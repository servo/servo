/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ClientBinding::FrameType;
use dom::bindings::codegen::Bindings::ClientBinding::{ClientMethods, Wrap};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::JS;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString};
use dom::serviceworker::ServiceWorker;
use dom::window::Window;
use url::Url;
use uuid::Uuid;

#[dom_struct]
pub struct Client {
    reflector_: Reflector,
    active_worker: Option<JS<ServiceWorker>>,
    url: USVString,
    frame_type: FrameType,
    #[ignore_heap_size_of = "Defined in uuid"]
    id: Uuid
}

impl Client {
    fn new_inherited(url: Url) -> Client {
        Client {
            reflector_: Reflector::new(),
            active_worker: None,
            url: USVString(url.as_str().to_owned()),
            frame_type: FrameType::None,
            id: Uuid::new_v4()
        }
    }

    pub fn new(window: &Window) -> Root<Client> {
        reflect_dom_object(box Client::new_inherited(window.get_url()), GlobalRef::Window(window), Wrap)
    }
}

impl ClientMethods for Client {
    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#client-url-attribute
    fn Url(&self) -> USVString {
        self.url.clone()
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
