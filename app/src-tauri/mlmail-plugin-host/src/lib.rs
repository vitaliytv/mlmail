//! Domain plugin host for mlmail — metadata read + draft create over Gmail.
//!
//! Wraps [`plugin_mail::MailHost`] with Gmail HTTP and [`GrantGatedMailHost`].
//! Mutating `mail:draft.create` is audited via [`AuditStore`] (30d, no body).

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use plugin_mail::{
    scope_draft_account, scope_metadata_message, DraftCreateRequest, DraftCreateResult,
    GrantGatedMailHost, MailError, MailHost, MessageMetadata,
};
use plugin_permissions::{AuditEntry, AuditStore, Grant, GrantStore};
use plugin_runtime::{PluginHandle, PluginRuntime, RuntimeError};
use serde_json::{json, Value};

pub use plugin_a2ui::{
    sample_sidebar_stream, validate_stream, SurfaceState, CATALOG_NITRA_CORE, PROTOCOL, SCHEMA_REV,
};
pub use plugin_mail::{MockMailHost, CAP_MAIL_DRAFT_CREATE, CAP_MAIL_METADATA_READ};
pub use plugin_permissions::{audit_store_path, AUDIT_RETENTION_SECS};
pub use plugin_runtime::{ResourceLimits, DRAFT_HELPER_WAT, MAIL_READER_WAT};

mod manager;
pub use manager::*;

/// Errors from the mlmail plugin host façade.
#[derive(Debug, thiserror::Error)]
pub enum HostError {
    #[error(transparent)]
    Mail(#[from] MailError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Permissions(#[from] plugin_permissions::PermissionsError),
    #[error(transparent)]
    A2ui(#[from] plugin_a2ui::A2uiError),
    #[error("gmail http {status}: {body}")]
    Http { status: u16, body: String },
    #[error("network: {0}")]
    Network(String),
    #[error("parse: {0}")]
    Parse(String),
}

impl From<reqwest::Error> for HostError {
    fn from(value: reqwest::Error) -> Self {
        HostError::Network(value.to_string())
    }
}

/// Sync Gmail client (metadata + drafts) for [`MailHost`].
#[derive(Debug, Clone)]
pub struct GmailMailHost {
    /// Base URL ending at `/messages` (same as mlmail `Endpoints.gmail_messages_list`).
    pub messages_base: String,
    pub access_token: String,
}

impl GmailMailHost {
    pub fn new(messages_base: impl Into<String>, access_token: impl Into<String>) -> Self {
        Self {
            messages_base: messages_base.into(),
            access_token: access_token.into(),
        }
    }

    fn drafts_url(&self) -> String {
        let base = self.messages_base.trim_end_matches('/');
        if let Some(prefix) = base.strip_suffix("/messages") {
            format!("{prefix}/drafts")
        } else {
            format!("{base}/../drafts")
        }
    }

    fn fetch_meta(&self, message_id: &str) -> Result<MessageMetadata, HostError> {
        let url = format!("{}/{message_id}", self.messages_base.trim_end_matches('/'));
        let resp = reqwest::blocking::Client::new()
            .get(&url)
            .bearer_auth(&self.access_token)
            .query(&[
                ("format", "metadata"),
                ("metadataHeaders", "From"),
                ("metadataHeaders", "Subject"),
                ("metadataHeaders", "Date"),
            ])
            .send()?;

        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        if !status.is_success() {
            return Err(HostError::Http {
                status: status.as_u16(),
                body,
            });
        }

        let v: Value = serde_json::from_str(&body).map_err(|e| HostError::Parse(e.to_string()))?;
        let empty: Vec<Value> = Vec::new();
        let headers = v
            .get("payload")
            .and_then(|p| p.get("headers"))
            .and_then(|h| h.as_array())
            .unwrap_or(&empty);

        Ok(MessageMetadata {
            id: message_id.to_string(),
            from: extract_header(headers, "From"),
            subject: extract_header(headers, "Subject"),
            date: extract_header(headers, "Date"),
        })
    }

    fn post_draft(&self, req: &DraftCreateRequest) -> Result<DraftCreateResult, HostError> {
        let raw = encode_rfc822_raw(req);
        let payload = json!({ "message": { "raw": raw } });
        let resp = reqwest::blocking::Client::new()
            .post(self.drafts_url())
            .bearer_auth(&self.access_token)
            .json(&payload)
            .send()?;

        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        if !status.is_success() {
            return Err(HostError::Http {
                status: status.as_u16(),
                body,
            });
        }

        let v: Value = serde_json::from_str(&body).map_err(|e| HostError::Parse(e.to_string()))?;
        let draft_id = v
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| HostError::Parse("draft id missing".into()))?
            .to_string();
        Ok(DraftCreateResult { draft_id })
    }
}

impl MailHost for GmailMailHost {
    fn get_message_metadata(&self, message_id: &str) -> Result<MessageMetadata, MailError> {
        self.fetch_meta(message_id).map_err(host_to_mail)
    }

    fn create_draft(&self, req: &DraftCreateRequest) -> Result<DraftCreateResult, MailError> {
        self.post_draft(req).map_err(host_to_mail)
    }
}

fn host_to_mail(e: HostError) -> MailError {
    match e {
        HostError::Http { status: 401, .. } => MailError::Unavailable("reauth required".into()),
        HostError::Http { status: 404, body } => MailError::NotFound(body),
        HostError::Http { status, body } => {
            MailError::Unavailable(format!("http {status}: {body}"))
        }
        HostError::Network(m) | HostError::Parse(m) => MailError::Unavailable(m),
        HostError::Mail(m) => m,
        other => MailError::Other(other.to_string()),
    }
}

fn encode_rfc822_raw(req: &DraftCreateRequest) -> String {
    let msg = format!(
        "To: {}\r\nSubject: {}\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\n{}",
        req.to, req.subject, req.body
    );
    URL_SAFE_NO_PAD.encode(msg.as_bytes())
}

fn extract_header(headers: &[Value], name: &str) -> String {
    let target = name.to_ascii_lowercase();
    for h in headers {
        let key = h.get("name").and_then(Value::as_str).unwrap_or("");
        if key.to_ascii_lowercase() == target {
            return h
                .get("value")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
        }
    }
    String::new()
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Orchestrates runtime + grants + audit for a single plugin/user pair.
pub struct MailPluginSession {
    pub runtime: PluginRuntime,
    pub grants: Arc<Mutex<GrantStore>>,
    pub audit: Arc<Mutex<AuditStore>>,
    pub plugin_id: String,
    pub plugin_version: String,
    pub user_id: String,
}

impl MailPluginSession {
    pub fn new(
        plugin_id: impl Into<String>,
        plugin_version: impl Into<String>,
        user_id: impl Into<String>,
        grants: GrantStore,
        audit: AuditStore,
        limits: ResourceLimits,
    ) -> Result<Self, HostError> {
        Ok(Self {
            runtime: PluginRuntime::new(limits)?,
            grants: Arc::new(Mutex::new(grants)),
            audit: Arc::new(Mutex::new(audit)),
            plugin_id: plugin_id.into(),
            plugin_version: plugin_version.into(),
            user_id: user_id.into(),
        })
    }

    pub fn grant_metadata_message(&self, message_id: &str) -> Result<(), HostError> {
        let mut grants = self.grants.lock().expect("grants");
        grants.grant(Grant {
            plugin_id: self.plugin_id.clone(),
            user_id: self.user_id.clone(),
            scope: scope_metadata_message(message_id),
            granted_at_unix: now_unix(),
        })?;
        Ok(())
    }

    pub fn grant_draft_account(&self, account_id: &str) -> Result<(), HostError> {
        let mut grants = self.grants.lock().expect("grants");
        grants.grant(Grant {
            plugin_id: self.plugin_id.clone(),
            user_id: self.user_id.clone(),
            scope: scope_draft_account(account_id),
            granted_at_unix: now_unix(),
        })?;
        Ok(())
    }

    pub fn load_sample_reader(&self) -> Result<PluginHandle, HostError> {
        Ok(self.runtime.load_wat(&self.plugin_id, MAIL_READER_WAT)?)
    }

    pub fn load_sample_draft_helper(&self) -> Result<PluginHandle, HostError> {
        Ok(self.runtime.load_wat(&self.plugin_id, DRAFT_HELPER_WAT)?)
    }

    /// Gate `inner` with this session's grants and run sample `read_meta`.
    pub fn read_meta_via_sample(
        &self,
        handle: &PluginHandle,
        inner: Arc<dyn MailHost>,
    ) -> Result<MessageMetadata, HostError> {
        let gated: Arc<dyn MailHost> = Arc::new(GrantGatedMailHost::new(
            InnerMail(inner),
            Arc::clone(&self.grants),
            self.plugin_id.clone(),
            self.user_id.clone(),
        ));
        Ok(self.runtime.read_meta_via_plugin(handle, gated)?)
    }

    /// Run sample `handle_action` → create_draft and append an audit entry.
    pub fn create_draft_via_sample(
        &self,
        handle: &PluginHandle,
        inner: Arc<dyn MailHost>,
        correlation_id: &str,
    ) -> Result<DraftCreateResult, HostError> {
        let gated: Arc<dyn MailHost> = Arc::new(GrantGatedMailHost::new(
            InnerMail(inner),
            Arc::clone(&self.grants),
            self.plugin_id.clone(),
            self.user_id.clone(),
        ));
        let scope = scope_draft_account("acct_1");
        let outcome = self.runtime.handle_action_via_plugin(handle, gated);
        let (result_str, mapped) = match outcome {
            Ok(r) => ("ok".to_string(), Ok(r)),
            Err(e) => {
                let msg = e.to_string();
                let result = if msg.to_lowercase().contains("denied") {
                    format!("denied:{msg}")
                } else {
                    format!("error:{msg}")
                };
                (result, Err(HostError::from(e)))
            }
        };

        {
            let mut audit = self.audit.lock().expect("audit");
            audit.append(
                AuditEntry {
                    plugin_id: self.plugin_id.clone(),
                    plugin_version: self.plugin_version.clone(),
                    action_id: "createDraft".into(),
                    capability: CAP_MAIL_DRAFT_CREATE.to_string(),
                    scope,
                    result: result_str,
                    correlation_id: correlation_id.to_string(),
                    timestamp_unix: now_unix(),
                },
                now_unix(),
            )?;
        }

        mapped
    }
}

/// Validate the sample sidebar A2UI stream and return the surface for Vue.
pub fn sample_sidebar_surface() -> Result<SurfaceState, HostError> {
    let reg = validate_stream(&sample_sidebar_stream())?;
    reg.get("sidebar.draft-helper")
        .cloned()
        .ok_or_else(|| HostError::Parse("sidebar.draft-helper missing after validate".into()))
}

/// Validate the mlmail-only detail A2UI fixture and return surface `detail.draft-helper`.
pub fn sample_detail_surface() -> Result<SurfaceState, HostError> {
    let messages: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../fixtures/detail_sample.json"))
            .map_err(|e| HostError::Parse(e.to_string()))?;
    let reg = validate_stream(&messages)?;
    reg.get("detail.draft-helper")
        .cloned()
        .ok_or_else(|| HostError::Parse("detail.draft-helper missing after validate".into()))
}

/// Demo path for Tauri UI: mock draft + audit without a live Gmail token.
pub fn demo_create_draft_with_audit(
    app_data: &std::path::Path,
) -> Result<(DraftCreateResult, AuditEntry), HostError> {
    let grants = GrantStore::open(app_data.join("plugins").join("grants-demo.json"))?;
    let audit = AuditStore::open(audit_store_path(app_data))?;
    let session = MailPluginSession::new(
        "com.example.draft-helper",
        "0.1.0",
        "demo-user",
        grants,
        audit,
        ResourceLimits {
            wall_clock: std::time::Duration::from_secs(5),
            ..ResourceLimits::default()
        },
    )?;
    session.grant_draft_account("acct_1")?;
    let handle = session.load_sample_draft_helper()?;
    session.runtime.activate(&handle)?;
    let mock: Arc<dyn MailHost> = Arc::new(MockMailHost::default());
    let result = session.create_draft_via_sample(&handle, mock, "ui-sidebar")?;
    let entry = session
        .audit
        .lock()
        .expect("audit")
        .list()
        .last()
        .cloned()
        .ok_or_else(|| HostError::Parse("audit entry missing".into()))?;
    Ok((result, entry))
}

/// Thin adapter so `GrantGatedMailHost` can wrap `Arc<dyn MailHost>`.
struct InnerMail(Arc<dyn MailHost>);

impl MailHost for InnerMail {
    fn get_message_metadata(&self, message_id: &str) -> Result<MessageMetadata, MailError> {
        self.0.get_message_metadata(message_id)
    }

    fn create_draft(&self, req: &DraftCreateRequest) -> Result<DraftCreateResult, MailError> {
        self.0.create_draft(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn meta_body() -> &'static str {
        r#"{"payload":{"headers":[
          {"name":"From","value":"a@example.com"},
          {"name":"Subject","value":"Hello from Gmail"},
          {"name":"Date","value":"Mon, 3 Aug 2026"}
        ]}}"#
    }

    fn session_for(dir: &tempfile::TempDir) -> MailPluginSession {
        MailPluginSession::new(
            "com.example.mail-reader",
            "0.1.0",
            "u1",
            GrantStore::open(dir.path().join("grants.json")).unwrap(),
            AuditStore::open(dir.path().join("audit.json")).unwrap(),
            ResourceLimits {
                wall_clock: std::time::Duration::from_secs(5),
                ..ResourceLimits::default()
            },
        )
        .unwrap()
    }

    #[test]
    fn gmail_host_fetches_metadata() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/messages/msg_1")
            .match_header("authorization", "Bearer tok")
            .match_query(mockito::Matcher::Regex("format=metadata".into()))
            .with_status(200)
            .with_body(meta_body())
            .create();

        let host = GmailMailHost::new(format!("{}/messages", server.url()), "tok");
        let meta = host.get_message_metadata("msg_1").unwrap();
        assert_eq!(meta.subject, "Hello from Gmail");
        assert_eq!(meta.from, "a@example.com");
    }

    #[test]
    fn gmail_host_creates_draft() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("POST", "/drafts")
            .match_header("authorization", "Bearer tok")
            .with_status(200)
            .with_body(r#"{"id":"draft_gmail_1"}"#)
            .create();

        let host = GmailMailHost::new(format!("{}/messages", server.url()), "tok");
        let result = host
            .create_draft(&DraftCreateRequest {
                account_id: "acct_1".into(),
                to: "b@example.com".into(),
                subject: "Re: Hello".into(),
                body: "Thanks".into(),
            })
            .unwrap();
        assert_eq!(result.draft_id, "draft_gmail_1");
    }

    #[test]
    fn grant_gated_gmail_without_wasm() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/messages/msg_1")
            .match_header("authorization", "Bearer tok")
            .match_query(mockito::Matcher::Regex("format=metadata".into()))
            .with_status(200)
            .with_body(meta_body())
            .create();

        let dir = tempdir().unwrap();
        let grants = Arc::new(Mutex::new(
            GrantStore::open(dir.path().join("grants.json")).unwrap(),
        ));
        {
            let mut g = grants.lock().unwrap();
            g.grant(Grant {
                plugin_id: "com.example.mail-reader".into(),
                user_id: "u1".into(),
                scope: scope_metadata_message("msg_1"),
                granted_at_unix: 1,
            })
            .unwrap();
        }
        let host = GrantGatedMailHost::new(
            GmailMailHost::new(format!("{}/messages", server.url()), "tok"),
            grants,
            "com.example.mail-reader",
            "u1",
        );
        let meta = host.get_message_metadata("msg_1").unwrap();
        assert_eq!(meta.subject, "Hello from Gmail");
    }

    #[test]
    fn session_sample_plugin_reads_with_mock() {
        let dir = tempdir().unwrap();
        let session = session_for(&dir);
        session.grant_metadata_message("msg_1").unwrap();

        let handle = session.load_sample_reader().unwrap();
        session.runtime.activate(&handle).unwrap();

        let mock: Arc<dyn MailHost> = Arc::new(MockMailHost {
            messages: vec![MessageMetadata {
                id: "msg_1".into(),
                from: "a@example.com".into(),
                subject: "Hello from Mock".into(),
                date: "Mon, 3 Aug 2026".into(),
            }],
            ..Default::default()
        });
        let meta = session.read_meta_via_sample(&handle, mock).unwrap();
        assert_eq!(meta.subject, "Hello from Mock");
    }

    #[test]
    fn session_sample_plugin_reads_with_grant() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/messages/msg_1")
            .match_header("authorization", "Bearer tok")
            .match_query(mockito::Matcher::Regex("format=metadata".into()))
            .with_status(200)
            .with_body(meta_body())
            .create();

        let dir = tempdir().unwrap();
        let session = session_for(&dir);
        session.grant_metadata_message("msg_1").unwrap();

        let handle = session.load_sample_reader().unwrap();
        session.runtime.activate(&handle).unwrap();

        let gmail: Arc<dyn MailHost> = Arc::new(GmailMailHost::new(
            format!("{}/messages", server.url()),
            "tok",
        ));
        let meta = session.read_meta_via_sample(&handle, gmail).unwrap();
        assert_eq!(meta.subject, "Hello from Gmail");
    }

    #[test]
    fn session_sample_plugin_denied_without_grant() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/messages/msg_1")
            .with_status(200)
            .with_body(meta_body())
            .create();

        let dir = tempdir().unwrap();
        let session = session_for(&dir);

        let handle = session.load_sample_reader().unwrap();
        let gmail: Arc<dyn MailHost> = Arc::new(GmailMailHost::new(
            format!("{}/messages", server.url()),
            "tok",
        ));
        let err = session.read_meta_via_sample(&handle, gmail).unwrap_err();
        assert!(
            err.to_string().to_lowercase().contains("denied")
                || matches!(err, HostError::Mail(MailError::Denied(_)))
                || matches!(err, HostError::Runtime(_)),
            "expected denied, got {err}"
        );
    }

    #[test]
    fn sample_sidebar_a2ui_validates() {
        let surface = sample_sidebar_surface().unwrap();
        assert_eq!(surface.surface_id, "sidebar.draft-helper");
        assert_eq!(surface.catalog_id, CATALOG_NITRA_CORE);
        assert!(surface.root().is_some());
        assert!(surface.components.contains_key("title"));
    }

    #[test]
    fn sample_detail_a2ui_validates() {
        let surface = sample_detail_surface().unwrap();
        assert_eq!(surface.surface_id, "detail.draft-helper");
        assert_eq!(surface.catalog_id, CATALOG_NITRA_CORE);
        assert!(surface.root().is_some());
        assert!(surface.components.contains_key("title"));
        assert!(surface.components.contains_key("draft_btn"));
    }

    #[test]
    fn session_create_draft_audited() {
        let dir = tempdir().unwrap();
        let session = MailPluginSession::new(
            "com.example.draft-helper",
            "0.1.0",
            "u1",
            GrantStore::open(dir.path().join("grants.json")).unwrap(),
            AuditStore::open(dir.path().join("audit.json")).unwrap(),
            ResourceLimits {
                wall_clock: std::time::Duration::from_secs(5),
                ..ResourceLimits::default()
            },
        )
        .unwrap();
        session.grant_draft_account("acct_1").unwrap();
        let handle = session.load_sample_draft_helper().unwrap();
        session.runtime.activate(&handle).unwrap();
        let mock: Arc<dyn MailHost> = Arc::new(MockMailHost::default());
        let result = session
            .create_draft_via_sample(&handle, mock, "corr-1")
            .unwrap();
        assert_eq!(result.draft_id, "draft_1");
        let audit = session.audit.lock().unwrap();
        assert_eq!(audit.list().len(), 1);
        assert_eq!(audit.list()[0].result, "ok");
        assert_eq!(audit.list()[0].action_id, "createDraft");
        assert_eq!(audit.list()[0].capability, CAP_MAIL_DRAFT_CREATE);
        assert!(!audit.list()[0].result.contains("Thanks"));
    }

    #[test]
    fn session_create_draft_denied_still_audited() {
        let dir = tempdir().unwrap();
        let session = MailPluginSession::new(
            "com.example.draft-helper",
            "0.1.0",
            "u1",
            GrantStore::open(dir.path().join("grants.json")).unwrap(),
            AuditStore::open(dir.path().join("audit.json")).unwrap(),
            ResourceLimits {
                wall_clock: std::time::Duration::from_secs(5),
                ..ResourceLimits::default()
            },
        )
        .unwrap();
        let handle = session.load_sample_draft_helper().unwrap();
        let mock: Arc<dyn MailHost> = Arc::new(MockMailHost::default());
        let err = session
            .create_draft_via_sample(&handle, mock, "corr-2")
            .unwrap_err();
        assert!(
            err.to_string().to_lowercase().contains("denied")
                || matches!(err, HostError::Runtime(_))
        );
        let audit = session.audit.lock().unwrap();
        assert_eq!(audit.list().len(), 1);
        assert!(audit.list()[0].result.starts_with("denied"));
    }

    #[test]
    fn demo_create_draft_with_audit_smoke() {
        let dir = tempdir().unwrap();
        let (result, entry) = demo_create_draft_with_audit(dir.path()).unwrap();
        assert_eq!(result.draft_id, "draft_1");
        assert_eq!(entry.result, "ok");
    }
}
