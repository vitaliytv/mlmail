pub mod error;
pub mod message;

use crate::auth::{self, state::AuthState, storage::SharedStorage};
use crate::endpoints::Endpoints;
use crate::gmail::error::GmailError;
use serde::Deserialize;
use std::sync::Mutex;
use tauri::State;

pub const GMAIL_LABEL_INBOX_URL: &str =
    "https://gmail.googleapis.com/gmail/v1/users/me/labels/INBOX";

#[derive(Deserialize)]
struct LabelResponse {
    #[serde(rename = "messagesTotal")]
    messages_total: Option<u64>,
}

pub fn parse_messages_total(body: &str) -> Result<u64, GmailError> {
    let v: LabelResponse =
        serde_json::from_str(body).map_err(|e| GmailError::Parse(e.to_string()))?;
    v.messages_total
        .ok_or_else(|| GmailError::Parse("messagesTotal missing".into()))
}

pub(crate) async fn fetch_inbox_count_at(
    endpoint: &str,
    access_token: &str,
) -> Result<u64, GmailError> {
    let resp = reqwest::Client::new()
        .get(endpoint)
        .bearer_auth(access_token)
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if status.is_success() {
        return parse_messages_total(&body);
    }
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(GmailError::ReauthRequired);
    }
    Err(GmailError::Http {
        status: status.as_u16(),
        body,
    })
}

#[tauri::command]
pub async fn gmail_inbox_count(
    endpoints: State<'_, Endpoints>,
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<u64, GmailError> {
    let token = auth::acquire_access_token(
        &endpoints.google_token,
        storage.inner().as_ref(),
        state.inner(),
    )
    .await?;
    fetch_inbox_count_at(&endpoints.gmail_label_inbox, &token).await
}

pub const GMAIL_MESSAGES_LIST_URL: &str =
    "https://gmail.googleapis.com/gmail/v1/users/me/messages";

pub(crate) async fn list_inbox_ids_at(
    endpoint: &str,
    access_token: &str,
) -> Result<Vec<String>, GmailError> {
    let resp = reqwest::Client::new()
        .get(endpoint)
        .bearer_auth(access_token)
        .query(&[
            ("labelIds", "INBOX"),
            ("maxResults", "100"),
            ("fields", "messages/id"),
        ])
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(GmailError::ReauthRequired);
    }
    if !status.is_success() {
        return Err(GmailError::Http {
            status: status.as_u16(),
            body,
        });
    }

    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| GmailError::Parse(e.to_string()))?;
    let arr = match v.get("messages").and_then(|m| m.as_array()) {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };
    let mut ids = Vec::with_capacity(arr.len());
    for m in arr {
        let id = m
            .get("id")
            .and_then(|x| x.as_str())
            .ok_or_else(|| GmailError::Parse("message without id".into()))?;
        ids.push(id.to_string());
    }
    Ok(ids)
}

use crate::gmail::message::{extract_header, extract_plain_text, GmailMessage};

pub(crate) async fn get_message_at(
    base_endpoint: &str,
    access_token: &str,
    id: &str,
) -> Result<GmailMessage, GmailError> {
    let resp = reqwest::Client::new()
        .get(format!("{base_endpoint}/{id}"))
        .bearer_auth(access_token)
        .query(&[("format", "full")])
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(GmailError::ReauthRequired);
    }
    if !status.is_success() {
        return Err(GmailError::Http {
            status: status.as_u16(),
            body,
        });
    }

    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| GmailError::Parse(e.to_string()))?;
    let payload = v
        .get("payload")
        .ok_or_else(|| GmailError::Parse("message has no payload".into()))?;
    let empty_headers: Vec<serde_json::Value> = Vec::new();
    let headers = payload
        .get("headers")
        .and_then(|h| h.as_array())
        .unwrap_or(&empty_headers);

    let body_text = extract_plain_text(payload);
    let body_truncated: String = body_text.chars().take(10_000).collect();

    Ok(GmailMessage {
        id: id.to_string(),
        from: extract_header(headers, "From"),
        subject: extract_header(headers, "Subject"),
        date: extract_header(headers, "Date"),
        body: body_truncated,
    })
}

#[tauri::command]
pub async fn gmail_random_message(
    endpoints: State<'_, Endpoints>,
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<GmailMessage, GmailError> {
    let token = auth::acquire_access_token(
        &endpoints.google_token,
        storage.inner().as_ref(),
        state.inner(),
    )
    .await?;
    let ids = list_inbox_ids_at(&endpoints.gmail_messages_list, &token).await?;
    if ids.is_empty() {
        return Err(GmailError::Empty);
    }
    let i = rand::random::<u64>() as usize % ids.len();
    get_message_at(&endpoints.gmail_messages_list, &token, &ids[i]).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_messages_total_extracts_number() {
        let body = r#"{"id":"INBOX","name":"INBOX","messagesTotal":348,"threadsTotal":210}"#;
        assert_eq!(parse_messages_total(body).unwrap(), 348);
    }

    #[test]
    fn parse_messages_total_missing_field_returns_parse_error() {
        let body = r#"{"id":"INBOX","name":"INBOX"}"#;
        let err = parse_messages_total(body).unwrap_err();
        assert!(matches!(err, GmailError::Parse(_)));
    }

    #[test]
    fn parse_messages_total_invalid_json_returns_parse_error() {
        let err = parse_messages_total("not json").unwrap_err();
        assert!(matches!(err, GmailError::Parse(_)));
    }

    #[tokio::test]
    async fn fetch_inbox_count_returns_parsed_total_on_200() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("GET", "/inbox")
            .match_header("authorization", "Bearer AT-1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"messagesTotal":42}"#)
            .create_async()
            .await;

        let n = fetch_inbox_count_at(&format!("{}/inbox", server.url()), "AT-1")
            .await
            .unwrap();
        assert_eq!(n, 42);
        m.assert_async().await;
    }

    #[tokio::test]
    async fn fetch_inbox_count_maps_401_to_reauth_required() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/inbox")
            .with_status(401)
            .with_body(r#"{"error":{"code":401,"message":"Invalid Credentials"}}"#)
            .create_async()
            .await;

        let err = fetch_inbox_count_at(&format!("{}/inbox", server.url()), "AT-1")
            .await
            .unwrap_err();
        assert!(matches!(err, GmailError::ReauthRequired));
    }

    #[tokio::test]
    async fn fetch_inbox_count_maps_5xx_to_http_error() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/inbox")
            .with_status(503)
            .with_body("upstream down")
            .create_async()
            .await;

        let err = fetch_inbox_count_at(&format!("{}/inbox", server.url()), "AT-1")
            .await
            .unwrap_err();
        match err {
            GmailError::Http { status, .. } => assert_eq!(status, 503),
            other => panic!("expected Http, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn fetch_inbox_count_maps_403_to_http_error() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/inbox")
            .with_status(403)
            .with_body("forbidden")
            .create_async()
            .await;

        let err = fetch_inbox_count_at(&format!("{}/inbox", server.url()), "AT-1")
            .await
            .unwrap_err();
        match err {
            GmailError::Http { status, .. } => assert_eq!(status, 403),
            other => panic!("expected Http, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn fetch_inbox_count_returns_parse_error_when_body_lacks_field() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/inbox")
            .with_status(200)
            .with_body(r#"{"id":"INBOX"}"#)
            .create_async()
            .await;

        let err = fetch_inbox_count_at(&format!("{}/inbox", server.url()), "AT-1")
            .await
            .unwrap_err();
        assert!(matches!(err, GmailError::Parse(_)));
    }

    #[tokio::test]
    async fn list_inbox_ids_returns_ids_on_200() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("labelIds".into(), "INBOX".into()),
                mockito::Matcher::UrlEncoded("maxResults".into(), "100".into()),
                mockito::Matcher::UrlEncoded("fields".into(), "messages/id".into()),
            ]))
            .match_header("authorization", "Bearer AT-1")
            .with_status(200)
            .with_body(r#"{"messages":[{"id":"a"},{"id":"b"},{"id":"c"}]}"#)
            .create_async()
            .await;

        let ids = list_inbox_ids_at(&format!("{}/messages", server.url()), "AT-1")
            .await
            .unwrap();
        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    #[tokio::test]
    async fn list_inbox_ids_returns_empty_when_no_messages_field() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body(r#"{"resultSizeEstimate":0}"#)
            .create_async()
            .await;

        let ids = list_inbox_ids_at(&format!("{}/messages", server.url()), "AT-1")
            .await
            .unwrap();
        assert!(ids.is_empty());
    }

    #[tokio::test]
    async fn list_inbox_ids_maps_401_to_reauth() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(401)
            .with_body("nope")
            .create_async()
            .await;

        let err = list_inbox_ids_at(&format!("{}/messages", server.url()), "AT-1")
            .await
            .unwrap_err();
        assert!(matches!(err, GmailError::ReauthRequired));
    }

    #[tokio::test]
    async fn list_inbox_ids_maps_5xx_to_http() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(503)
            .with_body("boom")
            .create_async()
            .await;

        let err = list_inbox_ids_at(&format!("{}/messages", server.url()), "AT-1")
            .await
            .unwrap_err();
        match err {
            GmailError::Http { status, .. } => assert_eq!(status, 503),
            other => panic!("expected Http, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_message_returns_parsed_gmail_message() {
        use base64::Engine;
        let body_data = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"hello body");
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages/m1")
            .match_query(mockito::Matcher::UrlEncoded("format".into(), "full".into()))
            .match_header("authorization", "Bearer AT-1")
            .with_status(200)
            .with_body(
                serde_json::json!({
                    "id": "m1",
                    "payload": {
                        "mimeType": "text/plain",
                        "headers": [
                            {"name": "From",    "value": "alice@example.com"},
                            {"name": "Subject", "value": "Greetings"},
                            {"name": "Date",    "value": "Mon, 15 May 2026 10:00:00 +0300"}
                        ],
                        "body": {"data": body_data}
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let msg = get_message_at(&format!("{}/messages", server.url()), "AT-1", "m1")
            .await
            .unwrap();
        assert_eq!(msg.id, "m1");
        assert_eq!(msg.from, "alice@example.com");
        assert_eq!(msg.subject, "Greetings");
        assert_eq!(msg.date, "Mon, 15 May 2026 10:00:00 +0300");
        assert_eq!(msg.body, "hello body");
    }

    #[tokio::test]
    async fn get_message_maps_401_to_reauth() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages/m1")
            .match_query(mockito::Matcher::Any)
            .with_status(401)
            .with_body("nope")
            .create_async()
            .await;

        let err = get_message_at(&format!("{}/messages", server.url()), "AT-1", "m1")
            .await
            .unwrap_err();
        assert!(matches!(err, GmailError::ReauthRequired));
    }

    #[tokio::test]
    async fn get_message_truncates_long_body() {
        use base64::Engine;
        let long = "x".repeat(11_000);
        let data = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(long.as_bytes());
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages/m1")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body(
                serde_json::json!({
                    "id": "m1",
                    "payload": {
                        "mimeType": "text/plain",
                        "headers": [],
                        "body": {"data": data}
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let msg = get_message_at(&format!("{}/messages", server.url()), "AT-1", "m1")
            .await
            .unwrap();
        assert_eq!(msg.body.chars().count(), 10_000);
    }
}
