/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


/* References:
	Student page: https://github.com/servo/servo/wiki/WebSocket-student-project
	Servo error enums for throwing errors: https://github.com/servo/servo/wiki/WebSocket-student-project
	Servo core - a list of all the modules like cell, char, u 16:
		https://github.com/servo/servo/wiki/WebSocket-student-project
*/
use dom::bindings::codegen::Bindings::WebSocketBinding;
use dom::bindings::codegen::Bindings::WebSocketBinding::WebSocketMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::codegen::InheritTypes::EventCast;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::error::Error::InvalidAccess;
use dom::bindings::error::Error::Syntax;
use dom::event::{Event, EventBubbles, EventCancelable, EventHelpers};
use dom::closeevent::CloseEvent;
use dom::bindings::global::{GlobalField, GlobalRef};
use dom::bindings::js::{Temporary, JSRef, Rootable};
use dom::bindings::utils::reflect_dom_object;
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use util::str::DOMString;
use script_task::Runnable;
use script_task::ScriptMsg;
use dom::bindings::refcounted::Trusted;

use dom::bindings::cell::DOMRefCell;
use dom::bindings::trace::JSTraceable;
use websocket::Message;
use websocket::ws::sender::Sender as Sender_Object;
use websocket::client::sender::Sender;
use websocket::client::receiver::Receiver;
use websocket::stream::WebSocketStream;
use websocket::client::request::Url;
use websocket::Client;
use std::cell::{Cell, RefCell};
use std::borrow::ToOwned;


#[derive(PartialEq, Copy)]
#[jstraceable]
enum WebSocketRequestState {
    Connecting = 0,
    Open = 1,
    Closing = 2,
    Closed = 3,
}

no_jsmanaged_fields!(Sender<WebSocketStream>);
no_jsmanaged_fields!(Receiver<WebSocketStream>);

#[dom_struct]
pub struct WebSocket { //Websocket attributes defined here
    eventtarget: EventTarget,
    url: DOMString,
    global: GlobalField,
    ready_state: Cell<WebSocketRequestState>,
    sender: RefCell<Option<Sender<WebSocketStream> > >,
    receiver: RefCell<Option<Receiver<WebSocketStream> > >,
    failed: Cell<bool>, //Flag to tell if websocket was closed due to failure
    full: Cell<bool>, //Flag to tell if websocket queue is full
    clean_close: Cell<bool>, //Flag to tell if the websocket closed cleanly (not due to full or fail)
    code: Cell<u16>, //Closing code
    reason: DOMRefCell<DOMString>, //Closing reason
    data: DOMRefCell<DOMString>, //Data from send 
	sendCloseFrame: Cell<bool>
}

impl WebSocket {
    pub fn new_inherited(global: GlobalRef, url: DOMString) -> WebSocket {
        println!("Creating websocket...");
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

    pub fn new(global: GlobalRef, url: DOMString) -> Temporary<WebSocket> {
	/*TODO: This constructor is only a prototype, it does not accomplish the specs
	  defined here:
	  http://html.spec.whatwg.org/multipage/comms.html#client-specified-protocols
	  All 9 items must be satisfied.
	  TODO: This constructor should be responsible for spawning a thread for the
		receive loop after ws_root.r().Open() - See comment
	*/
        let ws_root = reflect_dom_object(box WebSocket::new_inherited(global, url),
                           global,
                           WebSocketBinding::Wrap).root();
        let _ = ws_root.r().Open(); //Calls open on websocket
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
        Temporary::from_rooted(ws_root.r()) //Return the websocket from the original thread that called this constructor
    }

    pub fn Constructor(global: GlobalRef, url: DOMString) -> Fallible<Temporary<WebSocket>> {
        Ok(WebSocket::new(global, url))
    }
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
        let global_root = self.global.root();
        let addr: Trusted<WebSocket> = Trusted::new(global_root.r().get_cx(), self, global_root.r().script_chan().clone());
        let open_task = box WebSocketTaskHandler::new(addr.clone(), WebSocketTask::Open);
        global_root.r().script_chan().send(ScriptMsg::RunnableMsg(open_task)).unwrap();
        Ok(())
    }

	fn Send(self, data: Option<DOMString>)-> Fallible<()>{
	/*TODO: This is not up to spec see http://html.spec.whatwg.org/multipage/comms.html search for "If argument is a string"
	  TODO: Need to buffer data
	  TODO: bufferedAmount attribute returns the size of the buffer in bytes - this is a required attribute defined in the websocket.webidle file
	  TODO: The send function needs to flag when full by using the following
	  	self.full.set(true). This needs to be done when the buffer is full
	*/
        *self.data.borrow_mut() = data.unwrap(); //storing the message to be sent for the this websocket
        let global_root = self.global.root();
        let addr: Trusted<WebSocket> = Trusted::new(global_root.r().get_cx(), self, global_root.r().script_chan().clone());
        let send_task = box WebSocketTaskHandler::new(addr.clone(), WebSocketTask::Send);
        global_root.r().script_chan().send(ScriptMsg::RunnableMsg(send_task)).unwrap();
        Ok(())
    }

	/*TODO: 3 other types of send as specified in webidle/WebSocket.webidl
		The specs for each of these functions are described here:
			https://html.spec.whatwg.org/multipage/comms.html
	*/		
    //void send(Blob data); //Search above reference for "If the argument is a blob"
    //void send(ArrayBuffer data); //Search above reference for "If the argument is an ArrayBuffer"
    //void send(ArrayBufferView data); //Search above reference for "If the argument is an object that matches the ArrayBufferView"

	//Close is up to spec! :)
    fn Close(self, code: Option<u16>, reason: Option<DOMString>) -> Fallible<()>{
		if code.is_some() { //Code defined
		   if !(code==Some(1000) ||
		   (code>=Some(3000) && code<=Some(4999)
		       )
		   )
		   { //Check code is NOT 1000 NOR in the range of 3000-4999 (inclusive)
		        return Err(Error::InvalidAccess); //Throw InvalidAccessError and abort
		   }
		}
		if reason.is_some() { //reason defined
		   if reason.as_ref().unwrap().as_bytes().len() > 123 //reason cannot be larger than 123 bytes
		   {
		    return Err(Error::Syntax); //Throw SyntaxError and abort
		   }
		}

		match self.ready_state.get() { //Returns the value of the cell
			WebSocketRequestState::Closing => {} //Do nothing
			WebSocketRequestState::Closed => {} //Do nothing
			WebSocketRequestState::Connecting => { //Connection is not yet established
				/*By setting the state to closing, the open function
				 will abort connecting the websocket*/
				self.ready_state.set(WebSocketRequestState::Closing); //Set state to closing
				self.failed.set(true); //Set failed flag
				self.sendCloseFrame.set(true); //Set close frame flag
				//-----Dispatch send task to send close frame------//
				let global_root = self.global.root();
				let addr: Trusted<WebSocket> = Trusted::new(global_root.r().get_cx(), self, global_root.r().script_chan().clone());
				let send_task = box WebSocketTaskHandler::new(addr.clone(), WebSocketTask::Send);
				global_root.r().script_chan().send(ScriptMsg::RunnableMsg(send_task)).unwrap();
				//Note: After sending the close message, the receive loop confirms a close message from the server and must fire a close event
			}
			WebSocketRequestState::Open => {//Closing handshake not started - still in open
			   //Start the closing by setting the code and reason if they exist
				if code.is_some() {
					self.code.set(code.unwrap());
				}
				if reason.is_some(){
				   *self.reason.borrow_mut() = reason.unwrap();
				}
				self.ready_state.set(WebSocketRequestState::Closing); //Set state to closing
				self.sendCloseFrame.set(true); //Set close frame flag
				//Dispatch send task to send close frame
				let global_root = self.global.root();
				let addr: Trusted<WebSocket> = Trusted::new(global_root.r().get_cx(), self, global_root.r().script_chan().clone());
				let send_task = box WebSocketTaskHandler::new(addr.clone(), WebSocketTask::Send);
				global_root.r().script_chan().send(ScriptMsg::RunnableMsg(send_task)).unwrap();
				//Note: After sending the close message, the receive loop confirms a close message from the server and must fire a close event
			}
		   //_ => { self.ready_state.set(WebSocketRequestState::Closing); } //Unreachable - Uncomment if you add more states to WebSocketRequestState
		}
		Ok(()) //Return Ok
    }
}


pub enum WebSocketTask {
    Open,
    Close,
    Send,
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
		https://html.spec.whatwg.org/multipage/comms.html#feedback-from-the-protocol
	*/
        println!("Trying to connect.");
        let ws = self.addr.to_temporary().root(); //Get root
        let ws = ws.r(); //Get websocket reference
        let parsed_url = Url::parse(ws.url.as_slice()).unwrap(); //Get url
        let request = Client::connect(parsed_url).unwrap(); //Set up the request
        let response = request.send().unwrap(); //Send the request to the server
        response.validate().unwrap(); //Validate the server's response
        println!("Successful connection.");
		ws.ready_state.set(WebSocketRequestState::Open); //Set state to open
        //Check to see if ready_state is Closing or Closed and failed = true - means we failed the websocket
        //if so return without setting any states
    if ((ws.ready_state.get() == WebSocketRequestState::Closed) || (ws.ready_state.get() == WebSocketRequestState::Closing)) && ws.failed.get() {
        //Do nothing else. Let the close finish.
    }
    else {
        let (temp_sender, temp_receiver) = response.begin().split(); //Set send and receiver in the attributes
        let mut other_sender = ws.sender.borrow_mut();
        let mut other_receiver = ws.receiver.borrow_mut();
        *other_sender = Some(temp_sender);
        *other_receiver = Some(temp_receiver);
        let global = ws.global.root();
        let event = Event::new(global.r(),
                               "open".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::Cancelable).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(ws);
        event.r().fire(target);
        println!("Fired open event.");
        }
    }

	/*TODO: Fix this to use the data buffer. Right now it reads the send_message stored
		in the attribute "data" - incorrect.
	*/
    fn dispatch_send(&self){
        println!("Trying to send");
        let ws = self.addr.to_temporary().root();
        let ws = ws.r();
        let mut other_sender = ws.sender.borrow_mut();
        let my_sender = other_sender.as_mut().unwrap();
		if ws.sendCloseFrame.get() { //TODO: Also check if the buffer is full
			ws.sendCloseFrame.set(false); //Deflag the close frame request
			let _ = my_sender.send_message(Message::Close(None)); //Send close frame
			return;
		}
        let data = ws.data.borrow();
        let data_clone = data.clone();
        let _ = my_sender.send_message(Message::Text(data_clone));
		return;
    }


    fn dispatch_close(&self) {
        let ws = self.addr.to_temporary().root();
        let global = ws.r().global.root();
        ws.r().ready_state.set(WebSocketRequestState::Closed); //Set to closed state
        //If failed or full, fire error event
        if ws.r().failed.get()||ws.r().full.get() {
            //Unset failed flag so we don't cause false positives
            ws.r().failed.set(false);
            //Unset full flag so we don't cause false positives
            ws.r().full.set(false);
            //A Bad close
            ws.r().clean_close.set(false);
            let event = Event::new(global.r(),
                                    "error".to_owned(),
                                    EventBubbles::DoesNotBubble,
                                    EventCancelable::Cancelable).root();
            let target: JSRef<EventTarget> = EventTargetCast::from_ref(ws.r());
            event.r().fire(target);
            println!("Fired error event.");
        }
        let ws = ws.r();
        let rsn = ws.reason.borrow();
        let rsn_clone = rsn.clone();
        /*In addition, we also have to fire a close even if error event fired
         https://html.spec.whatwg.org/multipage/comms.html#closeWebSocket
		*/
        let close_event = CloseEvent::new(global.r(),
                                        "close".to_owned(),
                                        EventBubbles::DoesNotBubble,
                        EventCancelable::Cancelable,
                        ws.clean_close.get(),
                        ws.code.get(),
                        rsn_clone
                        ).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(ws);
        let event: JSRef<Event> = EventCast::from_ref(close_event.r());
        event.set_trusted(true);
        event.fire(target);
        println!("Fired close event.");
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
            WebSocketTask::Send => {
                self.dispatch_send();
            }
        }
    }
}

