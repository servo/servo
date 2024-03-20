/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;

use dom_struct::dom_struct;
use servo_url::ServoUrl;
use uuid::Uuid;

use crate::dom::bindings::codegen::Bindings::ClientBinding::{ClientMethods, FrameType};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::serviceworker::ServiceWorker;
use crate::dom::window::Window;

#[dom_struct]
pub struct Client {
    reflector_: Reflector,
    active_worker: MutNullableDom<ServiceWorker>,
    #[no_trace]
    url: ServoUrl,
    frame_type: FrameType,
    #[ignore_malloc_size_of = "Defined in uuid"]
    #[no_trace]
    id: Uuid,
}

impl Client {
    fn new_inherited(url: ServoUrl) -> Client {
        Client {
            reflector_: Reflector::new(),
            active_worker: Default::default(),
            url,
            frame_type: FrameType::None,
            id: Uuid::new_v4(),
        }
    }

    pub fn new(window: &Window) -> DomRoot<Client> {
        reflect_dom_object(Box::new(Client::new_inherited(window.get_url())), window)
    }

    pub fn creation_url(&self) -> ServoUrl {
        self.url.clone()
    }

    pub fn get_controller(&self) -> Option<DomRoot<ServiceWorker>> {
        self.active_worker.get()
    }

    #[allow(dead_code)]
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
