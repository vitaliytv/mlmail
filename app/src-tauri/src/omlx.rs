//! Local omlx server config, read from `~/.omlx/settings.json` — the same
//! file the omlx server itself reads. `@7n/tauri-components` used to expose
//! this as `plugin:agent|omlx_config`; that command was dropped in 0.11.0
//! along with the whole direct-chat omlx path (see app/src/omlx.js), so
//! mlmail now reads the file itself.

use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmlxConfig {
    base_url: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
}

/// Read `(base_url, api_key)` from `~/.omlx/settings.json`. Any read error →
/// `(None, None)`, so callers fall back to their own defaults.
fn omlx_from_settings() -> (Option<String>, Option<String>) {
    let Some(home) = std::env::var_os("HOME") else {
        return (None, None);
    };
    let path = PathBuf::from(home).join(".omlx/settings.json");
    let Ok(raw) = fs::read_to_string(&path) else {
        return (None, None);
    };
    let Ok(json) = serde_json::from_str::<Value>(&raw) else {
        return (None, None);
    };
    let str_at = |obj: &str, key: &str| {
        json.get(obj)
            .and_then(|o| o.get(key))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
    };
    let host = str_at("server", "host");
    let port = json
        .get("server")
        .and_then(|s| s.get("port"))
        .and_then(Value::as_u64);
    let base_url = match (host, port) {
        (Some(h), Some(p)) => Some(format!("http://{h}:{p}/v1")),
        _ => None,
    };
    (base_url, str_at("auth", "api_key"))
}

#[tauri::command]
pub fn omlx_config() -> OmlxConfig {
    let (base_url, api_key) = omlx_from_settings();
    OmlxConfig {
        base_url,
        model: None,
        api_key,
    }
}
