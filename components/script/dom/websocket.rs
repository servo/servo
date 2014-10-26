/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WebSocketBinding;
use dom::bindings::codegen::Bindings::WebSocketBinding::WebSocketMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Temporary,JSRef};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use servo_util::str::DOMString;

#[jstraceable]
pub enum WebSocketType {
    WebSocketTypeId
}

#[dom_struct]
pub struct WebSocket {
    reflector_: Reflector,
    type_: WebSocketType,
    url: DOMString    
}

impl WebSocket {
    pub fn new_inherited(url:DOMString) -> WebSocket {
        WebSocket {
            reflector_: Reflector::new(),
            type_: WebSocketTypeId,
            url: url            
        }
    }

    pub fn new(global: &GlobalRef,url: DOMString) -> Temporary<WebSocket> {
        reflect_dom_object(box WebSocket::new_inherited(url),
                           global,
                           WebSocketBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalRef,url:DOMString) -> Fallible<Temporary<WebSocket>> {
        Ok(WebSocket::new(global,url))
    }
    
    pub fn url<'a>(&'a self) -> &'a DOMString {
        &self.url
    }
           
}

impl Reflectable for WebSocket {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

impl<'a> WebSocketMethods for JSRef<'a, WebSocket> {
    fn Url(self) ->  DOMString {
        self.url.clone()
    }    
}
