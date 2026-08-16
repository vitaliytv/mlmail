//! Product-local persistence and enforcement for Booking Finder Gmail search consent.
//!
//! The generic n-plugin runtime stays domain-agnostic. mlmail maps its generated
//! `nitra:gmail/search` host interface to `mail:search` and records approval for
//! one exact Component release and authenticated Gmail account.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use n_plugin_package::ReleaseIdentity;
use serde::{Deserialize, Serialize};

/// Product-defined capability required by the Gmail search host interface.
pub const MAIL_SEARCH_CAPABILITY: &str = "mail:search";

/// One persistent approval for an exact plugin release and Gmail account.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GmailSearchConsent {
    /// Immutable release whose embedded Component digest was approved.
    pub release: ReleaseIdentity,
    /// Authenticated Gmail account covered by this approval.
    pub account_id: String,
}

/// Product-local store for exact `mail:search` approvals.
pub struct GmailSearchConsentStore {
    path: PathBuf,
    consents: Vec<GmailSearchConsent>,
}

impl GmailSearchConsentStore {
    /// Opens an existing store or starts an empty one at `path`.
    ///
    /// # Errors
    ///
    /// Returns an error when the existing JSON store cannot be read or decoded.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let consents = match fs::read_to_string(&path) {
            Ok(json) => serde_json::from_str(&json).with_context(|| {
                format!("failed to parse Gmail plugin consent `{}`", path.display())
            })?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("failed to read Gmail plugin consent `{}`", path.display())
                });
            }
        };
        Ok(Self { path, consents })
    }

    /// Persists `mail:search` consent for one exact Component and Gmail account.
    ///
    /// # Errors
    ///
    /// Returns an error when `account_id` is empty or the updated store cannot be written.
    pub fn grant(&mut self, release: ReleaseIdentity, account_id: impl Into<String>) -> Result<()> {
        let account_id = account_id.into();
        if account_id.trim().is_empty() {
            bail!("Gmail consent account identity cannot be empty");
        }
        if !self.allows(&release, &account_id) {
            self.consents.push(GmailSearchConsent {
                release,
                account_id,
            });
            self.save()?;
        }
        Ok(())
    }

    /// Reports whether this exact release may search the selected Gmail account.
    #[must_use]
    pub fn allows(&self, release: &ReleaseIdentity, account_id: &str) -> bool {
        self.consents
            .iter()
            .any(|consent| consent.release == *release && consent.account_id == account_id)
    }

    /// Rejects a host call unless exact `mail:search` consent exists.
    ///
    /// # Errors
    ///
    /// Returns a stable `grant-required` error without exposing tokens, query text, or message data.
    pub fn require(&self, release: &ReleaseIdentity, account_id: &str) -> Result<()> {
        if self.allows(release, account_id) {
            return Ok(());
        }
        bail!(
            "grant-required: `{MAIL_SEARCH_CAPABILITY}` is not approved for `{}` on account `{account_id}`",
            release.package
        );
    }

    fn save(&self) -> Result<()> {
        let parent = self
            .path
            .parent()
            .context("Gmail plugin consent path must have a parent directory")?;
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create Gmail plugin consent directory `{}`",
                parent.display()
            )
        })?;
        let file_name = self
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .context("Gmail plugin consent path must have a UTF-8 file name")?;
        let temporary = parent.join(format!(".{file_name}.{}.tmp", std::process::id()));
        let json = serde_json::to_vec_pretty(&self.consents)?;
        fs::write(&temporary, json).with_context(|| {
            format!(
                "failed to write Gmail plugin consent `{}`",
                temporary.display()
            )
        })?;
        fs::rename(&temporary, &self.path).with_context(|| {
            format!(
                "failed to publish Gmail plugin consent `{}`",
                self.path.display()
            )
        })
    }
}

/// Returns the standard per-application path for persistent Gmail search consent.
#[must_use]
pub fn consent_store_path(app_data: &Path) -> PathBuf {
    app_data.join("plugins").join("gmail-search-consents.json")
}

#[cfg(test)]
mod tests {
    use n_plugin_package::ReleaseIdentity;

    use super::{GmailSearchConsentStore, MAIL_SEARCH_CAPABILITY};

    fn release(digest: &str) -> ReleaseIdentity {
        ReleaseIdentity {
            package: "vitaliytv:booking-finder".into(),
            version: "0.1.0".into(),
            digest: digest.into(),
        }
    }

    #[test]
    fn persists_an_exact_account_scoped_search_grant() {
        let directory = tempfile::tempdir().expect("temporary consent directory should open");
        let path = directory.path().join("consent.json");
        let exact =
            release("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let mut store = GmailSearchConsentStore::open(&path).expect("store should open");

        assert!(!store.allows(&exact, "person@example.com"));
        assert!(store.require(&exact, "person@example.com").is_err());
        store
            .grant(exact.clone(), "person@example.com")
            .expect("grant should persist");

        let reopened = GmailSearchConsentStore::open(&path).expect("store should reopen");
        assert!(reopened.allows(&exact, "person@example.com"));
        assert!(!reopened.allows(&exact, "other@example.com"));
        assert!(!reopened.allows(
            &release("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            "person@example.com"
        ));
        assert!(reopened
            .require(&exact, "other@example.com")
            .expect_err("different account must require consent")
            .to_string()
            .contains(MAIL_SEARCH_CAPABILITY));
    }
}
