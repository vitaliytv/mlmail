use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri::Manager;

use mlmail_lib::auth::state::AuthState;
use mlmail_lib::auth::storage::in_memory::InMemoryStorage;
use mlmail_lib::auth::storage::SharedStorage;
use mlmail_lib::endpoints::Endpoints;
use mlmail_lib::gmail::error::GmailError;
use mlmail_lib::gmail::{gmail_inbox_count, gmail_random_message};

fn make_app(endpoints: Endpoints, access_token: &str) -> tauri::App<tauri::test::MockRuntime> {
    let app = mock_builder()
        .build(mock_context(noop_assets()))
        .expect("failed to build mock app");
    let state = AuthState {
        email: Some("u@example.com".into()),
        access_token: Some(access_token.into()),
        access_token_expires_at: Some(Instant::now() + Duration::from_secs(3600)),
    };
    app.manage(Mutex::new(state));
    app.manage(endpoints);
    let storage: SharedStorage = Arc::new(InMemoryStorage::new());
    app.manage(storage);
    app
}

#[tokio::test]
async fn gmail_inbox_count_returns_count_from_gmail_endpoint() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/labels/INBOX")
        .match_header("authorization", "Bearer AT-fresh")
        .with_status(200)
        .with_body(r#"{"messagesTotal":42}"#)
        .create_async()
        .await;

    let endpoints = Endpoints {
        google_token: format!("{}/token", server.url()),
        gmail_label_inbox: format!("{}/labels/INBOX", server.url()),
        gmail_messages_list: format!("{}/messages", server.url()),
    };
    let app = make_app(endpoints, "AT-fresh");

    let n = gmail_inbox_count(app.state(), app.state(), app.state())
        .await
        .unwrap();
    assert_eq!(n, 42);
}

#[tokio::test]
async fn gmail_inbox_count_maps_401_to_reauth() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/labels/INBOX")
        .with_status(401)
        .with_body("nope")
        .create_async()
        .await;

    let endpoints = Endpoints {
        google_token: format!("{}/token", server.url()),
        gmail_label_inbox: format!("{}/labels/INBOX", server.url()),
        gmail_messages_list: format!("{}/messages", server.url()),
    };
    let app = make_app(endpoints, "AT-fresh");

    let err = gmail_inbox_count(app.state(), app.state(), app.state())
        .await
        .unwrap_err();
    assert!(matches!(err, GmailError::ReauthRequired));
}

#[tokio::test]
async fn gmail_random_message_returns_message_when_inbox_has_entries() {
    let mut server = mockito::Server::new_async().await;
    let body_data = URL_SAFE_NO_PAD.encode(b"hello");
    server
        .mock("GET", "/messages")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("labelIds".into(), "INBOX".into()),
            mockito::Matcher::UrlEncoded("maxResults".into(), "100".into()),
            mockito::Matcher::UrlEncoded("fields".into(), "messages/id".into()),
        ]))
        .with_status(200)
        .with_body(r#"{"messages":[{"id":"m-only"}]}"#)
        .create_async()
        .await;
    server
        .mock("GET", "/messages/m-only")
        .match_query(mockito::Matcher::UrlEncoded(
            "format".into(),
            "full".into(),
        ))
        .with_status(200)
        .with_body(
            serde_json::json!({
                "id": "m-only",
                "payload": {
                    "mimeType": "text/plain",
                    "headers": [
                        {"name": "From", "value": "alice@example.com"},
                        {"name": "Subject", "value": "Hi"},
                        {"name": "Date", "value": "Mon, 16 May 2026 10:00:00 +0300"}
                    ],
                    "body": {"data": body_data}
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let endpoints = Endpoints {
        google_token: format!("{}/token", server.url()),
        gmail_label_inbox: format!("{}/labels/INBOX", server.url()),
        gmail_messages_list: format!("{}/messages", server.url()),
    };
    let app = make_app(endpoints, "AT-fresh");

    let msg = gmail_random_message(app.state(), app.state(), app.state())
        .await
        .unwrap();
    assert_eq!(msg.id, "m-only");
    assert_eq!(msg.from, "alice@example.com");
    assert_eq!(msg.subject, "Hi");
    assert_eq!(msg.body, "hello");
}

#[tokio::test]
async fn gmail_random_message_returns_empty_when_inbox_is_empty() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/messages")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_body(r#"{"resultSizeEstimate":0}"#)
        .create_async()
        .await;

    let endpoints = Endpoints {
        google_token: format!("{}/token", server.url()),
        gmail_label_inbox: format!("{}/labels/INBOX", server.url()),
        gmail_messages_list: format!("{}/messages", server.url()),
    };
    let app = make_app(endpoints, "AT-fresh");

    let err = gmail_random_message(app.state(), app.state(), app.state())
        .await
        .unwrap_err();
    assert!(matches!(err, GmailError::Empty));
}
