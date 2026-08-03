//! Platform-agnostic chat command: Vue calls one `llm_chat`/`llm_list_models`/
//! `llm_providers` surface regardless of OS, mirroring the tool-surface
//! `dispatch` pattern (see `docs/adr/n-tool-surface-llm-доступний-бекенд.md`)
//! — the desktop/mobile split lives here, not in the frontend.
//!
//! Desktop: local OpenAI-compatible HTTP servers (omlx, litellm,
//! turbofieldfare, ...) via the `llm-lib` crate's `LocalCloud`, the Rust
//! mirror of `@7n/llm-lib`'s `local-providers.mjs` env-var convention
//! (`N_<PREFIX>_BASE_URL`/`N_<PREFIX>_API_KEY`).
//!
//! Android: LiteRT-LM on-device (Gemma 3n) via a JNI bridge — not yet
//! implemented (see ADR "Наступні кроки"), so these commands return a clear
//! error there rather than silently no-op.

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use llm_lib::local_cloud::LocalProvider;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::collections::HashMap;

/// `LiteRT-LM ще не реалізовано` — shared error text for the mobile stubs
/// below, so every command fails identically until the JNI bridge lands.
#[cfg(any(target_os = "android", target_os = "ios"))]
const LITERT_NOT_IMPLEMENTED: &str = "LiteRT-LM ще не реалізовано на цій платформі";

/// Read `(base_url, api_key)` from `~/.omlx/settings.json` — the same file
/// the omlx server itself reads. `None` on any error (missing file, bad
/// JSON, missing keys), so callers fall back to their own defaults.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn omlx_from_settings() -> (Option<String>, Option<String>) {
    use serde_json::Value;
    use std::path::PathBuf;

    let Some(home) = std::env::var_os("HOME") else {
        return (None, None);
    };
    let path = PathBuf::from(home).join(".omlx/settings.json");
    let Ok(raw) = std::fs::read_to_string(&path) else {
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
    // Trailing slash required — `LocalProvider::base_url` joins the request
    // path onto it (see llm-lib's local_cloud.rs doc comment).
    let base_url = match (host, port) {
        (Some(h), Some(p)) => Some(format!("http://{h}:{p}/v1/")),
        _ => None,
    };
    (base_url, str_at("auth", "api_key"))
}

/// Registered local providers, keyed by the prefix used in `"provider/model-id"`
/// specs — the same env-var names and defaults as `@7n/llm-lib`'s
/// `defaultLocalProviders()`, so a developer's existing `N_OMLX_*`/`N_LITELLM_*`
/// env stays valid whether the call goes through Node tooling or this app.
/// `omlx`'s default additionally falls back to `~/.omlx/settings.json`
/// (the omlx server's own config), read before the env override is applied.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn build_local_providers() -> HashMap<String, LocalProvider> {
    use std::env;

    let (settings_base_url, settings_api_key) = omlx_from_settings();
    let mut providers = HashMap::new();
    providers.insert(
        "omlx".to_string(),
        LocalProvider {
            base_url: env::var("N_OMLX_BASE_URL")
                .ok()
                .or(settings_base_url)
                .unwrap_or_else(|| "http://127.0.0.1:8000/v1/".to_string()),
            api_key: env::var("N_OMLX_API_KEY")
                .ok()
                .or_else(|| env::var("OMLX_API_KEY").ok())
                .or(settings_api_key),
        },
    );
    providers.insert(
        "litellm".to_string(),
        LocalProvider {
            base_url: env::var("N_LITELLM_BASE_URL")
                .unwrap_or_else(|_| "https://llm.7n.ai/v1/".to_string()),
            api_key: env::var("N_LITELLM_API_KEY")
                .ok()
                .or_else(|| env::var("LITELLM_API_KEY").ok()),
        },
    );
    providers.insert(
        "turbofieldfare".to_string(),
        LocalProvider {
            base_url: env::var("N_TURBOFIELDFARE_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8080/v1/".to_string()),
            api_key: env::var("N_TURBOFIELDFARE_API_KEY").ok(),
        },
    );
    providers
}

/// The provider prefixes Vue may pick from — desktop returns the
/// `build_local_providers()` keys (sorted for a stable UI order), mobile
/// returns an empty list until LiteRT-LM is wired up.
#[tauri::command]
pub fn llm_providers() -> Vec<String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let mut names: Vec<String> = build_local_providers().into_keys().collect();
        names.sort();
        names
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        Vec::new()
    }
}

/// Loaded model ids for `provider`, via `GET {baseUrl}/models` (generic
/// OpenAI shape `{ data: [{ id }] }`). Empty on any failure — mirrors the
/// old `listOmlxModels()` JS helper's degrade-gracefully contract.
#[tauri::command]
pub async fn llm_list_models(provider: String) -> Result<Vec<String>, String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let providers = build_local_providers();
        let config = providers
            .get(&provider)
            .ok_or_else(|| format!("невідомий провайдер {provider:?}"))?;
        let url = format!("{}models", config.base_url);
        let client = reqwest::Client::new();
        let mut request = client.get(&url);
        if let Some(key) = &config.api_key {
            request = request.bearer_auth(key);
        }
        let Ok(response) = request.send().await else {
            return Ok(Vec::new());
        };
        if !response.status().is_success() {
            return Ok(Vec::new());
        }
        let Ok(data) = response.json::<serde_json::Value>().await else {
            return Ok(Vec::new());
        };
        Ok(data
            .get("data")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|m| m.get("id").and_then(|i| i.as_str()).map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default())
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = provider;
        Ok(Vec::new())
    }
}

/// One-shot chat call for the summarize/ask/pattern/newsletter-render/
/// call-analysis composables (always system+single-user, never tools —
/// `LocalCloud::one_shot_with_spec` fits exactly, no genai `ChatRequest`
/// plumbing needed in Vue). `model_spec` is `"provider/model-id"`, e.g.
/// `"omlx/gemma-4-e4b"` or `"turbofieldfare/gemma-4-26b-a4b-it"`.
#[tauri::command]
pub async fn llm_chat(model_spec: String, system: Option<String>, user: String) -> Result<String, String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let cascade = llm_lib::LocalCloud::new(build_local_providers());
        cascade
            .one_shot_with_spec(&model_spec, system.as_deref(), &user)
            .await
            .map_err(|e| e.to_string())
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = (model_spec, system, user);
        Err(LITERT_NOT_IMPLEMENTED.to_string())
    }
}
