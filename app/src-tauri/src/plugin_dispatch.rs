//! Exact installed Component selection boundary for typed plugin commands.

use std::path::Path;

use anyhow::{bail, Context, Result};
use n_plugin_compatibility::GraphLifecycleState;
use n_plugin_package::{ReleaseIdentity, WitExportRef};
use n_plugin_runtime::{ActivationGeneration, GenerationStatus};

use crate::{
    plugin_context::{self, PluginContextEntry},
    plugin_contracts::MlmailPluginContractRegistry,
    plugin_grants::{grant_store_path, PluginGrantKey, PluginGrantScope, PluginGrantStore},
    plugins::{load_index, registry},
};

/// Exact Component and durable context identity selected for one typed invocation.
pub struct PluginDispatchSelection {
    /// Immutable installed release selected by the caller.
    pub release: ReleaseIdentity,
    /// Exact composed Component bytes loaded from the activation CAS.
    pub component: Vec<u8>,
    /// Deterministic durable context instance for this exact release.
    pub context_id: String,
    /// Exact committed activation generation selected for invocation.
    pub generation: u64,
    /// Product host interfaces imported by the stored activation generation.
    pub host_interfaces: Vec<String>,
}

/// Resolves one exact installed release for a required typed trigger.
///
/// # Errors
///
/// Returns an error unless the committed projection, immutable generation, active pointer,
/// lifecycle, trigger contract and CAS artifact all agree on the caller's exact target.
pub(crate) fn dispatch_component_at(
    app_data: &Path,
    target: &ReleaseIdentity,
    required_trigger: &str,
    contracts: &MlmailPluginContractRegistry,
) -> Result<PluginDispatchSelection> {
    let installed = load_index(app_data)?
        .plugins
        .into_iter()
        .find(|plugin| plugin.release == *target)
        .with_context(|| format!("exact plugin release `{}` is not installed", target.digest))?;
    if !installed.enabled {
        bail!("exact plugin release is disabled")
    }

    let registry = registry(app_data)?;
    let generation = ActivationGeneration::new(installed.activation_generation)?;
    let stored = registry.generation(generation)?;
    if stored.root != *target
        || stored.generation != generation
        || stored.status != GenerationStatus::Active
    {
        bail!("installed plugin projection does not match its active immutable generation")
    }
    if registry.graph_lifecycle(target)?.state != GraphLifecycleState::active() {
        bail!("exact plugin release is disabled or unavailable")
    }
    let active = registry
        .active(target)?
        .context("installed plugin has no active generation")?;
    if active.root != *target
        || active.generation != generation
        || active.status != GenerationStatus::Active
    {
        bail!("active plugin pointer does not match the exact installed release")
    }

    let trigger = WitExportRef::parse(required_trigger)?;
    contracts.action_for(&trigger).with_context(|| {
        format!("typed trigger `{required_trigger}` is not registered by mlmail")
    })?;
    if !stored.triggers.contains(&trigger) {
        bail!("exact plugin release does not implement required trigger `{required_trigger}`")
    }
    let component = registry.cas().read(&stored.composed_digest)?;
    Ok(PluginDispatchSelection {
        release: target.clone(),
        component,
        context_id: durable_context_id(target),
        generation: generation.get(),
        host_interfaces: stored.host_interfaces,
    })
}

/// Returns the stable durable instance identity for one package-scoped plugin slot.
#[must_use]
pub fn durable_context_id(release: &ReleaseIdentity) -> String {
    format!("plugin:{}", release.package)
}

/// Publishes the exact desired context for every committed installed plugin projection.
///
/// # Errors
///
/// Returns an error when an installed projection no longer identifies its immutable generation,
/// a stored host import is invalid, or durable context publication fails.
pub(crate) fn publish_context_at(app_data: &Path) -> Result<()> {
    let registry = registry(app_data)?;
    let mut entries = Vec::new();
    for installed in load_index(app_data)?.plugins {
        let generation = ActivationGeneration::new(installed.activation_generation)?;
        let stored = registry.generation(generation)?;
        if stored.root != installed.release || stored.generation != generation {
            bail!("installed plugin projection does not match its immutable generation")
        }
        entries.push(PluginContextEntry {
            context_id: durable_context_id(&installed.release),
            release: installed.release,
            host_interfaces: stored
                .host_interfaces
                .iter()
                .map(WitExportRef::parse)
                .collect::<std::result::Result<Vec<_>, _>>()?,
            enabled: installed.enabled,
        });
    }
    let desired = plugin_context::plugin_context(entries)?;
    let store =
        n_plugin_runtime::DurableContextStore::open(plugin_context::context_database(app_data))?;
    if store
        .active()?
        .is_none_or(|active| active.entries != desired)
    {
        store.publish(desired)?;
    }
    Ok(())
}

/// Enforces every exact product capability required by the selected stored host imports.
///
/// # Errors
///
/// Returns `grant-required` before token acquisition when any exact release/account/host grant is
/// absent, and rejects stored host interfaces without product-owned capability metadata.
pub(crate) fn require_dispatch_grants(
    app_data: &Path,
    selection: &PluginDispatchSelection,
    contracts: &MlmailPluginContractRegistry,
    account_id: &str,
) -> Result<()> {
    let store = PluginGrantStore::open(grant_store_path(app_data))?;
    for interface in &selection.host_interfaces {
        let identity = WitExportRef::parse(interface)?;
        let requirements = contracts
            .capability_requirements_for(&identity)
            .with_context(|| {
                format!("stored host interface `{interface}` has no product capability mapping")
            })?;
        if requirements.is_empty() {
            bail!("stored host interface `{interface}` has no product capability requirement")
        }
        for requirement in requirements {
            let key = if requirement.account_scoped {
                PluginGrantKey::root_host(
                    selection.release.clone(),
                    interface,
                    requirement.capability,
                    account_id,
                )?
            } else {
                PluginGrantKey::new(
                    selection.release.clone(),
                    selection.release.clone(),
                    format!("root-host:{interface}"),
                    interface,
                    requirement.capability,
                    None,
                    PluginGrantScope::Application,
                )?
            };
            store.require(&key)?;
        }
    }
    Ok(())
}
