/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::WebSocketBinding;
use dom::bindings::codegen::Bindings::WebSocketBinding::WebSocketMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::codegen::InheritTypes::EventCast;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::error::Error::{InvalidAccess, Syntax};
use dom::bindings::global::{GlobalField, GlobalRef};
use dom::bindings::js::Root;
use dom::bindings::refcounted::Trusted;
use dom::bindings::str::USVString;
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::reflect_dom_object;
use dom::closeevent::CloseEvent;
use dom::event::{Event, EventBubbles, EventCancelable, EventHelpers};
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use script_task::Runnable;
use script_task::ScriptMsg;
use std::cell::{Cell, RefCell};
use std::borrow::ToOwned;
use util::str::DOMString;

use hyper::header::Host;
use websocket::Message;
use websocket::ws::sender::Sender as Sender_Object;
use websocket::client::sender::Sender;
use websocket::client::receiver::Receiver;
use websocket::stream::WebSocketStream;
use websocket::client::request::Url;
use websocket::Client;
use websocket::header::Origin;
use websocket::result::WebSocketResult;
use websocket::ws::util::url::parse_url;

#[derive(JSTraceable, PartialEq, Copy, Clone)]
enum WebSocketRequestState {
    Connecting = 0,
    Open = 1,
    Closing = 2,
    Closed = 3,
}

no_jsmanaged_fields!(Sender<WebSocketStream>);
no_jsmanaged_fields!(Receiver<WebSocketStream>);

#[dom_struct]
pub struct WebSocket {
    eventtarget: EventTarget,
    url: Url,
    global: GlobalField,
    ready_state: Cell<WebSocketRequestState>,
    sender: RefCell<Option<Sender<WebSocketStream>>>,
    receiver: RefCell<Option<Receiver<WebSocketStream>>>,
    failed: Cell<bool>, //Flag to tell if websocket was closed due to failure
    full: Cell<bool>, //Flag to tell if websocket queue is full
    clean_close: Cell<bool>, //Flag to tell if the websocket closed cleanly (not due to full or fail)
    code: Cell<u16>, //Closing code
    reason: DOMRefCell<DOMString>, //Closing reason
    data: DOMRefCell<DOMString>, //Data from send - TODO: Remove after buffer is added.
}

/// *Establish a WebSocket Connection* as defined in RFC 6455.
fn establish_a_websocket_connection(url: (Host, String, bool), origin: String)
    -> WebSocketResult<(Sender<WebSocketStream>, Receiver<WebSocketStream>)> {
    let mut request = try!(Client::connect(url));
    request.headers.set(Origin(origin));

    let response = try!(request.send());
    try!(response.validate());

    Ok(response.begin().split())
}


impl WebSocket {
    pub fn new_inherited(global: GlobalRef, url: Url) -> WebSocket {
        WebSocket {
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::WebSocket),
            url: url,
            global: GlobalField::from_rooted(&global),
            ready_state: Cell::new(WebSocketRequestState::Connecting),
            failed: Cell::new(false),
            sender: RefCell::new(None),
            receiver: RefCell::new(None),
            full: Cell::new(false),
            clean_close: Cell::new(true),
            code: Cell::new(0),
            reason: DOMRefCell::new("".to_owned()),
            data: DOMRefCell::new("".to_owned()),
        }

    }

    pub fn new(global: GlobalRef, url: DOMString) -> Fallible<Root<WebSocket>> {
        // Step 1.
        let parsed_url = try!(Url::parse(&url).map_err(|_| Error::Syntax));
        let url = try!(parse_url(&parsed_url).map_err(|_| Error::Syntax));

        /*TODO: This constructor is only a prototype, it does not accomplish the specs
          defined here:
          http://html.spec.whatwg.org
          The remaining 8 items must be satisfied.
          TODO: This constructor should be responsible for spawning a thread for the
          receive loop after ws.r().Open() - See comment
        */
        let ws = reflect_dom_object(box WebSocket::new_inherited(global, parsed_url),
                                    global,
                                    WebSocketBinding::Wrap);

        let channel = establish_a_websocket_connection(url, global.get_url().serialize());
        let (temp_sender, temp_receiver) = match channel {
            Ok(channel) => channel,
            Err(e) => {
                debug!("Failed to establish a WebSocket connection: {:?}", e);
                let global_root = ws.r().global.root();
                let address = Trusted::new(global_root.r().get_cx(), ws.r(), global_root.r().script_chan().clone());
                let task = box WebSocketTaskHandler::new(address, WebSocketTask::Close);
                global_root.r().script_chan().send(ScriptMsg::RunnableMsg(task)).unwrap();
                return Ok(ws);
            }
        };

        *ws.r().sender.borrow_mut() = Some(temp_sender);
        *ws.r().receiver.borrow_mut() = Some(temp_receiver);

        //Create everything necessary for starting the open asynchronous task, then begin the task.
        let global_root = ws.r().global.root();
        let addr: Trusted<WebSocket> =
            Trusted::new(global_root.r().get_cx(), ws.r(), global_root.r().script_chan().clone());
        let open_task = box WebSocketTaskHandler::new(addr, WebSocketTask::ConnectionEstablished);
        global_root.r().script_chan().send(ScriptMsg::RunnableMsg(open_task)).unwrap();
        //TODO: Spawn thread here for receive loop
        /*TODO: Add receive loop here and make new thread run this
          Receive is an infinite loop "similiar" the one shown here:
          https://github.com/cyderize/rust-websocket/blob/master/examples/client.rs#L64
          TODO: The receive loop however does need to follow the spec. These are outlined here
          under "WebSocket message has been received" items 1-5:
          https://github.com/cyderize/rust-websocket/blob/master/examples/client.rs#L64
          TODO: The receive loop also needs to dispatch an asynchronous event as stated here:
          https://github.com/cyderize/rust-websocket/blob/master/examples/client.rs#L64
          TODO: When the receive loop receives a close message from the server,
          it confirms the websocket is now closed. This requires the close event
          to be fired (dispatch_close fires the close event - see implementation below)
        */
        Ok(ws)
    }

    pub fn Constructor(global: GlobalRef, url: DOMString) -> Fallible<Root<WebSocket>> {
        WebSocket::new(global, url)
    }
}

impl<'a> WebSocketMethods for &'a WebSocket {
    event_handler!(open, GetOnopen, SetOnopen);
    event_handler!(close, GetOnclose, SetOnclose);
    event_handler!(error, GetOnerror, SetOnerror);

    fn Url(self) -> DOMString {
        self.url.serialize()
    }

    fn ReadyState(self) -> u16 {
        self.ready_state.get() as u16
    }

    fn Send(self, data: Option<USVString>) -> Fallible<()> {
        match self.ready_state.get() {
            WebSocketRequestState::Connecting => {
                return Err(Error::InvalidState);
            },
            WebSocketRequestState::Open => (),
            WebSocketRequestState::Closing | WebSocketRequestState::Closed => {
                // TODO: Update bufferedAmount.
                return Ok(());
            }
        }

        /*TODO: This is not up to spec see http://html.spec.whatwg.org/multipage/comms.html search for
                "If argument is a string"
          TODO: Need to buffer data
          TODO: bufferedAmount attribute returns the size of the buffer in bytes -
                this is a required attribute defined in the websocket.webidl file
          TODO: The send function needs to flag when full by using the following
          self.full.set(true). This needs to be done when the buffer is full
        */
        let mut other_sender = self.sender.borrow_mut();
        let my_sender = other_sender.as_mut().unwrap();
        let _ = my_sender.send_message(Message::Text(data.unwrap().0));
        return Ok(())
    }

    fn Close(self, code: Option<u16>, reason: Option<USVString>) -> Fallible<()>{
        fn send_close(this: &WebSocket) {
            this.ready_state.set(WebSocketRequestState::Closing);

            let mut sender = this.sender.borrow_mut();
            //TODO: Also check if the buffer is full
            if let Some(sender) = sender.as_mut() {
                let _ = sender.send_message(Message::Close(None));
            }
        }


        if let Some(code) = code {
            //Check code is NOT 1000 NOR in the range of 3000-4999 (inclusive)
            if  code != 1000 && (code < 3000 || code > 4999) {
                return Err(Error::InvalidAccess);
            }
        }
        if let Some(ref reason) = reason {
            if reason.0.as_bytes().len() > 123 { //reason cannot be larger than 123 bytes
                return Err(Error::Syntax);
            }
        }

        match self.ready_state.get() {
            WebSocketRequestState::Closing | WebSocketRequestState::Closed  => {} //Do nothing
            WebSocketRequestState::Connecting => { //Connection is not yet established
                /*By setting the state to closing, the open function
                  will abort connecting the websocket*/
                self.failed.set(true);
                send_close(self);
                //Note: After sending the close message, the receive loop confirms a close message from the server and
                //      must fire a close event
            }
            WebSocketRequestState::Open => {
                //Closing handshake not started - still in open
                //Start the closing by setting the code and reason if they exist
                if let Some(code) = code {
                    self.code.set(code);
                }
                if let Some(reason) = reason {
                    *self.reason.borrow_mut() = reason.0;
                }
                send_close(self);
                //Note: After sending the close message, the receive loop confirms a close message from the server and
                //      must fire a close event
            }
        }
        Ok(()) //Return Ok
    }
}


pub enum WebSocketTask {
    /// Task queued when *the WebSocket connection is established*.
    ConnectionEstablished,
    Close,
}

pub struct WebSocketTaskHandler {
    addr: Trusted<WebSocket>,
    task: WebSocketTask,
}

impl WebSocketTaskHandler {
    pub fn new(addr: Trusted<WebSocket>, task: WebSocketTask) -> WebSocketTaskHandler {
        WebSocketTaskHandler {
            addr: addr,
            task: task,
        }
    }

    fn connection_established(&self) {
        /*TODO: Items 1, 3, 4, & 5 under "WebSocket connection is established" as specified here:
          https://html.spec.whatwg.org/multipage/#feedback-from-the-protocol
        */
        let ws = self.addr.root();

        // Step 1: Protocols.

        // Step 2.
        ws.ready_state.set(WebSocketRequestState::Open);

        // Step 3: Extensions.
        // Step 4: Protocols.
        // Step 5: Cookies.

        // Step 6.
        let global = ws.global.root();
        let event = Event::new(global.r(), "open".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable);
        event.fire(EventTargetCast::from_ref(ws.r()));
    }

    fn dispatch_close(&self) {
        let ws = self.addr.root();
        let ws = ws.r();
        let global = ws.global.root();
        ws.ready_state.set(WebSocketRequestState::Closed);
        //If failed or full, fire error event
        if ws.failed.get() || ws.full.get() {
            ws.failed.set(false);
            ws.full.set(false);
            //A Bad close
            ws.clean_close.set(false);
            let event = Event::new(global.r(),
                                   "error".to_owned(),
                                   EventBubbles::DoesNotBubble,
                                   EventCancelable::Cancelable);
            let target = EventTargetCast::from_ref(ws);
            event.r().fire(target);
        }
        let rsn = ws.reason.borrow();
        let rsn_clone = rsn.clone();
        /*In addition, we also have to fire a close even if error event fired
         https://html.spec.whatwg.org/multipage/#closeWebSocket
        */
        let close_event = CloseEvent::new(global.r(),
                                          "close".to_owned(),
                                          EventBubbles::DoesNotBubble,
                                          EventCancelable::NotCancelable,
                                          ws.clean_close.get(),
                                          ws.code.get(),
                                          rsn_clone);
        let target = EventTargetCast::from_ref(ws);
        let event = EventCast::from_ref(close_event.r());
        event.fire(target);
    }
}

impl Runnable for WebSocketTaskHandler {
    fn handler(self: Box<WebSocketTaskHandler>) {
        match self.task {
            WebSocketTask::ConnectionEstablished => {
                self.connection_established();
            }
            WebSocketTask::Close => {
                self.dispatch_close();
            }
        }
    }
}

