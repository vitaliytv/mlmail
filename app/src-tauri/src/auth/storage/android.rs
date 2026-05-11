use crate::auth::error::StorageError;
use crate::auth::storage::{RefreshTokenStorage, StoredSession};
use tauri::AppHandle;

pub struct EncryptedPrefs {
    app: AppHandle,
}

impl EncryptedPrefs {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl RefreshTokenStorage for EncryptedPrefs {
    fn save(&self, email: &str, refresh_token: &str) -> Result<(), StorageError> {
        crate::auth::flow::android::call_save_session(&self.app, email, refresh_token)
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn load(&self) -> Result<Option<StoredSession>, StorageError> {
        crate::auth::flow::android::call_load_session(&self.app)
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn clear(&self) -> Result<(), StorageError> {
        crate::auth::flow::android::call_clear_session(&self.app)
            .map_err(|e| StorageError::Backend(e.to_string()))
    }
}
