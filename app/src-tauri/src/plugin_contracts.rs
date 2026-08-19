//! Product-owned typed registry for plugin triggers, host interfaces and consent metadata.

use std::{collections::BTreeMap, path::Path};

use anyhow::{bail, Context, Result};
use n_plugin_compatibility::{ApplicationIdentity, RuntimeEnvironment, WitInterfaceDescriptor};
use n_plugin_package::WitExportRef;
use n_plugin_runtime::{
    ApplicationTriggerInventory, HostInterfaceInventory, PluginEnvironmentContext,
    PluginHostInterfaceRegistry, PluginRuntime, PluginRuntimeBuilder,
};
use n_plugin_wkg::load_locked_package;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::gmail::plugin_bindings::{self, GmailSearchHost};

/// Exact WIT package that owns mlmail's Gmail contracts.
pub const GMAIL_CONTRACT_PACKAGE: &str = "nitra:gmail";
/// Exact package requirement compiled into the current mlmail release.
pub const GMAIL_CONTRACT_REQUIREMENT: &str = "=0.1.0";
/// Exact generated host interface for Gmail search.
pub const GMAIL_SEARCH_INTERFACE: &str = "nitra:gmail/search@0.1.0";
/// Exact generated host interface for authenticated draft creation.
pub const GMAIL_DRAFTS_INTERFACE: &str = "nitra:gmail/drafts@0.1.0";
/// Exact generated trigger implemented by Booking Finder Components.
pub const GMAIL_BOOKING_FINDER_INTERFACE: &str = "nitra:gmail/booking-finder@0.1.0";
/// Exact generated trigger implemented by Draft Helper Components.
pub const GMAIL_DRAFT_HELPER_INTERFACE: &str = "nitra:gmail/draft-helper@0.1.0";
/// Public stable identity placed in plugin environment metadata.
pub const MLMAIL_APPLICATION_ID: &str = "vitaliytv:mlmail";

const MAIL_SEARCH_CAPABILITY: &str = "mail:search";
const MAIL_DRAFT_CREATE_CAPABILITY: &str = "mail:draft.create";

/// Product command family that can deliver one registered plugin trigger.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MlmailPluginActionKind {
    /// Invoke the typed Booking Finder search adapter.
    BookingFinderFind,
    /// Invoke the typed Draft Helper creation adapter.
    DraftHelperCreate,
}

/// Product-local consent requirement attached to an exact host interface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginCapabilityRequirement {
    /// Stable capability name shown by the installation preview.
    pub capability: &'static str,
    /// Whether consent must be scoped to one authenticated account.
    pub account_scoped: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HostRegistrationKind {
    GmailSearch,
    GmailDrafts,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HostContract {
    descriptor: WitInterfaceDescriptor,
    registration: HostRegistrationKind,
    capabilities: Vec<PluginCapabilityRequirement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TriggerContract {
    descriptor: WitInterfaceDescriptor,
    action: MlmailPluginActionKind,
}

impl TriggerContract {
    fn new(descriptor: WitInterfaceDescriptor, action: MlmailPluginActionKind) -> Self {
        Self { descriptor, action }
    }
}

/// Static typed inventory owned by mlmail and derived from its exact WKG lock.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MlmailPluginContractRegistry {
    hosts: BTreeMap<WitExportRef, HostContract>,
    triggers: BTreeMap<WitExportRef, TriggerContract>,
}

impl MlmailPluginContractRegistry {
    /// Loads every contract compiled into mlmail from one exact `wkg.lock` release.
    ///
    /// # Errors
    ///
    /// Returns an error when the package is missing, selects another release, has an invalid
    /// digest, or produces duplicate interface identities.
    pub async fn load(lock_path: impl AsRef<Path>) -> Result<Self> {
        let locked = load_locked_package(
            lock_path,
            GMAIL_CONTRACT_PACKAGE,
            GMAIL_CONTRACT_REQUIREMENT,
        )
        .await?;
        if locked.version != "0.1.0" {
            bail!(
                "Gmail contracts require `{GMAIL_CONTRACT_PACKAGE}@0.1.0`, but wkg.lock selected `{}`",
                locked.version
            );
        }

        let host_contracts = [
            HostContract {
                descriptor: descriptor(GMAIL_SEARCH_INTERFACE, &locked.digest)?,
                registration: HostRegistrationKind::GmailSearch,
                capabilities: vec![PluginCapabilityRequirement {
                    capability: MAIL_SEARCH_CAPABILITY,
                    account_scoped: true,
                }],
            },
            HostContract {
                descriptor: descriptor(GMAIL_DRAFTS_INTERFACE, &locked.digest)?,
                registration: HostRegistrationKind::GmailDrafts,
                capabilities: vec![PluginCapabilityRequirement {
                    capability: MAIL_DRAFT_CREATE_CAPABILITY,
                    account_scoped: true,
                }],
            },
        ];
        let trigger_contracts = [
            TriggerContract::new(
                descriptor(GMAIL_BOOKING_FINDER_INTERFACE, &locked.digest)?,
                MlmailPluginActionKind::BookingFinderFind,
            ),
            TriggerContract::new(
                descriptor(GMAIL_DRAFT_HELPER_INTERFACE, &locked.digest)?,
                MlmailPluginActionKind::DraftHelperCreate,
            ),
        ];

        Self::from_contracts(host_contracts, trigger_contracts)
    }

    /// Returns the exact host interfaces accepted by activation compilation.
    #[must_use]
    pub fn host_inventory(&self) -> HostInterfaceInventory {
        HostInterfaceInventory::new(self.hosts.keys().cloned())
    }

    /// Returns the exact triggers accepted by installation preflight.
    #[must_use]
    pub fn trigger_inventory(&self) -> ApplicationTriggerInventory {
        ApplicationTriggerInventory::from_descriptors(self.trigger_descriptors())
    }

    /// Returns WKG descriptors for all generated host linker registrations.
    #[must_use]
    pub fn host_descriptors(&self) -> Vec<WitInterfaceDescriptor> {
        self.hosts
            .values()
            .map(|contract| contract.descriptor.clone())
            .collect()
    }

    /// Returns WKG descriptors for all product trigger adapters.
    #[must_use]
    pub fn trigger_descriptors(&self) -> Vec<WitInterfaceDescriptor> {
        self.triggers
            .values()
            .map(|contract| contract.descriptor.clone())
            .collect()
    }

    /// Resolves the typed product action for an exact trigger identity.
    #[must_use]
    pub fn action_for(&self, trigger: &WitExportRef) -> Option<MlmailPluginActionKind> {
        self.triggers.get(trigger).map(|contract| contract.action)
    }

    /// Returns consent metadata for an exact registered host interface.
    #[must_use]
    pub fn capability_requirements_for(
        &self,
        interface: &WitExportRef,
    ) -> Option<&[PluginCapabilityRequirement]> {
        self.hosts
            .get(interface)
            .map(|contract| contract.capabilities.as_slice())
    }

    /// Returns the descriptor compiled for an exact host interface or trigger.
    #[must_use]
    pub fn descriptor_for(&self, identity: &WitExportRef) -> Option<WitInterfaceDescriptor> {
        self.hosts
            .get(identity)
            .map(|contract| contract.descriptor.clone())
            .or_else(|| {
                self.triggers
                    .get(identity)
                    .map(|contract| contract.descriptor.clone())
            })
    }

    /// Returns a deterministic fingerprint of contracts, actions, and consent mappings.
    #[must_use]
    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        for (identity, contract) in &self.hosts {
            hasher.update(b"host\0");
            hasher.update(identity.as_str().as_bytes());
            hasher.update(b"\0");
            hasher.update(contract.descriptor.package_digest.as_bytes());
            for requirement in &contract.capabilities {
                hasher.update(b"\0capability\0");
                hasher.update(requirement.capability.as_bytes());
                hasher.update([u8::from(requirement.account_scoped)]);
            }
        }
        for (identity, contract) in &self.triggers {
            hasher.update(b"trigger\0");
            hasher.update(identity.as_str().as_bytes());
            hasher.update(b"\0");
            hasher.update(contract.descriptor.package_digest.as_bytes());
            hasher.update(b"\0");
            hasher.update(match contract.action {
                MlmailPluginActionKind::BookingFinderFind => b"booking-finder-find".as_slice(),
                MlmailPluginActionKind::DraftHelperCreate => b"draft-helper-create".as_slice(),
            });
        }
        format!("sha256:{:x}", hasher.finalize())
    }

    /// Builds the runtime from the same typed registrations represented by this inventory.
    ///
    /// # Errors
    ///
    /// Returns an error when the Component engine or a generated linker registration fails.
    pub fn build_runtime(&self) -> Result<PluginRuntime<GmailSearchHost>> {
        Ok(PluginRuntimeBuilder::new()?
            .register_host_interfaces(self.linker_registrations()?)?
            .register_triggers(self.trigger_inventory())
            .build())
    }

    /// Builds public application metadata shared by compatibility checks and runtime activation.
    #[must_use]
    pub fn environment_context(&self, application_version: &str) -> PluginEnvironmentContext {
        PluginEnvironmentContext {
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
        }
    }

    fn from_contracts(
        hosts: impl IntoIterator<Item = HostContract>,
        triggers: impl IntoIterator<Item = TriggerContract>,
    ) -> Result<Self> {
        let mut host_map = BTreeMap::new();
        for contract in hosts {
            let identity = contract.descriptor.identity.clone();
            if host_map.insert(identity.clone(), contract).is_some() {
                bail!(
                    "host interface `{}` is already registered",
                    identity.as_str()
                );
            }
        }

        let mut trigger_map = BTreeMap::new();
        for contract in triggers {
            let identity = contract.descriptor.identity.clone();
            if trigger_map.insert(identity.clone(), contract).is_some() {
                bail!("trigger `{}` is already registered", identity.as_str());
            }
        }

        Ok(Self {
            hosts: host_map,
            triggers: trigger_map,
        })
    }

    fn linker_registrations(&self) -> Result<PluginHostInterfaceRegistry<GmailSearchHost>> {
        let mut registrations = PluginHostInterfaceRegistry::new();
        for contract in self.hosts.values() {
            match contract.registration {
                HostRegistrationKind::GmailSearch => {
                    registrations.register(contract.descriptor.clone(), |linker| {
                        plugin_bindings::nitra::gmail::search::add_to_linker::<_, GmailSearchHost>(
                            linker,
                            |state| state,
                        )
                    })?
                }
                HostRegistrationKind::GmailDrafts => {
                    registrations.register(contract.descriptor.clone(), |linker| {
                        plugin_bindings::nitra::gmail::drafts::add_to_linker::<_, GmailSearchHost>(
                            linker,
                            |state| state,
                        )
                    })?
                }
            }
        }
        Ok(registrations)
    }
}

fn descriptor(identity: &str, digest: &str) -> Result<WitInterfaceDescriptor> {
    WitInterfaceDescriptor::new(
        WitExportRef::parse(identity).context("compiled mlmail WIT identity must remain valid")?,
        digest,
    )
    .context("locked mlmail package must use a canonical WKG content digest")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gmail::plugin_runtime::{
        GMAIL_BOOKING_FINDER_INTERFACE, GMAIL_DRAFTS_INTERFACE, GMAIL_DRAFT_HELPER_INTERFACE,
        GMAIL_SEARCH_INTERFACE,
    };
    use n_plugin_package::WitExportRef;

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

    async fn registry() -> MlmailPluginContractRegistry {
        let directory = tempfile::tempdir().expect("temporary lock directory should open");
        let lock_path = directory.path().join("wkg.lock");
        std::fs::write(&lock_path, LOCK).expect("standard lock fixture should write");
        MlmailPluginContractRegistry::load(&lock_path)
            .await
            .expect("exact mlmail contracts should load")
    }

    #[tokio::test]
    async fn contains_exact_product_triggers_and_actions() {
        let registry = registry().await;

        let booking = WitExportRef::parse(GMAIL_BOOKING_FINDER_INTERFACE).expect("valid trigger");
        let draft = WitExportRef::parse(GMAIL_DRAFT_HELPER_INTERFACE).expect("valid trigger");

        assert_eq!(
            registry.action_for(&booking),
            Some(MlmailPluginActionKind::BookingFinderFind)
        );
        assert_eq!(
            registry.action_for(&draft),
            Some(MlmailPluginActionKind::DraftHelperCreate)
        );
    }

    #[tokio::test]
    async fn derives_host_and_trigger_descriptors_from_one_locked_digest() {
        let registry = registry().await;
        let host_descriptors = registry.host_descriptors();
        let trigger_descriptors = registry.trigger_descriptors();

        assert_eq!(host_descriptors.len(), 2);
        assert_eq!(trigger_descriptors.len(), 2);
        for identity in [GMAIL_SEARCH_INTERFACE, GMAIL_DRAFTS_INTERFACE] {
            assert!(host_descriptors.iter().any(|descriptor| {
                descriptor.identity.as_str() == identity
                    && descriptor.package_digest
                        == "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            }));
        }
        assert!(host_descriptors
            .iter()
            .chain(trigger_descriptors.iter())
            .all(|descriptor| descriptor.package_digest == host_descriptors[0].package_digest));
    }

    #[tokio::test]
    async fn unknown_trigger_does_not_resolve() {
        let registry = registry().await;
        let unknown = WitExportRef::parse("nitra:gmail/unknown@0.1.0").expect("valid trigger");

        assert_eq!(registry.action_for(&unknown), None);
        assert_eq!(registry.capability_requirements_for(&unknown), None);
    }

    #[tokio::test]
    async fn maps_host_interfaces_to_account_scoped_capabilities() {
        let registry = registry().await;
        let search = WitExportRef::parse(GMAIL_SEARCH_INTERFACE).expect("valid interface");
        let drafts = WitExportRef::parse(GMAIL_DRAFTS_INTERFACE).expect("valid interface");

        assert_eq!(
            registry.capability_requirements_for(&search),
            Some(
                [PluginCapabilityRequirement {
                    capability: "mail:search",
                    account_scoped: true,
                }]
                .as_slice()
            )
        );
        assert_eq!(
            registry.capability_requirements_for(&drafts),
            Some(
                [PluginCapabilityRequirement {
                    capability: "mail:draft.create",
                    account_scoped: true,
                }]
                .as_slice()
            )
        );
    }

    #[test]
    fn rejects_duplicate_trigger_identity() {
        let descriptor = test_descriptor(GMAIL_DRAFT_HELPER_INTERFACE);
        let error = MlmailPluginContractRegistry::from_contracts(
            Vec::new(),
            [
                TriggerContract::new(
                    descriptor.clone(),
                    MlmailPluginActionKind::DraftHelperCreate,
                ),
                TriggerContract::new(descriptor, MlmailPluginActionKind::BookingFinderFind),
            ],
        )
        .expect_err("duplicate trigger identity must be rejected");

        assert!(error.to_string().contains("already registered"));
    }

    #[tokio::test]
    async fn public_environment_matches_linker_registrations() {
        let registry = registry().await;
        let expected_host = registry.host_inventory();
        let expected_triggers = registry.trigger_inventory();
        let host_descriptors = registry.host_descriptors();
        let trigger_descriptors = registry.trigger_descriptors();
        let runtime = registry
            .build_runtime()
            .expect("typed linker registrations should build");
        let environment = runtime
            .plugin_environment(registry.environment_context("0.28.0"))
            .expect("public plugin environment should build");

        let _linker = runtime
            .new_linker()
            .expect("every advertised host interface should register");
        assert_eq!(runtime.host_inventory(), expected_host);
        assert_eq!(runtime.trigger_inventory(), &expected_triggers);
        assert_eq!(environment.host_interfaces, host_descriptors);
        assert_eq!(environment.triggers, trigger_descriptors);
    }

    fn test_descriptor(identity: &str) -> n_plugin_compatibility::WitInterfaceDescriptor {
        n_plugin_compatibility::WitInterfaceDescriptor::new(
            WitExportRef::parse(identity).expect("test identity should parse"),
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .expect("test descriptor should build")
    }
}
