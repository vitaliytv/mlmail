use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AuthError {
    #[error("login cancelled by user")]
    Cancelled,
    #[error("network error: {0}")]
    Network(String),
    #[error("oauth error: {0}")]
    OAuth(String),
    #[error("oauth client not configured: {0}")]
    ConfigMissing(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("re-authentication required")]
    ReauthRequired,
    #[error("platform error: {0}")]
    Platform(String),
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("backend error: {0}")]
    Backend(String),
}

impl From<StorageError> for AuthError {
    fn from(e: StorageError) -> Self {
        AuthError::Storage(e.to_string())
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(e: reqwest::Error) -> Self {
        AuthError::Network(e.to_string())
    }
}
