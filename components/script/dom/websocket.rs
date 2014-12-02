/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::WebSocketBinding;
use dom::bindings::codegen::Bindings::WebSocketBinding::WebSocketMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Temporary, JSRef};
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::eventtarget::{EventTarget, EventTargetHelpers, WebSocketTypeId};
use servo_net::resource_task::{Load, LoadData, LoadResponse};
use servo_util::str::DOMString;
use std::comm::channel;
use url::Url;

#[dom_struct]
pub struct WebSocket {
    eventtarget: EventTarget,
    url: DOMString,
    response: LoadResponse
}

impl WebSocket {
    pub fn new_inherited(url: DOMString, resp: LoadResponse) -> WebSocket {
        WebSocket {
            eventtarget: EventTarget::new_inherited(WebSocketTypeId),
            url: url,
            response: resp
        }
    }

    pub fn new(global: &GlobalRef, url: DOMString) -> Temporary<WebSocket> {
        let resource_task = global.resource_task();
        let ws_url = Url::parse(url.as_slice()).unwrap();
        let(start_chan, start_port) = channel();
        resource_task.send(Load(LoadData::new(ws_url), start_chan));
        let resp = start_port.recv();
        reflect_dom_object(box WebSocket::new_inherited(url, resp),
                           global,
                           WebSocketBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalRef, url: DOMString) -> Fallible<Temporary<WebSocket>> {
        Ok(WebSocket::new(global, url))
    }
}

impl Reflectable for WebSocket {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }
}

impl<'a> WebSocketMethods for JSRef<'a, WebSocket> {
    event_handler!(open, GetOnopen, SetOnopen)
    event_handler!(error, GetOnerror, SetOnerror)
    event_handler!(close, GetOnclose, SetOnclose)
    event_handler!(message, GetOnmessage, SetOnmessage)

    fn Url(self) -> DOMString {
        self.url.clone()
    }
}
