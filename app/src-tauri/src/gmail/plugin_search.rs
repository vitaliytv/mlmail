//! Product-owned native representation of the `vitaliytv:gmail/search` WIT contract.

use serde::{Deserialize, Serialize};

use super::error::GmailError;

/// Request fields forwarded directly to Gmail `users.messages.list`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GmailListRequest {
    /// Unrestricted Gmail query syntax; empty keeps Gmail's standard semantics.
    pub q: String,
    /// Optional Gmail API page size.
    pub max_results: Option<u32>,
    /// Opaque continuation token from the preceding API page.
    pub page_token: Option<String>,
    /// Optional Gmail label identifiers without an application-imposed default.
    pub label_ids: Vec<String>,
    /// Optional Gmail `includeSpamTrash` flag.
    pub include_spam_trash: Option<bool>,
}

/// One message identity in the exact Gmail list response shape.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailMessageRef {
    /// Gmail message identifier.
    pub id: String,
    /// Optional Gmail thread identifier.
    pub thread_id: Option<String>,
}

/// One page from Gmail `users.messages.list`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailListResponse {
    /// Omitted when Gmail found no messages for this page.
    pub messages: Option<Vec<GmailMessageRef>>,
    /// Opaque token for the following page, omitted on the final page.
    pub next_page_token: Option<String>,
    /// Gmail's optional approximate result count.
    pub result_size_estimate: Option<u32>,
}

/// Fetches exactly one native Gmail list page without inspecting or restricting `q`.
pub async fn list_messages_page_at(
    endpoint: &str,
    access_token: &str,
    request: &GmailListRequest,
) -> Result<GmailListResponse, GmailError> {
    let mut query = Vec::new();
    query.push(("q", request.q.clone()));
    if let Some(max_results) = request.max_results {
        query.push(("maxResults", max_results.to_string()));
    }
    if let Some(page_token) = &request.page_token {
        query.push(("pageToken", page_token.clone()));
    }
    if let Some(include_spam_trash) = request.include_spam_trash {
        query.push(("includeSpamTrash", include_spam_trash.to_string()));
    }
    for label_id in &request.label_ids {
        query.push(("labelIds", label_id.clone()));
    }

    let response = reqwest::Client::new()
        .get(endpoint)
        .bearer_auth(access_token)
        .query(&query)
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
    serde_json::from_str(&body).map_err(|error| GmailError::Parse(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{list_messages_page_at, GmailListRequest, GmailListResponse};

    #[tokio::test]
    async fn forwards_unrestricted_query_and_preserves_gmail_page_shape() {
        let mut server = mockito::Server::new_async().await;
        let response = r#"{"messages":[{"id":"m1","threadId":"t1"}],"nextPageToken":"next","resultSizeEstimate":42}"#;
        let mock = server
            .mock("GET", "/messages")
            .match_header("authorization", "Bearer token")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("q".into(), "from:booking.com newer_than:30d".into()),
                mockito::Matcher::UrlEncoded("maxResults".into(), "50".into()),
                mockito::Matcher::UrlEncoded("pageToken".into(), "previous".into()),
                mockito::Matcher::UrlEncoded("labelIds".into(), "INBOX".into()),
                mockito::Matcher::UrlEncoded("includeSpamTrash".into(), "true".into()),
            ]))
            .with_status(200)
            .with_body(response)
            .create_async()
            .await;
        let request = GmailListRequest {
            q: "from:booking.com newer_than:30d".into(),
            max_results: Some(50),
            page_token: Some("previous".into()),
            label_ids: vec!["INBOX".into()],
            include_spam_trash: Some(true),
        };

        let page = list_messages_page_at(&format!("{}/messages", server.url()), "token", &request)
            .await
            .expect("Gmail page should decode");

        mock.assert_async().await;
        assert_eq!(
            page,
            GmailListResponse {
                messages: Some(vec![super::GmailMessageRef {
                    id: "m1".into(),
                    thread_id: Some("t1".into()),
                }]),
                next_page_token: Some("next".into()),
                result_size_estimate: Some(42),
            }
        );
    }
}
