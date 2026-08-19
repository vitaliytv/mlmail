//! Typed execution of the Draft Helper demo over mlmail's generic plugin runtime.

use anyhow::{anyhow, Result};
use n_plugin_runtime::{ActivationGeneration, ActivationRegistry};
use wasmtime::{component::Component, Store};

use super::{plugin_bindings::GmailSearchHost, plugin_runtime::GmailPluginRuntime};

wasmtime::component::bindgen!({
    path: "wit",
    world: "draft-helper-plugin",
    imports: { default: async },
    exports: { default: async },
});

/// Runs one packaged Draft Helper Component with the authenticated Gmail host state.
///
/// # Errors
///
/// Returns an error when bytes are not a Component, required typed imports cannot be linked,
/// Component instantiation fails, or Gmail rejects the draft creation request. The product
/// command enforces the exact generic grant before passing Component bytes or OAuth credentials.
pub(crate) async fn invoke_draft_helper(
    runtime: &GmailPluginRuntime,
    activation_registry: &ActivationRegistry,
    generation: ActivationGeneration,
    component_bytes: &[u8],
    messages_endpoint: impl Into<String>,
    access_token: impl Into<String>,
) -> Result<nitra::gmail::drafts::DraftRef> {
    n_plugin_runtime::ensure_component(component_bytes)?;
    let component = Component::from_binary(runtime.runtime().engine(), component_bytes)?;
    let mut linker = runtime.runtime().new_linker()?;
    n_plugin_runtime::register_generation_edge_guards(
        &mut linker,
        activation_registry,
        generation,
    )?;
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    let mut store = Store::new(
        runtime.runtime().engine(),
        GmailSearchHost::new(messages_endpoint, access_token),
    );
    let plugin = DraftHelperPlugin::instantiate_async(&mut store, &component, &linker).await?;
    let draft = store
        .run_concurrent(async |accessor| {
            plugin
                .nitra_gmail_draft_helper()
                .call_create(accessor)
                .await
        })
        .await??;

    draft.map_err(|error| anyhow!("Draft Helper Gmail create failed: {error:?}"))
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use n_plugin_package::{embed_manifest, inspect_component, PluginManifest};

    use super::*;
    use crate::gmail::plugin_runtime::{
        build_gmail_plugin_runtime, publish_test_generation, GMAIL_DRAFT_HELPER_INTERFACE,
        MLMAIL_APPLICATION_ID,
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
    #[ignore = "requires the Draft Helper guest Component built for wasm32-wasip2"]
    async fn invokes_packaged_draft_helper_through_typed_gmail_drafts() -> Result<()> {
        let component_path = std::env::var_os("MLMAIL_DRAFT_HELPER_COMPONENT")
            .context("MLMAIL_DRAFT_HELPER_COMPONENT must point to the built guest Component")?;
        let component = std::fs::read(component_path)?;
        let packaged = match inspect_component(&component) {
            Ok(_) => component,
            Err(_) => {
                let manifest = PluginManifest::from_toml(include_str!(
                    "../../plugins/draft-helper/.n-plugin.toml"
                ))?;
                embed_manifest(&component, &manifest)?
            }
        };
        let inspected = inspect_component(&packaged)?;
        assert_eq!(
            inspected.manifest.entrypoints["create"].as_str(),
            GMAIL_DRAFT_HELPER_INTERFACE
        );

        let mut server = mockito::Server::new_async().await;
        let created = server
            .mock("POST", "/drafts")
            .match_header("authorization", "Bearer token")
            .with_status(200)
            .with_body(r#"{"id":"draft-1"}"#)
            .create_async()
            .await;
        let temporary = tempfile::tempdir()?;
        let lock_path = temporary.path().join("wkg.lock");
        std::fs::write(&lock_path, LOCK)?;
        let runtime = build_gmail_plugin_runtime(&lock_path, "0.23.0").await?;
        let (activation_registry, generation) =
            publish_test_generation(&packaged, &lock_path, temporary.path()).await?;
        let draft = invoke_draft_helper(
            &runtime,
            &activation_registry,
            generation,
            &packaged,
            format!("{}/messages", server.url()),
            "token",
        )
        .await?;

        created.assert_async().await;
        assert_eq!(runtime.environment().application.id, MLMAIL_APPLICATION_ID);
        assert_eq!(draft.id, "draft-1");
        Ok(())
    }
}
