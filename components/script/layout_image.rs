/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Infrastructure to initiate network requests for images needed by layout. The script thread needs
//! to be responsible for them because there's no guarantee that the responsible nodes will still
//! exist in the future if layout holds on to them during asynchronous operations.

use std::sync::{Arc, Mutex};

use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::image_cache::{ImageCache, PendingImageId};
use net_traits::request::{Destination, RequestBuilder as FetchRequestInit};
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
use crate::network_listener::{self, NetworkListener, PreInvoke, ResourceTimingListener};

struct LayoutImageContext {
    id: PendingImageId,
    cache: Arc<dyn ImageCache>,
    resource_timing: ResourceFetchTiming,
    doc: Trusted<Document>,
    url: ServoUrl,
}

impl FetchResponseListener for LayoutImageContext {
    fn process_request_body(&mut self) {}
    fn process_request_eof(&mut self) {}
    fn process_response(&mut self, metadata: Result<FetchMetadata, NetworkError>) {
        self.cache
            .notify_pending_response(self.id, FetchResponseMsg::ProcessResponse(metadata));
    }

    fn process_response_chunk(&mut self, payload: Vec<u8>) {
        self.cache
            .notify_pending_response(self.id, FetchResponseMsg::ProcessResponseChunk(payload));
    }

    fn process_response_eof(&mut self, response: Result<ResourceFetchTiming, NetworkError>) {
        self.cache
            .notify_pending_response(self.id, FetchResponseMsg::ProcessResponseEOF(response));
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

    let context = Arc::new(Mutex::new(LayoutImageContext {
        id,
        cache,
        resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
        doc: Trusted::new(&document),
        url: url.clone(),
    }));

    let document = document_from_node(node);

    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let (task_source, canceller) = document
        .window()
        .task_manager()
        .networking_task_source_with_canceller();
    let listener = NetworkListener {
        context,
        task_source,
        canceller: Some(canceller),
    };
    ROUTER.add_route(
        action_receiver.to_opaque(),
        Box::new(move |message| {
            listener.notify_fetch(message.to().unwrap());
        }),
    );

    let request = FetchRequestInit::new(url, document.global().get_referrer())
        .origin(document.origin().immutable().clone())
        .destination(Destination::Image)
        .pipeline_id(Some(document.global().pipeline_id()));

    // Layout image loads do not delay the document load event.
    document
        .loader_mut()
        .fetch_async_background(request, action_sender);
}
