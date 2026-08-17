//! Registers mlmail's generated Gmail WIT binding in the domain-agnostic plugin runtime.
//!
//! The product reads the exact Gmail package from the standard `wkg.lock`, then
//! combines that package digest with the generated linker registration. It never
//! invents a parallel descriptor or a product-local package resolver.

use std::path::Path;

use anyhow::{bail, Context, Result};
use n_plugin_compatibility::{
    ApplicationIdentity, PluginEnvironment, RuntimeEnvironment, WitInterfaceDescriptor,
};
use n_plugin_package::WitExportRef;
use n_plugin_runtime::{
    ApplicationTriggerInventory, PluginEnvironmentContext, PluginHostInterfaceRegistry,
    PluginRuntime, PluginRuntimeBuilder,
};
use n_plugin_wkg::load_locked_package;

use super::plugin_bindings::{self, GmailSearchHost};

/// Exact WIT package that owns the product Gmail search interface.
pub const GMAIL_SEARCH_PACKAGE: &str = "nitra:gmail";
/// Exact M0 package requirement used when looking up Gmail in `wkg.lock`.
pub const GMAIL_SEARCH_REQUIREMENT: &str = "=0.1.0";
/// Exact generated interface identity made available to installed Components.
pub const GMAIL_SEARCH_INTERFACE: &str = "nitra:gmail/search@0.1.0";
/// Exact generated interface identity for authenticated Gmail draft creation.
pub const GMAIL_DRAFTS_INTERFACE: &str = "nitra:gmail/drafts@0.1.0";
/// Exact generated Component trigger implemented by the Booking Finder demo.
pub const GMAIL_BOOKING_FINDER_INTERFACE: &str = "nitra:gmail/booking-finder@0.1.0";
/// Exact generated Component trigger implemented by the Draft Helper demo.
pub const GMAIL_DRAFT_HELPER_INTERFACE: &str = "nitra:gmail/draft-helper@0.1.0";
/// Public, stable application identity used in plugin environment metadata.
pub const MLMAIL_APPLICATION_ID: &str = "vitaliytv:mlmail";

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
    let locked =
        load_locked_package(lock_path, GMAIL_SEARCH_PACKAGE, GMAIL_SEARCH_REQUIREMENT).await?;
    if locked.version != "0.1.0" {
        bail!(
            "Gmail search requires `{GMAIL_SEARCH_PACKAGE}@0.1.0`, but wkg.lock selected `{}`",
            locked.version
        );
    }
    WitInterfaceDescriptor::new(
        WitExportRef::parse(GMAIL_SEARCH_INTERFACE)
            .context("compiled Gmail WIT identity must remain valid")?,
        locked.digest,
    )
    .context("locked Gmail package must use a canonical WKG content digest")
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
    let locked =
        load_locked_package(lock_path, GMAIL_SEARCH_PACKAGE, GMAIL_SEARCH_REQUIREMENT).await?;
    WitInterfaceDescriptor::new(
        WitExportRef::parse(GMAIL_BOOKING_FINDER_INTERFACE)
            .context("compiled Booking Finder WIT identity must remain valid")?,
        locked.digest,
    )
    .context("locked Gmail package must use a canonical WKG content digest")
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
    let locked =
        load_locked_package(lock_path, GMAIL_SEARCH_PACKAGE, GMAIL_SEARCH_REQUIREMENT).await?;
    WitInterfaceDescriptor::new(
        WitExportRef::parse(interface).context("compiled Gmail WIT identity must remain valid")?,
        locked.digest,
    )
    .context("locked Gmail package must use a canonical WKG content digest")
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
    let lock_path = lock_path.as_ref();
    let descriptor = gmail_search_descriptor(lock_path).await?;
    let drafts_descriptor = gmail_drafts_descriptor(lock_path).await?;
    let booking_trigger = gmail_booking_finder_descriptor(lock_path).await?;
    let draft_helper_trigger = gmail_draft_helper_descriptor(lock_path).await?;
    let mut interfaces = PluginHostInterfaceRegistry::<GmailSearchHost>::new();
    interfaces.register(descriptor, |linker| {
        plugin_bindings::nitra::gmail::search::add_to_linker::<_, GmailSearchHost>(
            linker,
            |state| state,
        )
    })?;
    interfaces.register(drafts_descriptor, |linker| {
        plugin_bindings::nitra::gmail::drafts::add_to_linker::<_, GmailSearchHost>(
            linker,
            |state| state,
        )
    })?;

    let runtime = PluginRuntimeBuilder::new()?
        .register_host_interfaces(interfaces)?
        .register_triggers(ApplicationTriggerInventory::from_descriptors([
            booking_trigger,
            draft_helper_trigger,
        ]))
        .build();
    let environment = runtime.plugin_environment(PluginEnvironmentContext {
        application: ApplicationIdentity {
            id: MLMAIL_APPLICATION_ID.to_owned(),
            version: application_version.to_owned(),
        },
        runtime: RuntimeEnvironment {
            runtime_lts: 48,
            component_model_profile: "nitra-component-v1".to_owned(),
            async_abi_snapshot: "wasmtime-48".to_owned(),
        },
        plugin_manifest_versions: vec![1],
        vue_a2ui_schema: 1,
        required_features: vec!["component-model-async".to_owned()],
    })?;

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
