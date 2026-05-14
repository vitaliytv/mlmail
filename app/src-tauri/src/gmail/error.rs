use crate::auth::error::AuthError;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum GmailError {
    #[error("network error: {0}")]
    Network(String),
    #[error("gmail http {status}: {body}")]
    Http { status: u16, body: String },
    #[error("could not parse gmail response: {0}")]
    Parse(String),
    #[error("re-authentication required")]
    ReauthRequired,
    #[error("platform error: {0}")]
    Platform(String),
}

impl From<reqwest::Error> for GmailError {
    fn from(e: reqwest::Error) -> Self {
        GmailError::Network(e.to_string())
    }
}

impl From<AuthError> for GmailError {
    fn from(e: AuthError) -> Self {
        match e {
            AuthError::ReauthRequired => GmailError::ReauthRequired,
            AuthError::Network(m) => GmailError::Network(m),
            AuthError::Platform(m) => GmailError::Platform(m),
            other => GmailError::Platform(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_reauth_maps_to_gmail_reauth() {
        let e: GmailError = AuthError::ReauthRequired.into();
        assert!(matches!(e, GmailError::ReauthRequired));
    }

    #[test]
    fn auth_network_maps_to_gmail_network() {
        let e: GmailError = AuthError::Network("dns".into()).into();
        match e {
            GmailError::Network(m) => assert_eq!(m, "dns"),
            _ => panic!("expected Network"),
        }
    }

    #[test]
    fn serializes_with_tagged_kind() {
        let e = GmailError::Http { status: 503, body: "boom".into() };
        let s = serde_json::to_string(&e).unwrap();
        assert!(s.contains("\"kind\":\"Http\""));
        assert!(s.contains("\"status\":503"));
    }
}
