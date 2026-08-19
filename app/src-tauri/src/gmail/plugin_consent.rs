//! Product-local persistence and enforcement for Booking Finder Gmail search consent.
//!
//! The generic n-plugin runtime stays domain-agnostic. mlmail maps its generated
//! `nitra:gmail/search` host interface to `mail:search` and records approval for
//! one exact Component release and authenticated Gmail account.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use n_plugin_package::ReleaseIdentity;
use serde::{Deserialize, Serialize};

use crate::{
    plugin_contracts::GMAIL_SEARCH_INTERFACE,
    plugin_grants::{grant_store_path, PluginGrantKey, PluginGrantStore},
};

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
    grants: PluginGrantStore,
}

impl GmailSearchConsentStore {
    /// Opens an existing store or starts an empty one at `path`.
    ///
    /// # Errors
    ///
    /// Returns an error when the existing JSON store cannot be read or decoded.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let grants = match PluginGrantStore::open(&path) {
            Ok(grants) => grants,
            Err(generic_error) => {
                let source = std::fs::read(&path).with_context(|| {
                    format!("failed to read Gmail plugin consent `{}`", path.display())
                })?;
                let legacy = serde_json::from_slice::<Vec<GmailSearchConsent>>(&source)
                    .with_context(|| {
                        format!(
                            "failed to parse Gmail plugin consent `{}` after generic grant error: {generic_error:#}",
                            path.display()
                        )
                    })?;
                let exact = legacy
                    .into_iter()
                    .map(|consent| {
                        PluginGrantKey::root_host(
                            consent.release,
                            GMAIL_SEARCH_INTERFACE,
                            MAIL_SEARCH_CAPABILITY,
                            consent.account_id,
                        )
                    })
                    .collect::<Result<Vec<_>>>()?;
                PluginGrantStore::migrate_exact(&path, exact)?
            }
        };
        Ok(Self { grants })
    }

    /// Persists `mail:search` consent for one exact Component and Gmail account.
    ///
    /// # Errors
    ///
    /// Returns an error when `account_id` is empty or the updated store cannot be written.
    pub fn grant(&mut self, release: ReleaseIdentity, account_id: impl Into<String>) -> Result<()> {
        self.grants.grant(PluginGrantKey::root_host(
            release,
            GMAIL_SEARCH_INTERFACE,
            MAIL_SEARCH_CAPABILITY,
            account_id,
        )?)
    }

    /// Reports whether this exact release may search the selected Gmail account.
    #[must_use]
    pub fn allows(&self, release: &ReleaseIdentity, account_id: &str) -> bool {
        PluginGrantKey::root_host(
            release.clone(),
            GMAIL_SEARCH_INTERFACE,
            MAIL_SEARCH_CAPABILITY,
            account_id,
        )
        .is_ok_and(|key| self.grants.allows(&key))
    }

    /// Rejects a host call unless exact `mail:search` consent exists.
    ///
    /// # Errors
    ///
    /// Returns a stable `grant-required` error without exposing tokens, query text, or message data.
    pub fn require(&self, release: &ReleaseIdentity, account_id: &str) -> Result<()> {
        self.grants.require(&PluginGrantKey::root_host(
            release.clone(),
            GMAIL_SEARCH_INTERFACE,
            MAIL_SEARCH_CAPABILITY,
            account_id,
        )?)
    }
}

/// Returns the standard per-application path for persistent Gmail search consent.
#[must_use]
pub fn consent_store_path(app_data: &Path) -> PathBuf {
    grant_store_path(app_data)
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
