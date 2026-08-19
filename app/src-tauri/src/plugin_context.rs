//! Durable product context for the built-in Gmail provider and exact installed plugin roots.
//!
//! The native provider and guest plugin are distinct exact-release instances. The coordinator
//! replays their desired state before Gmail side effects are allowed and keeps OAuth credentials
//! outside durable Component metadata.

use std::{
    collections::BTreeMap,
    future::Future,
    path::Path,
    pin::Pin,
    sync::{Arc, Mutex},
};

use anyhow::{bail, Context, Result};
use n_plugin_package::{ReleaseIdentity, WitExportRef};
use n_plugin_runtime::{
    ContextActivation, ContextComponent, ContextComponentId, ContextDeactivation,
    ContextExecutorSnapshot, ContextLifecycleState, ContextRecovery, ContextRestartRepair,
    ContextTransitionDriver, DurableContextEntry, DurableContextPublication, DurableContextRuntime,
    DurableContextStore, RevertibleEffectScope,
};

use crate::gmail::plugin_runtime::{GMAIL_DRAFTS_INTERFACE, GMAIL_SEARCH_INTERFACE};

const GMAIL_PROVIDER_INSTANCE: &str = "mlmail:gmail-provider";
const GMAIL_PROVIDER_PACKAGE: &str = "vitaliytv:mlmail-gmail-provider";
const GMAIL_PROVIDER_VERSION: &str = "0.1.0";
const GMAIL_PROVIDER_DIGEST: &str =
    "sha256:d788813ab60e9a038928ba5aece756d72a24e20bf8e2edc11f1d2d2fab65a0dd";

type DriverFuture<'a> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

/// Compatibility helper for one installed Draft Helper release.
///
/// The Gmail provider stays manually enabled. Disabling Draft Helper preserves its exact entry
/// without allowing dependency availability to override the user's choice.
pub fn draft_helper_context(
    draft_helper: Option<(ReleaseIdentity, bool)>,
) -> Result<Vec<DurableContextEntry>> {
    let drafts = WitExportRef::parse(GMAIL_DRAFTS_INTERFACE)?;
    plugin_context(
        draft_helper
            .into_iter()
            .map(|(release, enabled)| PluginContextEntry {
                context_id: "plugin:nitra:draft-helper".to_owned(),
                release,
                host_interfaces: vec![drafts.clone()],
                enabled,
            }),
    )
}

/// One exact installed root represented in the durable product context.
pub struct PluginContextEntry {
    /// Stable package-scoped instance identity used by command dispatch.
    pub context_id: String,
    /// Exact immutable release expected at the context boundary.
    pub release: ReleaseIdentity,
    /// Exact product host interfaces required by this installed generation.
    pub host_interfaces: Vec<WitExportRef>,
    /// Explicit user enablement retained in durable desired state.
    pub enabled: bool,
}

/// Builds the complete desired context for the native Gmail provider and installed roots.
///
/// # Errors
///
/// Returns an error when a durable component identity or compiled Gmail interface is invalid.
pub fn plugin_context(
    plugins: impl IntoIterator<Item = PluginContextEntry>,
) -> Result<Vec<DurableContextEntry>> {
    context_entries(native_gmail_provider_release(), plugins)
}

/// Path of the product context database below the application data directory.
#[must_use]
pub fn context_database(app_data: &Path) -> std::path::PathBuf {
    app_data.join("n-plugin").join("context.sqlite3")
}

/// Running mlmail coordinator that gates online plugin work on a settled durable context.
pub struct PluginContextCoordinator {
    runtime: DurableContextRuntime,
    desired_releases: BTreeMap<ContextComponentId, ReleaseIdentity>,
}

impl PluginContextCoordinator {
    /// Repairs interrupted process state and activates the last committed context without network.
    ///
    /// # Errors
    ///
    /// Returns an error when durable repair, context replay or lifecycle execution fails.
    pub async fn start(database: impl AsRef<Path>) -> Result<Self> {
        Self::start_with_driver(database, MlmailContextDriver::default()).await
    }

    async fn start_with_driver(
        database: impl AsRef<Path>,
        driver: MlmailContextDriver,
    ) -> Result<Self> {
        let store = DurableContextStore::open(database)?;
        let desired_releases = desired_release_map(
            store
                .active()?
                .into_iter()
                .flat_map(|desired| desired.entries),
        );
        let runtime = DurableContextRuntime::start(
            store,
            driver.clone(),
            MlmailRestartRepair {
                events: driver.events,
            },
        )
        .await?;
        Ok(Self {
            runtime,
            desired_releases,
        })
    }

    /// Atomically replaces the complete desired context and waits for all runnable transitions.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid context, persistence failure or executor shutdown.
    pub async fn replace_desired(
        &mut self,
        entries: impl IntoIterator<Item = DurableContextEntry>,
    ) -> Result<ContextExecutorSnapshot> {
        let entries = entries.into_iter().collect::<Vec<_>>();
        let desired_releases = desired_release_map(entries.iter().cloned());
        let publication = self.runtime.publish_desired(entries)?;
        let snapshot = self.wait_for_publication(publication).await?;
        self.desired_releases = desired_releases;
        Ok(snapshot)
    }

    /// Executes an online plugin action only while provider and exact target instances are active.
    ///
    /// Gmail draft creation is an emission, not a reversible lifecycle acquisition. The action is
    /// therefore delayed until activation commits and is never registered for automatic rollback.
    ///
    /// # Errors
    ///
    /// Returns an error when either instance is unavailable or the action itself fails.
    pub async fn run_plugin<T, F, Fut>(
        &self,
        context_id: &str,
        target: &ReleaseIdentity,
        action: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let snapshot = self.runtime.snapshot();
        require_active(&snapshot, &component_id(GMAIL_PROVIDER_INSTANCE)?)?;
        let component = component_id(context_id)?;
        require_active(&snapshot, &component)?;
        if self.desired_releases.get(&component) != Some(target) {
            bail!(
                "Component context instance `{component}` does not match the exact target release"
            )
        }
        action().await
    }

    /// Gracefully unloads process instances while retaining the desired generation for restart.
    ///
    /// # Errors
    ///
    /// Returns an error when the executor cannot reach its recovery boundary.
    pub async fn shutdown(self) -> Result<ContextExecutorSnapshot> {
        Ok(self.runtime.shutdown().await?)
    }

    async fn wait_for_publication(
        &mut self,
        publication: DurableContextPublication,
    ) -> Result<ContextExecutorSnapshot> {
        Ok(self.runtime.wait_for_settled(publication).await?)
    }
}

fn require_active(
    snapshot: &ContextExecutorSnapshot,
    component: &ContextComponentId,
) -> Result<()> {
    if snapshot.lifecycle(component) == Some(ContextLifecycleState::Active) {
        return Ok(());
    }
    bail!("Component context instance `{component}` is not active")
}

fn native_gmail_provider_release() -> ReleaseIdentity {
    ReleaseIdentity {
        package: GMAIL_PROVIDER_PACKAGE.to_owned(),
        version: GMAIL_PROVIDER_VERSION.to_owned(),
        digest: GMAIL_PROVIDER_DIGEST.to_owned(),
    }
}

fn context_entries(
    provider_release: ReleaseIdentity,
    plugins: impl IntoIterator<Item = PluginContextEntry>,
) -> Result<Vec<DurableContextEntry>> {
    let drafts = WitExportRef::parse(GMAIL_DRAFTS_INTERFACE)?;
    let search = WitExportRef::parse(GMAIL_SEARCH_INTERFACE)?;
    let provider = ContextComponent::new(
        component_id(GMAIL_PROVIDER_INSTANCE)?,
        provider_release,
        [],
        [drafts, search],
    );
    let mut entries = vec![DurableContextEntry::enabled(provider)];
    for plugin in plugins {
        let component = ContextComponent::new(
            component_id(&plugin.context_id)?,
            plugin.release,
            plugin.host_interfaces,
            [],
        );
        entries.push(if plugin.enabled {
            DurableContextEntry::enabled(component)
        } else {
            DurableContextEntry::disabled(component)
        });
    }
    Ok(entries)
}

fn desired_release_map(
    entries: impl IntoIterator<Item = DurableContextEntry>,
) -> BTreeMap<ContextComponentId, ReleaseIdentity> {
    entries
        .into_iter()
        .map(|entry| (entry.component.id, entry.component.release))
        .collect()
}

fn component_id(value: &str) -> Result<ContextComponentId> {
    ContextComponentId::parse(value).map_err(anyhow::Error::new)
}

#[derive(Clone, Default)]
struct MlmailContextDriver {
    active: Arc<Mutex<BTreeMap<ContextComponentId, ReleaseIdentity>>>,
    events: Arc<Mutex<Vec<String>>>,
}

impl MlmailContextDriver {
    fn record(&self, event: String) -> Result<()> {
        self.events
            .lock()
            .map_err(|_| anyhow::anyhow!("plugin context event lock is poisoned"))?
            .push(event);
        Ok(())
    }
}

impl ContextTransitionDriver for MlmailContextDriver {
    fn load<'a>(
        &'a self,
        activation: ContextActivation,
        effects: &'a mut RevertibleEffectScope,
    ) -> DriverFuture<'a> {
        Box::pin(async move {
            let mut active = self
                .active
                .lock()
                .map_err(|_| anyhow::anyhow!("plugin context active lock is poisoned"))?;
            if active
                .insert(activation.component.clone(), activation.release.clone())
                .is_some()
            {
                bail!(
                    "Component context instance `{}` is already active",
                    activation.component
                );
            }
            drop(active);
            self.record(format!(
                "load:{}:{}",
                activation.component, activation.release.digest
            ))?;

            let active = Arc::clone(&self.active);
            let events = Arc::clone(&self.events);
            let component = activation.component;
            effects.track_acquisition(
                format!("context-instance:{component}"),
                move || async move {
                    active
                        .lock()
                        .map_err(|_| anyhow::anyhow!("plugin context active lock is poisoned"))?
                        .remove(&component);
                    events
                        .lock()
                        .map_err(|_| anyhow::anyhow!("plugin context event lock is poisoned"))?
                        .push(format!("recover:{component}"));
                    Ok(())
                },
            );
            Ok(())
        })
    }

    fn drain(&self, deactivation: ContextDeactivation) -> DriverFuture<'_> {
        Box::pin(async move {
            let active = self
                .active
                .lock()
                .map_err(|_| anyhow::anyhow!("plugin context active lock is poisoned"))?;
            let release = active
                .get(&deactivation.component)
                .with_context(|| format!("Component `{}` is not active", deactivation.component))?;
            if release != &deactivation.release {
                bail!(
                    "Component `{}` changed release before drain",
                    deactivation.component
                );
            }
            drop(active);
            self.record(format!(
                "drain:{}:{}",
                deactivation.component, deactivation.release.digest
            ))
        })
    }

    fn recovered(&self, recovery: ContextRecovery) -> DriverFuture<'_> {
        Box::pin(async move { self.record(format!("recovered:{}", recovery.component)) })
    }
}

struct MlmailRestartRepair {
    events: Arc<Mutex<Vec<String>>>,
}

impl ContextRestartRepair for MlmailRestartRepair {
    fn repair(&self, instance: n_plugin_runtime::DurableContextInstance) -> DriverFuture<'_> {
        Box::pin(async move {
            self.events
                .lock()
                .map_err(|_| anyhow::anyhow!("plugin context event lock is poisoned"))?
                .push(format!("repair:{}", instance.component));
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft_entries(
        provider: ReleaseIdentity,
        helper: ReleaseIdentity,
        enabled: bool,
    ) -> Result<Vec<DurableContextEntry>> {
        context_entries(
            provider,
            [PluginContextEntry {
                context_id: "plugin:nitra:draft-helper".to_owned(),
                release: helper,
                host_interfaces: vec![WitExportRef::parse(GMAIL_DRAFTS_INTERFACE)?],
                enabled,
            }],
        )
    }

    fn release(package: &str, digest: &str) -> ReleaseIdentity {
        ReleaseIdentity {
            package: package.to_owned(),
            version: "1.0.0".to_owned(),
            digest: digest.to_owned(),
        }
    }

    fn recorded(driver: &MlmailContextDriver) -> Vec<String> {
        driver
            .events
            .lock()
            .expect("plugin context event lock should remain available")
            .clone()
    }

    #[tokio::test]
    async fn replaces_provider_around_the_dependent_instance() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let driver = MlmailContextDriver::default();
        let mut coordinator = PluginContextCoordinator::start_with_driver(
            directory.path().join("context.sqlite3"),
            driver.clone(),
        )
        .await?;
        let helper = release("nitra:draft-helper", "sha256:helper");
        let provider_v1 = release("vitaliytv:gmail-provider", "sha256:provider-v1");
        let provider_v2 = release("vitaliytv:gmail-provider", "sha256:provider-v2");

        coordinator
            .replace_desired(draft_entries(provider_v1, helper.clone(), true)?)
            .await?;
        coordinator
            .replace_desired(draft_entries(provider_v2, helper, true)?)
            .await?;

        let events = recorded(&driver);
        let helper_drain = events
            .iter()
            .position(|event| event.starts_with("drain:plugin:nitra:draft-helper"))
            .context("Draft Helper should drain for provider replacement")?;
        let provider_drain = events
            .iter()
            .position(|event| event.starts_with("drain:mlmail:gmail-provider"))
            .context("old Gmail provider should drain")?;
        let provider_v2_load = events
            .iter()
            .position(|event| event == "load:mlmail:gmail-provider:sha256:provider-v2")
            .context("replacement Gmail provider should load")?;
        let helper_v2_load = events
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, event)| {
                (event == "load:plugin:nitra:draft-helper:sha256:helper").then_some(index)
            })
            .context("Draft Helper should reload")?;

        assert!(helper_drain < provider_drain);
        assert!(provider_drain < provider_v2_load);
        assert!(provider_v2_load < helper_v2_load);
        coordinator.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn replays_offline_context_and_repairs_an_interrupted_process() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let database = directory.path().join("context.sqlite3");
        let first_driver = MlmailContextDriver::default();
        let mut first =
            PluginContextCoordinator::start_with_driver(&database, first_driver.clone()).await?;
        first
            .replace_desired(draft_helper_context(Some((
                release("nitra:draft-helper", "sha256:helper"),
                true,
            )))?)
            .await?;
        drop(first);

        let restarted_driver = MlmailContextDriver::default();
        let restarted =
            PluginContextCoordinator::start_with_driver(&database, restarted_driver.clone())
                .await?;
        let helper = release("nitra:draft-helper", "sha256:helper");
        restarted
            .run_plugin("plugin:nitra:draft-helper", &helper, || async { Ok(()) })
            .await?;

        let events = recorded(&restarted_driver);
        assert_eq!(
            &events[..2],
            &[
                "repair:plugin:nitra:draft-helper",
                "repair:mlmail:gmail-provider",
            ]
        );
        assert_eq!(
            &events[2..4],
            &[
                format!("load:mlmail:gmail-provider:{GMAIL_PROVIDER_DIGEST}"),
                "load:plugin:nitra:draft-helper:sha256:helper".to_owned(),
            ]
        );
        restarted.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn gates_online_emission_until_both_instances_are_active() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let mut coordinator =
            PluginContextCoordinator::start(directory.path().join("context.sqlite3")).await?;
        let helper = release("nitra:draft-helper", "sha256:helper");
        let blocked = coordinator
            .run_plugin("plugin:nitra:draft-helper", &helper, || async {
                Ok("emitted")
            })
            .await
            .expect_err("emission must be blocked without committed instances");
        assert!(blocked.to_string().contains("not active"));

        coordinator
            .replace_desired(draft_helper_context(Some((helper.clone(), true)))?)
            .await?;
        let wrong_release = release("nitra:draft-helper", "sha256:other");
        let mismatch = coordinator
            .run_plugin("plugin:nitra:draft-helper", &wrong_release, || async {
                Ok("must-not-run")
            })
            .await
            .expect_err("exact release mismatch must block the action boundary");
        assert!(mismatch.to_string().contains("exact target release"));
        let emitted = coordinator
            .run_plugin("plugin:nitra:draft-helper", &helper, || async {
                Ok("emitted")
            })
            .await?;

        assert_eq!(emitted, "emitted");
        coordinator.shutdown().await?;
        Ok(())
    }
}
