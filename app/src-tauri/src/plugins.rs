//! Tauri commands for plugins / A2UI / Manager (M4–M6).

use mlmail_plugin_host::{ConsentGrant, InstallPreview, ManagedPlugin};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Manager};

/// JSON DTO for a validated A2UI surface (camelCase for the Vue renderer).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct A2uiSurfaceDto {
    pub surface_id: String,
    pub catalog_id: String,
    pub components: Value,
    pub data_model: Value,
}

/// Result of a sidebar createDraft action (demo mock + audit).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginDraftActionDto {
    pub draft_id: String,
    pub audit_result: String,
    pub correlation_id: String,
    pub capability: String,
}

fn app_data(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {e}"))
}

/// Return the sample sidebar surface after Rust `plugin-a2ui` validation.
#[tauri::command]
pub fn plugin_a2ui_sample_sidebar() -> Result<A2uiSurfaceDto, String> {
    let surface = mlmail_plugin_host::sample_sidebar_surface().map_err(|e| e.to_string())?;
    let components = serde_json::to_value(&surface.components).map_err(|e| e.to_string())?;
    Ok(A2uiSurfaceDto {
        surface_id: surface.surface_id,
        catalog_id: surface.catalog_id,
        components,
        data_model: surface.data_model,
    })
}

/// Run sample Wasm `handle_action` → mock draft create + append audit (M5).
/// If the sample plugin is installed and disabled, refuse.
#[tauri::command]
pub fn plugin_sidebar_create_draft(app: AppHandle) -> Result<PluginDraftActionDto, String> {
    let dir = app_data(&app)?;
    let sample_id = "com.example.mail-draft-helper";
    if let Ok(list) = mlmail_plugin_host::list_managed(&dir, "demo-user") {
        if list.iter().any(|p| p.id == sample_id) {
            mlmail_plugin_host::ensure_invocable(&dir, sample_id).map_err(|e| e.to_string())?;
        }
    }
    let (result, entry) =
        mlmail_plugin_host::demo_create_draft_with_audit(&dir).map_err(|e| e.to_string())?;
    Ok(PluginDraftActionDto {
        draft_id: result.draft_id,
        audit_result: entry.result,
        correlation_id: entry.correlation_id,
        capability: entry.capability,
    })
}

/// List installed plugins for the Manager UI.
#[tauri::command]
pub fn plugin_manager_list(app: AppHandle) -> Result<Vec<ManagedPlugin>, String> {
    let dir = app_data(&app)?;
    mlmail_plugin_host::list_managed(&dir, "demo-user").map_err(|e| e.to_string())
}

/// Preview local package install (consent diff).
#[tauri::command]
pub fn plugin_manager_preview_install(
    app: AppHandle,
    path: String,
) -> Result<InstallPreview, String> {
    let dir = app_data(&app)?;
    mlmail_plugin_host::preview_install(&dir, std::path::Path::new(&path), true)
        .map_err(|e| e.to_string())
}

/// Install after consent; `grants` from the consent dialog.
#[tauri::command]
pub fn plugin_manager_install(
    app: AppHandle,
    path: String,
    grants: Vec<ConsentGrant>,
    tofu_accept: bool,
) -> Result<ManagedPlugin, String> {
    let dir = app_data(&app)?;
    mlmail_plugin_host::install_with_consent(
        &dir,
        std::path::Path::new(&path),
        "demo-user",
        &grants,
        tofu_accept,
        true,
    )
    .map_err(|e| e.to_string())
}

/// Pack + install the sample Draft Helper (debug / acceptance path).
#[tauri::command]
pub fn plugin_manager_install_sample(app: AppHandle) -> Result<ManagedPlugin, String> {
    let dir = app_data(&app)?;
    mlmail_plugin_host::install_sample_draft_helper(&dir, "demo-user").map_err(|e| e.to_string())
}

/// Disable or re-enable a plugin (keeps package + grants + settings).
#[tauri::command]
pub fn plugin_manager_set_disabled(
    app: AppHandle,
    plugin_id: String,
    disabled: bool,
) -> Result<(), String> {
    let dir = app_data(&app)?;
    mlmail_plugin_host::set_disabled(&dir, &plugin_id, disabled).map_err(|e| e.to_string())
}

/// Uninstall and purge grants/settings/audit for the plugin.
#[tauri::command]
pub fn plugin_manager_uninstall(app: AppHandle, plugin_id: String) -> Result<(), String> {
    let dir = app_data(&app)?;
    mlmail_plugin_host::uninstall_purge(&dir, &plugin_id).map_err(|e| e.to_string())
}
