/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ClientBinding::{ClientMethods, Wrap};
use dom::bindings::codegen::Bindings::ClientBinding::FrameType;
use dom::bindings::js::{JS, Root, MutNullableHeap};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString};
use dom::serviceworker::ServiceWorker;
use dom::window::Window;
use std::default::Default;
use url::Url;
use uuid::Uuid;

#[dom_struct]
pub struct Client {
    reflector_: Reflector,
    active_worker: MutNullableHeap<JS<ServiceWorker>>,
    url: USVString,
    frame_type: FrameType,
    #[ignore_heap_size_of = "Defined in uuid"]
    id: Uuid
}

impl Client {
    fn new_inherited(url: Url) -> Client {
        Client {
            reflector_: Reflector::new(),
            active_worker: Default::default(),
            url: USVString(url.as_str().to_owned()),
            frame_type: FrameType::None,
            id: Uuid::new_v4()
        }
    }

    pub fn new(window: &Window) -> Root<Client> {
        reflect_dom_object(box Client::new_inherited(window.get_url()),
                           window,
                           Wrap)
    }

    pub fn creation_url(&self) -> Url {
        let USVString(ref url_str) = self.url;
        Url::parse(url_str).unwrap()
    }

    pub fn get_controller(&self) -> Option<Root<ServiceWorker>> {
        self.active_worker.get()
    }

    pub fn set_controller(&self, worker: &ServiceWorker) {
        self.active_worker.set(Some(worker));
    }
}

impl ClientMethods for Client {
    // https://w3c.github.io/ServiceWorker/#client-url-attribute
    fn Url(&self) -> USVString {
        self.url.clone()
    }

    // https://w3c.github.io/ServiceWorker/#client-frametype
    fn FrameType(&self) -> FrameType {
        self.frame_type
    }

    // https://w3c.github.io/ServiceWorker/#client-id
    fn Id(&self) -> DOMString {
        let uid_str = format!("{}", self.id);
        DOMString::from_string(uid_str)
    }
}
