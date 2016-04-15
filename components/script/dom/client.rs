/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ClientBinding::FrameType;
use dom::bindings::codegen::Bindings::ClientBinding::{ClientMethods, Wrap};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::js::{JS, MutNullableHeap};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::USVString;
use dom::serviceworker::ServiceWorker;
use dom::window::Window;
use std::default::Default;
use util::str::DOMString;
use uuid::Uuid;

#[dom_struct]
pub struct Client {
reflector_: Reflector,
    active_worker: MutNullableHeap<JS<ServiceWorker>>,
    url: DOMString,
    frame_type: FrameType,
    #[ignore_heap_size_of = "Defined in uuid"]
    id: Uuid
}

impl Client {
    fn new_inherited() -> Client {
        Client {
            reflector_: Reflector::new(),
            active_worker: Default::default(),
            url: DOMString::new(),
            frame_type: FrameType::None,
            id: Uuid::new_v4()
        }
    }

    pub fn new(window: &Window) -> Root<Client> {
        reflect_dom_object(box Client::new_inherited(), GlobalRef::Window(window), Wrap)
    }
}

impl ClientMethods for Client {
    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#client-url
    fn Url(&self) -> USVString {
        USVString((*self.url).to_owned())
    }
    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#client-frametype
    fn FrameType(&self) -> FrameType {
        self.frame_type
    }
    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#client-id
    fn Id(&self) -> DOMString {
        let uid_str = format!("{}", self.id);
        DOMString::from_string(uid_str)
    }
}
