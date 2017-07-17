/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Infrastructure to initiate network requests for images needed by the layout
//! thread. The script thread needs to be responsible for them because there's
//! no guarantee that the responsible nodes will still exist in the future if the
//! layout thread holds on to them during asynchronous operations.

use dom::bindings::reflector::DomObject;
use dom::node::{Node, document_from_node};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::{FetchResponseMsg, FetchResponseListener, FetchMetadata, NetworkError};
use net_traits::image_cache::{ImageCache, PendingImageId};
use net_traits::request::{Type as RequestType, RequestInit as FetchRequestInit};
use network_listener::{NetworkListener, PreInvoke};
use servo_url::ServoUrl;
use std::sync::{Arc, Mutex};

struct LayoutImageContext {
    id: PendingImageId,
    cache: Arc<ImageCache>,
}

impl FetchResponseListener for LayoutImageContext {
    fn process_request_body(&mut self) {}
    fn process_request_eof(&mut self) {}
    fn process_response(&mut self, metadata: Result<FetchMetadata, NetworkError>) {
        self.cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponse(metadata));
    }

    fn process_response_chunk(&mut self, payload: Vec<u8>) {
        self.cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponseChunk(payload));
    }

    fn process_response_eof(&mut self, response: Result<(), NetworkError>) {
        self.cache.notify_pending_response(self.id,
                                           FetchResponseMsg::ProcessResponseEOF(response));
    }
}

impl PreInvoke for LayoutImageContext {}

pub fn fetch_image_for_layout(url: ServoUrl,
                              node: &Node,
                              id: PendingImageId,
                              cache: Arc<ImageCache>) {
    let context = Arc::new(Mutex::new(LayoutImageContext {
        id: id,
        cache: cache,
    }));

    let document = document_from_node(node);
    let window = document.window();

    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let listener = NetworkListener {
        context: context,
        task_source: window.networking_task_source(),
        wrapper: Some(window.get_runnable_wrapper()),
    };
    ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
        listener.notify_fetch(message.to().unwrap());
    });

    let request = FetchRequestInit {
        url: url,
        origin: document.origin().immutable().clone(),
        type_: RequestType::Image,
        pipeline_id: Some(document.global().pipeline_id()),
        .. FetchRequestInit::default()
    };

    // Layout image loads do not delay the document load event.
    document.mut_loader().fetch_async_background(request, action_sender);
}
