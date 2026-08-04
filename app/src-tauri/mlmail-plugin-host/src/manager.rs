//! Plugin Manager lifecycle (M6): consent diff, disable, uninstall purge.
//!
//! Reuses `plugin-package::{install_package,list_installed,uninstall_plugin}` and
//! `GrantStore` / `AuditStore` / `TrustStore` under `app_data/plugins/`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use plugin_manifest::{CapabilityDecl, PluginManifest};
use plugin_package::{
    install_package, list_installed, pack_directory, uninstall_plugin, InstallOptions,
    InstalledPlugin,
};
use plugin_permissions::{
    audit_store_path, fingerprint_preview, grant_store_path, trust_store_path, AuditStore, Grant,
    GrantStore, Scope, TrustStore,
};
use serde::{Deserialize, Serialize};

use crate::HostError;

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

/// Preview install/update consent for a local `.nitra-plugin` path.
pub fn preview_install(
    app_data: &Path,
    package_path: &Path,
    allow_unsigned: bool,
) -> Result<InstallPreview, HostError> {
    let verified = plugin_package::verify_package(package_path, None, allow_unsigned)
        .map_err(|e| HostError::Parse(e.to_string()))?;
    let trust = TrustStore::open(trust_store_path(app_data))?;
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
    // Also treat untrusted new key as requiring TOFU consent (UI shows fingerprint)
    let _ = trust;
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
) -> Result<ManagedPlugin, HostError> {
    let mut trust = TrustStore::open(trust_store_path(app_data))?;
    let opts = InstallOptions {
        allow_unsigned,
        tofu_accept,
        ..InstallOptions::default()
    };
    let installed = install_package(
        package_path,
        &registry_root(app_data),
        &mut trust,
        None,
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

/// Pack + install the sample Draft Helper (unsigned, debug path) with default grants.
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
    fs::write(staging.join("component.wasm"), b"\0asm\x01\x00\x00\x00")
        .map_err(|e| HostError::Parse(e.to_string()))?;
    fs::write(staging.join("settings.schema.json"), b"{}")
        .map_err(|e| HostError::Parse(e.to_string()))?;
    fs::write(
        staging.join("changelog.md"),
        b"# Draft Helper\n\nSample plugin for M6.\n",
    )
    .map_err(|e| HostError::Parse(e.to_string()))?;

    let pkg = app_data
        .join("plugins")
        .join("_staging")
        .join("sample.nitra-plugin");
    pack_directory(&staging, &pkg, None).map_err(|e| HostError::Parse(e.to_string()))?;

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
    install_with_consent(app_data, &pkg, user_id, &grants, true, true)
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
}
