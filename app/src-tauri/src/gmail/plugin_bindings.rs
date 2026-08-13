//! Generated Component Model bindings for mlmail's Gmail plugin contract.

use super::{
    plugin_search::list_messages_page_at, GmailError, GmailListRequest, GmailListResponse,
};
use wasmtime::component::{Accessor, HasData, ResourceTable, StreamReader};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

#[cfg(test)]
use super::GmailMessageRef;

wasmtime::component::bindgen!({
    path: "wit",
    world: "gmail-plugin",
    imports: { default: async },
});

/// Host state that serves the generated Gmail search import for one plugin instance.
pub struct GmailSearchHost {
    endpoint: String,
    access_token: String,
    wasi: WasiCtx,
    table: ResourceTable,
}

impl GmailSearchHost {
    /// Creates one host state using the authenticated Gmail messages endpoint.
    #[must_use]
    pub fn new(endpoint: impl Into<String>, access_token: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            access_token: access_token.into(),
            wasi: WasiCtxBuilder::new().build(),
            table: ResourceTable::default(),
        }
    }
}

impl HasData for GmailSearchHost {
    type Data<'a> = &'a mut Self;
}

impl WasiView for GmailSearchHost {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl<T> nitra::gmail::search::HostWithStore<T> for GmailSearchHost {
    async fn list_pages(
        host: &Accessor<T, Self>,
        request: nitra::gmail::search::ListRequest,
    ) -> Result<
        StreamReader<Result<nitra::gmail::search::ListResponse, nitra::gmail::search::Error>>,
        nitra::gmail::search::Error,
    > {
        let (endpoint, access_token) = host.with(|mut access| {
            let state = access.get();
            (state.endpoint.clone(), state.access_token.clone())
        });
        let pages = list_pages(&endpoint, &access_token, request).await;
        host.with(|access| {
            StreamReader::new(access, pages).map_err(|_| nitra::gmail::search::Error::Unavailable)
        })
    }
}

impl nitra::gmail::search::Host for GmailSearchHost {}

async fn list_pages(
    endpoint: &str,
    access_token: &str,
    request: nitra::gmail::search::ListRequest,
) -> Vec<Result<nitra::gmail::search::ListResponse, nitra::gmail::search::Error>> {
    let mut request = GmailListRequest {
        q: request.q,
        max_results: request.max_results,
        page_token: request.page_token,
        label_ids: request.label_ids,
        include_spam_trash: request.include_spam_trash,
    };
    let mut pages = Vec::new();

    loop {
        let page = match list_messages_page_at(endpoint, access_token, &request).await {
            Ok(page) => page,
            Err(error) => {
                pages.push(Err(map_error(error)));
                return pages;
            }
        };
        request.page_token = page.next_page_token.clone();
        pages.push(Ok(to_component_page(page)));

        if request.page_token.is_none() {
            return pages;
        }
    }
}

fn to_component_page(page: GmailListResponse) -> nitra::gmail::search::ListResponse {
    nitra::gmail::search::ListResponse {
        messages: page.messages.map(|messages| {
            messages
                .into_iter()
                .map(|message| nitra::gmail::search::MessageRef {
                    id: message.id,
                    thread_id: message.thread_id,
                })
                .collect()
        }),
        next_page_token: page.next_page_token,
        result_size_estimate: page.result_size_estimate,
    }
}

fn map_error(error: GmailError) -> nitra::gmail::search::Error {
    match error {
        GmailError::ReauthRequired => nitra::gmail::search::Error::Unauthenticated,
        GmailError::Parse(_) => nitra::gmail::search::Error::InvalidResponse,
        GmailError::Network(_) | GmailError::Http { .. } | GmailError::Platform(_) => {
            nitra::gmail::search::Error::Unavailable
        }
        GmailError::Empty | GmailError::EmptyQuery => nitra::gmail::search::Error::InvalidResponse,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_raw_gmail_page_fields_when_projecting_to_wit() {
        let response = to_component_page(GmailListResponse {
            messages: Some(vec![GmailMessageRef {
                id: "message-1".to_owned(),
                thread_id: Some("thread-1".to_owned()),
            }]),
            next_page_token: Some("next".to_owned()),
            result_size_estimate: Some(7),
        });

        assert_eq!(response.messages.expect("messages")[0].id, "message-1");
        assert_eq!(response.next_page_token.as_deref(), Some("next"));
        assert_eq!(response.result_size_estimate, Some(7));
    }

    #[test]
    fn maps_reauthentication_to_the_wit_error() {
        assert!(matches!(
            map_error(GmailError::ReauthRequired),
            nitra::gmail::search::Error::Unauthenticated
        ));
    }

    #[tokio::test]
    async fn follows_gmail_next_page_tokens_without_rewriting_query() {
        let mut server = mockito::Server::new_async().await;
        let first = server
            .mock("GET", "/messages")
            .match_header("authorization", "Bearer token")
            .match_query(mockito::Matcher::UrlEncoded(
                "q".into(),
                "from:booking.com newer_than:30d".into(),
            ))
            .with_status(200)
            .with_body(r#"{"messages":[{"id":"one"}],"nextPageToken":"two"}"#)
            .create_async()
            .await;
        let second = server
            .mock("GET", "/messages")
            .match_header("authorization", "Bearer token")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("q".into(), "from:booking.com newer_than:30d".into()),
                mockito::Matcher::UrlEncoded("pageToken".into(), "two".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"messages":[{"id":"three"}]}"#)
            .create_async()
            .await;
        let host = GmailSearchHost::new(format!("{}/messages", server.url()), "token");

        let pages = list_pages(
            &host.endpoint,
            &host.access_token,
            nitra::gmail::search::ListRequest {
                q: "from:booking.com newer_than:30d".to_owned(),
                max_results: None,
                page_token: None,
                label_ids: Vec::new(),
                include_spam_trash: None,
            },
        )
        .await;

        first.assert_async().await;
        second.assert_async().await;
        assert_eq!(pages.len(), 2);
        assert_eq!(
            pages[0]
                .as_ref()
                .expect("first page")
                .messages
                .as_ref()
                .expect("first messages")[0]
                .id,
            "one"
        );
        assert_eq!(
            pages[1]
                .as_ref()
                .expect("second page")
                .messages
                .as_ref()
                .expect("second messages")[0]
                .id,
            "three"
        );
    }
}
