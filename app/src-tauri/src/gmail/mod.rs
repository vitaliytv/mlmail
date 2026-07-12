pub mod error;
pub mod message;

use crate::auth::{self, state::AuthState, storage::SharedStorage};
use crate::endpoints::Endpoints;
use crate::gmail::error::GmailError;
use serde::Deserialize;
use std::sync::Mutex;
#[cfg(target_os = "macos")]
use tauri::Manager;
use tauri::State;

pub const GMAIL_LABEL_INBOX_URL: &str =
    "https://gmail.googleapis.com/gmail/v1/users/me/labels/INBOX";
pub const GMAIL_LABELS_URL: &str = "https://gmail.googleapis.com/gmail/v1/users/me/labels";

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
pub async fn gmail_inbox_count<R: tauri::Runtime>(
    #[cfg_attr(not(target_os = "macos"), allow(unused_variables))] app: tauri::AppHandle<R>,
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
    let count = fetch_inbox_count_at(&endpoints.gmail_label_inbox, &token).await?;
    #[cfg(target_os = "macos")]
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_badge_count(Some(count as i64));
    }
    Ok(count)
}

pub const GMAIL_MESSAGES_LIST_URL: &str = "https://gmail.googleapis.com/gmail/v1/users/me/messages";

pub const GMAIL_BATCH_MODIFY_URL: &str =
    "https://gmail.googleapis.com/gmail/v1/users/me/messages/batchModify";

pub const GMAIL_FILTERS_URL: &str =
    "https://gmail.googleapis.com/gmail/v1/users/me/settings/filters";

pub(crate) async fn list_inbox_ids_at(
    endpoint: &str,
    access_token: &str,
) -> Result<Vec<String>, GmailError> {
    list_inbox_ids_at_q(endpoint, access_token, "").await
}

pub(crate) async fn list_inbox_ids_at_q(
    endpoint: &str,
    access_token: &str,
    q: &str,
) -> Result<Vec<String>, GmailError> {
    let mut params: Vec<(&str, &str)> = vec![
        ("labelIds", "INBOX"),
        ("maxResults", "100"),
        ("fields", "messages/id"),
    ];
    if !q.is_empty() {
        params.push(("q", q));
    }
    let resp = reqwest::Client::new()
        .get(endpoint)
        .bearer_auth(access_token)
        .query(&params)
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

use crate::gmail::message::{
    extract_header, extract_html_body, extract_plain_text, parse_unsubscribe, GmailMessage,
    UnsubscribeAction,
};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

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
    let html_body = extract_html_body(payload);

    Ok(GmailMessage {
        id: id.to_string(),
        from: extract_header(headers, "From"),
        subject: extract_header(headers, "Subject"),
        date: extract_header(headers, "Date"),
        body: body_truncated,
        html_body,
        unsubscribe: parse_unsubscribe(headers),
    })
}

pub(crate) async fn post_one_click(url: &str) -> Result<(), GmailError> {
    let resp = reqwest::Client::new()
        .post(url)
        .header("content-type", "application/x-www-form-urlencoded")
        .body("List-Unsubscribe=One-Click")
        .send()
        .await?;
    let status = resp.status();
    if status.is_success() {
        return Ok(());
    }
    let body = resp.text().await.unwrap_or_default();
    Err(GmailError::Http {
        status: status.as_u16(),
        body,
    })
}

#[tauri::command]
pub async fn gmail_unsubscribe(
    app: AppHandle,
    action: UnsubscribeAction,
) -> Result<(), GmailError> {
    match action {
        UnsubscribeAction::OneClick { url } => post_one_click(&url).await,
        UnsubscribeAction::Url { url } => app
            .opener()
            .open_url(&url, None::<&str>)
            .map_err(|e| GmailError::Platform(format!("open browser: {e}"))),
        UnsubscribeAction::Mailto { to, subject } => {
            let mut url = format!("mailto:{to}");
            if let Some(s) = subject {
                url.push_str("?subject=");
                url.push_str(&s);
            }
            app.opener()
                .open_url(&url, None::<&str>)
                .map_err(|e| GmailError::Platform(format!("open mailto: {e}")))
        }
    }
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

#[tauri::command]
pub async fn gmail_random_newsletter(
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
    let ids = list_inbox_ids_at_q(
        &endpoints.gmail_messages_list,
        &token,
        "has:list-unsubscribe",
    )
    .await?;
    if ids.is_empty() {
        return Err(GmailError::Empty);
    }
    let i = rand::random::<u64>() as usize % ids.len();
    get_message_at(&endpoints.gmail_messages_list, &token, &ids[i]).await
}

/// Lightweight inbox-message summary (no body) for search results.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailSummary {
    pub id: String,
    pub from: String,
    pub subject: String,
    pub date: String,
}

/// Fetch only From/Subject/Date headers (format=metadata) for one message.
pub(crate) async fn get_message_meta_at(
    base_endpoint: &str,
    access_token: &str,
    id: &str,
) -> Result<GmailSummary, GmailError> {
    let resp = reqwest::Client::new()
        .get(format!("{base_endpoint}/{id}"))
        .bearer_auth(access_token)
        .query(&[
            ("format", "metadata"),
            ("metadataHeaders", "From"),
            ("metadataHeaders", "Subject"),
            ("metadataHeaders", "Date"),
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
    let empty_headers: Vec<serde_json::Value> = Vec::new();
    let headers = v
        .get("payload")
        .and_then(|p| p.get("headers"))
        .and_then(|h| h.as_array())
        .unwrap_or(&empty_headers);
    Ok(GmailSummary {
        id: id.to_string(),
        from: extract_header(headers, "From"),
        subject: extract_header(headers, "Subject"),
        date: extract_header(headers, "Date"),
    })
}

/// Search the inbox with a Gmail query (`q`); returns up to 15 message
/// summaries (id/from/subject/date), newest first.
#[tauri::command]
pub async fn gmail_search(
    q: String,
    endpoints: State<'_, Endpoints>,
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<Vec<GmailSummary>, GmailError> {
    let token = auth::acquire_access_token(
        &endpoints.google_token,
        storage.inner().as_ref(),
        state.inner(),
    )
    .await?;
    let ids = list_inbox_ids_at_q(&endpoints.gmail_messages_list, &token, &q).await?;
    let mut out = Vec::new();
    for id in ids.into_iter().take(15) {
        out.push(get_message_meta_at(&endpoints.gmail_messages_list, &token, &id).await?);
    }
    Ok(out)
}

/// Read one inbox message in full (headers + plain-text body) by id.
#[tauri::command]
pub async fn gmail_read(
    id: String,
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
    get_message_at(&endpoints.gmail_messages_list, &token, &id).await
}

/// Move one message to Trash by id (reversible; Gmail purges Trash after 30d).
#[tauri::command]
pub async fn gmail_trash(
    id: String,
    endpoints: State<'_, Endpoints>,
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<(), GmailError> {
    let token = auth::acquire_access_token(
        &endpoints.google_token,
        storage.inner().as_ref(),
        state.inner(),
    )
    .await?;
    let url = format!("{}/{}/trash", endpoints.gmail_messages_list, id);
    let resp = reqwest::Client::new()
        .post(&url)
        .bearer_auth(&token)
        .header("content-length", "0")
        .send()
        .await?;
    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(GmailError::ReauthRequired);
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(GmailError::Http {
            status: status.as_u16(),
            body,
        });
    }
    Ok(())
}

/// Gmail caps `batchModify` at 1000 message ids per request.
const BATCH_MODIFY_MAX: usize = 1000;

/// List **all** INBOX message ids matching `q`, following `nextPageToken`
/// through every page (unlike [`list_inbox_ids_at_q`], which returns one page).
pub(crate) async fn list_all_ids_at_q(
    endpoint: &str,
    access_token: &str,
    q: &str,
) -> Result<Vec<String>, GmailError> {
    let mut ids = Vec::new();
    let mut page_token: Option<String> = None;
    loop {
        let mut params: Vec<(&str, &str)> = vec![
            ("labelIds", "INBOX"),
            ("maxResults", "500"),
            ("fields", "messages/id,nextPageToken"),
        ];
        if !q.is_empty() {
            params.push(("q", q));
        }
        if let Some(ref t) = page_token {
            params.push(("pageToken", t));
        }
        let resp = reqwest::Client::new()
            .get(endpoint)
            .bearer_auth(access_token)
            .query(&params)
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
        if let Some(arr) = v.get("messages").and_then(|m| m.as_array()) {
            for m in arr {
                let id = m
                    .get("id")
                    .and_then(|x| x.as_str())
                    .ok_or_else(|| GmailError::Parse("message without id".into()))?;
                ids.push(id.to_string());
            }
        }
        match v.get("nextPageToken").and_then(|t| t.as_str()) {
            Some(t) if !t.is_empty() => page_token = Some(t.to_string()),
            _ => break,
        }
    }
    Ok(ids)
}

/// POST `payload` to `endpoint` with a bearer token; returns the response on
/// success, translating an unauthorized status into `GmailError::ReauthRequired`.
/// The caller decides how (or whether) to parse the response body, since some
/// Gmail endpoints (e.g. `batchModify`) return an empty body on success.
async fn authed_post(
    endpoint: &str,
    access_token: &str,
    payload: &serde_json::Value,
) -> Result<reqwest::Response, GmailError> {
    let resp = reqwest::Client::new()
        .post(endpoint)
        .bearer_auth(access_token)
        .json(payload)
        .send()
        .await?;
    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(GmailError::ReauthRequired);
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(GmailError::Http {
            status: status.as_u16(),
            body,
        });
    }
    Ok(resp)
}

/// Move one batch of message ids (≤1000) to Trash via `batchModify`
/// (adds the `TRASH` label, removes `INBOX`). Returns 204 on success.
pub(crate) async fn batch_trash_at(
    endpoint: &str,
    access_token: &str,
    ids: &[String],
) -> Result<(), GmailError> {
    let payload = serde_json::json!({
        "ids": ids,
        "addLabelIds": ["TRASH"],
        "removeLabelIds": ["INBOX"],
    });
    authed_post(endpoint, access_token, &payload).await?;
    Ok(())
}

/// Outcome of [`gmail_trash_query`]: how many messages were moved to Trash.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashQueryResult {
    pub trashed: u32,
}

/// Move **every** inbox message matching a Gmail query `q` to Trash. Paginates
/// the full match set, then trashes it in `batchModify` chunks (Gmail caps each
/// call at 1000 ids). Reversible — Gmail purges Trash after 30 days.
///
/// Rejects an empty query: that would match the whole inbox.
#[tauri::command]
pub async fn gmail_trash_query(
    q: String,
    endpoints: State<'_, Endpoints>,
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<TrashQueryResult, GmailError> {
    if q.trim().is_empty() {
        return Err(GmailError::EmptyQuery);
    }
    let token = auth::acquire_access_token(
        &endpoints.google_token,
        storage.inner().as_ref(),
        state.inner(),
    )
    .await?;
    let ids = list_all_ids_at_q(&endpoints.gmail_messages_list, &token, &q).await?;
    for chunk in ids.chunks(BATCH_MODIFY_MAX) {
        batch_trash_at(&endpoints.gmail_batch_modify, &token, chunk).await?;
    }
    Ok(TrashQueryResult {
        trashed: ids.len() as u32,
    })
}

/// Identifier of a newly created Gmail filter.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterResult {
    pub id: String,
}

/// Create a Gmail filter that auto-trashes future matches (action: add `TRASH`,
/// remove `INBOX`). `from`/`subject` are matched as Gmail filter criteria; at
/// least one must be non-empty. A 403 carrying a scope complaint is mapped to
/// [`GmailError::ReauthRequired`] (the `gmail.settings.basic` consent is new).
pub(crate) async fn create_filter_at(
    endpoint: &str,
    access_token: &str,
    from: &str,
    subject: &str,
) -> Result<FilterResult, GmailError> {
    let mut criteria = serde_json::Map::new();
    if !from.is_empty() {
        criteria.insert("from".into(), from.into());
    }
    if !subject.is_empty() {
        criteria.insert("subject".into(), subject.into());
    }
    let payload = serde_json::json!({
        "criteria": serde_json::Value::Object(criteria),
        "action": {
            "addLabelIds": ["TRASH"],
            "removeLabelIds": ["INBOX"],
        },
    });
    let resp = reqwest::Client::new()
        .post(endpoint)
        .bearer_auth(access_token)
        .json(&payload)
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(GmailError::ReauthRequired);
    }
    // A fresh install may still hold a token minted before gmail.settings.basic
    // was requested; Google answers 403 with a scope complaint. Nudge re-login.
    if status == reqwest::StatusCode::FORBIDDEN && body.to_lowercase().contains("scope") {
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
    let id = v
        .get("id")
        .and_then(|x| x.as_str())
        .ok_or_else(|| GmailError::Parse("filter response missing id".into()))?;
    Ok(FilterResult { id: id.to_string() })
}

/// Create a Gmail filter that automatically moves future matching mail to Trash.
#[tauri::command]
pub async fn gmail_create_filter(
    from: String,
    subject: String,
    endpoints: State<'_, Endpoints>,
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<FilterResult, GmailError> {
    let from = from.trim();
    let subject = subject.trim();
    if from.is_empty() && subject.is_empty() {
        return Err(GmailError::EmptyQuery);
    }
    let token = auth::acquire_access_token(
        &endpoints.google_token,
        storage.inner().as_ref(),
        state.inner(),
    )
    .await?;
    create_filter_at(&endpoints.gmail_filters, &token, from, subject).await
}

/// Match criteria of an existing Gmail filter, as returned by the list endpoint.
///
/// Mirrors the full `Filter.Criteria` schema from the Gmail API so no
/// condition type is silently dropped during deserialization.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterCriteria {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub negated_query: Option<String>,
    #[serde(default)]
    pub has_attachment: Option<bool>,
    #[serde(default)]
    pub exclude_chats: Option<bool>,
    #[serde(default)]
    pub size: Option<i64>,
    #[serde(default)]
    pub size_comparison: Option<String>,
}

/// Action of an existing Gmail filter, as returned by the list endpoint.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterAction {
    #[serde(default)]
    pub add_label_ids: Vec<String>,
    #[serde(default)]
    pub remove_label_ids: Vec<String>,
    #[serde(default)]
    pub forward: Option<String>,
}

/// One existing Gmail filter, as returned by the list endpoint.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterListItem {
    pub id: String,
    #[serde(default)]
    pub criteria: FilterCriteria,
    #[serde(default)]
    pub action: FilterAction,
}

/// GET `endpoint` with a bearer token and return the parsed JSON body,
/// translating auth-related HTTP statuses into `GmailError::ReauthRequired`.
async fn authed_get_json(
    endpoint: &str,
    access_token: &str,
) -> Result<serde_json::Value, GmailError> {
    let resp = reqwest::Client::new()
        .get(endpoint)
        .bearer_auth(access_token)
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(GmailError::ReauthRequired);
    }
    if status == reqwest::StatusCode::FORBIDDEN && body.to_lowercase().contains("scope") {
        return Err(GmailError::ReauthRequired);
    }
    if !status.is_success() {
        return Err(GmailError::Http {
            status: status.as_u16(),
            body,
        });
    }
    serde_json::from_str(&body).map_err(|e| GmailError::Parse(e.to_string()))
}

/// List all Gmail filters configured for the account.
pub(crate) async fn list_filters_at(
    endpoint: &str,
    access_token: &str,
) -> Result<Vec<FilterListItem>, GmailError> {
    let v = authed_get_json(endpoint, access_token).await?;
    let items = match v.get("filter") {
        Some(arr) => {
            serde_json::from_value(arr.clone()).map_err(|e| GmailError::Parse(e.to_string()))?
        }
        None => Vec::new(),
    };
    Ok(items)
}

/// List all Gmail filters for the authenticated user.
#[tauri::command]
pub async fn gmail_list_filters(
    endpoints: State<'_, Endpoints>,
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<Vec<FilterListItem>, GmailError> {
    let token = auth::acquire_access_token(
        &endpoints.google_token,
        storage.inner().as_ref(),
        state.inner(),
    )
    .await?;
    list_filters_at(&endpoints.gmail_filters, &token).await
}

/// One Gmail label, as returned by the labels list endpoint.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GmailLabel {
    pub id: String,
    pub name: String,
}

/// List all Gmail labels (system and user-created) for the authenticated account.
pub(crate) async fn list_labels_at(
    endpoint: &str,
    access_token: &str,
) -> Result<Vec<GmailLabel>, GmailError> {
    let v = authed_get_json(endpoint, access_token).await?;
    let labels = v
        .get("labels")
        .and_then(|l| l.as_array())
        .cloned()
        .unwrap_or_default();
    Ok(labels
        .into_iter()
        .filter_map(|l| {
            let id = l.get("id")?.as_str()?.to_string();
            let name = l.get("name")?.as_str()?.to_string();
            Some(GmailLabel { id, name })
        })
        .collect())
}

/// List all Gmail labels for the authenticated user.
#[tauri::command]
pub async fn gmail_list_labels(
    endpoints: State<'_, Endpoints>,
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<Vec<GmailLabel>, GmailError> {
    let token = auth::acquire_access_token(
        &endpoints.google_token,
        storage.inner().as_ref(),
        state.inner(),
    )
    .await?;
    list_labels_at(&endpoints.gmail_labels, &token).await
}

/// Delete a Gmail filter by id.
pub(crate) async fn delete_filter_at(
    endpoint: &str,
    access_token: &str,
    id: &str,
) -> Result<(), GmailError> {
    let url = format!("{endpoint}/{id}");
    let resp = reqwest::Client::new()
        .delete(&url)
        .bearer_auth(access_token)
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(GmailError::ReauthRequired);
    }
    if status == reqwest::StatusCode::FORBIDDEN && body.to_lowercase().contains("scope") {
        return Err(GmailError::ReauthRequired);
    }
    if !status.is_success() {
        return Err(GmailError::Http {
            status: status.as_u16(),
            body,
        });
    }
    Ok(())
}

/// Delete a Gmail filter by id.
#[tauri::command]
pub async fn gmail_delete_filter(
    id: String,
    endpoints: State<'_, Endpoints>,
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<(), GmailError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(GmailError::EmptyQuery);
    }
    let token = auth::acquire_access_token(
        &endpoints.google_token,
        storage.inner().as_ref(),
        state.inner(),
    )
    .await?;
    delete_filter_at(&endpoints.gmail_filters, &token, id).await
}

const SAVED_LABEL_NAME: &str = "Збережено";
const TASK_LABEL_NAME: &str = "Задача";

/// Ensure a user label named `name` exists; return its id. Creates it
/// (with the given colours) if absent.
async fn ensure_label(
    labels_url: &str,
    access_token: &str,
    name: &str,
    bg_color: &str,
    text_color: &str,
) -> Result<String, GmailError> {
    // List existing labels.
    let v = authed_get_json(labels_url, access_token).await?;
    if let Some(labels) = v.get("labels").and_then(|l| l.as_array()) {
        for label in labels {
            if label.get("name").and_then(|n| n.as_str()) == Some(name) {
                if let Some(id) = label.get("id").and_then(|i| i.as_str()) {
                    return Ok(id.to_string());
                }
            }
        }
    }
    // Create the label.
    let payload = serde_json::json!({
        "name": name,
        "labelListVisibility": "labelShow",
        "messageListVisibility": "show",
        "color": { "backgroundColor": bg_color, "textColor": text_color },
    });
    let resp = authed_post(labels_url, access_token, &payload).await?;
    let v: serde_json::Value = serde_json::from_str(&resp.text().await.unwrap_or_default())
        .map_err(|e| GmailError::Parse(e.to_string()))?;
    v.get("id")
        .and_then(|i| i.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| GmailError::Parse("label create response missing id".into()))
}

/// Ensure the "Збережено" user label exists; return its id.
async fn ensure_saved_label(labels_url: &str, access_token: &str) -> Result<String, GmailError> {
    ensure_label(
        labels_url,
        access_token,
        SAVED_LABEL_NAME,
        "#16a766",
        "#ffffff",
    )
    .await
}

/// Ensure the "Задача" user label exists; return its id.
async fn ensure_task_label(labels_url: &str, access_token: &str) -> Result<String, GmailError> {
    ensure_label(
        labels_url,
        access_token,
        TASK_LABEL_NAME,
        "#fb4c2f",
        "#ffffff",
    )
    .await
}

/// POST one `batchModify` call (add/remove label ids on a single message id)
/// and translate the response into a `Result`.
async fn modify_message(
    batch_modify_url: &str,
    access_token: &str,
    id: &str,
    add_label_ids: &[&str],
    remove_label_ids: &[&str],
) -> Result<(), GmailError> {
    let payload = serde_json::json!({
        "ids": [id],
        "addLabelIds": add_label_ids,
        "removeLabelIds": remove_label_ids,
    });
    let resp = reqwest::Client::new()
        .post(batch_modify_url)
        .bearer_auth(access_token)
        .json(&payload)
        .send()
        .await?;
    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(GmailError::ReauthRequired);
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(GmailError::Http {
            status: status.as_u16(),
            body,
        });
    }
    Ok(())
}

/// Apply the "Збережено" label to one message and remove it from INBOX.
#[tauri::command]
pub async fn gmail_save(
    id: String,
    endpoints: State<'_, Endpoints>,
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<(), GmailError> {
    let token = auth::acquire_access_token(
        &endpoints.google_token,
        storage.inner().as_ref(),
        state.inner(),
    )
    .await?;
    let label_id = ensure_saved_label(&endpoints.gmail_labels, &token).await?;
    modify_message(
        &endpoints.gmail_batch_modify,
        &token,
        &id,
        &[&label_id],
        &["INBOX"],
    )
    .await
}

/// Apply the "Задача" label to one message. Unlike [`gmail_save`], the message
/// stays wherever it is (INBOX included) — flagging a task doesn't archive mail.
#[tauri::command]
pub async fn gmail_flag_task(
    id: String,
    endpoints: State<'_, Endpoints>,
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<(), GmailError> {
    let token = auth::acquire_access_token(
        &endpoints.google_token,
        storage.inner().as_ref(),
        state.inner(),
    )
    .await?;
    let label_id = ensure_task_label(&endpoints.gmail_labels, &token).await?;
    modify_message(
        &endpoints.gmail_batch_modify,
        &token,
        &id,
        &[&label_id],
        &[],
    )
    .await
}

/// Remove the "Задача" label from one message (marks the task done without
/// otherwise touching the message).
#[tauri::command]
pub async fn gmail_unflag_task(
    id: String,
    endpoints: State<'_, Endpoints>,
    storage: State<'_, SharedStorage>,
    state: State<'_, Mutex<AuthState>>,
) -> Result<(), GmailError> {
    let token = auth::acquire_access_token(
        &endpoints.google_token,
        storage.inner().as_ref(),
        state.inner(),
    )
    .await?;
    let label_id = ensure_task_label(&endpoints.gmail_labels, &token).await?;
    modify_message(
        &endpoints.gmail_batch_modify,
        &token,
        &id,
        &[],
        &[&label_id],
    )
    .await
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
    async fn list_inbox_ids_passes_q_param_when_non_empty() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("labelIds".into(), "INBOX".into()),
                mockito::Matcher::UrlEncoded("q".into(), "has:list-unsubscribe".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"messages":[{"id":"n1"}]}"#)
            .create_async()
            .await;
        let ids = list_inbox_ids_at_q(
            &format!("{}/messages", server.url()),
            "AT-1",
            "has:list-unsubscribe",
        )
        .await
        .unwrap();
        assert_eq!(ids, vec!["n1"]);
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
    async fn get_message_parses_one_click_unsubscribe_header() {
        use crate::gmail::message::UnsubscribeAction;
        use base64::Engine;
        let data = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"hi");
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
                        "headers": [
                            {"name": "From",    "value": "n@l"},
                            {"name": "Subject", "value": "s"},
                            {"name": "Date",    "value": "d"},
                            {"name": "List-Unsubscribe",
                             "value": "<mailto:u@l.com>, <https://l.com/u/abc>"},
                            {"name": "List-Unsubscribe-Post",
                             "value": "List-Unsubscribe=One-Click"}
                        ],
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
        assert_eq!(
            msg.unsubscribe,
            Some(UnsubscribeAction::OneClick {
                url: "https://l.com/u/abc".into(),
            })
        );
    }

    #[tokio::test]
    async fn get_message_unsubscribe_is_none_when_header_absent() {
        use base64::Engine;
        let data = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"hi");
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
                        "headers": [{"name": "From", "value": "a@b"}],
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
        assert_eq!(msg.unsubscribe, None);
    }

    #[tokio::test]
    async fn post_one_click_sends_form_body() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("POST", "/unsub")
            .match_header("content-type", "application/x-www-form-urlencoded")
            .match_body("List-Unsubscribe=One-Click")
            .with_status(200)
            .create_async()
            .await;
        post_one_click(&format!("{}/unsub", server.url()))
            .await
            .unwrap();
        m.assert_async().await;
    }

    #[tokio::test]
    async fn post_one_click_maps_5xx_to_http_error() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/unsub")
            .with_status(503)
            .with_body("down")
            .create_async()
            .await;
        let err = post_one_click(&format!("{}/unsub", server.url()))
            .await
            .unwrap_err();
        match err {
            GmailError::Http { status, .. } => assert_eq!(status, 503),
            other => panic!("expected Http, got {other:?}"),
        }
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

    #[tokio::test]
    async fn list_all_ids_follows_pagination() {
        let mut server = mockito::Server::new_async().await;
        // Most-specific mock first: mockito returns the first matching mock, so
        // the page-2 request (carrying pageToken) is served here, not by page 1.
        let p2 = server
            .mock("GET", "/messages")
            .match_query(mockito::Matcher::UrlEncoded(
                "pageToken".into(),
                "PT2".into(),
            ))
            .with_status(200)
            .with_body(r#"{"messages":[{"id":"c"}]}"#)
            .create_async()
            .await;
        let p1 = server
            .mock("GET", "/messages")
            .match_query(mockito::Matcher::UrlEncoded("q".into(), "from:x".into()))
            .with_status(200)
            .with_body(r#"{"messages":[{"id":"a"},{"id":"b"}],"nextPageToken":"PT2"}"#)
            .create_async()
            .await;

        let ids = list_all_ids_at_q(&format!("{}/messages", server.url()), "AT-1", "from:x")
            .await
            .unwrap();
        assert_eq!(ids, vec!["a", "b", "c"]);
        p1.assert_async().await;
        p2.assert_async().await;
    }

    #[tokio::test]
    async fn list_all_ids_maps_401_to_reauth() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(401)
            .with_body("nope")
            .create_async()
            .await;
        let err = list_all_ids_at_q(&format!("{}/messages", server.url()), "AT-1", "from:x")
            .await
            .unwrap_err();
        assert!(matches!(err, GmailError::ReauthRequired));
    }

    #[tokio::test]
    async fn batch_trash_posts_trash_label() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("POST", "/batchModify")
            .match_header("authorization", "Bearer AT-1")
            .match_body(mockito::Matcher::PartialJsonString(
                r#"{"addLabelIds":["TRASH"],"removeLabelIds":["INBOX"],"ids":["a","b"]}"#.into(),
            ))
            .with_status(204)
            .create_async()
            .await;
        batch_trash_at(
            &format!("{}/batchModify", server.url()),
            "AT-1",
            &["a".into(), "b".into()],
        )
        .await
        .unwrap();
        m.assert_async().await;
    }

    #[tokio::test]
    async fn batch_trash_maps_5xx_to_http() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/batchModify")
            .with_status(503)
            .with_body("boom")
            .create_async()
            .await;
        let err = batch_trash_at(
            &format!("{}/batchModify", server.url()),
            "AT-1",
            &["a".into()],
        )
        .await
        .unwrap_err();
        match err {
            GmailError::Http { status, .. } => assert_eq!(status, 503),
            other => panic!("expected Http, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn create_filter_returns_id_on_success() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("POST", "/filters")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::PartialJsonString(
                    r#"{"criteria":{"from":"a@b.com","subject":"Hi"}}"#.into(),
                ),
                mockito::Matcher::PartialJsonString(
                    r#"{"action":{"addLabelIds":["TRASH"],"removeLabelIds":["INBOX"]}}"#.into(),
                ),
            ]))
            .with_status(200)
            .with_body(r#"{"id":"FILTER_1","criteria":{},"action":{}}"#)
            .create_async()
            .await;
        let res = create_filter_at(
            &format!("{}/filters", server.url()),
            "AT-1",
            "a@b.com",
            "Hi",
        )
        .await
        .unwrap();
        assert_eq!(res.id, "FILTER_1");
        m.assert_async().await;
    }

    #[tokio::test]
    async fn create_filter_maps_scope_403_to_reauth() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/filters")
            .with_status(403)
            .with_body(r#"{"error":{"message":"Request had insufficient authentication scopes."}}"#)
            .create_async()
            .await;
        let err = create_filter_at(
            &format!("{}/filters", server.url()),
            "AT-1",
            "a@b.com",
            "Hi",
        )
        .await
        .unwrap_err();
        assert!(matches!(err, GmailError::ReauthRequired));
    }

    #[tokio::test]
    async fn create_filter_maps_non_scope_403_to_http() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/filters")
            .with_status(403)
            .with_body(r#"{"error":{"message":"Number of filters exceeded the limit."}}"#)
            .create_async()
            .await;
        let err = create_filter_at(
            &format!("{}/filters", server.url()),
            "AT-1",
            "a@b.com",
            "Hi",
        )
        .await
        .unwrap_err();
        match err {
            GmailError::Http { status, .. } => assert_eq!(status, 403),
            other => panic!("expected Http, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn list_filters_at_parses_filter_array() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/filters")
            .with_status(200)
            .with_body(r#"{"filter":[{"id":"f1","criteria":{"from":"a@b.com"}}]}"#)
            .create_async()
            .await;
        let res = list_filters_at(&format!("{}/filters", server.url()), "AT-1")
            .await
            .unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].id, "f1");
        assert_eq!(res[0].criteria.from.as_deref(), Some("a@b.com"));
    }

    #[tokio::test]
    async fn list_filters_at_returns_empty_when_key_absent() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/filters")
            .with_status(200)
            .with_body("{}")
            .create_async()
            .await;
        let res = list_filters_at(&format!("{}/filters", server.url()), "AT-1")
            .await
            .unwrap();
        assert!(res.is_empty());
    }

    #[tokio::test]
    async fn list_filters_at_maps_401_to_reauth() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/filters")
            .with_status(401)
            .create_async()
            .await;
        let err = list_filters_at(&format!("{}/filters", server.url()), "AT-1")
            .await
            .unwrap_err();
        assert!(matches!(err, GmailError::ReauthRequired));
    }

    #[tokio::test]
    async fn delete_filter_at_sends_delete_to_id_path() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("DELETE", "/filters/f1")
            .with_status(204)
            .create_async()
            .await;
        delete_filter_at(&format!("{}/filters", server.url()), "AT-1", "f1")
            .await
            .unwrap();
        m.assert_async().await;
    }

    #[tokio::test]
    async fn delete_filter_at_maps_401_to_reauth() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("DELETE", "/filters/f1")
            .with_status(401)
            .create_async()
            .await;
        let err = delete_filter_at(&format!("{}/filters", server.url()), "AT-1", "f1")
            .await
            .unwrap_err();
        assert!(matches!(err, GmailError::ReauthRequired));
    }

    #[tokio::test]
    async fn delete_filter_at_maps_5xx_to_http_error() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("DELETE", "/filters/f1")
            .with_status(503)
            .create_async()
            .await;
        let err = delete_filter_at(&format!("{}/filters", server.url()), "AT-1", "f1")
            .await
            .unwrap_err();
        match err {
            GmailError::Http { status, .. } => assert_eq!(status, 503),
            other => panic!("expected Http, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn ensure_label_finds_existing_without_creating() {
        let mut server = mockito::Server::new_async().await;
        let list_mock = server
            .mock("GET", "/labels")
            .with_status(200)
            .with_body(r#"{"labels":[{"id":"Label_1","name":"Задача"}]}"#)
            .create_async()
            .await;
        let create_mock = server
            .mock("POST", "/labels")
            .expect(0)
            .create_async()
            .await;
        let id = ensure_label(
            &format!("{}/labels", server.url()),
            "AT-1",
            "Задача",
            "#fb4c2f",
            "#ffffff",
        )
        .await
        .unwrap();
        assert_eq!(id, "Label_1");
        list_mock.assert_async().await;
        create_mock.assert_async().await;
    }

    #[tokio::test]
    async fn ensure_label_creates_when_absent() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/labels")
            .with_status(200)
            .with_body(r#"{"labels":[]}"#)
            .create_async()
            .await;
        server
            .mock("POST", "/labels")
            .match_body(mockito::Matcher::PartialJsonString(
                r#"{"name":"Задача"}"#.into(),
            ))
            .with_status(200)
            .with_body(r#"{"id":"Label_new"}"#)
            .create_async()
            .await;
        let id = ensure_label(
            &format!("{}/labels", server.url()),
            "AT-1",
            "Задача",
            "#fb4c2f",
            "#ffffff",
        )
        .await
        .unwrap();
        assert_eq!(id, "Label_new");
    }

    #[tokio::test]
    async fn modify_message_posts_add_and_remove_label_ids() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("POST", "/batchModify")
            .match_header("authorization", "Bearer AT-1")
            .match_body(mockito::Matcher::PartialJsonString(
                r#"{"ids":["m1"],"addLabelIds":["Label_1"],"removeLabelIds":[]}"#.into(),
            ))
            .with_status(204)
            .create_async()
            .await;
        modify_message(
            &format!("{}/batchModify", server.url()),
            "AT-1",
            "m1",
            &["Label_1"],
            &[],
        )
        .await
        .unwrap();
        m.assert_async().await;
    }

    #[tokio::test]
    async fn modify_message_maps_401_to_reauth() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/batchModify")
            .with_status(401)
            .create_async()
            .await;
        let err = modify_message(
            &format!("{}/batchModify", server.url()),
            "AT-1",
            "m1",
            &[],
            &["Label_1"],
        )
        .await
        .unwrap_err();
        assert!(matches!(err, GmailError::ReauthRequired));
    }
}
