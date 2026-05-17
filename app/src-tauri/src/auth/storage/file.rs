use crate::auth::error::StorageError;
use crate::auth::storage::{RefreshTokenStorage, StoredSession};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
struct FileSession {
    email: String,
    refresh_token: String,
}

pub struct FileStorage {
    path: PathBuf,
}

impl FileStorage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

fn io_to_backend(e: std::io::Error) -> StorageError {
    StorageError::Backend(e.to_string())
}

fn json_to_backend(e: serde_json::Error) -> StorageError {
    StorageError::Backend(e.to_string())
}

fn ensure_parent_dir(path: &Path) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(io_to_backend)?;
        }
    }
    Ok(())
}

// Write data to a tmp file with 0600 mode from the start, then rename atomically.
// This avoids the race window that exists when writing with default umask then chmod.
#[cfg(unix)]
fn write_atomic_secret(path: &Path, data: &[u8]) -> Result<(), StorageError> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    let tmp = path.with_extension("tmp");
    {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp)
            .map_err(io_to_backend)?;
        f.write_all(data).map_err(io_to_backend)?;
    }
    fs::rename(&tmp, path).map_err(io_to_backend)
}

#[cfg(not(unix))]
fn write_atomic_secret(path: &Path, data: &[u8]) -> Result<(), StorageError> {
    fs::write(path, data).map_err(io_to_backend)
}

impl RefreshTokenStorage for FileStorage {
    fn save(&self, email: &str, refresh_token: &str) -> Result<(), StorageError> {
        ensure_parent_dir(&self.path)?;
        let session = FileSession {
            email: email.to_string(),
            refresh_token: refresh_token.to_string(),
        };
        let data = serde_json::to_vec_pretty(&session).map_err(json_to_backend)?;
        write_atomic_secret(&self.path, &data)
    }

    fn load(&self) -> Result<Option<StoredSession>, StorageError> {
        let data = match fs::read(&self.path) {
            Ok(v) => v,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(io_to_backend(e)),
        };
        let session: FileSession = serde_json::from_slice(&data).map_err(json_to_backend)?;
        Ok(Some(StoredSession {
            email: session.email,
            refresh_token: session.refresh_token,
        }))
    }

    fn clear(&self) -> Result<(), StorageError> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
            Err(e) => Err(io_to_backend(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn new_storage(dir: &Path) -> FileStorage {
        FileStorage::new(dir.join("nested").join("session.json"))
    }

    #[test]
    fn save_then_load_roundtrips_session() {
        let dir = tempdir().unwrap();
        let s = new_storage(dir.path());
        s.save("a@b.test", "rt-1").unwrap();

        let loaded = s.load().unwrap().unwrap();
        assert_eq!(loaded.email, "a@b.test");
        assert_eq!(loaded.refresh_token, "rt-1");
    }

    #[test]
    fn save_overwrites_existing_file() {
        let dir = tempdir().unwrap();
        let s = new_storage(dir.path());
        s.save("a@b.test", "rt-1").unwrap();
        s.save("a@b.test", "rt-2").unwrap();

        let loaded = s.load().unwrap().unwrap();
        assert_eq!(loaded.refresh_token, "rt-2");
    }

    #[test]
    fn load_returns_none_when_file_missing() {
        let dir = tempdir().unwrap();
        let s = new_storage(dir.path());
        assert!(s.load().unwrap().is_none());
    }

    #[test]
    fn load_returns_backend_error_on_invalid_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("session.json");
        fs::write(&path, b"not json").unwrap();
        let s = FileStorage::new(path);

        let err = s.load().unwrap_err();
        assert!(matches!(err, StorageError::Backend(_)));
    }

    #[test]
    fn clear_removes_file() {
        let dir = tempdir().unwrap();
        let s = new_storage(dir.path());
        s.save("a@b.test", "rt-1").unwrap();
        s.clear().unwrap();
        assert!(s.load().unwrap().is_none());
    }

    #[test]
    fn clear_is_noop_when_file_missing() {
        let dir = tempdir().unwrap();
        let s = new_storage(dir.path());
        s.clear().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn save_sets_permissions_to_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().unwrap();
        let s = new_storage(dir.path());
        s.save("a@b.test", "rt-1").unwrap();

        let mode = fs::metadata(&s.path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
