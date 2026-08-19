//! Product boundary for installed n-plugin Components in mlmail.
//!
//! The manager accepts only raw WebAssembly Components with an embedded
//! `nitra.plugin-manifest/v1`. It records their immutable activation generation
//! in the generic n-plugin registry; OAuth credentials never enter plugin files.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use n_plugin_compatibility::GraphLifecycleState;
use n_plugin_oci::{ResolvedNode, ResolvedPluginGraph};
use n_plugin_package::{inspect_component, ReleaseIdentity};
use n_plugin_runtime::{ActivationCompiler, ActivationGeneration, ActivationRegistry};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::{
    auth::{self, state::AuthState, storage::SharedStorage},
    endpoints::Endpoints,
    gmail::{plugin_draft_helper::invoke_draft_helper, plugin_runtime::build_gmail_plugin_runtime},
    plugin_context::{self, PluginContextCoordinator},
    plugin_contracts::MlmailPluginContractRegistry,
    plugin_install::{preflight_component, PluginInstallPreview},
};

const DRAFT_HELPER_TRIGGER: &str = "nitra:gmail/draft-helper@0.1.0";
const GMAIL_WKG_LOCK: &str = include_str!("../wkg.lock");

/// One installed root Component shown by the Vue Plugin Manager.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPlugin {
    /// Exact immutable release selected at installation time.
    pub release: ReleaseIdentity,
    /// Product triggers declared by the embedded manifest.
    pub triggers: Vec<String>,
    /// Explicit user intent; disabled Components cannot be invoked.
    pub enabled: bool,
}

/// Result returned after the installed Draft Helper creates a native Gmail draft.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginDraftActionDto {
    /// Opaque Gmail draft identifier returned by Gmail.
    pub draft_id: String,
    /// Exact Component release that initiated the operation.
    pub release: ReleaseIdentity,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
struct PluginIndex {
    next_generation: u64,
    plugins: Vec<InstalledPlugin>,
}

fn app_data(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| format!("app_data_dir: {error}"))
}

fn plugin_root(app_data: &Path) -> PathBuf {
    app_data.join("n-plugin")
}

fn registry(app_data: &Path) -> anyhow::Result<ActivationRegistry> {
    let root = plugin_root(app_data);
    ActivationRegistry::open(root.join("registry.sqlite3"), root.join("cas"))
}

fn index_path(app_data: &Path) -> PathBuf {
    plugin_root(app_data).join("installed.json")
}

fn load_index(app_data: &Path) -> anyhow::Result<PluginIndex> {
    let path = index_path(app_data);
    match fs::read(&path) {
        Ok(source) => Ok(serde_json::from_slice(&source)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(PluginIndex::default()),
        Err(error) => Err(error.into()),
    }
}

fn write_index(app_data: &Path, index: &PluginIndex) -> anyhow::Result<()> {
    let path = index_path(app_data);
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("plugin index has no parent directory"))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".installed-{}.tmp", std::process::id()));
    fs::write(&temporary, serde_json::to_vec_pretty(index)?)?;
    fs::rename(temporary, path)?;
    Ok(())
}

fn ensure_gmail_wkg_lock(app_data: &Path) -> anyhow::Result<PathBuf> {
    let path = plugin_root(app_data).join("wkg.lock");
    if let Ok(existing) = fs::read_to_string(&path) {
        if existing == GMAIL_WKG_LOCK {
            return Ok(path);
        }
        anyhow::bail!("installed Gmail WKG lock differs from this mlmail release");
    }
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Gmail WKG lock has no parent directory"))?;
    fs::create_dir_all(parent)?;
    fs::write(&path, GMAIL_WKG_LOCK)?;
    Ok(path)
}

async fn contract_registry(app_data: &Path) -> anyhow::Result<MlmailPluginContractRegistry> {
    let lock_path = ensure_gmail_wkg_lock(app_data)?;
    MlmailPluginContractRegistry::load(lock_path).await
}

async fn preflight_plugin_at(app_data: &Path, path: &Path) -> anyhow::Result<PluginInstallPreview> {
    let component = fs::read(path)?;
    let contracts = contract_registry(app_data).await?;
    preflight_component(&component, &contracts)
}

async fn install_plugin_at(app_data: &Path, path: &Path) -> anyhow::Result<InstalledPlugin> {
    let component = fs::read(path)?;
    let contracts = contract_registry(app_data).await?;
    let preview = preflight_component(&component, &contracts)?;
    if !preview.compatible {
        anyhow::bail!(
            "plugin is incompatible with this mlmail release: {}",
            preview
                .reason
                .as_deref()
                .unwrap_or("installation preflight did not provide a reason")
        );
    }
    let embedded = inspect_component(&component)?;
    if embedded.release != preview.release {
        anyhow::bail!("plugin release changed during installation preflight");
    }
    let mut index = load_index(app_data)?;
    let next_generation = index
        .next_generation
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("plugin activation generation overflow"))?;
    let generation = ActivationGeneration::new(next_generation)?;
    let graph = ResolvedPluginGraph {
        root: embedded.release.clone(),
        nodes: vec![ResolvedNode {
            release: embedded.release.clone(),
            manifest: embedded.manifest.clone(),
            reference: "local-install".to_owned(),
            component,
        }],
        edges: Vec::new(),
        lock_file: PathBuf::new(),
    };
    let plan = ActivationCompiler::new()?.compile(
        &graph,
        generation,
        &contracts.host_inventory(),
        &contracts.trigger_inventory(),
    )?;
    registry(app_data)?.publish(&plan, &graph)?;

    let installed = InstalledPlugin {
        release: embedded.release,
        triggers: graph
            .nodes
            .first()
            .expect("one root node was constructed")
            .manifest
            .triggers
            .iter()
            .map(|trigger| trigger.as_str().to_owned())
            .collect(),
        enabled: true,
    };
    index
        .plugins
        .retain(|existing| existing.release.package != installed.release.package);
    index.plugins.push(installed.clone());
    index
        .plugins
        .sort_by(|left, right| left.release.package.cmp(&right.release.package));
    index.next_generation = next_generation;
    publish_plugin_context(app_data, &index)?;
    write_index(app_data, &index)?;
    Ok(installed)
}

fn find_draft_helper(app_data: &Path) -> anyhow::Result<InstalledPlugin> {
    load_index(app_data)?
        .plugins
        .into_iter()
        .find(|plugin| {
            plugin.enabled
                && plugin
                    .triggers
                    .iter()
                    .any(|trigger| trigger == DRAFT_HELPER_TRIGGER)
        })
        .ok_or_else(|| anyhow::anyhow!("install and enable a Draft Helper Component first"))
}

fn publish_plugin_context(app_data: &Path, index: &PluginIndex) -> anyhow::Result<()> {
    let draft_helper = index
        .plugins
        .iter()
        .find(|plugin| {
            plugin
                .triggers
                .iter()
                .any(|trigger| trigger == DRAFT_HELPER_TRIGGER)
        })
        .map(|plugin| (plugin.release.clone(), plugin.enabled));
    let entries = plugin_context::draft_helper_context(draft_helper)?;
    let store =
        n_plugin_runtime::DurableContextStore::open(plugin_context::context_database(app_data))?;
    if store
        .active()?
        .is_none_or(|desired| desired.entries != entries)
    {
        store.publish(entries)?;
    }
    Ok(())
}

/// Lists root Components installed through the n-plugin Component manager.
#[tauri::command]
pub fn plugin_manager_list(app: AppHandle) -> Result<Vec<InstalledPlugin>, String> {
    load_index(&app_data(&app)?)
        .map(|index| index.plugins)
        .map_err(|error| error.to_string())
}

/// Returns a read-only compatibility and consent preview for one local Component.
#[tauri::command]
pub async fn plugin_manager_preflight(
    app: AppHandle,
    path: String,
) -> Result<PluginInstallPreview, String> {
    preflight_plugin_at(&app_data(&app)?, Path::new(&path))
        .await
        .map_err(|error| error.to_string())
}

/// Compatibility wrapper that validates, composes, and atomically activates one local Component.
#[tauri::command]
pub async fn plugin_manager_install(
    app: AppHandle,
    path: String,
) -> Result<InstalledPlugin, String> {
    install_plugin_at(&app_data(&app)?, Path::new(&path))
        .await
        .map_err(|error| error.to_string())
}

/// Changes explicit user enablement without deleting the immutable Component generation.
#[tauri::command]
pub fn plugin_manager_set_disabled(
    app: AppHandle,
    package: String,
    disabled: bool,
) -> Result<(), String> {
    let app_data = app_data(&app)?;
    let mut index = load_index(&app_data).map_err(|error| error.to_string())?;
    let plugin = index
        .plugins
        .iter_mut()
        .find(|plugin| plugin.release.package == package)
        .ok_or_else(|| format!("plugin `{package}` is not installed"))?;
    plugin.enabled = !disabled;
    let lifecycle = if disabled {
        GraphLifecycleState::manually_disabled()
    } else {
        GraphLifecycleState::active()
    };
    registry(&app_data)
        .and_then(|registry| registry.set_graph_lifecycle(&plugin.release, lifecycle))
        .map_err(|error| error.to_string())?;
    publish_plugin_context(&app_data, &index).map_err(|error| error.to_string())?;
    write_index(&app_data, &index).map_err(|error| error.to_string())
}

/// Disables and forgets one installed root Component; unreachable CAS data is collected later.
#[tauri::command]
pub fn plugin_manager_uninstall(app: AppHandle, package: String) -> Result<(), String> {
    let app_data = app_data(&app)?;
    let mut index = load_index(&app_data).map_err(|error| error.to_string())?;
    let position = index
        .plugins
        .iter()
        .position(|plugin| plugin.release.package == package)
        .ok_or_else(|| format!("plugin `{package}` is not installed"))?;
    let plugin = index.plugins.remove(position);
    registry(&app_data)
        .and_then(|registry| {
            registry.set_graph_lifecycle(&plugin.release, GraphLifecycleState::manually_disabled())
        })
        .map_err(|error| error.to_string())?;
    publish_plugin_context(&app_data, &index).map_err(|error| error.to_string())?;
    write_index(&app_data, &index).map_err(|error| error.to_string())
}

/// Runs the enabled Draft Helper Component with the current account's app-owned OAuth token.
#[tauri::command]
pub async fn plugin_draft_helper_create(
    app: AppHandle,
    endpoints: State<'_, Endpoints>,
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<PluginDraftActionDto, String> {
    let app_data = app_data(&app)?;
    let plugin = find_draft_helper(&app_data).map_err(|error| error.to_string())?;
    let index = load_index(&app_data).map_err(|error| error.to_string())?;
    publish_plugin_context(&app_data, &index).map_err(|error| error.to_string())?;
    let registry = registry(&app_data).map_err(|error| error.to_string())?;
    let active = registry
        .active(&plugin.release)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "installed Draft Helper has no active generation".to_owned())?;
    let lifecycle = registry
        .graph_lifecycle(&plugin.release)
        .map_err(|error| error.to_string())?;
    if lifecycle.state != GraphLifecycleState::active() {
        return Err("Draft Helper is disabled or unavailable".to_owned());
    }
    let component = registry
        .cas()
        .read(&active.composed_digest)
        .map_err(|error| error.to_string())?;
    let lock_path = ensure_gmail_wkg_lock(&app_data).map_err(|error| error.to_string())?;
    let runtime = build_gmail_plugin_runtime(&lock_path, env!("CARGO_PKG_VERSION"))
        .await
        .map_err(|error| error.to_string())?;
    let context = PluginContextCoordinator::start(plugin_context::context_database(&app_data))
        .await
        .map_err(|error| error.to_string())?;
    let action = context
        .run_draft_helper(|| async {
            let token = auth::acquire_access_token(
                &endpoints.google_token,
                storage.inner().as_ref(),
                state.inner(),
            )
            .await?;
            invoke_draft_helper(&runtime, &component, &endpoints.gmail_messages_list, token).await
        })
        .await
        .map_err(|error| error.to_string());
    if let Err(error) = context.shutdown().await {
        log::error!("Draft Helper context shutdown failed after emission boundary: {error}");
    }
    let draft = action?;
    Ok(PluginDraftActionDto {
        draft_id: draft.id,
        release: plugin.release,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::{Context, Result};
    use n_plugin_package::{embed_manifest, PluginManifest, ReleaseIdentity};
    use n_plugin_runtime::ensure_component;

    use super::{
        install_plugin_at, load_index, plugin_root, preflight_plugin_at, registry, write_index,
        InstalledPlugin, PluginIndex,
    };
    use crate::plugin_contracts::{GMAIL_DRAFTS_INTERFACE, GMAIL_DRAFT_HELPER_INTERFACE};

    #[test]
    fn persists_exact_component_identity_and_user_enablement() {
        let temporary = tempfile::tempdir().expect("temporary plugin directory should open");
        let index = PluginIndex {
            next_generation: 3,
            plugins: vec![InstalledPlugin {
                release: ReleaseIdentity {
                    package: "nitra:draft-helper".to_owned(),
                    version: "0.1.0".to_owned(),
                    digest:
                        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                            .to_owned(),
                },
                triggers: vec!["nitra:gmail/draft-helper@0.1.0".to_owned()],
                enabled: false,
            }],
        };

        write_index(temporary.path(), &index).expect("index should persist");
        assert_eq!(
            load_index(temporary.path()).expect("index should reopen"),
            index
        );
    }

    #[tokio::test]
    async fn previews_a_local_component_without_activation_writes() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let manifest = PluginManifest::from_toml(&format!(
            r#"
schema = "nitra.plugin-manifest/v1"
publisher_id = "other"
package = "draft-helper"
version = "0.1.0"
triggers = ["{GMAIL_DRAFT_HELPER_INTERFACE}"]

[entrypoints]
create = "{GMAIL_DRAFT_HELPER_INTERFACE}"
"#,
        ))?;
        let raw = wat::parse_str(format!(
            r#"
(component
  (type $host-contract (instance))
  (import "{GMAIL_DRAFTS_INTERFACE}" (instance (type $host-contract)))
  (core module $module)
  (core instance $core (instantiate $module))
  (instance $api)
  (export "{GMAIL_DRAFT_HELPER_INTERFACE}" (instance $api))
)
"#,
        ))?;
        let path = temporary.path().join("draft-helper.n-plugin");
        fs::write(&path, embed_manifest(&raw, &manifest)?)?;

        let preview = preflight_plugin_at(temporary.path(), &path).await?;

        assert!(
            preview.compatible,
            "unexpected reason: {:?}",
            preview.reason
        );
        let root = plugin_root(temporary.path());
        for relative in [
            "registry.sqlite3",
            "cas",
            "installed.json",
            "context.sqlite3",
            ".n-plugin.lock",
        ] {
            assert!(!root.join(relative).exists(), "unexpected {relative}");
        }
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires MLMAIL_DRAFT_HELPER_COMPONENT to point to a packaged .n-plugin"]
    async fn installs_packaged_component_into_the_activation_registry() -> Result<()> {
        let component = std::env::var_os("MLMAIL_DRAFT_HELPER_COMPONENT")
            .context("MLMAIL_DRAFT_HELPER_COMPONENT must point to a packaged .n-plugin")?;
        let temporary = tempfile::tempdir()?;
        let installed =
            install_plugin_at(temporary.path(), std::path::Path::new(&component)).await?;
        let registry = registry(temporary.path())?;
        let active = registry
            .active(&installed.release)?
            .context("installed Component must have an active generation")?;

        assert_eq!(
            registry.graph_lifecycle(&installed.release)?.state,
            n_plugin_compatibility::GraphLifecycleState::active()
        );
        ensure_component(&registry.cas().read(&active.composed_digest)?)?;
        Ok(())
    }
}
