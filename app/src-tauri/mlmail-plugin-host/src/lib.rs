//! Domain plugin host for mlmail — `mail:metadata.read` over Gmail.
//!
//! Wraps [`plugin_mail::MailHost`] with Gmail `format=metadata` HTTP and
//! [`GrantGatedMailHost`] so Wasm plugins only see headers they are granted.

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use plugin_mail::{
    scope_metadata_message, GrantGatedMailHost, MailError, MailHost, MessageMetadata,
};
use plugin_permissions::{Grant, GrantStore};
use plugin_runtime::{PluginHandle, PluginRuntime, RuntimeError};
use serde_json::Value;

pub use plugin_a2ui::{
    sample_sidebar_stream, validate_stream, SurfaceState, CATALOG_NITRA_CORE, PROTOCOL, SCHEMA_REV,
};
pub use plugin_mail::{MockMailHost, CAP_MAIL_METADATA_READ};
pub use plugin_runtime::{ResourceLimits, MAIL_READER_WAT};

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

/// Sync Gmail metadata client (no body) for [`MailHost`].
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
}

impl MailHost for GmailMailHost {
    fn get_message_metadata(&self, message_id: &str) -> Result<MessageMetadata, MailError> {
        self.fetch_meta(message_id).map_err(|e| match e {
            HostError::Http { status: 401, .. } => MailError::Unavailable("reauth required".into()),
            HostError::Http { status: 404, .. } => MailError::NotFound(message_id.to_string()),
            HostError::Http { status, body } => {
                MailError::Unavailable(format!("http {status}: {body}"))
            }
            HostError::Network(m) | HostError::Parse(m) => MailError::Unavailable(m),
            HostError::Mail(m) => m,
            other => MailError::Other(other.to_string()),
        })
    }
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

/// Orchestrates runtime + grants for a single plugin/user pair.
pub struct MailPluginSession {
    pub runtime: PluginRuntime,
    pub grants: Arc<Mutex<GrantStore>>,
    pub plugin_id: String,
    pub user_id: String,
}

impl MailPluginSession {
    pub fn new(
        plugin_id: impl Into<String>,
        user_id: impl Into<String>,
        grants: GrantStore,
        limits: ResourceLimits,
    ) -> Result<Self, HostError> {
        Ok(Self {
            runtime: PluginRuntime::new(limits)?,
            grants: Arc::new(Mutex::new(grants)),
            plugin_id: plugin_id.into(),
            user_id: user_id.into(),
        })
    }

    pub fn grant_metadata_message(&self, message_id: &str) -> Result<(), HostError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut grants = self.grants.lock().expect("grants");
        grants.grant(Grant {
            plugin_id: self.plugin_id.clone(),
            user_id: self.user_id.clone(),
            scope: scope_metadata_message(message_id),
            granted_at_unix: now,
        })?;
        Ok(())
    }

    pub fn load_sample_reader(&self) -> Result<PluginHandle, HostError> {
        Ok(self.runtime.load_wat(&self.plugin_id, MAIL_READER_WAT)?)
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
}

/// Validate the sample sidebar A2UI stream and return the surface for Vue.
pub fn sample_sidebar_surface() -> Result<SurfaceState, HostError> {
    let reg = validate_stream(&sample_sidebar_stream())?;
    reg.get("sidebar.draft-helper")
        .cloned()
        .ok_or_else(|| HostError::Parse("sidebar.draft-helper missing after validate".into()))
}

/// Thin adapter so `GrantGatedMailHost` can wrap `Arc<dyn MailHost>`.
struct InnerMail(Arc<dyn MailHost>);

impl MailHost for InnerMail {
    fn get_message_metadata(&self, message_id: &str) -> Result<MessageMetadata, MailError> {
        self.0.get_message_metadata(message_id)
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
        let grants = GrantStore::open(dir.path().join("grants.json")).unwrap();
        let session = MailPluginSession::new(
            "com.example.mail-reader",
            "u1",
            grants,
            ResourceLimits {
                wall_clock: std::time::Duration::from_secs(5),
                ..ResourceLimits::default()
            },
        )
        .unwrap();
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
        let grants = GrantStore::open(dir.path().join("grants.json")).unwrap();
        let session = MailPluginSession::new(
            "com.example.mail-reader",
            "u1",
            grants,
            ResourceLimits {
                wall_clock: std::time::Duration::from_secs(5),
                ..ResourceLimits::default()
            },
        )
        .unwrap();
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
        let grants = GrantStore::open(dir.path().join("grants.json")).unwrap();
        let session = MailPluginSession::new(
            "com.example.mail-reader",
            "u1",
            grants,
            ResourceLimits {
                wall_clock: std::time::Duration::from_secs(5),
                ..ResourceLimits::default()
            },
        )
        .unwrap();

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
}
