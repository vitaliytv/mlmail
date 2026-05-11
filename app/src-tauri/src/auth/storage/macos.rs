use crate::auth::error::StorageError;
use crate::auth::storage::{RefreshTokenStorage, StoredSession};
use keyring::Entry;

const SERVICE: &str = "com.vitaliytv.mlmail";
const REFRESH_KEY: &str = "google.refresh_token";
const EMAIL_KEY: &str = "google.email";

pub struct Keychain;

impl Keychain {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Keychain {
    fn default() -> Self {
        Self::new()
    }
}

fn to_backend_err(e: keyring::Error) -> StorageError {
    StorageError::Backend(e.to_string())
}

impl RefreshTokenStorage for Keychain {
    fn save(&self, email: &str, refresh_token: &str) -> Result<(), StorageError> {
        Entry::new(SERVICE, EMAIL_KEY)
            .map_err(to_backend_err)?
            .set_password(email)
            .map_err(to_backend_err)?;
        Entry::new(SERVICE, REFRESH_KEY)
            .map_err(to_backend_err)?
            .set_password(refresh_token)
            .map_err(to_backend_err)?;
        Ok(())
    }

    fn load(&self) -> Result<Option<StoredSession>, StorageError> {
        let email = match Entry::new(SERVICE, EMAIL_KEY)
            .map_err(to_backend_err)?
            .get_password()
        {
            Ok(v) => v,
            Err(keyring::Error::NoEntry) => return Ok(None),
            Err(e) => return Err(to_backend_err(e)),
        };
        let refresh_token = match Entry::new(SERVICE, REFRESH_KEY)
            .map_err(to_backend_err)?
            .get_password()
        {
            Ok(v) => v,
            Err(keyring::Error::NoEntry) => return Ok(None),
            Err(e) => return Err(to_backend_err(e)),
        };
        Ok(Some(StoredSession { email, refresh_token }))
    }

    fn clear(&self) -> Result<(), StorageError> {
        let _ = Entry::new(SERVICE, EMAIL_KEY)
            .map_err(to_backend_err)?
            .delete_credential();
        let _ = Entry::new(SERVICE, REFRESH_KEY)
            .map_err(to_backend_err)?
            .delete_credential();
        Ok(())
    }
}
