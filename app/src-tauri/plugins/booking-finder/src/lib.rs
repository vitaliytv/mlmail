//! Booking Finder is a product demo Component that consumes typed Gmail search pages.

#[allow(
    clippy::same_length_and_capacity,
    unsafe_code,
    reason = "generated Canonical ABI glue owns raw buffer reconstruction"
)]
mod bindings {
    wit_bindgen::generate!({
        path: "wit",
        world: "booking-finder-plugin",
        with: {
            "vitaliytv:gmail/search@0.1.0": generate,
        },
    });

    use super::BookingFinder;
    export!(BookingFinder);
}

use bindings::{
    exports::vitaliytv::booking_finder::booking_finder::{BookingResults, Guest},
    vitaliytv::gmail::search::{self, Error, ListRequest},
};

const BOOKING_QUERY: &str = "from:(booking.com)";

struct BookingFinder;

impl Guest for BookingFinder {
    async fn find() -> Result<BookingResults, Error> {
        let mut pages = search::list_pages(ListRequest {
            q: BOOKING_QUERY.into(),
            max_results: None,
            page_token: None,
            label_ids: Vec::new(),
            include_spam_trash: None,
        })
        .await?;
        let mut messages = Vec::new();

        while let Some(page) = pages.next().await {
            messages.extend(page?.messages.unwrap_or_default());
        }

        Ok(BookingResults {
            query: BOOKING_QUERY.into(),
            messages,
        })
    }
}
