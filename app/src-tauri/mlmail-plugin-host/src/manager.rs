//! Plugin Manager lifecycle (M6): consent diff, disable, uninstall purge.
//!
//! Reuses `plugin-package::{install_package,list_installed,uninstall_plugin}` and
//! `GrantStore` / `AuditStore` / `TrustStore` under `app_data/plugins/`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use plugin_manifest::{CapabilityDecl, PluginManifest};
use std::io::Read;
use std::sync::Arc;

use plugin_mail::{DraftCreateResult, MailHost};
use plugin_package::{
    generate_keypair, install_package, list_installed, pack_directory, signing_key_from_bytes,
    uninstall_plugin, write_public_key_file, InstallOptions, InstalledPlugin,
    DEFAULT_PACKAGE_FILENAME, PACKAGE_EXTENSION,
};
use plugin_permissions::{
    audit_store_path, fingerprint_preview, grant_store_path, trust_store_path, AuditEntry,
    AuditStore, Grant, GrantStore, Scope, TrustStore,
};
use serde::{Deserialize, Serialize};

use crate::{HostError, MailPluginSession, MockMailHost, ResourceLimits, DRAFT_HELPER_WAT};

/// Capability grant request accepted from the consent UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsentGrant {
    pub capability: String,
    pub resource_kind: String,
    pub resource_id: Option<String>,
}

/// Diff between currently granted / installed caps and a candidate package.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConsentDiff {
    pub added: Vec<CapabilityDecl>,
    pub removed: Vec<CapabilityDecl>,
    pub unchanged: Vec<CapabilityDecl>,
    pub key_changed: bool,
}

/// One managed plugin row for the Manager UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub publisher_key_id: String,
    pub fingerprint: Option<String>,
    pub capabilities: Vec<CapabilityDecl>,
    pub granted: Vec<Grant>,
    pub disabled: bool,
    pub install_dir: String,
    pub changelog: String,
}

/// Preview before install/update (manifest + consent diff + fingerprint).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPreview {
    pub manifest: PluginManifest,
    pub diff: ConsentDiff,
    pub fingerprint: Option<String>,
    pub already_installed: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PluginStateFile {
    /// plugin_id → disabled
    plugins: BTreeMap<String, PluginStateEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PluginStateEntry {
    disabled: bool,
    #[serde(default)]
    active_version: Option<String>,
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// `app_data/plugins/registry`
pub fn registry_root(app_data: &Path) -> PathBuf {
    app_data.join("plugins").join("registry")
}

/// `app_data/plugins/state.json`
pub fn state_path(app_data: &Path) -> PathBuf {
    app_data.join("plugins").join("state.json")
}

/// `app_data/plugins/settings/<plugin_id>/`
pub fn settings_dir(app_data: &Path, plugin_id: &str) -> PathBuf {
    app_data.join("plugins").join("settings").join(plugin_id)
}

fn load_state(app_data: &Path) -> Result<PluginStateFile, HostError> {
    let path = state_path(app_data);
    if !path.exists() {
        return Ok(PluginStateFile::default());
    }
    let text = fs::read_to_string(&path).map_err(|e| HostError::Parse(e.to_string()))?;
    serde_json::from_str(&text).map_err(|e| HostError::Parse(e.to_string()))
}

fn save_state(app_data: &Path, state: &PluginStateFile) -> Result<(), HostError> {
    let path = state_path(app_data);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| HostError::Parse(e.to_string()))?;
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(state).map_err(|e| HostError::Parse(e.to_string()))?,
    )
    .map_err(|e| HostError::Parse(e.to_string()))
}

fn cap_key(c: &CapabilityDecl) -> String {
    let mut kinds = c.resource_kinds.clone();
    kinds.sort();
    format!("{}|{}", c.name, kinds.join(","))
}

/// Pure consent diff between previous and candidate capability declarations.
pub fn diff_capabilities(
    previous: &[CapabilityDecl],
    candidate: &[CapabilityDecl],
    key_changed: bool,
) -> ConsentDiff {
    let prev: BTreeSet<String> = previous.iter().map(cap_key).collect();
    let next: BTreeSet<String> = candidate.iter().map(cap_key).collect();
    let by_key_prev: BTreeMap<String, CapabilityDecl> =
        previous.iter().map(|c| (cap_key(c), c.clone())).collect();
    let by_key_next: BTreeMap<String, CapabilityDecl> =
        candidate.iter().map(|c| (cap_key(c), c.clone())).collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut unchanged = Vec::new();
    for k in next.difference(&prev) {
        if let Some(c) = by_key_next.get(k) {
            added.push(c.clone());
        }
    }
    for k in prev.difference(&next) {
        if let Some(c) = by_key_prev.get(k) {
            removed.push(c.clone());
        }
    }
    for k in prev.intersection(&next) {
        if let Some(c) = by_key_next.get(k) {
            unchanged.push(c.clone());
        }
    }
    ConsentDiff {
        added,
        removed,
        unchanged,
        key_changed,
    }
}

fn installed_for_id(
    registry: &Path,
    plugin_id: &str,
) -> Result<Option<InstalledPlugin>, HostError> {
    let list = list_installed(registry).map_err(|e| HostError::Parse(e.to_string()))?;
    Ok(list
        .into_iter()
        .filter(|p| p.manifest.id == plugin_id)
        .max_by(|a, b| a.manifest.version.cmp(&b.manifest.version)))
}

fn read_changelog(install_dir: &Path) -> String {
    fs::read_to_string(install_dir.join("changelog.md")).unwrap_or_default()
}

/// Companion public key path: `{package_filename}.pub` (same as `write_public_key_file`).
fn companion_public_key_path(package_path: &Path) -> PathBuf {
    package_path.with_file_name(format!(
        "{}.pub",
        package_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(DEFAULT_PACKAGE_FILENAME)
    ))
}

fn read_companion_public_key(package_path: &Path) -> Option<String> {
    let path = companion_public_key_path(package_path);
    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn peek_manifest(package_path: &Path) -> Result<PluginManifest, HostError> {
    let file = fs::File::open(package_path).map_err(|e| HostError::Parse(e.to_string()))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| HostError::Parse(e.to_string()))?;
    let mut entry = archive
        .by_name("plugin.toml")
        .map_err(|e| HostError::Parse(e.to_string()))?;
    let mut toml = String::new();
    entry
        .read_to_string(&mut toml)
        .map_err(|e| HostError::Parse(e.to_string()))?;
    PluginManifest::parse(&toml).map_err(|e| HostError::Parse(e.to_string()))
}

/// Resolve verifying key: explicit → companion `.pub` → trust store by publisher_key_id.
fn resolve_public_key_hex(
    app_data: &Path,
    package_path: &Path,
    public_key_hex: Option<&str>,
) -> Result<Option<String>, HostError> {
    if let Some(k) = public_key_hex {
        return Ok(Some(k.to_string()));
    }
    if let Some(k) = read_companion_public_key(package_path) {
        return Ok(Some(k));
    }
    if let Ok(manifest) = peek_manifest(package_path) {
        let trust = TrustStore::open(trust_store_path(app_data))?;
        if let Some(trusted) = trust.get(&manifest.publisher_key_id) {
            return Ok(Some(trusted.public_key_hex.clone()));
        }
    }
    Ok(None)
}

/// List managed plugins (registry + grants + disabled state).
pub fn list_managed(app_data: &Path, user_id: &str) -> Result<Vec<ManagedPlugin>, HostError> {
    let registry = registry_root(app_data);
    let state = load_state(app_data)?;
    let grants = GrantStore::open(grant_store_path(app_data))?;
    let installed = list_installed(&registry).map_err(|e| HostError::Parse(e.to_string()))?;

    // Prefer newest version per id
    let mut best: BTreeMap<String, InstalledPlugin> = BTreeMap::new();
    for p in installed {
        best.entry(p.manifest.id.clone())
            .and_modify(|cur| {
                if p.manifest.version > cur.manifest.version {
                    *cur = p.clone();
                }
            })
            .or_insert(p);
    }

    let mut out = Vec::new();
    for (_, p) in best {
        let disabled = state
            .plugins
            .get(&p.manifest.id)
            .map(|e| e.disabled)
            .unwrap_or(false);
        let granted = grants
            .list_for_plugin(&p.manifest.id)
            .into_iter()
            .filter(|g| g.user_id == user_id)
            .collect();
        out.push(ManagedPlugin {
            id: p.manifest.id.clone(),
            name: p.manifest.name.clone(),
            version: p.manifest.version.clone(),
            publisher: p.manifest.publisher.clone(),
            publisher_key_id: p.manifest.publisher_key_id.clone(),
            fingerprint: p.public_key_hex.as_deref().map(fingerprint_preview),
            capabilities: p.manifest.capabilities.clone(),
            granted,
            disabled,
            install_dir: p.install_dir.display().to_string(),
            changelog: read_changelog(&p.install_dir),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Preview install/update consent for a local `.n-plugin` path.
pub fn preview_install(
    app_data: &Path,
    package_path: &Path,
    allow_unsigned: bool,
) -> Result<InstallPreview, HostError> {
    let key = resolve_public_key_hex(app_data, package_path, None)?;
    let verified = plugin_package::verify_package(package_path, key.as_deref(), allow_unsigned)
        .map_err(|e| HostError::Parse(e.to_string()))?;
    let existing = installed_for_id(&registry_root(app_data), &verified.manifest.id)?;
    let previous_caps = existing
        .as_ref()
        .map(|p| p.manifest.capabilities.clone())
        .unwrap_or_default();
    let key_changed = match (&existing, &verified.public_key_hex) {
        (Some(prev), Some(new_key)) => prev
            .public_key_hex
            .as_ref()
            .map(|old| old != new_key)
            .unwrap_or(false),
        _ => false,
    };
    let diff = diff_capabilities(&previous_caps, &verified.manifest.capabilities, key_changed);
    Ok(InstallPreview {
        fingerprint: verified.public_key_hex.as_deref().map(fingerprint_preview),
        manifest: verified.manifest,
        diff,
        already_installed: existing.is_some(),
    })
}

/// Install (or update) after user accepted grants. Purges nothing on update.
pub fn install_with_consent(
    app_data: &Path,
    package_path: &Path,
    user_id: &str,
    grants: &[ConsentGrant],
    tofu_accept: bool,
    allow_unsigned: bool,
    public_key_hex: Option<&str>,
) -> Result<ManagedPlugin, HostError> {
    let mut trust = TrustStore::open(trust_store_path(app_data))?;
    let opts = InstallOptions {
        allow_unsigned,
        tofu_accept,
        ..InstallOptions::default()
    };
    let key = resolve_public_key_hex(app_data, package_path, public_key_hex)?;
    let installed = install_package(
        package_path,
        &registry_root(app_data),
        &mut trust,
        key.as_deref(),
        &opts,
    )
    .map_err(|e| HostError::Parse(e.to_string()))?;

    let mut store = GrantStore::open(grant_store_path(app_data))?;
    for g in grants {
        store.grant(Grant {
            plugin_id: installed.manifest.id.clone(),
            user_id: user_id.to_string(),
            scope: Scope {
                capability: g.capability.clone(),
                resource_kind: g.resource_kind.clone(),
                resource_id: g.resource_id.clone(),
            },
            granted_at_unix: now_unix(),
        })?;
    }

    // Ensure enabled in state
    let mut state = load_state(app_data)?;
    state.plugins.insert(
        installed.manifest.id.clone(),
        PluginStateEntry {
            disabled: false,
            active_version: Some(installed.manifest.version.clone()),
        },
    );
    save_state(app_data, &state)?;

    // settings dir scaffold
    let settings = settings_dir(app_data, &installed.manifest.id);
    fs::create_dir_all(&settings).map_err(|e| HostError::Parse(e.to_string()))?;

    let list = list_managed(app_data, user_id)?;
    list.into_iter()
        .find(|p| p.id == installed.manifest.id)
        .ok_or_else(|| HostError::Parse("installed plugin missing from list".into()))
}

/// Disable/enable: stop invocations when disabled; keep package + grants + settings.
pub fn set_disabled(app_data: &Path, plugin_id: &str, disabled: bool) -> Result<(), HostError> {
    if installed_for_id(&registry_root(app_data), plugin_id)?.is_none() {
        return Err(HostError::Parse(format!(
            "plugin not installed: {plugin_id}"
        )));
    }
    let mut state = load_state(app_data)?;
    let entry = state.plugins.entry(plugin_id.to_string()).or_default();
    entry.disabled = disabled;
    save_state(app_data, &state)
}

/// Whether the plugin may be invoked (installed and not disabled).
pub fn ensure_invocable(app_data: &Path, plugin_id: &str) -> Result<(), HostError> {
    if installed_for_id(&registry_root(app_data), plugin_id)?.is_none() {
        return Err(HostError::Parse(format!(
            "plugin not installed: {plugin_id}"
        )));
    }
    let state = load_state(app_data)?;
    if state
        .plugins
        .get(plugin_id)
        .map(|e| e.disabled)
        .unwrap_or(false)
    {
        return Err(HostError::Parse(format!("plugin disabled: {plugin_id}")));
    }
    Ok(())
}

/// Uninstall: remove package + settings + grants + audit rows + state entry.
pub fn uninstall_purge(app_data: &Path, plugin_id: &str) -> Result<(), HostError> {
    uninstall_plugin(&registry_root(app_data), plugin_id)
        .map_err(|e| HostError::Parse(e.to_string()))?;

    let settings = settings_dir(app_data, plugin_id);
    if settings.exists() {
        fs::remove_dir_all(&settings).map_err(|e| HostError::Parse(e.to_string()))?;
    }

    let mut grants = GrantStore::open(grant_store_path(app_data))?;
    grants.purge_plugin(plugin_id)?;

    let mut audit = AuditStore::open(audit_store_path(app_data))?;
    audit.purge_plugin(plugin_id)?;

    let mut state = load_state(app_data)?;
    state.plugins.remove(plugin_id);
    save_state(app_data, &state)?;
    Ok(())
}

/// Pack + install the signed sample Draft Helper with default grants.
pub fn install_sample_draft_helper(
    app_data: &Path,
    user_id: &str,
) -> Result<ManagedPlugin, HostError> {
    let staging = app_data.join("plugins").join("_staging").join("sample");
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|e| HostError::Parse(e.to_string()))?;
    }
    fs::create_dir_all(&staging).map_err(|e| HostError::Parse(e.to_string()))?;
    fs::write(
        staging.join("plugin.toml"),
        r#"id = "com.example.mail-draft-helper"
name = "Draft Helper"
version = "0.1.0"
publisher = "example"
publisher_key_id = "ext_example_2026"
[a2ui]
protocol = "1.0"
schema_rev = "ae2785521b33222f775bac50d080066bac110b4ab5214945c4d8c5bee6a35416"
[requires]
platform = "^0.1"
"nitra:mail" = "^0.1"
[[capabilities]]
name = "mail:metadata.read"
resource_kinds = ["message"]
[[capabilities]]
name = "mail:draft.create"
resource_kinds = ["account"]
[[surfaces]]
id = "sidebar.draft-helper"
kind = "sidebar"
"#,
    )
    .map_err(|e| HostError::Parse(e.to_string()))?;
    let wasm = wat::parse_str(DRAFT_HELPER_WAT).map_err(|e| HostError::Parse(e.to_string()))?;
    fs::write(staging.join("component.wasm"), wasm).map_err(|e| HostError::Parse(e.to_string()))?;
    fs::write(staging.join("settings.schema.json"), b"{}")
        .map_err(|e| HostError::Parse(e.to_string()))?;
    fs::write(
        staging.join("changelog.md"),
        b"# Draft Helper\n\nSample plugin for M6.\n",
    )
    .map_err(|e| HostError::Parse(e.to_string()))?;

    let staging_root = app_data.join("plugins").join("_staging");
    let pkg = staging_root.join(format!("sample.{PACKAGE_EXTENSION}"));
    let sk_path = staging_root.join("sample.ed25519");
    let pub_cache = staging_root.join("sample.pubhex");
    let (sk, pub_hex) = if sk_path.exists() && pub_cache.exists() {
        let bytes = fs::read(&sk_path).map_err(|e| HostError::Parse(e.to_string()))?;
        let sk = signing_key_from_bytes(&bytes).map_err(|e| HostError::Parse(e.to_string()))?;
        let pub_hex = fs::read_to_string(&pub_cache)
            .map_err(|e| HostError::Parse(e.to_string()))?
            .trim()
            .to_string();
        (sk, pub_hex)
    } else {
        let (sk, pub_hex) = generate_keypair();
        fs::create_dir_all(&staging_root).map_err(|e| HostError::Parse(e.to_string()))?;
        fs::write(&sk_path, sk.to_bytes()).map_err(|e| HostError::Parse(e.to_string()))?;
        fs::write(&pub_cache, &pub_hex).map_err(|e| HostError::Parse(e.to_string()))?;
        (sk, pub_hex)
    };
    pack_directory(&staging, &pkg, Some(&sk)).map_err(|e| HostError::Parse(e.to_string()))?;
    write_public_key_file(&pkg, &pub_hex).map_err(|e| HostError::Parse(e.to_string()))?;

    let grants = vec![
        ConsentGrant {
            capability: "mail:metadata.read".into(),
            resource_kind: "message".into(),
            resource_id: Some("msg_1".into()),
        },
        ConsentGrant {
            capability: "mail:draft.create".into(),
            resource_kind: "account".into(),
            resource_id: Some("acct_1".into()),
        },
    ];
    install_with_consent(
        app_data,
        &pkg,
        user_id,
        &grants,
        true,
        false,
        Some(&pub_hex),
    )
}

/// Invoke installed plugin Wasm (`component.wasm`) for draft.create via MockMailHost.
pub fn create_draft_from_installed(
    app_data: &Path,
    user_id: &str,
    plugin_id: &str,
) -> Result<(DraftCreateResult, AuditEntry), HostError> {
    ensure_invocable(app_data, plugin_id)?;
    let installed = installed_for_id(&registry_root(app_data), plugin_id)?
        .ok_or_else(|| HostError::Parse(format!("plugin not installed: {plugin_id}")))?;
    let wasm_path = installed.install_dir.join("component.wasm");
    let wasm = fs::read(&wasm_path).map_err(|e| HostError::Parse(e.to_string()))?;

    let grants = GrantStore::open(grant_store_path(app_data))?;
    let audit = AuditStore::open(audit_store_path(app_data))?;
    let session = MailPluginSession::new(
        installed.manifest.id.clone(),
        installed.manifest.version.clone(),
        user_id,
        grants,
        audit,
        ResourceLimits {
            wall_clock: std::time::Duration::from_secs(5),
            ..ResourceLimits::default()
        },
    )?;
    let handle = session.runtime.load_wasm(&session.plugin_id, &wasm)?;
    session.runtime.activate(&handle)?;
    let mock: Arc<dyn MailHost> = Arc::new(MockMailHost::default());
    let result = session.create_draft_via_sample(&handle, mock, "ui-sidebar")?;
    let entry = session
        .audit
        .lock()
        .expect("audit")
        .list()
        .last()
        .cloned()
        .ok_or_else(|| HostError::Parse("audit entry missing".into()))?;
    Ok((result, entry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_caps() -> Vec<CapabilityDecl> {
        vec![
            CapabilityDecl {
                name: "mail:metadata.read".into(),
                resource_kinds: vec!["message".into()],
            },
            CapabilityDecl {
                name: "mail:draft.create".into(),
                resource_kinds: vec!["account".into()],
            },
        ]
    }

    #[test]
    fn consent_diff_same_caps() {
        let caps = sample_caps();
        let diff = diff_capabilities(&caps, &caps, false);
        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert_eq!(diff.unchanged.len(), 2);
        assert!(!diff.key_changed);
    }

    #[test]
    fn consent_diff_escalation() {
        let prev = vec![CapabilityDecl {
            name: "mail:metadata.read".into(),
            resource_kinds: vec!["message".into()],
        }];
        let next = sample_caps();
        let diff = diff_capabilities(&prev, &next, false);
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].name, "mail:draft.create");
        assert_eq!(diff.unchanged.len(), 1);
    }

    #[test]
    fn install_disable_uninstall_purge() {
        let dir = tempdir().unwrap();
        let app_data = dir.path();
        let managed = install_sample_draft_helper(app_data, "u1").unwrap();
        assert_eq!(managed.id, "com.example.mail-draft-helper");
        assert!(!managed.disabled);
        assert_eq!(managed.granted.len(), 2);
        assert!(PathBuf::from(&managed.install_dir).exists());
        assert!(settings_dir(app_data, &managed.id).exists());

        set_disabled(app_data, &managed.id, true).unwrap();
        assert!(ensure_invocable(app_data, &managed.id).is_err());
        let listed = list_managed(app_data, "u1").unwrap();
        assert!(listed[0].disabled);
        // package + grants kept while disabled
        assert!(PathBuf::from(&listed[0].install_dir).exists());
        assert_eq!(listed[0].granted.len(), 2);

        set_disabled(app_data, &managed.id, false).unwrap();
        ensure_invocable(app_data, &managed.id).unwrap();

        uninstall_purge(app_data, &managed.id).unwrap();
        assert!(list_managed(app_data, "u1").unwrap().is_empty());
        assert!(!settings_dir(app_data, &managed.id).exists());
        let grants = GrantStore::open(grant_store_path(app_data)).unwrap();
        assert!(grants.list_for_plugin(&managed.id).is_empty());
    }

    #[test]
    fn update_same_caps_keeps_grants() {
        let dir = tempdir().unwrap();
        let app_data = dir.path();
        let first = install_sample_draft_helper(app_data, "u1").unwrap();
        assert_eq!(first.granted.len(), 2);

        // Re-install same sample (same caps) — grants survive (idempotent grant)
        let second = install_sample_draft_helper(app_data, "u1").unwrap();
        assert_eq!(second.granted.len(), 2);
        let caps = sample_caps();
        let diff = diff_capabilities(&caps, &caps, false);
        assert!(diff.added.is_empty());
    }

    #[test]
    fn create_draft_from_installed_sample() {
        let dir = tempdir().unwrap();
        let app_data = dir.path();
        install_sample_draft_helper(app_data, "u1").unwrap();
        let (result, entry) =
            create_draft_from_installed(app_data, "u1", "com.example.mail-draft-helper").unwrap();
        assert!(!result.draft_id.is_empty());
        assert_eq!(entry.action_id, "createDraft");
        assert!(entry.result == "ok" || entry.result.starts_with("ok"));
    }
}
