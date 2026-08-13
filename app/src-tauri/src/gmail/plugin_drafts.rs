//! Product-owned native implementation of the `nitra:gmail/drafts` WIT contract.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::Deserialize;

use super::GmailError;

/// One requested Gmail draft message without OAuth or HTTP transport details.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailDraftRequest {
    /// Recipient used for the RFC 822 `To` header.
    pub to: String,
    /// Subject used for the RFC 822 `Subject` header.
    pub subject: String,
    /// Plain-text RFC 822 message body.
    pub body: String,
}

/// Opaque Gmail identifier returned after a successful draft creation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailDraftRef {
    /// Gmail draft identifier.
    pub id: String,
}

#[derive(Deserialize)]
struct GmailDraftResponse {
    id: Option<String>,
}

/// Creates a Gmail draft at a caller-supplied endpoint using an authenticated token.
pub async fn create_draft_at(
    endpoint: &str,
    access_token: &str,
    request: &GmailDraftRequest,
) -> Result<GmailDraftRef, GmailError> {
    let raw = URL_SAFE_NO_PAD.encode(format!(
        "To: {}\r\nSubject: {}\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\n{}",
        request.to, request.subject, request.body
    ));
    let response = reqwest::Client::new()
        .post(endpoint)
        .bearer_auth(access_token)
        .json(&serde_json::json!({ "message": { "raw": raw } }))
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(GmailError::ReauthRequired);
    }
    if !status.is_success() {
        return Err(GmailError::Http {
            status: status.as_u16(),
            body,
        });
    }

    let response: GmailDraftResponse =
        serde_json::from_str(&body).map_err(|error| GmailError::Parse(error.to_string()))?;
    response
        .id
        .map(|id| GmailDraftRef { id })
        .ok_or_else(|| GmailError::Parse("draft id missing".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn creates_a_gmail_draft_with_rfc822_content() {
        let mut server = mockito::Server::new_async().await;
        let raw_prefix = URL_SAFE_NO_PAD.encode("To: a@example.com");
        let created = server
            .mock("POST", "/drafts")
            .match_header("authorization", "Bearer token")
            .match_body(mockito::Matcher::Regex(format!("\"raw\":\"{raw_prefix}")))
            .with_status(200)
            .with_body(r#"{"id":"draft-1"}"#)
            .create_async()
            .await;

        let draft = create_draft_at(
            &format!("{}/drafts", server.url()),
            "token",
            &GmailDraftRequest {
                to: "a@example.com".to_owned(),
                subject: "Hello".to_owned(),
                body: "From Component".to_owned(),
            },
        )
        .await
        .expect("Gmail should create the draft");

        created.assert_async().await;
        assert_eq!(draft.id, "draft-1");
    }
}
