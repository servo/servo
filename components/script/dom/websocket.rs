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

use websocket::{Message, Sender, Receiver};
use websocket::client::request::Url;
use websocket::Client;
use std::cell::Cell;

#[derive(PartialEq, Copy)]
#[jstraceable]
enum WebsocketRequestState {
	Unsent = 0,
	Opened = 1,
	Sending = 2,
	Receiving = 3,
}

#[dom_struct]
pub struct WebSocket {
    eventtarget: EventTarget,
    url: DOMString,
	ready_state: Cell<WebsocketRequestState>
}

impl WebSocket {
    pub fn new_inherited(url: DOMString) -> WebSocket {
        WebSocket {
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::WebSocket),
            url: url,
		ready_state: Cell::new(WebsocketRequestState::Unsent)
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
    fn Url(self) -> DOMString {
       self.url.clone()
    }
	
   fn ReadyState(self) -> u16 {
   	self.ready_state.get() as u16
   }
}
