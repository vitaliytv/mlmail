use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use std::sync::{Arc, Mutex};

use mlmail_lib::auth::error::AuthError;
use mlmail_lib::auth::finalize_login;
use mlmail_lib::auth::state::AuthState;
use mlmail_lib::auth::storage::in_memory::InMemoryStorage;
use mlmail_lib::auth::storage::RefreshTokenStorage;
use mlmail_lib::auth::token_exchange::TokenResponse;

fn build_jwt(payload_json: &str) -> String {
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"RS256","typ":"JWT"}"#);
    let payload = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(b"fake-signature");
    format!("{header}.{payload}.{signature}")
}

#[test]
fn finalize_login_saves_refresh_and_updates_state_on_success() {
    let storage = Arc::new(InMemoryStorage::new());
    let state = Mutex::new(AuthState::default());

    let resp = TokenResponse {
        access_token: "AT-1".into(),
        expires_in: 3600,
        refresh_token: Some("RT-1".into()),
        id_token: Some(build_jwt(r#"{"email":"user@example.com"}"#)),
    };

    let session = finalize_login(resp, storage.as_ref(), &state).unwrap();
    assert_eq!(session.email, "user@example.com");

    let stored = storage.load().unwrap().unwrap();
    assert_eq!(stored.email, "user@example.com");
    assert_eq!(stored.refresh_token, "RT-1");

    let s = state.lock().unwrap();
    assert_eq!(s.email.as_deref(), Some("user@example.com"));
    assert_eq!(s.access_token.as_deref(), Some("AT-1"));
    assert!(s.access_token_expires_at.is_some());
}

#[test]
fn finalize_login_returns_oauth_error_when_id_token_lacks_email() {
    let storage = Arc::new(InMemoryStorage::new());
    let state = Mutex::new(AuthState::default());

    let resp = TokenResponse {
        access_token: "AT".into(),
        expires_in: 3600,
        refresh_token: Some("RT".into()),
        id_token: Some(build_jwt(r#"{"sub":"123"}"#)),
    };

    let err = finalize_login(resp, storage.as_ref(), &state).unwrap_err();
    assert!(matches!(err, AuthError::OAuth(_)));
    assert_eq!(storage.load().unwrap(), None);
    assert!(state.lock().unwrap().email.is_none());
}

#[test]
fn finalize_login_returns_oauth_error_when_id_token_missing() {
    let storage = Arc::new(InMemoryStorage::new());
    let state = Mutex::new(AuthState::default());

    let resp = TokenResponse {
        access_token: "AT".into(),
        expires_in: 3600,
        refresh_token: Some("RT".into()),
        id_token: None,
    };

    let err = finalize_login(resp, storage.as_ref(), &state).unwrap_err();
    assert!(matches!(err, AuthError::OAuth(_)));
    assert_eq!(storage.load().unwrap(), None);
}

#[test]
fn finalize_login_returns_oauth_error_when_refresh_token_missing() {
    let storage = Arc::new(InMemoryStorage::new());
    let state = Mutex::new(AuthState::default());

    let resp = TokenResponse {
        access_token: "AT".into(),
        expires_in: 3600,
        refresh_token: None,
        id_token: Some(build_jwt(r#"{"email":"user@example.com"}"#)),
    };

    let err = finalize_login(resp, storage.as_ref(), &state).unwrap_err();
    assert!(matches!(err, AuthError::OAuth(_)));
    assert_eq!(storage.load().unwrap(), None);
    assert!(state.lock().unwrap().email.is_none());
}
