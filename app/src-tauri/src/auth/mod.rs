pub mod config;
pub mod error;
pub mod flow;
pub mod id_token;
pub mod pkce;
pub mod state;
pub mod storage;
pub mod token_exchange;

use crate::auth::error::AuthError;
use crate::auth::state::AuthState;
use crate::auth::storage::{RefreshTokenStorage, StoredSession};
use crate::auth::token_exchange::TokenResponse;
use serde::Serialize;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, State};

#[derive(Serialize)]
pub struct AuthSession {
    pub email: String,
}

#[cfg(target_os = "macos")]
fn make_storage(_app: &AppHandle) -> Box<dyn RefreshTokenStorage> {
    storage::platform_storage()
}

#[cfg(target_os = "android")]
fn make_storage(app: &AppHandle) -> Box<dyn RefreshTokenStorage> {
    storage::platform_storage(app)
}

#[cfg(not(any(target_os = "macos", target_os = "android")))]
fn make_storage(_app: &AppHandle) -> Box<dyn RefreshTokenStorage> {
    // Other platforms unsupported; in-memory keeps the binary buildable for tooling.
    Box::new(storage::in_memory::InMemoryStorage::new())
}

pub fn on_startup(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let storage = make_storage(app);
    let loaded = storage.load().map_err(|e| AuthError::from(e).to_string())?;
    if let Some(session) = loaded {
        let state: State<'_, Mutex<AuthState>> = app.state();
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.email = Some(session.email);
    }
    Ok(())
}

fn apply_token_response(state: &mut AuthState, resp: &TokenResponse, email: Option<String>) {
    state.access_token = Some(resp.access_token.clone());
    state.access_token_expires_at = Some(Instant::now() + Duration::from_secs(resp.expires_in));
    if let Some(e) = email {
        state.email = Some(e);
    }
}

#[cfg(target_os = "macos")]
async fn run_login(app: &AppHandle) -> Result<TokenResponse, AuthError> {
    flow::macos::run_login_flow(app, config::desktop_client_id()).await
}

#[cfg(target_os = "android")]
async fn run_login(app: &AppHandle) -> Result<TokenResponse, AuthError> {
    flow::android::run_login_flow(app, config::android_web_client_id(), config::android_client_id()).await
}

#[cfg(not(any(target_os = "macos", target_os = "android")))]
async fn run_login(_app: &AppHandle) -> Result<TokenResponse, AuthError> {
    Err(AuthError::Platform("unsupported platform".into()))
}

#[cfg(target_os = "android")]
fn client_id_for_refresh() -> &'static str {
    config::android_client_id()
}

#[cfg(not(target_os = "android"))]
fn client_id_for_refresh() -> &'static str {
    config::desktop_client_id()
}

#[cfg(target_os = "android")]
fn client_secret_for_refresh() -> Option<&'static str> {
    None
}

#[cfg(not(target_os = "android"))]
fn client_secret_for_refresh() -> Option<&'static str> {
    Some(config::desktop_client_secret())
}

#[tauri::command]
pub async fn auth_start_login(
    app: AppHandle,
    state: State<'_, Mutex<AuthState>>,
) -> Result<AuthSession, AuthError> {
    let resp = run_login(&app).await?;

    let email = resp
        .id_token
        .as_deref()
        .and_then(id_token::extract_email)
        .ok_or_else(|| AuthError::OAuth("no email in id_token".into()))?;

    let refresh = resp
        .refresh_token
        .clone()
        .ok_or_else(|| AuthError::OAuth("no refresh_token in response".into()))?;

    let storage = make_storage(&app);
    storage.save(&email, &refresh)?;

    {
        let mut s = state.lock().map_err(|e| AuthError::Platform(e.to_string()))?;
        apply_token_response(&mut s, &resp, Some(email.clone()));
    }

    Ok(AuthSession { email })
}

pub async fn acquire_access_token(
    app: &AppHandle,
    state: &State<'_, Mutex<AuthState>>,
) -> Result<String, AuthError> {
    {
        let s = state.lock().map_err(|e| AuthError::Platform(e.to_string()))?;
        if s.is_access_token_fresh() {
            return Ok(s.access_token.clone().unwrap());
        }
    }

    let storage = make_storage(app);
    let stored = storage
        .load()?
        .ok_or(AuthError::ReauthRequired)?;

    let resp = token_exchange::exchange_refresh(
        client_id_for_refresh(),
        client_secret_for_refresh(),
        &stored.refresh_token,
    )
    .await;

    match resp {
        Ok(resp) => {
            if let Some(new_rt) = resp.refresh_token.as_deref() {
                storage.save(&stored.email, new_rt)?;
            }
            let token = resp.access_token.clone();
            let mut s = state.lock().map_err(|e| AuthError::Platform(e.to_string()))?;
            apply_token_response(&mut s, &resp, None);
            Ok(token)
        }
        Err(AuthError::ReauthRequired) => {
            let _ = storage.clear();
            if let Ok(mut s) = state.lock() {
                s.reset();
            }
            Err(AuthError::ReauthRequired)
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn auth_get_access_token(
    app: AppHandle,
    state: State<'_, Mutex<AuthState>>,
) -> Result<String, AuthError> {
    acquire_access_token(&app, &state).await
}

#[tauri::command]
pub fn auth_is_authenticated(state: State<'_, Mutex<AuthState>>) -> bool {
    state
        .lock()
        .map(|s| s.email.is_some())
        .unwrap_or(false)
}

#[tauri::command]
pub fn auth_current_email(state: State<'_, Mutex<AuthState>>) -> Option<String> {
    state.lock().ok().and_then(|s| s.email.clone())
}

#[tauri::command]
pub async fn auth_logout(
    app: AppHandle,
    state: State<'_, Mutex<AuthState>>,
) -> Result<(), AuthError> {
    let storage = make_storage(&app);
    storage.clear()?;
    let mut s = state.lock().map_err(|e| AuthError::Platform(e.to_string()))?;
    s.reset();
    Ok(())
}

pub fn collect_session(app: &AppHandle) -> Result<Option<StoredSession>, AuthError> {
    let storage = make_storage(app);
    Ok(storage.load()?)
}

#[tauri::command]
pub async fn get_inbox_count(
    app: AppHandle,
    state: State<'_, Mutex<AuthState>>,
) -> Result<u32, AuthError> {
    let access_token = auth_get_access_token(app, state).await?;
    let resp = reqwest::Client::new()
        .get("https://gmail.googleapis.com/gmail/v1/users/me/labels/INBOX")
        .bearer_auth(&access_token)
        .send()
        .await?;
    let json: serde_json::Value = resp.json().await?;
    Ok(json["messagesTotal"].as_u64().unwrap_or(0) as u32)
}
