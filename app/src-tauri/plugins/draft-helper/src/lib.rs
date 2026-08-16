//! Draft Helper is a trusted product demo Component that creates one typed Gmail draft.

#[allow(
    clippy::same_length_and_capacity,
    unsafe_code,
    reason = "generated Canonical ABI glue owns raw buffer reconstruction"
)]
mod bindings {
    wit_bindgen::generate!({
        path: "wit",
        world: "draft-helper-plugin",
        with: {
            "vitaliytv:gmail/drafts@0.1.0": generate,
        },
    });

    use super::DraftHelper;
    export!(DraftHelper);
}

use bindings::{
    exports::vitaliytv::draft_helper::draft_helper::Guest,
    vitaliytv::gmail::drafts::{self, CreateRequest, DraftRef, Error},
};

const DEMO_RECIPIENT: &str = "a@example.com";
const DEMO_SUBJECT: &str = "Hello from mlmail";
const DEMO_BODY: &str = "Draft created by the typed vitaliytv:gmail/drafts Component import.";

struct DraftHelper;

impl Guest for DraftHelper {
    async fn create() -> Result<DraftRef, Error> {
        drafts::create(CreateRequest {
            to: DEMO_RECIPIENT.into(),
            subject: DEMO_SUBJECT.into(),
            body: DEMO_BODY.into(),
        })
        .await
    }
}
