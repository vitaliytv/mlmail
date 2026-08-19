//! Typed execution of the Booking Finder demo over mlmail's generic plugin runtime.

use anyhow::{anyhow, Result};
use n_plugin_runtime::{ActivationGeneration, ActivationRegistry};
use wasmtime::{component::Component, Store};

use super::{plugin_bindings::GmailSearchHost, plugin_runtime::GmailPluginRuntime};

wasmtime::component::bindgen!({
    path: "wit",
    world: "booking-finder-plugin",
    imports: { default: async },
    exports: { default: async },
});

/// Runs one packaged Booking Finder Component with the authenticated Gmail host state.
///
/// # Errors
///
/// Returns an error when bytes are not a Component, required typed imports cannot be linked,
/// Component instantiation fails, or the demo returns a Gmail host error. The product command
/// enforces the exact generic grant before passing Component bytes or OAuth credentials here.
pub(crate) async fn invoke_booking_finder(
    runtime: &GmailPluginRuntime,
    activation_registry: &ActivationRegistry,
    generation: ActivationGeneration,
    component_bytes: &[u8],
    endpoint: impl Into<String>,
    access_token: impl Into<String>,
) -> Result<exports::nitra::gmail::booking_finder::BookingResults> {
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
        GmailSearchHost::new(endpoint, access_token),
    );
    let plugin = BookingFinderPlugin::instantiate_async(&mut store, &component, &linker).await?;
    let results = store
        .run_concurrent(async |accessor| {
            plugin
                .nitra_gmail_booking_finder()
                .call_find(accessor)
                .await
        })
        .await??;

    results.map_err(|error| anyhow!("Booking Finder Gmail search failed: {error:?}"))
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use n_plugin_package::{embed_manifest, inspect_component, PluginManifest};

    use super::*;
    use crate::gmail::plugin_runtime::{
        build_gmail_plugin_runtime, publish_test_generation, GMAIL_BOOKING_FINDER_INTERFACE,
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
    #[ignore = "requires a built guest or an installed packaged Booking Finder Component"]
    async fn invokes_packaged_booking_finder_through_typed_gmail_search() -> Result<()> {
        let component_path = std::env::var_os("MLMAIL_BOOKING_FINDER_COMPONENT").context(
            "MLMAIL_BOOKING_FINDER_COMPONENT must point to the Booking Finder Component",
        )?;
        let component = std::fs::read(component_path)?;
        let packaged = packaged_component(component)?;
        let inspected = inspect_component(&packaged)?;
        assert_eq!(
            inspected.manifest.entrypoints["find"].as_str(),
            GMAIL_BOOKING_FINDER_INTERFACE
        );

        let mut server = mockito::Server::new_async().await;
        let first = server
            .mock("GET", "/messages")
            .match_header("authorization", "Bearer token")
            .match_query(mockito::Matcher::UrlEncoded(
                "q".into(),
                "from:(booking.com)".into(),
            ))
            .with_status(200)
            .with_body(r#"{"messages":[{"id":"reservation-1","threadId":"thread-1"}],"nextPageToken":"page-2"}"#)
            .create_async()
            .await;
        let second = server
            .mock("GET", "/messages")
            .match_header("authorization", "Bearer token")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("q".into(), "from:(booking.com)".into()),
                mockito::Matcher::UrlEncoded("pageToken".into(), "page-2".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"messages":[{"id":"reservation-2"}]}"#)
            .create_async()
            .await;

        let temporary = tempfile::tempdir()?;
        let lock_path = temporary.path().join("wkg.lock");
        std::fs::write(&lock_path, LOCK)?;
        let runtime = build_gmail_plugin_runtime(&lock_path, "0.22.0").await?;
        let (activation_registry, generation) =
            publish_test_generation(&packaged, &lock_path, temporary.path()).await?;
        let results = invoke_booking_finder(
            &runtime,
            &activation_registry,
            generation,
            &packaged,
            format!("{}/messages", server.url()),
            "token",
        )
        .await?;

        first.assert_async().await;
        second.assert_async().await;
        assert_eq!(runtime.environment().application.id, MLMAIL_APPLICATION_ID);
        assert_eq!(results.query, "from:(booking.com)");
        assert_eq!(results.messages.len(), 2);
        assert_eq!(results.messages[0].id, "reservation-1");
        assert_eq!(results.messages[0].thread_id.as_deref(), Some("thread-1"));
        assert_eq!(results.messages[1].id, "reservation-2");
        Ok(())
    }

    fn packaged_component(component: Vec<u8>) -> Result<Vec<u8>> {
        match inspect_component(&component) {
            Ok(_) => Ok(component),
            Err(error)
                if error.to_string() == "Component does not contain `nitra.plugin-manifest/v1`" =>
            {
                let manifest = PluginManifest::from_toml(include_str!(
                    "../../plugins/booking-finder/.n-plugin.toml"
                ))?;
                Ok(embed_manifest(&component, &manifest)?)
            }
            Err(error) => Err(error.into()),
        }
    }
}
