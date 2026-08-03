//! Tauri commands for the plugin / A2UI surface (M4–M5).

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
#[tauri::command]
pub fn plugin_sidebar_create_draft(app: AppHandle) -> Result<PluginDraftActionDto, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {e}"))?;
    let (result, entry) =
        mlmail_plugin_host::demo_create_draft_with_audit(&dir).map_err(|e| e.to_string())?;
    Ok(PluginDraftActionDto {
        draft_id: result.draft_id,
        audit_result: entry.result,
        correlation_id: entry.correlation_id,
        capability: entry.capability,
    })
}
