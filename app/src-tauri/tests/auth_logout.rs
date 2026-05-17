use std::sync::{Arc, Mutex};
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri::Manager;

use mlmail_lib::auth::auth_logout;
use mlmail_lib::auth::state::AuthState;
use mlmail_lib::auth::storage::in_memory::InMemoryStorage;
use mlmail_lib::auth::storage::SharedStorage;

fn make_app(storage: SharedStorage) -> tauri::App<tauri::test::MockRuntime> {
    let app = mock_builder()
        .build(mock_context(noop_assets()))
        .expect("failed to build mock app");
    app.manage(Mutex::new(AuthState::default()));
    app.manage(storage);
    app
}

#[tokio::test]
async fn auth_logout_clears_storage_and_resets_state() {
    let storage: SharedStorage = Arc::new(InMemoryStorage::new());
    storage.save("user@example.com", "rt-1").unwrap();

    let app = make_app(storage.clone());
    {
        let state = app.state::<Mutex<AuthState>>();
        let mut s = state.lock().unwrap();
        s.email = Some("user@example.com".into());
        s.access_token = Some("AT".into());
    }

    auth_logout(app.state(), app.state()).await.unwrap();

    assert_eq!(storage.load().unwrap(), None);
    let state = app.state::<Mutex<AuthState>>();
    let s = state.lock().unwrap();
    assert!(s.email.is_none());
    assert!(s.access_token.is_none());
    assert!(s.access_token_expires_at.is_none());
}

#[tokio::test]
async fn auth_logout_is_noop_when_already_logged_out() {
    let storage: SharedStorage = Arc::new(InMemoryStorage::new());
    let app = make_app(storage.clone());

    auth_logout(app.state(), app.state()).await.unwrap();

    assert_eq!(storage.load().unwrap(), None);
}
