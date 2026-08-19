//! Pure product-local preflight for typed n-plugin Components.

use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::Result;
use n_plugin_oci::{ResolvedEdge, ResolvedNode, ResolvedPluginGraph};
use n_plugin_package::{inspect_component, ReleaseIdentity, WitExportRef};
use n_plugin_runtime::{
    ActivationCompiler, ActivationGeneration, ActivationPlan, ActivationPolicy, CapabilityAccess,
    EdgeCapabilityPolicy,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    plugin_contracts::{
        is_no_consent_runtime_interface, MlmailPluginActionKind, MlmailPluginContractRegistry,
        PluginCapabilityRequirement,
    },
    plugin_grants::{PluginGrantKey, PluginGrantScope},
};

/// One product action that can deliver a supported plugin trigger.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginActionPreview {
    /// Stable action identifier consumed by the Vue Plugin Manager.
    pub kind: String,
    /// Human-readable action label.
    pub label: String,
    /// Exact typed trigger delivered by this action.
    pub trigger: String,
}

/// One required manifest dependency shown before graph resolution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginDependencyPreview {
    /// Stable logical edge name from the plugin manifest.
    pub name: String,
    /// Canonical package identity required by the edge.
    pub package: String,
    /// Author-declared SemVer requirement delegated to graph resolution.
    pub requirement: String,
    /// Typed imports that the dependency must provide.
    pub imports: Vec<String>,
    /// Exact caller release that declares this resolved edge.
    pub caller: ReleaseIdentity,
    /// Exact dependency release selected by the approved deployment lock.
    pub release: ReleaseIdentity,
}

/// Content-bound deployment lock retained for offline confirmation and restart diagnostics.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginDeploymentLock {
    /// Path below the product-owned `n-plugin` data directory.
    pub relative_path: String,
    /// SHA-256 digest of the exact normalized lock bytes reviewed by preflight.
    pub digest: String,
}

/// Consent scope attached to one product capability requirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginCapabilityAccountScope {
    /// Consent belongs to the currently authenticated account.
    CurrentAccount,
    /// Consent belongs to the application installation.
    Application,
}

/// One capability requirement derived from a typed host import.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCapabilityPreview {
    /// Opaque deterministic identifier returned unchanged by the consent UI.
    pub requirement_id: String,
    /// Exact Component node that imports the host interface.
    pub subject: ReleaseIdentity,
    /// Stable product-owned logical host edge independent from activation generation.
    pub logical_edge: String,
    /// Stable product capability identifier shown in consent UI.
    pub capability: String,
    /// Exact typed host interface that requires this capability.
    pub host_interface: String,
    /// Account or application scope for the eventual consent grant.
    pub account_scope: PluginCapabilityAccountScope,
    /// Exact public account identity covered by account-scoped consent.
    pub account_id: Option<String>,
    /// Structured payload-free capability scope.
    pub scope: PluginGrantScope,
}

impl PluginCapabilityPreview {
    /// Reconstructs the authoritative exact grant key represented by this preview row.
    ///
    /// # Errors
    ///
    /// Returns an error when preview fields no longer form a valid product grant key.
    pub fn grant_key(&self, root: ReleaseIdentity) -> Result<PluginGrantKey> {
        PluginGrantKey::new(
            root,
            self.subject.clone(),
            self.logical_edge.clone(),
            self.host_interface.clone(),
            self.capability.clone(),
            self.account_id.clone(),
            self.scope.clone(),
        )
    }
}

/// Read-only installation preview for one manifest-bearing WebAssembly Component.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInstallPreview {
    /// Digest-bound fingerprint of bytes, account, contracts, actions, and consent requirements.
    pub preview_id: String,
    /// Exact product contract registry fingerprint used for this compatibility decision.
    pub contract_fingerprint: String,
    /// Exact content-addressed plugin release inspected from the Component.
    pub release: ReleaseIdentity,
    /// Manifest triggers registered by this mlmail release.
    pub supported_triggers: Vec<String>,
    /// Product actions available for the supported entrypoints.
    pub actions: Vec<PluginActionPreview>,
    /// Required dependency declarations awaiting graph resolution.
    pub dependencies: Vec<PluginDependencyPreview>,
    /// Exact deployment lock used to resolve this complete graph, when resolution was available.
    pub deployment_lock: Option<PluginDeploymentLock>,
    /// Consent requirements derived from imports accepted by the host inventory.
    pub required_capabilities: Vec<PluginCapabilityPreview>,
    /// Whether the exact Component can be activated by the current dependency-free installer.
    pub compatible: bool,
    /// Stable diagnostic when activation compatibility fails.
    pub reason: Option<String>,
}

/// Inspects and compiles one in-memory singleton candidate without persistent writes.
///
/// # Errors
///
/// Returns an error when bytes are not a Component, the embedded manifest is missing or invalid,
/// or a runtime compiler cannot be constructed. Compatibility mismatches are returned as a
/// serializable preview so the caller can explain them before installation.
pub fn preflight_component(
    component: &[u8],
    contracts: &MlmailPluginContractRegistry,
) -> Result<PluginInstallPreview> {
    preflight_component_for_account(component, contracts, None)
}

/// Inspects one Component for an exact authenticated account without persistent writes.
///
/// # Errors
///
/// Returns the same structural failures as [`preflight_component`]. Account identity only enters
/// consent scope and the preview fingerprint; it never enters Component bytes or runtime metadata.
pub fn preflight_component_for_account(
    component: &[u8],
    contracts: &MlmailPluginContractRegistry,
    account_id: Option<&str>,
) -> Result<PluginInstallPreview> {
    n_plugin_runtime::ensure_component(component)?;
    let embedded = inspect_component(component)?;
    let supported_triggers = embedded
        .manifest
        .triggers
        .iter()
        .filter(|trigger| contracts.action_for(trigger).is_some())
        .map(|trigger| trigger.as_str().to_owned())
        .collect::<Vec<_>>();
    let actions = embedded
        .manifest
        .entrypoints
        .values()
        .filter(|entrypoint| embedded.manifest.triggers.contains(entrypoint))
        .filter_map(|entrypoint| {
            contracts
                .action_for(entrypoint)
                .map(|action| action_preview(action, entrypoint))
        })
        .collect::<Vec<_>>();
    let dependency_declarations = embedded
        .manifest
        .dependencies
        .iter()
        .map(|(name, dependency)| PluginDependencyPreview {
            name: name.clone(),
            package: dependency.package.clone(),
            requirement: dependency.requirement.clone(),
            imports: dependency
                .imports
                .iter()
                .map(|interface| interface.as_str().to_owned())
                .collect(),
            caller: embedded.release.clone(),
            release: ReleaseIdentity {
                package: dependency.package.clone(),
                version: dependency
                    .requirement
                    .strip_prefix('=')
                    .unwrap_or(&dependency.requirement)
                    .to_owned(),
                digest: String::new(),
            },
        })
        .collect::<Vec<_>>();
    let mut preview = PluginInstallPreview {
        preview_id: String::new(),
        contract_fingerprint: contracts.fingerprint(),
        release: embedded.release.clone(),
        supported_triggers,
        actions,
        dependencies: dependency_declarations,
        deployment_lock: None,
        required_capabilities: Vec::new(),
        compatible: false,
        reason: None,
    };

    if !preview.dependencies.is_empty() {
        preview.reason = Some(
            "required dependency graph installation is not supported by this mlmail release yet"
                .to_owned(),
        );
        return finalize_preview(component, preview);
    }

    let graph = ResolvedPluginGraph {
        root: embedded.release.clone(),
        nodes: vec![ResolvedNode {
            release: embedded.release,
            manifest: embedded.manifest,
            reference: "local-preflight".to_owned(),
            component: component.to_vec(),
        }],
        edges: Vec::new(),
        lock_file: PathBuf::new(),
    };
    let compiler = ActivationCompiler::new()?;
    let plan = match compiler.compile(
        &graph,
        ActivationGeneration::new(1)?,
        &contracts.activation_host_inventory(),
        &contracts.trigger_inventory(),
    ) {
        Ok(plan) => plan,
        Err(error) => {
            preview.reason = Some(format!("{error:#}"));
            return finalize_preview(component, preview);
        }
    };

    if preview.actions.is_empty() {
        preview.reason =
            Some("plugin has no entrypoint supported by this mlmail release".to_owned());
        return finalize_preview(component, preview);
    }

    preview.required_capabilities = capability_previews(
        &plan.host_interfaces,
        contracts,
        &preview.release,
        account_id,
    )?;
    preview.compatible = true;
    finalize_preview(component, preview)
}

/// Builds a typed installation preview from a complete exact dependency graph.
///
/// # Errors
///
/// Returns an error when the graph root differs from the supplied Component, deployment lock
/// evidence is absent, or the runtime compiler cannot be constructed. Compatibility failures are
/// returned in the preview without publishing activation state.
pub fn preflight_resolved_graph_for_account(
    component: &[u8],
    graph: &ResolvedPluginGraph,
    deployment_lock: PluginDeploymentLock,
    contracts: &MlmailPluginContractRegistry,
    account_id: Option<&str>,
) -> Result<PluginInstallPreview> {
    n_plugin_runtime::ensure_component(component)?;
    let embedded = inspect_component(component)?;
    if graph.root != embedded.release {
        anyhow::bail!("resolved dependency graph belongs to another exact root release");
    }
    let root = graph
        .nodes
        .iter()
        .find(|node| node.release == graph.root)
        .ok_or_else(|| anyhow::anyhow!("resolved dependency graph is missing its exact root"))?;
    if root.component != component || root.manifest != embedded.manifest {
        anyhow::bail!("resolved dependency graph root differs from preflight Component bytes");
    }

    let supported_triggers = embedded
        .manifest
        .triggers
        .iter()
        .filter(|trigger| contracts.action_for(trigger).is_some())
        .map(|trigger| trigger.as_str().to_owned())
        .collect::<Vec<_>>();
    let actions = embedded
        .manifest
        .entrypoints
        .values()
        .filter(|entrypoint| embedded.manifest.triggers.contains(entrypoint))
        .filter_map(|entrypoint| {
            contracts
                .action_for(entrypoint)
                .map(|action| action_preview(action, entrypoint))
        })
        .collect::<Vec<_>>();
    let dependencies = graph
        .edges
        .iter()
        .map(|edge| PluginDependencyPreview {
            name: edge.name.clone(),
            package: edge.callee.package.clone(),
            requirement: edge.requirement.clone(),
            imports: edge
                .imports
                .iter()
                .map(|interface| interface.as_str().to_owned())
                .collect(),
            caller: edge.caller.clone(),
            release: edge.callee.clone(),
        })
        .collect::<Vec<_>>();
    let mut preview = PluginInstallPreview {
        preview_id: String::new(),
        contract_fingerprint: contracts.fingerprint(),
        release: embedded.release,
        supported_triggers,
        actions,
        dependencies,
        deployment_lock: Some(deployment_lock),
        required_capabilities: Vec::new(),
        compatible: false,
        reason: None,
    };
    let compiler = ActivationCompiler::new()?;
    let plan = match compiler.compile(
        graph,
        ActivationGeneration::new(1)?,
        &contracts.activation_host_inventory(),
        &contracts.trigger_inventory(),
    ) {
        Ok(plan) => plan,
        Err(error) => {
            preview.reason = Some(format!("{error:#}"));
            return finalize_preview(component, preview);
        }
    };
    if preview.actions.is_empty() {
        preview.reason =
            Some("plugin has no entrypoint supported by this mlmail release".to_owned());
        return finalize_preview(component, preview);
    }
    preview.required_capabilities =
        graph_capability_previews(&plan, graph, contracts, &preview.release, account_id)?;
    preview.compatible = true;
    finalize_preview(component, preview)
}

/// Builds the immutable runtime edge policy for the exact approved dependency grants.
///
/// # Errors
///
/// Returns an error when a dependency grant cannot be matched to one generated activation edge or
/// when an unresolved product scope reaches confirmation.
pub fn activation_policy_for_grants(
    plan: &ActivationPlan,
    graph: &ResolvedPluginGraph,
    required: &[PluginGrantKey],
    approved: &[PluginGrantKey],
) -> Result<ActivationPolicy> {
    let mut policy = ActivationPolicy::for_plan(plan);
    let approved = approved.iter().cloned().collect::<BTreeSet<_>>();
    let required = required.iter().cloned().collect::<BTreeSet<_>>();
    if !approved.is_subset(&required) {
        anyhow::bail!("approved grants contain a requirement outside the exact preview");
    }
    for grant in required.iter().filter(|grant| grant.subject != graph.root) {
        let resolved_edge = graph
            .edges
            .iter()
            .find(|edge| {
                edge.callee == grant.subject
                    && dependency_host_edge(edge, &grant.host_interface) == grant.logical_edge
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "approved dependency grant does not match an exact inbound graph edge"
                )
            })?;
        let planned_edge = plan
            .edges
            .iter()
            .find(|edge| {
                edge.caller == resolved_edge.caller
                    && edge.name == resolved_edge.name
                    && edge.callee == resolved_edge.callee
            })
            .ok_or_else(|| anyhow::anyhow!("activation plan is missing an approved graph edge"))?;
        let access = CapabilityAccess {
            capability: grant.capability.clone(),
            scope: grant.scope.to_runtime_scope()?,
        };
        let edge = policy
            .edges
            .get_mut(&planned_edge.id)
            .expect("ActivationPolicy::for_plan creates every plan edge");
        edge.requirements.push(access.clone());
        if approved.contains(grant) {
            edge.grants.push(access);
        }
    }
    for edge in policy.edges.values_mut() {
        normalize_edge_policy(edge);
    }
    Ok(policy)
}

pub(crate) fn action_preview(
    action: MlmailPluginActionKind,
    trigger: &WitExportRef,
) -> PluginActionPreview {
    let (kind, label) = match action {
        MlmailPluginActionKind::BookingFinderFind => ("booking-finder-find", "Find bookings"),
        MlmailPluginActionKind::DraftHelperCreate => ("draft-helper-create", "Create draft"),
    };
    PluginActionPreview {
        kind: kind.to_owned(),
        label: label.to_owned(),
        trigger: trigger.as_str().to_owned(),
    }
}

fn capability_previews(
    host_interfaces: &[String],
    contracts: &MlmailPluginContractRegistry,
    release: &ReleaseIdentity,
    account_id: Option<&str>,
) -> Result<Vec<PluginCapabilityPreview>> {
    let mut requirements = Vec::new();
    for host_interface in host_interfaces {
        let identity = WitExportRef::parse(host_interface)?;
        if let Some(capabilities) = contracts.capability_requirements_for(&identity) {
            requirements.extend(
                capabilities
                    .iter()
                    .map(|capability| {
                        capability_preview(host_interface, *capability, release, account_id)
                    })
                    .collect::<Result<Vec<_>>>()?,
            );
        }
    }
    requirements.sort_by(|left, right| {
        left.capability
            .cmp(&right.capability)
            .then_with(|| left.host_interface.cmp(&right.host_interface))
    });
    Ok(requirements)
}

fn graph_capability_previews(
    plan: &ActivationPlan,
    graph: &ResolvedPluginGraph,
    contracts: &MlmailPluginContractRegistry,
    root: &ReleaseIdentity,
    account_id: Option<&str>,
) -> Result<Vec<PluginCapabilityPreview>> {
    let host_interfaces = plan
        .host_interfaces
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut requirements = Vec::new();
    for component in &plan.components {
        for host_interface in component
            .imports
            .iter()
            .filter(|name| host_interfaces.contains(name.as_str()))
        {
            let identity = WitExportRef::parse(host_interface)?;
            if is_no_consent_runtime_interface(&identity) {
                continue;
            }
            let capabilities = contracts
                .capability_requirements_for(&identity)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "compiled host interface `{host_interface}` has no capability mapping"
                    )
                })?;
            let inbound = if component.release == *root {
                Vec::new()
            } else {
                graph
                    .edges
                    .iter()
                    .filter(|edge| edge.callee == component.release)
                    .collect::<Vec<_>>()
            };
            if component.release != *root && inbound.is_empty() {
                anyhow::bail!("dependency host import has no explicit inbound graph edge");
            }
            if component.release == *root {
                requirements.extend(
                    capabilities
                        .iter()
                        .map(|capability| {
                            capability_preview_for_edge(
                                host_interface,
                                *capability,
                                root,
                                &component.release,
                                format!("root-host:{host_interface}"),
                                account_id,
                            )
                        })
                        .collect::<Result<Vec<_>>>()?,
                );
            } else {
                for edge in inbound {
                    requirements.extend(
                        capabilities
                            .iter()
                            .map(|capability| {
                                capability_preview_for_edge(
                                    host_interface,
                                    *capability,
                                    root,
                                    &component.release,
                                    dependency_host_edge(edge, host_interface),
                                    account_id,
                                )
                            })
                            .collect::<Result<Vec<_>>>()?,
                    );
                }
            }
        }
    }
    requirements.sort_by(|left, right| {
        left.subject
            .cmp(&right.subject)
            .then_with(|| left.logical_edge.cmp(&right.logical_edge))
            .then_with(|| left.capability.cmp(&right.capability))
            .then_with(|| left.host_interface.cmp(&right.host_interface))
    });
    requirements.dedup();
    Ok(requirements)
}

fn capability_preview(
    host_interface: &str,
    requirement: PluginCapabilityRequirement,
    release: &ReleaseIdentity,
    account_id: Option<&str>,
) -> Result<PluginCapabilityPreview> {
    capability_preview_for_edge(
        host_interface,
        requirement,
        release,
        release,
        format!("root-host:{host_interface}"),
        account_id,
    )
}

fn capability_preview_for_edge(
    host_interface: &str,
    requirement: PluginCapabilityRequirement,
    root: &ReleaseIdentity,
    subject: &ReleaseIdentity,
    logical_edge: String,
    account_id: Option<&str>,
) -> Result<PluginCapabilityPreview> {
    let account_id = requirement
        .account_scoped
        .then(|| account_id.map(str::to_owned))
        .flatten();
    let scope = if requirement.account_scoped {
        account_id
            .clone()
            .map_or(PluginGrantScope::UnresolvedMailAccount, |account_id| {
                PluginGrantScope::MailAccount { account_id }
            })
    } else {
        PluginGrantScope::Application
    };
    let requirement_id = fingerprint(&serde_json::to_vec(&(
        root,
        subject,
        &logical_edge,
        host_interface,
        requirement.capability,
        &account_id,
        &scope,
    ))?);
    Ok(PluginCapabilityPreview {
        requirement_id,
        subject: subject.clone(),
        logical_edge,
        capability: requirement.capability.to_owned(),
        host_interface: host_interface.to_owned(),
        account_scope: if requirement.account_scoped {
            PluginCapabilityAccountScope::CurrentAccount
        } else {
            PluginCapabilityAccountScope::Application
        },
        account_id,
        scope,
    })
}

fn dependency_host_edge(edge: &ResolvedEdge, host_interface: &str) -> String {
    format!(
        "dependency-host:{}:{}:{}#{}->{}:{}:{}:{host_interface}",
        edge.caller.package,
        edge.caller.version,
        edge.caller.digest,
        edge.name,
        edge.callee.package,
        edge.callee.version,
        edge.callee.digest,
    )
}

fn normalize_edge_policy(policy: &mut EdgeCapabilityPolicy) {
    let key = |access: &CapabilityAccess| {
        serde_json::to_string(access).expect("CapabilityAccess serialization is infallible")
    };
    policy.requirements.sort_by_key(&key);
    policy.requirements.dedup();
    policy.grants.sort_by_key(&key);
    policy.grants.dedup();
}

fn finalize_preview(
    component: &[u8],
    mut preview: PluginInstallPreview,
) -> Result<PluginInstallPreview> {
    let component_digest = fingerprint(component);
    let canonical = serde_json::to_vec(&(
        component_digest,
        &preview.contract_fingerprint,
        &preview.release,
        &preview.supported_triggers,
        &preview.actions,
        &preview.dependencies,
        &preview.deployment_lock,
        &preview.required_capabilities,
        preview.compatible,
        &preview.reason,
    ))?;
    preview.preview_id = fingerprint(&canonical);
    Ok(preview)
}

fn fingerprint(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use n_plugin_package::{embed_manifest, PluginManifest};

    use super::*;
    use crate::plugin_contracts::{
        MlmailPluginContractRegistry, GMAIL_BOOKING_FINDER_INTERFACE, GMAIL_DRAFTS_INTERFACE,
        GMAIL_DRAFT_HELPER_INTERFACE, GMAIL_SEARCH_INTERFACE,
    };

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
    async fn previews_draft_helper_and_booking_finder_through_one_path() -> Result<()> {
        let registry = registry().await?;
        let draft = packaged_component(
            "other",
            "draft-helper",
            "create",
            GMAIL_DRAFT_HELPER_INTERFACE,
            Some(GMAIL_DRAFTS_INTERFACE),
            &[GMAIL_DRAFT_HELPER_INTERFACE],
            "",
        )?;
        let booking = packaged_component(
            "another",
            "booking-finder",
            "find",
            GMAIL_BOOKING_FINDER_INTERFACE,
            Some(GMAIL_SEARCH_INTERFACE),
            &[GMAIL_BOOKING_FINDER_INTERFACE],
            "",
        )?;

        let draft = preflight_component(&draft, &registry)?;
        let booking = preflight_component(&booking, &registry)?;

        assert!(draft.compatible, "unexpected reason: {:?}", draft.reason);
        assert!(
            booking.compatible,
            "unexpected reason: {:?}",
            booking.reason
        );
        assert_eq!(draft.release.package, "other:draft-helper");
        assert_eq!(booking.release.package, "another:booking-finder");
        assert_eq!(draft.supported_triggers, [GMAIL_DRAFT_HELPER_INTERFACE]);
        assert_eq!(booking.supported_triggers, [GMAIL_BOOKING_FINDER_INTERFACE]);
        assert_eq!(draft.actions[0].kind, "draft-helper-create");
        assert_eq!(booking.actions[0].kind, "booking-finder-find");
        assert_eq!(
            draft.required_capabilities[0].capability,
            "mail:draft.create"
        );
        assert_eq!(booking.required_capabilities[0].capability, "mail:search");
        Ok(())
    }

    #[tokio::test]
    async fn never_downgrades_an_unresolved_account_requirement_to_application_scope() -> Result<()>
    {
        let registry = registry().await?;
        let component = packaged_component(
            "other",
            "booking-finder",
            "find",
            GMAIL_BOOKING_FINDER_INTERFACE,
            Some(GMAIL_SEARCH_INTERFACE),
            &[GMAIL_BOOKING_FINDER_INTERFACE],
            "",
        )?;

        let preview = preflight_component(&component, &registry)?;
        let requirement = &preview.required_capabilities[0];

        assert_eq!(requirement.account_id, None);
        assert_eq!(requirement.scope, PluginGrantScope::UnresolvedMailAccount);
        assert!(requirement.grant_key(preview.release).is_err());
        Ok(())
    }

    #[tokio::test]
    async fn binds_the_preview_to_the_exact_account_identity() -> Result<()> {
        let registry = registry().await?;
        let component = packaged_component(
            "other",
            "booking-finder",
            "find",
            GMAIL_BOOKING_FINDER_INTERFACE,
            Some(GMAIL_SEARCH_INTERFACE),
            &[GMAIL_BOOKING_FINDER_INTERFACE],
            "",
        )?;

        let first =
            preflight_component_for_account(&component, &registry, Some("first@example.com"))?;
        let second =
            preflight_component_for_account(&component, &registry, Some("second@example.com"))?;

        assert_ne!(first.preview_id, second.preview_id);
        assert_ne!(
            first.required_capabilities[0].requirement_id,
            second.required_capabilities[0].requirement_id
        );
        Ok(())
    }

    #[tokio::test]
    async fn returns_incompatible_preview_for_unknown_trigger() -> Result<()> {
        let registry = registry().await?;
        let component = packaged_component(
            "other",
            "unknown-trigger",
            "run",
            "other:unknown-trigger/run@0.1.0",
            None,
            &["other:unknown-trigger/run@0.1.0"],
            "",
        )?;

        let preview = preflight_component(&component, &registry)?;

        assert!(!preview.compatible);
        assert!(preview
            .reason
            .as_deref()
            .is_some_and(|reason| reason.contains("did not register")));
        assert!(preview.supported_triggers.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn returns_incompatible_preview_for_unknown_host_import() -> Result<()> {
        let registry = registry().await?;
        let component = packaged_component(
            "other",
            "unknown-host",
            "create",
            GMAIL_DRAFT_HELPER_INTERFACE,
            Some("other:host/unknown@0.1.0"),
            &[GMAIL_DRAFT_HELPER_INTERFACE],
            "",
        )?;

        let preview = preflight_component(&component, &registry)?;

        assert!(!preview.compatible);
        assert!(preview
            .reason
            .as_deref()
            .is_some_and(|reason| reason.contains("did not register")));
        Ok(())
    }

    #[tokio::test]
    async fn accepts_measured_wasi_without_adding_consent() -> Result<()> {
        let registry = registry().await?;
        let component = packaged_component(
            "other",
            "wasi-import",
            "create",
            GMAIL_DRAFT_HELPER_INTERFACE,
            Some("wasi:cli/environment@0.2.9"),
            &[GMAIL_DRAFT_HELPER_INTERFACE],
            "",
        )?;

        let preview = preflight_component(&component, &registry)?;

        assert!(preview.compatible, "{:?}", preview.reason);
        assert!(preview.required_capabilities.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn rejects_wasi_outside_the_measured_subset() -> Result<()> {
        let registry = registry().await?;
        let component = packaged_component(
            "other",
            "wasi-filesystem-import",
            "create",
            GMAIL_DRAFT_HELPER_INTERFACE,
            Some("wasi:filesystem/types@0.2.9"),
            &[GMAIL_DRAFT_HELPER_INTERFACE],
            "",
        )?;

        let preview = preflight_component(&component, &registry)?;

        assert!(!preview.compatible);
        assert!(preview
            .reason
            .as_deref()
            .is_some_and(|reason| reason.contains("wasi:filesystem/types@0.2.9")));
        Ok(())
    }

    #[tokio::test]
    async fn rejects_core_wasm_and_component_without_manifest() -> Result<()> {
        let registry = registry().await?;

        let core_error = preflight_component(b"\0asm\x01\0\0\0", &registry)
            .expect_err("core Wasm must be rejected");
        let component = wat::parse_str("(component)")?;
        let manifest_error = preflight_component(&component, &registry)
            .expect_err("manifest-free Component must be rejected");

        assert!(core_error.to_string().contains("Components only"));
        assert!(manifest_error.to_string().contains("plugin-manifest"));
        Ok(())
    }

    #[tokio::test]
    async fn returns_incompatible_preview_without_supported_entrypoints() -> Result<()> {
        let registry = registry().await?;
        let component = packaged_component(
            "other",
            "unsupported-action",
            "run",
            "other:unsupported-action/run@0.1.0",
            None,
            &[],
            "",
        )?;

        let preview = preflight_component(&component, &registry)?;

        assert!(!preview.compatible);
        assert!(preview.actions.is_empty());
        assert!(preview
            .reason
            .as_deref()
            .is_some_and(|reason| reason.contains("entrypoint")));
        Ok(())
    }

    #[tokio::test]
    async fn reports_required_dependency_without_activating_it() -> Result<()> {
        let registry = registry().await?;
        let dependency = r#"
[dependencies.provider]
package = "third:provider"
requirement = "=1.0.0"
imports = ["third:provider/api@1.0.0"]
"#;
        let component = packaged_component(
            "other",
            "dependent",
            "create",
            GMAIL_DRAFT_HELPER_INTERFACE,
            Some("third:provider/api@1.0.0"),
            &[GMAIL_DRAFT_HELPER_INTERFACE],
            dependency,
        )?;

        let preview = preflight_component(&component, &registry)?;

        assert!(!preview.compatible);
        assert_eq!(preview.dependencies.len(), 1);
        assert_eq!(preview.dependencies[0].name, "provider");
        assert_eq!(preview.dependencies[0].package, "third:provider");
        assert_eq!(preview.dependencies[0].requirement, "=1.0.0");
        assert!(preview
            .reason
            .as_deref()
            .is_some_and(|reason| reason.contains("dependency graph")));
        Ok(())
    }

    #[tokio::test]
    async fn binds_dependency_consent_to_one_exact_generated_edge() -> Result<()> {
        let contracts = registry().await?;
        let provider_interface = "third:provider/api@0.1.0";
        let dependency = format!(
            r#"
[dependencies.provider]
package = "third:provider"
requirement = "=0.1.0"
imports = ["{provider_interface}"]
"#,
        );
        let root_component = packaged_component(
            "other",
            "dependent",
            "create",
            GMAIL_DRAFT_HELPER_INTERFACE,
            Some(provider_interface),
            &[GMAIL_DRAFT_HELPER_INTERFACE],
            &dependency,
        )?;
        let provider_component = packaged_component(
            "third",
            "provider",
            "provide",
            provider_interface,
            Some(GMAIL_DRAFTS_INTERFACE),
            &[],
            "",
        )?;
        let root = inspect_component(&root_component)?;
        let provider = inspect_component(&provider_component)?;
        let graph = ResolvedPluginGraph {
            root: root.release.clone(),
            nodes: vec![
                ResolvedNode {
                    release: root.release.clone(),
                    manifest: root.manifest,
                    reference: "local:root".to_owned(),
                    component: root_component.clone(),
                },
                ResolvedNode {
                    release: provider.release.clone(),
                    manifest: provider.manifest,
                    reference: "local:provider".to_owned(),
                    component: provider_component,
                },
            ],
            edges: vec![ResolvedEdge {
                caller: root.release.clone(),
                name: "provider".to_owned(),
                requirement: "=0.1.0".to_owned(),
                imports: vec![WitExportRef::parse(provider_interface)?],
                callee: provider.release.clone(),
            }],
            lock_file: PathBuf::from("fixture.n-plugin.lock"),
        };
        let preview = preflight_resolved_graph_for_account(
            &root_component,
            &graph,
            PluginDeploymentLock {
                relative_path: "locks/fixture.n-plugin.lock".to_owned(),
                digest: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_owned(),
            },
            &contracts,
            Some("person@example.com"),
        )?;

        assert!(
            preview.compatible,
            "unexpected reason: {:?}",
            preview.reason
        );
        assert_eq!(preview.required_capabilities.len(), 1);
        assert_eq!(preview.required_capabilities[0].subject, provider.release);
        let required = preview
            .required_capabilities
            .iter()
            .map(|requirement| requirement.grant_key(preview.release.clone()))
            .collect::<Result<Vec<_>>>()?;
        let plan = ActivationCompiler::new()?.compile(
            &graph,
            ActivationGeneration::new(7)?,
            &contracts.activation_host_inventory(),
            &contracts.trigger_inventory(),
        )?;
        let denied = activation_policy_for_grants(&plan, &graph, &required, &[])?;
        let allowed = activation_policy_for_grants(&plan, &graph, &required, &required)?;
        let edge = &plan.edges[0].id;
        assert_eq!(denied.edges[edge].requirements.len(), 1);
        assert!(denied.edges[edge].grants.is_empty());
        assert_eq!(allowed.edges[edge].requirements, allowed.edges[edge].grants);
        Ok(())
    }

    #[tokio::test]
    async fn pure_preflight_does_not_write_plugin_state() -> Result<()> {
        let registry = registry().await?;
        let untouched = tempfile::tempdir()?;
        let component = packaged_component(
            "other",
            "draft-helper",
            "create",
            GMAIL_DRAFT_HELPER_INTERFACE,
            Some(GMAIL_DRAFTS_INTERFACE),
            &[GMAIL_DRAFT_HELPER_INTERFACE],
            "",
        )?;

        let preview = preflight_component(&component, &registry)?;

        assert!(preview.compatible);
        for relative in [
            "registry.sqlite3",
            "cas",
            "installed.json",
            "context.sqlite3",
            ".n-plugin.lock",
        ] {
            assert!(
                !untouched.path().join(relative).exists(),
                "unexpected {relative}"
            );
        }
        Ok(())
    }

    async fn registry() -> Result<MlmailPluginContractRegistry> {
        let directory = tempfile::tempdir()?;
        let lock_path = directory.path().join("wkg.lock");
        fs::write(&lock_path, LOCK)?;
        MlmailPluginContractRegistry::load(lock_path).await
    }

    fn packaged_component(
        publisher: &str,
        package: &str,
        entrypoint_name: &str,
        entrypoint: &str,
        host_import: Option<&str>,
        triggers: &[&str],
        dependencies: &str,
    ) -> Result<Vec<u8>> {
        let triggers = serde_json::to_string(triggers)?;
        let manifest = PluginManifest::from_toml(&format!(
            r#"
schema = "nitra.plugin-manifest/v1"
publisher_id = "{publisher}"
package = "{package}"
version = "0.1.0"
triggers = {triggers}

[entrypoints]
{entrypoint_name} = "{entrypoint}"
{dependencies}
"#,
        ))?;
        let component = component(entrypoint, host_import)?;
        Ok(embed_manifest(&component, &manifest)?)
    }

    fn component(entrypoint: &str, host_import: Option<&str>) -> Result<Vec<u8>> {
        let host = host_import.map_or_else(String::new, |interface| {
            format!(
                r#"
  (type $host-contract (instance))
  (import "{interface}" (instance (type $host-contract)))
"#,
            )
        });
        Ok(wat::parse_str(format!(
            r#"
(component
{host}
  (core module $module)
  (core instance $core (instantiate $module))
  (instance $api)
  (export "{entrypoint}" (instance $api))
)
"#,
        ))?)
    }
}
