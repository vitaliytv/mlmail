use crate::auth::error::StorageError;
use crate::auth::storage::{RefreshTokenStorage, StoredSession};
use std::sync::Mutex;

pub struct InMemoryStorage {
    inner: Mutex<Option<StoredSession>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self { inner: Mutex::new(None) }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl RefreshTokenStorage for InMemoryStorage {
    fn save(&self, email: &str, refresh_token: &str) -> Result<(), StorageError> {
        let mut g = self.inner.lock()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        *g = Some(StoredSession {
            email: email.into(),
            refresh_token: refresh_token.into(),
        });
        Ok(())
    }

    fn load(&self) -> Result<Option<StoredSession>, StorageError> {
        let g = self.inner.lock()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(g.clone())
    }

    fn clear(&self) -> Result<(), StorageError> {
        let mut g = self.inner.lock()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        *g = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::storage::{RefreshTokenStorage, StoredSession};

    #[test]
    fn load_returns_none_when_nothing_was_saved() {
        let s = InMemoryStorage::new();
        assert_eq!(s.load().unwrap(), None);
    }

    #[test]
    fn load_returns_saved_session() {
        let s = InMemoryStorage::new();
        s.save("u@example.com", "rt-1").unwrap();
        assert_eq!(
            s.load().unwrap(),
            Some(StoredSession {
                email: "u@example.com".into(),
                refresh_token: "rt-1".into()
            })
        );
    }

    #[test]
    fn second_save_overwrites_first() {
        let s = InMemoryStorage::new();
        s.save("a@x", "rt-1").unwrap();
        s.save("a@x", "rt-2").unwrap();
        assert_eq!(s.load().unwrap().unwrap().refresh_token, "rt-2");
    }

    #[test]
    fn clear_removes_saved_session() {
        let s = InMemoryStorage::new();
        s.save("u@example.com", "rt-1").unwrap();
        s.clear().unwrap();
        assert_eq!(s.load().unwrap(), None);
    }

    #[test]
    fn clear_on_empty_storage_is_noop() {
        let s = InMemoryStorage::new();
        s.clear().unwrap();
        assert_eq!(s.load().unwrap(), None);
    }
}
