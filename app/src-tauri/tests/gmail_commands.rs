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
use mlmail_lib::gmail::{
    gmail_create_filter, gmail_inbox_count, gmail_random_message, gmail_search, gmail_trash_query,
};

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
        gmail_batch_modify: format!("{}/batchModify", server.url()),
        gmail_filters: format!("{}/filters", server.url()),
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
        gmail_batch_modify: format!("{}/batchModify", server.url()),
        gmail_filters: format!("{}/filters", server.url()),
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
        gmail_batch_modify: format!("{}/batchModify", server.url()),
        gmail_filters: format!("{}/filters", server.url()),
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
        gmail_batch_modify: format!("{}/batchModify", server.url()),
        gmail_filters: format!("{}/filters", server.url()),
    };
    let app = make_app(endpoints, "AT-fresh");

    let err = gmail_random_message(app.state(), app.state(), app.state())
        .await
        .unwrap_err();
    assert!(matches!(err, GmailError::Empty));
}

#[tokio::test]
async fn gmail_search_passes_q_and_returns_summaries() {
    // Guards the UI→Rust contract: the `search` tool forwards its input key 1:1
    // into invoke('gmail_search', { q }), so the command's first arg is `q`.
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/messages")
        .match_query(mockito::Matcher::UrlEncoded("q".into(), "from:bob".into()))
        .with_status(200)
        .with_body(r#"{"messages":[{"id":"s1"}]}"#)
        .create_async()
        .await;
    server
        .mock("GET", "/messages/s1")
        .match_query(mockito::Matcher::UrlEncoded(
            "format".into(),
            "metadata".into(),
        ))
        .with_status(200)
        .with_body(
            serde_json::json!({
                "id": "s1",
                "payload": {
                    "headers": [
                        {"name": "From", "value": "bob@example.com"},
                        {"name": "Subject", "value": "Lunch"},
                        {"name": "Date", "value": "Mon, 16 May 2026 10:00:00 +0300"}
                    ]
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
        gmail_batch_modify: format!("{}/batchModify", server.url()),
        gmail_filters: format!("{}/filters", server.url()),
    };
    let app = make_app(endpoints, "AT-fresh");

    let out = gmail_search("from:bob".into(), app.state(), app.state(), app.state())
        .await
        .unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id, "s1");
    assert_eq!(out[0].from, "bob@example.com");
    assert_eq!(out[0].subject, "Lunch");
}

#[tokio::test]
async fn gmail_trash_query_trashes_all_matches_and_returns_count() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/messages")
        .match_query(mockito::Matcher::UrlEncoded("q".into(), "from:npm".into()))
        .with_status(200)
        .with_body(r#"{"messages":[{"id":"a"},{"id":"b"}]}"#)
        .create_async()
        .await;
    let batch = server
        .mock("POST", "/batchModify")
        .match_body(mockito::Matcher::PartialJsonString(
            r#"{"ids":["a","b"],"addLabelIds":["TRASH"],"removeLabelIds":["INBOX"]}"#.into(),
        ))
        .with_status(204)
        .create_async()
        .await;

    let endpoints = Endpoints {
        google_token: format!("{}/token", server.url()),
        gmail_label_inbox: format!("{}/labels/INBOX", server.url()),
        gmail_messages_list: format!("{}/messages", server.url()),
        gmail_batch_modify: format!("{}/batchModify", server.url()),
        gmail_filters: format!("{}/filters", server.url()),
    };
    let app = make_app(endpoints, "AT-fresh");

    let res = gmail_trash_query("from:npm".into(), app.state(), app.state(), app.state())
        .await
        .unwrap();
    assert_eq!(res.trashed, 2);
    batch.assert_async().await;
}

#[tokio::test]
async fn gmail_trash_query_rejects_empty_query() {
    let server = mockito::Server::new_async().await;
    let endpoints = Endpoints {
        google_token: format!("{}/token", server.url()),
        gmail_label_inbox: format!("{}/labels/INBOX", server.url()),
        gmail_messages_list: format!("{}/messages", server.url()),
        gmail_batch_modify: format!("{}/batchModify", server.url()),
        gmail_filters: format!("{}/filters", server.url()),
    };
    let app = make_app(endpoints, "AT-fresh");

    let err = gmail_trash_query("   ".into(), app.state(), app.state(), app.state())
        .await
        .unwrap_err();
    assert!(matches!(err, GmailError::EmptyQuery));
}

#[tokio::test]
async fn gmail_create_filter_posts_criteria_and_returns_id() {
    let mut server = mockito::Server::new_async().await;
    let filter = server
        .mock("POST", "/filters")
        .match_body(mockito::Matcher::AllOf(vec![
            mockito::Matcher::PartialJsonString(
                r#"{"criteria":{"from":"support@npmjs.com","subject":"Successfully published"}}"#
                    .into(),
            ),
            mockito::Matcher::PartialJsonString(
                r#"{"action":{"addLabelIds":["TRASH"],"removeLabelIds":["INBOX"]}}"#.into(),
            ),
        ]))
        .with_status(200)
        .with_body(r#"{"id":"FILTER_42"}"#)
        .create_async()
        .await;

    let endpoints = Endpoints {
        google_token: format!("{}/token", server.url()),
        gmail_label_inbox: format!("{}/labels/INBOX", server.url()),
        gmail_messages_list: format!("{}/messages", server.url()),
        gmail_batch_modify: format!("{}/batchModify", server.url()),
        gmail_filters: format!("{}/filters", server.url()),
    };
    let app = make_app(endpoints, "AT-fresh");

    let res = gmail_create_filter(
        "support@npmjs.com".into(),
        "Successfully published".into(),
        app.state(),
        app.state(),
        app.state(),
    )
    .await
    .unwrap();
    assert_eq!(res.id, "FILTER_42");
    filter.assert_async().await;
}
