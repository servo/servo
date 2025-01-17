/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The listener that encapsulates all state for an in-progress document request.
//! Any redirects that are encountered are followed. Whenever a non-redirect
//! response is received, it is forwarded to the appropriate script thread.

use std::cell::Cell;

use base::cross_process_instant::CrossProcessInstant;
use base::id::{BrowsingContextId, PipelineId, TopLevelBrowsingContextId};
use content_security_policy::Destination;
use crossbeam_channel::Sender;
use http::header;
use net_traits::request::{
    CredentialsMode, InsecureRequestsPolicy, RedirectMode, RequestBuilder, RequestMode,
};
use net_traits::response::ResponseInit;
use net_traits::{
    fetch_async, set_default_accept_language, BoxedFetchCallback, CoreResourceThread,
    FetchResponseMsg, Metadata, DOCUMENT_ACCEPT_HEADER_VALUE,
};
use script_traits::{DocumentActivity, LoadData, WindowSizeData};
use servo_url::{MutableOrigin, ServoUrl};

use crate::fetch::FetchCanceller;
use crate::messaging::MainThreadScriptMsg;

#[derive(Clone)]
pub struct NavigationListener {
    request_builder: RequestBuilder,
    main_thread_sender: Sender<MainThreadScriptMsg>,
    // Whether or not results are sent to the main thread. After a redirect results are no longer sent,
    // as the main thread has already started a new request.
    send_results_to_main_thread: Cell<bool>,
}

impl NavigationListener {
    pub(crate) fn into_callback(self) -> BoxedFetchCallback {
        Box::new(move |response_msg| self.notify_fetch(response_msg))
    }

    pub fn new(
        request_builder: RequestBuilder,
        main_thread_sender: Sender<MainThreadScriptMsg>,
    ) -> NavigationListener {
        NavigationListener {
            request_builder,
            main_thread_sender,
            send_results_to_main_thread: Cell::new(true),
        }
    }

    pub fn initiate_fetch(
        self,
        core_resource_thread: &CoreResourceThread,
        response_init: Option<ResponseInit>,
    ) {
        fetch_async(
            core_resource_thread,
            self.request_builder.clone(),
            response_init,
            self.into_callback(),
        );
    }

    fn notify_fetch(&self, message: FetchResponseMsg) {
        // If we've already asked the main thread to redirect the response, then stop sending results
        // for this fetch. The main thread has already replaced it.
        if !self.send_results_to_main_thread.get() {
            return;
        }

        // If this is a redirect, don't send any more message after this one.
        if Self::http_redirect_metadata(&message).is_some() {
            self.send_results_to_main_thread.set(false);
        }

        let pipeline_id = self
            .request_builder
            .pipeline_id
            .expect("Navigation should always have an associated Pipeline");
        let result = self
            .main_thread_sender
            .send(MainThreadScriptMsg::NavigationResponse {
                pipeline_id,
                message: Box::new(message),
            });

        if let Err(error) = result {
            warn!(
                "Failed to send network message to pipeline {:?}: {error:?}",
                pipeline_id
            );
        }
    }

    pub(crate) fn http_redirect_metadata(message: &FetchResponseMsg) -> Option<&Metadata> {
        let FetchResponseMsg::ProcessResponse(_, Ok(ref metadata)) = message else {
            return None;
        };

        // Don't allow redirects for non HTTP(S) URLs.
        let metadata = metadata.metadata();
        if !matches!(
            metadata.location_url,
            Some(Ok(ref location_url)) if matches!(location_url.scheme(), "http" | "https")
        ) {
            return None;
        }

        Some(metadata)
    }
}

/// A document load that is in the process of fetching the requested resource. Contains
/// data that will need to be present when the document and frame tree entry are created,
/// but is only easily available at initiation of the load and on a push basis (so some
/// data will be updated according to future resize events, viewport changes, etc.)
#[derive(JSTraceable)]
pub(crate) struct InProgressLoad {
    /// The pipeline which requested this load.
    #[no_trace]
    pub(crate) pipeline_id: PipelineId,
    /// The browsing context being loaded into.
    #[no_trace]
    pub(crate) browsing_context_id: BrowsingContextId,
    /// The top level ancestor browsing context.
    #[no_trace]
    pub(crate) top_level_browsing_context_id: TopLevelBrowsingContextId,
    /// The parent pipeline and frame type associated with this load, if any.
    #[no_trace]
    pub(crate) parent_info: Option<PipelineId>,
    /// The opener, if this is an auxiliary.
    #[no_trace]
    pub(crate) opener: Option<BrowsingContextId>,
    /// The current window size associated with this pipeline.
    #[no_trace]
    pub(crate) window_size: WindowSizeData,
    /// The activity level of the document (inactive, active or fully active).
    #[no_trace]
    pub(crate) activity: DocumentActivity,
    /// Window is throttled, running timers at a heavily limited rate.
    pub(crate) throttled: bool,
    /// The origin for the document
    #[no_trace]
    pub(crate) origin: MutableOrigin,
    /// Timestamp reporting the time when the browser started this load.
    #[no_trace]
    pub(crate) navigation_start: CrossProcessInstant,
    /// For cancelling the fetch
    pub(crate) canceller: FetchCanceller,
    /// The [`LoadData`] associated with this load.
    #[no_trace]
    pub(crate) load_data: LoadData,
    /// A list of URL to keep track of all the redirects that have happened during
    /// this load.
    #[no_trace]
    pub(crate) url_list: Vec<ServoUrl>,
}

impl InProgressLoad {
    /// Create a new InProgressLoad object.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: PipelineId,
        browsing_context_id: BrowsingContextId,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        parent_info: Option<PipelineId>,
        opener: Option<BrowsingContextId>,
        window_size: WindowSizeData,
        origin: MutableOrigin,
        load_data: LoadData,
    ) -> InProgressLoad {
        let url = load_data.url.clone();
        InProgressLoad {
            pipeline_id: id,
            browsing_context_id,
            top_level_browsing_context_id,
            parent_info,
            opener,
            window_size,
            activity: DocumentActivity::FullyActive,
            throttled: false,
            origin,
            navigation_start: CrossProcessInstant::now(),
            canceller: Default::default(),
            load_data,
            url_list: vec![url],
        }
    }

    pub(crate) fn request_builder(&mut self) -> RequestBuilder {
        let id = self.pipeline_id;
        let top_level_browsing_context_id = self.top_level_browsing_context_id;
        let mut request_builder = RequestBuilder::new(
            Some(top_level_browsing_context_id),
            self.load_data.url.clone(),
            self.load_data.referrer.clone(),
        )
        .method(self.load_data.method.clone())
        .destination(Destination::Document)
        .mode(RequestMode::Navigate)
        .credentials_mode(CredentialsMode::Include)
        .use_url_credentials(true)
        .pipeline_id(Some(id))
        .referrer_policy(self.load_data.referrer_policy)
        .insecure_requests_policy(
            self.load_data
                .inherited_insecure_requests_policy
                .unwrap_or(InsecureRequestsPolicy::DoNotUpgrade),
        )
        .headers(self.load_data.headers.clone())
        .body(self.load_data.data.clone())
        .redirect_mode(RedirectMode::Manual)
        .origin(self.origin.immutable().clone())
        .crash(self.load_data.crash.clone());
        request_builder.url_list = self.url_list.clone();

        if !request_builder.headers.contains_key(header::ACCEPT) {
            request_builder
                .headers
                .insert(header::ACCEPT, DOCUMENT_ACCEPT_HEADER_VALUE);
        }
        set_default_accept_language(&mut request_builder.headers);

        request_builder
    }
}
