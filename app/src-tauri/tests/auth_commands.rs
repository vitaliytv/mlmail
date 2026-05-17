use std::sync::Mutex;
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri::Manager;

use mlmail_lib::auth::{auth_current_email, auth_is_authenticated, state::AuthState};

fn make_app() -> tauri::App<tauri::test::MockRuntime> {
    mock_builder()
        .build(mock_context(noop_assets()))
        .expect("failed to build mock app")
}

#[test]
fn auth_is_authenticated_returns_false_when_no_email() {
    let app = make_app();
    app.manage(Mutex::new(AuthState::default()));

    assert!(!auth_is_authenticated(app.state()));
}

#[test]
fn auth_is_authenticated_returns_true_when_email_set() {
    let app = make_app();
    app.manage(Mutex::new(AuthState {
        email: Some("user@example.com".into()),
        ..Default::default()
    }));

    assert!(auth_is_authenticated(app.state()));
}

#[test]
fn auth_current_email_returns_none_when_not_signed_in() {
    let app = make_app();
    app.manage(Mutex::new(AuthState::default()));

    assert_eq!(auth_current_email(app.state()), None);
}

#[test]
fn auth_current_email_returns_email_when_signed_in() {
    let app = make_app();
    app.manage(Mutex::new(AuthState {
        email: Some("vitaliytv@nitralabs.com".into()),
        ..Default::default()
    }));

    assert_eq!(
        auth_current_email(app.state()),
        Some("vitaliytv@nitralabs.com".into())
    );
}
