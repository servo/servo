/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WebSocketBinding;
use dom::bindings::codegen::Bindings::WebSocketBinding::WebSocketMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::error::Error::{InvalidState, InvalidAccess};
use dom::bindings::error::Error::{Network, Syntax, Security, Abort, Timeout};
use dom::event::{Event, EventBubbles, EventCancelable, EventHelpers};
use dom::bindings::global::{GlobalField, GlobalRef, GlobalRoot};
use dom::bindings::js::{Temporary, JSRef};
use dom::bindings::utils::reflect_dom_object;
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use util::str::DOMString;

use websocket::{Message, Sender, Receiver};
use websocket::client::request::Url;
use websocket::Client;
use std::cell::Cell;
use std::borrow::ToOwned;

#[derive(PartialEq, Copy)]
#[jstraceable]
enum WebSocketRequestState {
	Connecting = 0,
    Open = 1,
    Closing = 2,
    Closed = 3,
}

#[dom_struct]
pub struct WebSocket {
    eventtarget: EventTarget,
    url: DOMString,
    global: GlobalField,
	ready_state: Cell<WebSocketRequestState>
}

impl WebSocket {
    pub fn new_inherited(global: GlobalRef, url: DOMString) -> WebSocket {
        println!("Creating websocket...");
	let copied_url = url.clone();
	WebSocket {
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::WebSocket),
            url: url,
            global: GlobalField::from_rooted(&global),
		ready_state: Cell::new(WebSocketRequestState::Connecting)
        }

    }

    pub fn new(global: GlobalRef, url: DOMString) -> Temporary<WebSocket> {
        let ws_root = reflect_dom_object(box WebSocket::new_inherited(global, url),
                           global,
                           WebSocketBinding::Wrap).root();
        ws_root.r().Open();
        Temporary::from_rooted(ws_root.r())
    }

    pub fn Constructor(global: GlobalRef, url: DOMString) -> Fallible<Temporary<WebSocket>> {
    	Ok(WebSocket::new(global, url))
    }
	
	/*fn send(self) -> ErrorResult {
		 let tx_1 = self.tx.clone();
		let send_loop = thread::scoped(move || {
			loop {
				let message = match self.rx.recv() {
					Ok(m) => m,
					Err(e) => {
						println!("Send loop: {:?}",e);
						return;
					}
				};
				match message {
					Message::Close(_) => {
						let _ = self.sender.send_message(message);
						// If it's a close message, send it and return
						return;
					}
					_ => (),
				}
				//Send the message
				match self.sender.send_message(message){
					Ok(()) => (),
					Err(e) => {
						println!("Send Loop: {:?}", e);
						let _ = self.sender.send_message(Message::Close(None));
						return;
					}
				}
			}	
		}); 
	}*/
}

impl<'a> WebSocketMethods for JSRef<'a, WebSocket> {
	event_handler!(open, GetOnopen, SetOnopen);
	event_handler!(close, GetOnclose, SetOnclose);
	event_handler!(error, GetOnerror, SetOnerror);

    fn Url(self) -> DOMString {
	println!("Cloning URL");
       self.url.clone()
    }
	
   fn ReadyState(self) -> u16 {
   	println!("Setting readystate");
	self.ready_state.get() as u16
   }

   fn Open(self) -> ErrorResult {
    	println!("Trying to connect.");
		let parsed_url = Url::parse(self.url.as_slice()).unwrap();
   		let request = Client::connect(parsed_url).unwrap();
		let response = request.send().unwrap();
		response.validate().unwrap();
		println!("Successful connection.");
    	let global = self.global.root();
        let event = Event::new(global.r(),
                               "open".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::Cancelable).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        event.r().fire(target);
        println!("Fired event.");
		Ok(())
    }

}
