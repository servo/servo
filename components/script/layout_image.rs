/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Infrastructure to initiate network requests for images needed by layout. The script thread needs
//! to be responsible for them because there's no guarantee that the responsible nodes will still
//! exist in the future if layout holds on to them during asynchronous operations.

use std::sync::Arc;

use net_traits::image_cache::{ImageCache, PendingImageId};
use net_traits::request::{Destination, RequestBuilder as FetchRequestInit, RequestId};
use net_traits::{
    FetchMetadata, FetchResponseListener, FetchResponseMsg, NetworkError, ResourceFetchTiming,
    ResourceTimingType,
};
use servo_url::ServoUrl;

use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::globalscope::GlobalScope;
use crate::dom::node::{document_from_node, Node};
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::network_listener::{self, PreInvoke, ResourceTimingListener};

struct LayoutImageContext {
    id: PendingImageId,
    cache: Arc<dyn ImageCache>,
    resource_timing: ResourceFetchTiming,
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
            FetchResponseMsg::ProcessResponseChunk(request_id, payload),
        );
    }

    fn process_response_eof(
        &mut self,
        request_id: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        self.cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponseEOF(request_id, response),
        );
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        network_listener::submit_timing(self)
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

impl PreInvoke for LayoutImageContext {}

pub fn fetch_image_for_layout(
    url: ServoUrl,
    node: &Node,
    id: PendingImageId,
    cache: Arc<dyn ImageCache>,
) {
    let document = document_from_node(node);
    let context = LayoutImageContext {
        id,
        cache,
        resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
        doc: Trusted::new(&document),
        url: url.clone(),
    };

    let request = FetchRequestInit::new(url, document.global().get_referrer())
        .origin(document.origin().immutable().clone())
        .destination(Destination::Image)
        .pipeline_id(Some(document.global().pipeline_id()));

    // Layout image loads do not delay the document load event.
    document.fetch_background(request, context, None);
}
