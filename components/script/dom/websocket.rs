/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WebSocketBinding;
use dom::bindings::codegen::Bindings::WebSocketBinding::WebSocketMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Temporary, JSRef};
use dom::bindings::utils::reflect_dom_object;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use util::str::DOMString;

// https://html.spec.whatwg.org/#the-websocket-interface
#[dom_struct]
pub struct WebSocket {
    eventtarget: EventTarget,
    url: DOMString
}

impl WebSocket {
    pub fn new_inherited(url: DOMString) -> WebSocket {
        WebSocket {
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::WebSocket),
            url: url
        }
    }

    pub fn new(global: GlobalRef, url: DOMString) -> Temporary<WebSocket> {
        reflect_dom_object(box WebSocket::new_inherited(url),
                           global,
                           WebSocketBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef, url: DOMString) -> Fallible<Temporary<WebSocket>> {
        Ok(WebSocket::new(global, url))
    }
}

impl<'a> WebSocketMethods for JSRef<'a, WebSocket> {
    // https://html.spec.whatwg.org/#dom-websocket-url
    fn Url(self) -> DOMString {
        self.url.clone()
    }
}
