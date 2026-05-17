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
use crate::auth::storage::{RefreshTokenStorage, SharedStorage, StoredSession};
use crate::auth::token_exchange::TokenResponse;
use crate::endpoints::Endpoints;
use serde::Serialize;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, State};

#[derive(Debug, Serialize)]
pub struct AuthSession {
    pub email: String,
}

#[cfg(target_os = "macos")]
pub fn make_storage(_app: &AppHandle) -> SharedStorage {
    storage::platform_storage()
}

#[cfg(target_os = "android")]
pub fn make_storage(app: &AppHandle) -> SharedStorage {
    storage::platform_storage(app)
}

#[cfg(not(any(target_os = "macos", target_os = "android")))]
pub fn make_storage(_app: &AppHandle) -> SharedStorage {
    use std::sync::Arc;
    // Other platforms unsupported; in-memory keeps the binary buildable for tooling.
    Arc::new(storage::in_memory::InMemoryStorage::new())
}

pub fn on_startup(
    app: &AppHandle,
    storage: &dyn RefreshTokenStorage,
) -> Result<(), Box<dyn std::error::Error>> {
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

pub fn finalize_login(
    resp: TokenResponse,
    storage: &dyn RefreshTokenStorage,
    state: &Mutex<AuthState>,
) -> Result<AuthSession, AuthError> {
    let email = resp
        .id_token
        .as_deref()
        .and_then(id_token::extract_email)
        .ok_or_else(|| AuthError::OAuth("no email in id_token".into()))?;

    let refresh = resp
        .refresh_token
        .clone()
        .ok_or_else(|| AuthError::OAuth("no refresh_token in response".into()))?;

    storage.save(&email, &refresh)?;

    {
        let mut s = state.lock().map_err(|e| AuthError::Platform(e.to_string()))?;
        apply_token_response(&mut s, &resp, Some(email.clone()));
    }

    Ok(AuthSession { email })
}

#[tauri::command]
pub async fn auth_start_login(
    app: AppHandle,
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<AuthSession, AuthError> {
    let resp = run_login(&app).await?;
    finalize_login(resp, storage.inner().as_ref(), state.inner())
}

pub async fn acquire_access_token(
    token_endpoint: &str,
    storage: &dyn RefreshTokenStorage,
    state: &Mutex<AuthState>,
) -> Result<String, AuthError> {
    {
        let s = state.lock().map_err(|e| AuthError::Platform(e.to_string()))?;
        if s.is_access_token_fresh() {
            return Ok(s.access_token.clone().unwrap());
        }
    }

    let stored = storage
        .load()?
        .ok_or(AuthError::ReauthRequired)?;

    let resp = token_exchange::exchange_refresh_at(
        token_endpoint,
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
    endpoints: State<'_, Endpoints>,
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<String, AuthError> {
    acquire_access_token(&endpoints.google_token, storage.inner().as_ref(), state.inner()).await
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
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<(), AuthError> {
    storage.clear()?;
    let mut s = state.lock().map_err(|e| AuthError::Platform(e.to_string()))?;
    s.reset();
    Ok(())
}

pub fn collect_session(storage: &dyn RefreshTokenStorage) -> Result<Option<StoredSession>, AuthError> {
    Ok(storage.load()?)
}
