use crate::auth::error::AuthError;
use crate::auth::storage::StoredSession;
use crate::auth::token_exchange::{self, FlowKind, TokenResponse};
use serde::{Deserialize, Serialize};
use tauri::plugin::PluginHandle;
use tauri::AppHandle;

const PLUGIN_NAME: &str = "mlmail-auth";
const ANDROID_REDIRECT_URI: &str = "";

#[derive(Serialize)]
struct SignInArgs {
    web_client_id: String,
    scopes: Vec<String>,
}

#[derive(Deserialize)]
struct SignInResult {
    server_auth_code: Option<String>,
    id_token: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct SaveSessionArgs {
    email: String,
    refresh_token: String,
}

#[derive(Deserialize)]
struct LoadSessionResult {
    email: Option<String>,
    refresh_token: Option<String>,
}

pub async fn run_login_flow(
    app: &AppHandle,
    web_client_id: &str,
    android_client_id: &str,
) -> Result<TokenResponse, AuthError> {
    let handle = plugin_handle(app)?;
    let scopes = vec![
        "openid".into(),
        "email".into(),
        "https://www.googleapis.com/auth/gmail.modify".into(),
        "https://www.googleapis.com/auth/gmail.settings.basic".into(),
    ];
    let args = SignInArgs {
        web_client_id: web_client_id.to_string(),
        scopes,
    };

    let result: SignInResult = handle
        .run_mobile_plugin("signInAndAuthorize", args)
        .map_err(|e| AuthError::Platform(format!("call signInAndAuthorize: {e}")))?;

    if let Some(err) = result.error {
        if err == "Cancelled" {
            return Err(AuthError::Cancelled);
        }
        return Err(AuthError::Platform(err));
    }

    let server_auth_code = result
        .server_auth_code
        .ok_or_else(|| AuthError::OAuth("missing server_auth_code".into()))?;

    token_exchange::exchange_code(
        android_client_id,
        None,
        &server_auth_code,
        "",
        ANDROID_REDIRECT_URI,
        FlowKind::Android,
    )
    .await
}

pub(crate) fn call_save_session(
    app: &AppHandle,
    email: &str,
    refresh_token: &str,
) -> Result<(), String> {
    let handle = plugin_handle(app).map_err(|e| e.to_string())?;
    handle
        .run_mobile_plugin::<()>(
            "saveSession",
            SaveSessionArgs {
                email: email.into(),
                refresh_token: refresh_token.into(),
            },
        )
        .map_err(|e| e.to_string())
}

pub(crate) fn call_load_session(app: &AppHandle) -> Result<Option<StoredSession>, String> {
    let handle = plugin_handle(app).map_err(|e| e.to_string())?;
    let res: LoadSessionResult = handle
        .run_mobile_plugin("loadSession", ())
        .map_err(|e| e.to_string())?;

    match (res.email, res.refresh_token) {
        (Some(email), Some(refresh_token)) => Ok(Some(StoredSession {
            email,
            refresh_token,
        })),
        _ => Ok(None),
    }
}

pub(crate) fn call_clear_session(app: &AppHandle) -> Result<(), String> {
    let handle = plugin_handle(app).map_err(|e| e.to_string())?;
    handle
        .run_mobile_plugin::<()>("clearSession", ())
        .map_err(|e| e.to_string())
}

fn plugin_handle(app: &AppHandle) -> Result<PluginHandle<tauri::Wry>, AuthError> {
    // The plugin is registered via `init_mobile_plugin` in lib.rs setup and the
    // resulting handle is stored in Tauri-managed state. Here we just retrieve it.
    use tauri::Manager;
    let cell: tauri::State<'_, std::sync::Mutex<Option<PluginHandle<tauri::Wry>>>> = app.state();
    let guard = cell
        .lock()
        .map_err(|e| AuthError::Platform(format!("plugin handle lock: {e}")))?;
    guard
        .clone()
        .ok_or_else(|| AuthError::Platform("mobile auth plugin not initialised".into()))
}
