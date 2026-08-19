//! Product-owned exact grants for typed plugin-to-host capability access.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use n_plugin_package::ReleaseIdentity;
use serde::{Deserialize, Serialize};

/// Sealed payload-free product scope attached to an exact capability grant.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PluginGrantScope {
    /// Account-scoped requirement previewed before an account identity is available.
    UnresolvedMailAccount,
    /// One exact authenticated mail account identity.
    MailAccount {
        /// Public account identity; never an OAuth credential.
        account_id: String,
    },
    /// The local application installation without an account resource.
    Application,
}

/// Exact product-owned authorization key for one typed plugin-to-host edge.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginGrantKey {
    /// Installed root graph that owns this authorization decision.
    pub root: ReleaseIdentity,
    /// Exact Component node importing the host interface.
    pub subject: ReleaseIdentity,
    /// Stable logical host edge independent from activation generation numbers.
    pub logical_edge: String,
    /// Exact typed host interface guarded by the capability.
    pub host_interface: String,
    /// Product capability mapped from the typed host interface.
    pub capability: String,
    /// Authenticated account covered by this grant, when account-scoped.
    pub account_id: Option<String>,
    /// Structured product scope without tokens, queries, or message bodies.
    pub scope: PluginGrantScope,
}

impl PluginGrantKey {
    /// Creates a validated exact product grant key.
    ///
    /// # Errors
    ///
    /// Returns an error when logical edge, interface, capability, or scoped account is invalid.
    pub fn new(
        root: ReleaseIdentity,
        subject: ReleaseIdentity,
        logical_edge: impl Into<String>,
        host_interface: impl Into<String>,
        capability: impl Into<String>,
        account_id: Option<String>,
        scope: PluginGrantScope,
    ) -> Result<Self> {
        let logical_edge = logical_edge.into();
        let host_interface = host_interface.into();
        let capability = capability.into();
        if logical_edge.trim().is_empty() {
            bail!("plugin grant logical edge cannot be empty");
        }
        if host_interface.trim().is_empty() {
            bail!("plugin grant host interface cannot be empty");
        }
        if capability.trim().is_empty() {
            bail!("plugin grant capability cannot be empty");
        }
        if account_id
            .as_ref()
            .is_some_and(|account| account.trim().is_empty())
        {
            bail!("plugin grant account identity cannot be empty");
        }
        match (&account_id, &scope) {
            (Some(account), PluginGrantScope::MailAccount { account_id: scoped })
                if account == scoped => {}
            (None, PluginGrantScope::Application) => {}
            (None, PluginGrantScope::UnresolvedMailAccount) => {
                bail!("account-scoped plugin grant requires an authenticated account identity")
            }
            _ => bail!("plugin grant account identity does not match its sealed scope"),
        }
        Ok(Self {
            root,
            subject,
            logical_edge,
            host_interface,
            capability,
            account_id,
            scope,
        })
    }

    /// Creates an account-scoped grant from one exact root Component to a typed host interface.
    ///
    /// # Errors
    ///
    /// Returns an error when interface, capability, or account identity is empty.
    pub fn root_host(
        release: ReleaseIdentity,
        host_interface: impl Into<String>,
        capability: impl Into<String>,
        account_id: impl Into<String>,
    ) -> Result<Self> {
        let host_interface = host_interface.into();
        let capability = capability.into();
        let account_id = account_id.into();
        if host_interface.trim().is_empty() {
            bail!("plugin grant host interface cannot be empty");
        }
        if capability.trim().is_empty() {
            bail!("plugin grant capability cannot be empty");
        }
        if account_id.trim().is_empty() {
            bail!("plugin grant account identity cannot be empty");
        }
        Self::new(
            release.clone(),
            release,
            format!("root-host:{host_interface}"),
            host_interface,
            capability,
            Some(account_id.clone()),
            PluginGrantScope::MailAccount { account_id },
        )
    }
}

/// Durable product-local store for exact plugin capability grants.
pub struct PluginGrantStore {
    path: PathBuf,
    grants: Vec<PluginGrantKey>,
}

impl PluginGrantStore {
    /// Opens an existing grant store or creates an empty in-memory projection.
    ///
    /// # Errors
    ///
    /// Returns an error when existing grant data cannot be read or decoded.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let grants = match fs::read(&path) {
            Ok(source) => serde_json::from_slice(&source)
                .with_context(|| format!("failed to parse plugin grants `{}`", path.display()))?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to read plugin grants `{}`", path.display()));
            }
        };
        Ok(Self { path, grants })
    }

    /// Replaces a legacy product grant file with a validated exact grant set.
    ///
    /// # Errors
    ///
    /// Returns an error when the migrated set cannot be published atomically.
    pub fn migrate_exact(
        path: impl Into<PathBuf>,
        keys: impl IntoIterator<Item = PluginGrantKey>,
    ) -> Result<Self> {
        let mut store = Self {
            path: path.into(),
            grants: Vec::new(),
        };
        store.grant_all(keys)?;
        Ok(store)
    }

    /// Reports whether the complete exact grant key is approved.
    #[must_use]
    pub fn allows(&self, key: &PluginGrantKey) -> bool {
        self.grants.contains(key)
    }

    /// Rejects access unless the complete exact grant key is approved.
    ///
    /// # Errors
    ///
    /// Returns a stable `grant-required` error without request payload data.
    pub fn require(&self, key: &PluginGrantKey) -> Result<()> {
        if self.allows(key) {
            return Ok(());
        }
        bail!(
            "grant-required: `{}` is not approved for exact plugin `{}`",
            key.capability,
            key.subject.package
        )
    }

    /// Persists one exact grant without weakening any existing scope.
    ///
    /// # Errors
    ///
    /// Returns an error when the complete updated store cannot be published atomically.
    pub fn grant(&mut self, key: PluginGrantKey) -> Result<()> {
        self.grant_all([key])
    }

    /// Persists a complete set of exact grants in one filesystem replacement.
    ///
    /// # Errors
    ///
    /// Returns an error when the complete updated store cannot be published atomically.
    pub fn grant_all(&mut self, keys: impl IntoIterator<Item = PluginGrantKey>) -> Result<()> {
        let mut updated = self.grants.clone();
        for key in keys {
            if !updated.contains(&key) {
                updated.push(key);
            }
        }
        updated.sort();
        updated.dedup();
        if updated != self.grants {
            let previous = std::mem::replace(&mut self.grants, updated);
            if let Err(error) = self.save() {
                self.grants = previous;
                return Err(error);
            }
        }
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let parent = self
            .path
            .parent()
            .context("plugin grant path must have a parent directory")?;
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create plugin grant directory `{}`",
                parent.display()
            )
        })?;
        let file_name = self
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .context("plugin grant path must have a UTF-8 file name")?;
        let temporary = parent.join(format!(".{file_name}.{}.tmp", std::process::id()));
        let mut file = fs::File::create(&temporary).with_context(|| {
            format!(
                "failed to create staged plugin grants `{}`",
                temporary.display()
            )
        })?;
        file.write_all(&serde_json::to_vec_pretty(&self.grants)?)
            .with_context(|| format!("failed to stage plugin grants `{}`", temporary.display()))?;
        file.sync_all().with_context(|| {
            format!(
                "failed to sync staged plugin grants `{}`",
                temporary.display()
            )
        })?;
        drop(file);
        fs::rename(&temporary, &self.path)
            .with_context(|| format!("failed to publish plugin grants `{}`", self.path.display()))
    }
}

/// Returns the canonical product grant path below the application data directory.
#[must_use]
pub fn grant_store_path(app_data: &Path) -> PathBuf {
    app_data.join("n-plugin").join("grants.json")
}

#[cfg(test)]
mod tests {
    use n_plugin_package::ReleaseIdentity;

    use super::*;

    fn release(digest: &str) -> ReleaseIdentity {
        ReleaseIdentity {
            package: "vitaliytv:booking-finder".to_owned(),
            version: "0.1.0".to_owned(),
            digest: digest.to_owned(),
        }
    }

    fn key(digest: &str, account: &str) -> PluginGrantKey {
        PluginGrantKey::root_host(
            release(digest),
            "nitra:gmail/search@0.1.0",
            "mail:search",
            account,
        )
        .expect("test grant key should be valid")
    }

    #[test]
    fn persists_only_the_exact_release_account_and_host_edge() {
        let directory = tempfile::tempdir().expect("temporary grant directory should open");
        let path = directory.path().join("grants.json");
        let approved = key(
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "person@example.com",
        );
        let mut store = PluginGrantStore::open(&path).expect("grant store should open");

        assert!(!store.allows(&approved));
        store
            .grant(approved.clone())
            .expect("exact grant should persist");

        let reopened = PluginGrantStore::open(&path).expect("grant store should reopen");
        assert!(reopened.allows(&approved));
        assert!(!reopened.allows(&key(
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "person@example.com",
        )));
        assert!(!reopened.allows(&key(
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "other@example.com",
        )));
    }

    #[test]
    fn rejects_empty_account_identity() {
        let error = PluginGrantKey::root_host(
            release("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            "nitra:gmail/search@0.1.0",
            "mail:search",
            " ",
        )
        .expect_err("empty account identity must fail closed");

        assert!(error.to_string().contains("account identity"));
    }
}
