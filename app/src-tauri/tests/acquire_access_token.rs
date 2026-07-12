use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use mlmail_lib::auth::acquire_access_token;
use mlmail_lib::auth::error::AuthError;
use mlmail_lib::auth::state::AuthState;
use mlmail_lib::auth::storage::in_memory::InMemoryStorage;
use mlmail_lib::auth::storage::RefreshTokenStorage;

#[tokio::test]
async fn returns_existing_token_without_network_when_state_is_fresh() {
    let storage = Arc::new(InMemoryStorage::new());
    let state = Mutex::new(AuthState {
        email: Some("u@x".into()),
        access_token: Some("AT-cached".into()),
        access_token_expires_at: Some(Instant::now() + Duration::from_secs(3600)),
    });

    let token = acquire_access_token("http://localhost:1", storage.as_ref(), &state)
        .await
        .unwrap();
    assert_eq!(token, "AT-cached");
}

#[tokio::test]
async fn refreshes_token_when_state_is_stale_using_stored_refresh_token() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/token")
        .with_status(200)
        .with_body(r#"{"access_token":"AT-new","expires_in":3600}"#)
        .create_async()
        .await;

    let storage = Arc::new(InMemoryStorage::new());
    storage.save("u@x", "RT-stored").unwrap();
    let state = Mutex::new(AuthState::default());

    let token = acquire_access_token(&format!("{}/token", server.url()), storage.as_ref(), &state)
        .await
        .unwrap();
    assert_eq!(token, "AT-new");

    let s = state.lock().unwrap();
    assert_eq!(s.access_token.as_deref(), Some("AT-new"));
}

#[tokio::test]
async fn rotates_refresh_token_when_response_includes_one() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/token")
        .with_status(200)
        .with_body(r#"{"access_token":"AT-new","expires_in":3600,"refresh_token":"RT-rotated"}"#)
        .create_async()
        .await;

    let storage = Arc::new(InMemoryStorage::new());
    storage.save("u@x", "RT-old").unwrap();
    let state = Mutex::new(AuthState::default());

    acquire_access_token(&format!("{}/token", server.url()), storage.as_ref(), &state)
        .await
        .unwrap();

    assert_eq!(storage.load().unwrap().unwrap().refresh_token, "RT-rotated");
}

#[tokio::test]
async fn returns_reauth_required_and_clears_storage_on_invalid_grant() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/token")
        .with_status(400)
        .with_body(r#"{"error":"invalid_grant"}"#)
        .create_async()
        .await;

    let storage = Arc::new(InMemoryStorage::new());
    storage.save("u@x", "RT-revoked").unwrap();
    let state = Mutex::new(AuthState {
        email: Some("u@x".into()),
        ..Default::default()
    });

    let err = acquire_access_token(&format!("{}/token", server.url()), storage.as_ref(), &state)
        .await
        .unwrap_err();
    assert!(matches!(err, AuthError::ReauthRequired));

    assert_eq!(storage.load().unwrap(), None);
    assert!(state.lock().unwrap().email.is_none());
}

#[tokio::test]
async fn returns_reauth_required_when_storage_has_no_session() {
    let storage = Arc::new(InMemoryStorage::new());
    let state = Mutex::new(AuthState::default());

    let err = acquire_access_token("http://localhost:1", storage.as_ref(), &state)
        .await
        .unwrap_err();
    assert!(matches!(err, AuthError::ReauthRequired));
}
