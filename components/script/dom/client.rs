/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ClientBinding::{ClientMethods, Wrap};
use dom::bindings::codegen::Bindings::ClientBinding::FrameType;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::bindings::str::{DOMString, USVString};
use dom::serviceworker::ServiceWorker;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_url::ServoUrl;
use std::default::Default;
use uuid::Uuid;

#[dom_struct]
pub struct Client {
    reflector_: Reflector,
    active_worker: MutNullableDom<ServiceWorker>,
    url: ServoUrl,
    frame_type: FrameType,
    #[ignore_heap_size_of = "Defined in uuid"]
    id: Uuid
}

impl Client {
    fn new_inherited(url: ServoUrl) -> Client {
        Client {
            reflector_: Reflector::new(),
            active_worker: Default::default(),
            url: url,
            frame_type: FrameType::None,
            id: Uuid::new_v4()
        }
    }

    pub fn new(window: &Window) -> DomRoot<Client> {
        reflect_dom_object(Box::new(Client::new_inherited(window.get_url())),
                           window,
                           Wrap)
    }

    pub fn creation_url(&self) -> ServoUrl {
        self.url.clone()
    }

    pub fn get_controller(&self) -> Option<DomRoot<ServiceWorker>> {
        self.active_worker.get()
    }

    pub fn set_controller(&self, worker: &ServiceWorker) {
        self.active_worker.set(Some(worker));
    }
}

impl ClientMethods for Client {
    // https://w3c.github.io/ServiceWorker/#client-url-attribute
    fn Url(&self) -> USVString {
        USVString(self.url.as_str().to_owned())
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
