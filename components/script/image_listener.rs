/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc;
use ipc_channel::ipc::IpcSender;
use ipc_channel::router::ROUTER;
use net_traits::image_cache::{ImageResponse, PendingImageResponse};

use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::node::{window_from_node, Node};
use crate::task_source::TaskSource;

pub trait ImageCacheListener {
    fn generation_id(&self) -> u32;
    fn process_image_response(&self, response: ImageResponse);
}

pub fn generate_cache_listener_for_element<
    T: ImageCacheListener + DerivedFrom<Node> + DomObject,
>(
    elem: &T,
) -> IpcSender<PendingImageResponse> {
    let trusted_node = Trusted::new(elem);
    let (responder_sender, responder_receiver) = ipc::channel().unwrap();

    let window = window_from_node(elem);
    let (task_source, canceller) = window
        .task_manager()
        .networking_task_source_with_canceller();
    let generation = elem.generation_id();
    ROUTER.add_route(
        responder_receiver.to_opaque(),
        Box::new(move |message| {
            let element = trusted_node.clone();
            let image = message.to().unwrap();
            debug!("Got image {:?}", image);
            let _ = task_source.queue_with_canceller(
                task!(process_image_response: move || {
                    let element = element.root();
                    // Ignore any image response for a previous request that has been discarded.
                    if generation == element.generation_id() {
                        element.process_image_response(image);
                    }
                }),
                &canceller,
            );
        }),
    );

    responder_sender
}
