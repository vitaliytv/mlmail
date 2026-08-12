//! OpenAI-compatible local LLM access for the desktop app.
//!
//! The frontend supplies an explicit endpoint for every request. A developer
//! may provide the initial endpoint through `N_LOCAL_OPENAI_BASE_URL`, while
//! the settings dialog stores the chosen URL and model locally. API keys stay
//! in memory and are never written by this module.

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use llm_lib::local_cloud::LocalProvider;
use serde::Serialize;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::collections::HashMap;

/// `LiteRT-LM ще не реалізовано` — shared error text for the mobile stubs
/// below, so every command fails identically until the JNI bridge lands.
#[cfg(any(target_os = "android", target_os = "ios"))]
const LITERT_NOT_IMPLEMENTED: &str = "LiteRT-LM ще не реалізовано на цій платформі";

/// The generic provider name used in every desktop model specification.
const LOCAL_OPENAI_PROVIDER: &str = "local-openai";

/// Initial non-secret desktop configuration exposed to the settings dialog.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmDefaultConfig {
    base_url: Option<String>,
}

/// Normalise and validate an OpenAI-compatible API root.
///
/// Only HTTP(S) roots ending at `/v1/` are accepted. Keeping this validation
/// at the native boundary prevents malformed URLs from being passed into the
/// shared HTTP client even when a caller bypasses the settings UI.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn normalize_base_url(raw: &str) -> Result<String, String> {
    let mut url = reqwest::Url::parse(raw.trim()).map_err(|_| {
        "Некоректна адреса LLM. Вкажіть повний URL на кшталт http://127.0.0.1:8080/v1/.".to_string()
    })?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("Адреса LLM має використовувати http:// або https://.".to_string());
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err("Адреса LLM не повинна містити query або fragment.".to_string());
    }
    let path = url.path().trim_end_matches('/');
    if path != "/v1" {
        return Err("Адреса LLM має завершуватися на /v1/.".to_string());
    }
    url.set_path("/v1/");
    Ok(url.to_string())
}

/// Build the one explicitly configured local provider for a request.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn build_local_providers(
    base_url: Option<String>,
    api_key: Option<String>,
) -> Result<HashMap<String, LocalProvider>, String> {
    let configured_url = base_url
        .filter(|value| !value.trim().is_empty())
        .or_else(|| std::env::var("N_LOCAL_OPENAI_BASE_URL").ok())
        .ok_or_else(|| "LLM не налаштовано. Вкажіть адресу сервера в Налаштуваннях LLM або задайте N_LOCAL_OPENAI_BASE_URL.".to_string())?;
    let mut providers = HashMap::new();
    providers.insert(
        LOCAL_OPENAI_PROVIDER.to_string(),
        LocalProvider {
            base_url: normalize_base_url(&configured_url)?,
            api_key: api_key
                .filter(|value| !value.is_empty())
                .or_else(|| std::env::var("N_LOCAL_OPENAI_API_KEY").ok()),
        },
    );
    Ok(providers)
}

/// Return the optional endpoint inherited by a freshly started desktop app.
/// The API key intentionally never crosses the Tauri boundary.
#[tauri::command]
pub fn llm_default_config() -> LlmDefaultConfig {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let base_url = std::env::var("N_LOCAL_OPENAI_BASE_URL")
            .ok()
            .and_then(|url| normalize_base_url(&url).ok());
        LlmDefaultConfig { base_url }
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        LlmDefaultConfig { base_url: None }
    }
}

/// The only provider prefix accepted by the desktop local-LLM surface.
#[tauri::command]
pub fn llm_providers() -> Vec<String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        vec![LOCAL_OPENAI_PROVIDER.to_string()]
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        Vec::new()
    }
}

/// Load model ids from an explicit OpenAI-compatible endpoint.
#[tauri::command]
pub async fn llm_list_models(
    base_url: Option<String>,
    api_key: Option<String>,
) -> Result<Vec<String>, String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let providers = build_local_providers(base_url, api_key)?;
        let config = providers
            .get(LOCAL_OPENAI_PROVIDER)
            .expect("local-openai provider is always registered");
        let url = format!("{}models", config.base_url);
        let client = reqwest::Client::new();
        let mut request = client.get(&url);
        if let Some(key) = &config.api_key {
            request = request.bearer_auth(key);
        }
        let response = request
            .send()
            .await
            .map_err(|error| format!("Не вдалося з’єднатися з LLM: {error}"))?
            .error_for_status()
            .map_err(|error| format!("LLM не прийняла запит моделей: {error}"))?;
        let data = response
            .json::<serde_json::Value>()
            .await
            .map_err(|error| format!("LLM повернула некоректний список моделей: {error}"))?;
        Ok(data
            .get("data")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|model| {
                        model
                            .get("id")
                            .and_then(|id| id.as_str())
                            .map(str::to_owned)
                    })
                    .collect()
            })
            .unwrap_or_default())
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = (base_url, api_key);
        Err(LITERT_NOT_IMPLEMENTED.to_string())
    }
}

/// Send one system-plus-user request to the explicitly configured local LLM.
#[tauri::command]
pub async fn llm_chat(
    model_spec: String,
    system: Option<String>,
    user: String,
    base_url: Option<String>,
    api_key: Option<String>,
) -> Result<String, String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let cascade = llm_lib::LocalCloud::new(build_local_providers(base_url, api_key)?);
        cascade
            .one_shot_with_spec(&model_spec, system.as_deref(), &user)
            .await
            .map_err(|error| error.to_string())
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = (model_spec, system, user, base_url, api_key);
        Err(LITERT_NOT_IMPLEMENTED.to_string())
    }
}

#[cfg(all(test, not(any(target_os = "android", target_os = "ios"))))]
mod tests {
    use super::normalize_base_url;

    #[test]
    fn normalizes_v1_url() {
        assert_eq!(
            normalize_base_url("http://127.0.0.1:8080/v1").unwrap(),
            "http://127.0.0.1:8080/v1/"
        );
    }

    #[test]
    fn rejects_non_v1_url() {
        assert!(normalize_base_url("http://127.0.0.1:8080").is_err());
    }
}
