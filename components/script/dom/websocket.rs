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

use websocket::Message;
use websocket::ws::sender::Sender as Sender_Object;
use websocket::client::sender::Sender;
use websocket::client::receiver::Receiver;
use websocket::stream::WebSocketStream;
use websocket::client::request::Url;
use websocket::Client;

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
    sendCloseFrame: Cell<bool>
}

fn parse_web_socket_url(url_str: &str) -> Fallible<(Url, String, u16, String, bool)> {
    // https://html.spec.whatwg.org/multipage/#parse-a-websocket-url's-components
    // Steps 1 and 2
    let parsed_url = Url::parse(url_str);
    let parsed_url = match parsed_url {
        Ok(parsed_url) => parsed_url,
        Err(_) => return Err(Error::Syntax),
    };

    // Step 4
    if parsed_url.fragment != None {
        return Err(Error::Syntax);
    }

    // Steps 3 and 5
    let secure = match parsed_url.scheme.as_ref() {
        "ws" => false,
        "wss" => true,
        _ => return Err(Error::Syntax), // step 3
    };

    let host = parsed_url.host().unwrap().serialize(); // Step 6
    let port = parsed_url.port_or_default().unwrap(); // Steps 7 and 8
    let mut resource = parsed_url.path().unwrap().connect("/"); // Step 9
    if resource.is_empty() {
        resource = "/".to_owned(); // Step 10
    }

    // Step 11
    if let Some(ref query) = parsed_url.query {
        resource.push('?');
        resource.push_str(query);
    }

    // Step 12
    // FIXME remove parsed_url once it's no longer used in WebSocket::new
    Ok((parsed_url, host, port, resource, secure))
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
            sendCloseFrame: Cell::new(false)
        }

    }

    pub fn new(global: GlobalRef, url: DOMString) -> Fallible<Root<WebSocket>> {
        // Step 1.
        // FIXME extract the right variables once Client::connect
        // implementation is fixed to follow the RFC 6455 properly.
        let (url, _, _, _, _) = try!(parse_web_socket_url(&url));

        /*TODO: This constructor is only a prototype, it does not accomplish the specs
          defined here:
          http://html.spec.whatwg.org
          The remaining 8 items must be satisfied.
          TODO: This constructor should be responsible for spawning a thread for the
          receive loop after ws.r().Open() - See comment
        */
        let ws = reflect_dom_object(box WebSocket::new_inherited(global, url.clone()),
                                    global,
                                    WebSocketBinding::Wrap);

        // TODO Client::connect does not conform to RFC 6455
        // see https://github.com/cyderize/rust-websocket/issues/38
        let request = match Client::connect(url) {
            Ok(request) => request,
            Err(_) => {
                let global_root = ws.r().global.root();
                let address = Trusted::new(global_root.r().get_cx(), ws.r(), global_root.r().script_chan().clone());
                let task = box WebSocketTaskHandler::new(address, WebSocketTask::Close);
                global_root.r().script_chan().send(ScriptMsg::RunnableMsg(task)).unwrap();
                return Ok(ws);
            }
        };
        let response = request.send().unwrap();
        response.validate().unwrap();
        ws.r().ready_state.set(WebSocketRequestState::Open);
        //Check to see if ready_state is Closing or Closed and failed = true - means we failed the websocket
        //if so return without setting any states
        let ready_state = ws.r().ready_state.get();
        let failed = ws.r().failed.get();
        if failed && (ready_state == WebSocketRequestState::Closed || ready_state == WebSocketRequestState::Closing) {
            //Do nothing else. Let the close finish.
            return Ok(ws);
        }

        let (temp_sender, temp_receiver) = response.begin().split();
        *ws.r().sender.borrow_mut() = Some(temp_sender);
        *ws.r().receiver.borrow_mut() = Some(temp_receiver);

        //Create everything necessary for starting the open asynchronous task, then begin the task.
        let global_root = ws.r().global.root();
        let addr: Trusted<WebSocket> =
            Trusted::new(global_root.r().get_cx(), ws.r(), global_root.r().script_chan().clone());
        let open_task = box WebSocketTaskHandler::new(addr.clone(), WebSocketTask::Open);
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

    fn Send(self, data: Option<USVString>)-> Fallible<()>{
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
        if self.sendCloseFrame.get() { //TODO: Also check if the buffer is full
            self.sendCloseFrame.set(false);
            let _ = my_sender.send_message(Message::Close(None));
            return Ok(());
        }
        let _ = my_sender.send_message(Message::Text(data.unwrap().0));
        return Ok(())
    }

    fn Close(self, code: Option<u16>, reason: Option<USVString>) -> Fallible<()>{
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
                self.ready_state.set(WebSocketRequestState::Closing);
                self.failed.set(true);
                self.sendCloseFrame.set(true);
                //Dispatch send task to send close frame
                //TODO: Sending here is just empty string, though no string is really needed. Another send, empty
                //      send, could be used.
                let _ = self.Send(None);
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
                self.ready_state.set(WebSocketRequestState::Closing);
                self.sendCloseFrame.set(true);
                //Dispatch send task to send close frame
                let _ = self.Send(None);
                //Note: After sending the close message, the receive loop confirms a close message from the server and
                //      must fire a close event
            }
        }
        Ok(()) //Return Ok
    }
}


pub enum WebSocketTask {
    Open,
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

    fn dispatch_open(&self) {
        /*TODO: Items 1, 3, 4, & 5 under "WebSocket connection is established" as specified here:
          https://html.spec.whatwg.org/multipage/#feedback-from-the-protocol
        */
        let ws = self.addr.root(); //Get root
        let ws = ws.r(); //Get websocket reference
        let global = ws.global.root();
        let event = Event::new(global.r(),
            "open".to_owned(),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable);
        let target = EventTargetCast::from_ref(ws);
        event.r().fire(target);
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
            WebSocketTask::Open => {
                self.dispatch_open();
            }
            WebSocketTask::Close => {
                self.dispatch_close();
            }
        }
    }
}

