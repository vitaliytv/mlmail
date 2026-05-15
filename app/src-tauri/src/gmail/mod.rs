pub mod error;
pub mod message;

use crate::auth::{self, state::AuthState};
use crate::gmail::error::GmailError;
use serde::Deserialize;
use std::sync::Mutex;
use tauri::{AppHandle, State};

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
    app: AppHandle,
    state: State<'_, Mutex<AuthState>>,
) -> Result<u64, GmailError> {
    let token = auth::acquire_access_token(&app, &state).await?;
    fetch_inbox_count_at(GMAIL_LABEL_INBOX_URL, &token).await
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
}
