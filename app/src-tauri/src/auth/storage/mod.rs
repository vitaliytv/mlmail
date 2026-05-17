use crate::auth::error::StorageError;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredSession {
    pub email: String,
    pub refresh_token: String,
}

pub trait RefreshTokenStorage: Send + Sync {
    fn save(&self, email: &str, refresh_token: &str) -> Result<(), StorageError>;
    fn load(&self) -> Result<Option<StoredSession>, StorageError>;
    fn clear(&self) -> Result<(), StorageError>;
}

pub type SharedStorage = Arc<dyn RefreshTokenStorage>;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "android")]
pub mod android;

pub mod in_memory;

#[cfg(target_os = "macos")]
pub fn platform_storage() -> SharedStorage {
    Arc::new(macos::Keychain::new())
}

#[cfg(target_os = "android")]
pub fn platform_storage(app: &tauri::AppHandle) -> SharedStorage {
    Arc::new(android::EncryptedPrefs::new(app.clone()))
}
