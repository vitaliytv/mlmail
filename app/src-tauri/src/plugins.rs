//! Tauri commands for the plugin / A2UI surface (M4).

use serde::Serialize;
use serde_json::Value;

/// JSON DTO for a validated A2UI surface (camelCase for the Vue renderer).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct A2uiSurfaceDto {
    pub surface_id: String,
    pub catalog_id: String,
    pub components: Value,
    pub data_model: Value,
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
