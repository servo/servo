/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Infrastructure to initiate network requests for images needed by layout. The script thread needs
//! to be responsible for them because there's no guarantee that the responsible nodes will still
//! exist in the future if layout holds on to them during asynchronous operations.

use std::sync::Arc;

use net_traits::image_cache::{ImageCache, PendingImageId};
use net_traits::request::{Destination, RequestBuilder, RequestId};
use net_traits::{FetchMetadata, FetchResponseMsg, NetworkError, ResourceFetchTiming};
use servo_url::ServoUrl;

use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::document::Document;
use crate::dom::globalscope::GlobalScope;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::performance::performanceresourcetiming::InitiatorType;
use crate::fetch::RequestWithGlobalScope;
use crate::network_listener::{self, FetchResponseListener, ResourceTimingListener};
use crate::script_runtime::CanGc;

struct LayoutImageContext {
    id: PendingImageId,
    cache: Arc<dyn ImageCache>,
    doc: Trusted<Document>,
    url: ServoUrl,
}

impl FetchResponseListener for LayoutImageContext {
    fn process_request_body(&mut self, _: RequestId) {}
    fn process_request_eof(&mut self, _: RequestId) {}
    fn process_response(
        &mut self,
        request_id: RequestId,
        metadata: Result<FetchMetadata, NetworkError>,
    ) {
        self.cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponse(request_id, metadata),
        );
    }

    fn process_response_chunk(&mut self, request_id: RequestId, payload: Vec<u8>) {
        self.cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponseChunk(request_id, payload.into()),
        );
    }

    fn process_response_eof(
        self,
        request_id: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        self.cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponseEOF(request_id, response.clone()),
        );
        network_listener::submit_timing(&self, &response, CanGc::note());
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(violations, None, None);
    }
}

impl ResourceTimingListener for LayoutImageContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (InitiatorType::Other, self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.doc.root().global()
    }
}

pub(crate) fn fetch_image_for_layout(
    url: ServoUrl,
    node: &Node,
    id: PendingImageId,
    cache: Arc<dyn ImageCache>,
) {
    let document = node.owner_document();
    let context = LayoutImageContext {
        id,
        cache,
        doc: Trusted::new(&document),
        url: url.clone(),
    };

    let global = node.owner_global();
    let request = RequestBuilder::new(Some(document.webview_id()), url, global.get_referrer())
        .destination(Destination::Image)
        .with_global_scope(&global);

    // Layout image loads do not delay the document load event.
    document.fetch_background(request, context);
}
