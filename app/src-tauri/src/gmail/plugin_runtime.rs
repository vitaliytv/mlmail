//! Registers mlmail's generated Gmail WIT binding in the domain-agnostic plugin runtime.
//!
//! The product reads the exact Gmail package from the standard `wkg.lock`, then
//! combines that package digest with the generated linker registration. It never
//! invents a parallel descriptor or a product-local package resolver.

use std::path::Path;

use anyhow::{Context, Result};
use n_plugin_compatibility::{PluginEnvironment, WitInterfaceDescriptor};
use n_plugin_package::WitExportRef;
use n_plugin_runtime::PluginRuntime;

use super::plugin_bindings::GmailSearchHost;
use crate::plugin_contracts::MlmailPluginContractRegistry;

pub use crate::plugin_contracts::{
    GMAIL_BOOKING_FINDER_INTERFACE, GMAIL_CONTRACT_PACKAGE as GMAIL_SEARCH_PACKAGE,
    GMAIL_CONTRACT_REQUIREMENT as GMAIL_SEARCH_REQUIREMENT, GMAIL_DRAFTS_INTERFACE,
    GMAIL_DRAFT_HELPER_INTERFACE, GMAIL_SEARCH_INTERFACE, MLMAIL_APPLICATION_ID,
};

/// Product runtime and its compatibility metadata derived from one exact WKG lock.
pub struct GmailPluginRuntime {
    runtime: PluginRuntime<GmailSearchHost>,
    environment: PluginEnvironment,
}

impl GmailPluginRuntime {
    /// Returns the generic runtime configured with mlmail's typed Gmail host binding.
    #[must_use]
    pub const fn runtime(&self) -> &PluginRuntime<GmailSearchHost> {
        &self.runtime
    }

    /// Returns metadata generated from the same linker registrations used at runtime.
    #[must_use]
    pub const fn environment(&self) -> &PluginEnvironment {
        &self.environment
    }
}

/// Reads the exact Gmail WIT release selected by an upstream-compatible lock file.
///
/// # Errors
///
/// Returns an error when the standard lock lacks the exact Gmail package, resolves a
/// different release, or contains an invalid content digest.
pub async fn gmail_search_descriptor(
    lock_path: impl AsRef<Path>,
) -> Result<WitInterfaceDescriptor> {
    registry_descriptor(lock_path, GMAIL_SEARCH_INTERFACE).await
}

/// Reads the exact Booking Finder trigger descriptor from the same Gmail WKG release.
///
/// # Errors
///
/// Returns an error when the standard lock lacks the exact Gmail package or its
/// content digest cannot be used as a canonical Component package descriptor.
pub async fn gmail_booking_finder_descriptor(
    lock_path: impl AsRef<Path>,
) -> Result<WitInterfaceDescriptor> {
    registry_descriptor(lock_path, GMAIL_BOOKING_FINDER_INTERFACE).await
}

/// Reads the exact Gmail drafts host descriptor from the standard WKG lock.
///
/// # Errors
///
/// Returns an error when the lock lacks the exact Gmail package or its digest is invalid.
pub async fn gmail_drafts_descriptor(
    lock_path: impl AsRef<Path>,
) -> Result<WitInterfaceDescriptor> {
    gmail_descriptor(lock_path, GMAIL_DRAFTS_INTERFACE).await
}

/// Reads the exact Draft Helper trigger descriptor from the standard WKG lock.
///
/// # Errors
///
/// Returns an error when the lock lacks the exact Gmail package or its digest is invalid.
pub async fn gmail_draft_helper_descriptor(
    lock_path: impl AsRef<Path>,
) -> Result<WitInterfaceDescriptor> {
    gmail_descriptor(lock_path, GMAIL_DRAFT_HELPER_INTERFACE).await
}

async fn gmail_descriptor(
    lock_path: impl AsRef<Path>,
    interface: &str,
) -> Result<WitInterfaceDescriptor> {
    registry_descriptor(lock_path, interface).await
}

async fn registry_descriptor(
    lock_path: impl AsRef<Path>,
    interface: &str,
) -> Result<WitInterfaceDescriptor> {
    let identity =
        WitExportRef::parse(interface).context("compiled Gmail WIT identity must remain valid")?;
    MlmailPluginContractRegistry::load(lock_path)
        .await?
        .descriptor_for(&identity)
        .with_context(|| format!("Gmail contract `{interface}` is not registered by mlmail"))
}

/// Builds the generic Component runtime from the product's exact Gmail package lock.
///
/// # Errors
///
/// Returns an error when the lock is invalid, the generated binding cannot register,
/// or plugin environment metadata cannot be constructed from that registration.
pub async fn build_gmail_plugin_runtime(
    lock_path: impl AsRef<Path>,
    application_version: &str,
) -> Result<GmailPluginRuntime> {
    let registry = MlmailPluginContractRegistry::load(lock_path).await?;
    let runtime = registry.build_runtime()?;
    let environment =
        runtime.plugin_environment(registry.environment_context(application_version))?;

    Ok(GmailPluginRuntime {
        runtime,
        environment,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCK: &str = r#"
version = 1

[[packages]]
name = "nitra:gmail"
registry = "mlmail"

[[packages.versions]]
requirement = "=0.1.0"
version = "0.1.0"
digest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
"#;

    #[tokio::test]
    async fn registers_gmail_from_the_standard_wkg_lock() {
        let directory = tempfile::tempdir().expect("temporary lock directory should open");
        let lock_path = directory.path().join("wkg.lock");
        std::fs::write(&lock_path, LOCK).expect("standard lock fixture should write");

        let registered = build_gmail_plugin_runtime(&lock_path, "0.20.0")
            .await
            .expect("exact locked Gmail WIT package should register");

        let _linker = registered
            .runtime()
            .new_linker()
            .expect("generated Gmail binding should configure a linker");
        assert_eq!(
            registered.environment().application.id,
            MLMAIL_APPLICATION_ID
        );
        assert_eq!(registered.environment().application.version, "0.20.0");
        assert_eq!(registered.environment().host_interfaces.len(), 2);
        assert_eq!(registered.environment().triggers.len(), 2);
        assert!(registered
            .environment()
            .host_interfaces
            .iter()
            .all(|interface| {
                interface.package_digest
                    == "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            }));
        assert!(registered
            .environment()
            .host_interfaces
            .iter()
            .any(|interface| interface.identity.as_str() == GMAIL_SEARCH_INTERFACE));
        assert!(registered
            .environment()
            .host_interfaces
            .iter()
            .any(|interface| interface.identity.as_str() == GMAIL_DRAFTS_INTERFACE));
        assert!(registered
            .environment()
            .triggers
            .iter()
            .any(|trigger| trigger.identity.as_str() == GMAIL_BOOKING_FINDER_INTERFACE));
        assert!(registered
            .environment()
            .triggers
            .iter()
            .any(|trigger| trigger.identity.as_str() == GMAIL_DRAFT_HELPER_INTERFACE));
    }

    #[tokio::test]
    async fn rejects_a_lock_for_another_gmail_release() {
        let directory = tempfile::tempdir().expect("temporary lock directory should open");
        let lock_path = directory.path().join("wkg.lock");
        std::fs::write(&lock_path, LOCK.replace("=0.1.0", "=0.2.0"))
            .expect("incompatible lock fixture should write");

        let error = gmail_search_descriptor(&lock_path)
            .await
            .expect_err("different requirement must not register the generated 0.1.0 binding");

        assert!(error.to_string().contains("absent from upstream wkg.lock"));
    }
}
