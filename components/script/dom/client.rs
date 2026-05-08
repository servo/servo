/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use script_bindings::reflector::Reflector;
use servo_url::ServoUrl;
use uuid::Uuid;

use crate::dom::bindings::codegen::Bindings::ClientBinding::{ClientMethods, FrameType};
use crate::dom::bindings::root::MutNullableDom;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::serviceworker::ServiceWorker;

#[dom_struct]
pub(crate) struct Client {
    reflector_: Reflector,
    active_worker: MutNullableDom<ServiceWorker>,
    #[no_trace]
    url: ServoUrl,
    frame_type: FrameType,
    #[no_trace]
    id: Uuid,
}

impl ClientMethods<crate::DomTypeHolder> for Client {
    /// <https://w3c.github.io/ServiceWorker/#client-url-attribute>
    fn Url(&self) -> USVString {
        USVString(self.url.as_str().to_owned())
    }

    /// <https://w3c.github.io/ServiceWorker/#client-frametype>
    fn FrameType(&self) -> FrameType {
        self.frame_type
    }

    /// <https://w3c.github.io/ServiceWorker/#client-id>
    fn Id(&self) -> DOMString {
        format!("{}", self.id).into()
    }
}
