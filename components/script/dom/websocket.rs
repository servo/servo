/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::WebSocketBinding;
use dom::bindings::codegen::Bindings::WebSocketBinding::{BinaryType, WebSocketMethods};
use dom::bindings::codegen::UnionTypes::StringOrStringSequence;
use dom::bindings::conversions::ToJSValConvertible;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString, is_token};
use dom::blob::{Blob, BlobImpl};
use dom::closeevent::CloseEvent;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::messageevent::MessageEvent;
use dom::urlhelper::UrlHelper;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use js::jsapi::{JS_GetArrayBufferData, JS_NewArrayBuffer};
use js::jsapi::JSAutoCompartment;
use js::jsval::UndefinedValue;
use libc::{uint32_t, uint8_t};
use net_traits::{WebSocketCommunicate, WebSocketConnectData, WebSocketDomAction, WebSocketNetworkEvent};
use net_traits::CookieSource::HTTP;
use net_traits::CoreResourceMsg::{SetCookiesForUrl, WebsocketConnect};
use net_traits::MessageData;
use net_traits::hosts::replace_hosts;
use net_traits::unwrap_websocket_protocol;
use script_runtime::CommonScriptMsg;
use script_runtime::ScriptThreadEventCategory::WebSocketEvent;
use script_thread::{Runnable, RunnableWrapper};
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::Cell;
use std::ptr;
use std::thread;
use task_source::TaskSource;
use task_source::networking::NetworkingTaskSource;
use websocket::client::request::Url;
use websocket::header::{Headers, WebSocketProtocol};
use websocket::ws::util::url::parse_url;

#[derive(JSTraceable, PartialEq, Copy, Clone, Debug, HeapSizeOf)]
enum WebSocketRequestState {
    Connecting = 0,
    Open = 1,
    Closing = 2,
    Closed = 3,
}

// list of bad ports according to
// https://fetch.spec.whatwg.org/#port-blocking
const BLOCKED_PORTS_LIST: &'static [u16] = &[
    1,    // tcpmux
    7,    // echo
    9,    // discard
    11,   // systat
    13,   // daytime
    15,   // netstat
    17,   // qotd
    19,   // chargen
    20,   // ftp-data
    21,   // ftp
    22,   // ssh
    23,   // telnet
    25,   // smtp
    37,   // time
    42,   // name
    43,   // nicname
    53,   // domain
    77,   // priv-rjs
    79,   // finger
    87,   // ttylink
    95,   // supdup
    101,  // hostriame
    102,  // iso-tsap
    103,  // gppitnp
    104,  // acr-nema
    109,  // pop2
    110,  // pop3
    111,  // sunrpc
    113,  // auth
    115,  // sftp
    117,  // uucp-path
    119,  // nntp
    123,  // ntp
    135,  // loc-srv / epmap
    139,  // netbios
    143,  // imap2
    179,  // bgp
    389,  // ldap
    465,  // smtp+ssl
    512,  // print / exec
    513,  // login
    514,  // shell
    515,  // printer
    526,  // tempo
    530,  // courier
    531,  // chat
    532,  // netnews
    540,  // uucp
    556,  // remotefs
    563,  // nntp+ssl
    587,  // smtp
    601,  // syslog-conn
    636,  // ldap+ssl
    993,  // imap+ssl
    995,  // pop3+ssl
    2049, // nfs
    3659, // apple-sasl
    4045, // lockd
    6000, // x11
    6665, // irc (alternate)
    6666, // irc (alternate)
    6667, // irc (default)
    6668, // irc (alternate)
    6669, // irc (alternate)
];

// Close codes defined in https://tools.ietf.org/html/rfc6455#section-7.4.1
// Names are from https://github.com/mozilla/gecko-dev/blob/master/netwerk/protocol/websocket/nsIWebSocketChannel.idl
#[allow(dead_code)]
mod close_code {
    pub const NORMAL: u16 = 1000;
    pub const GOING_AWAY: u16 = 1001;
    pub const PROTOCOL_ERROR: u16 = 1002;
    pub const UNSUPPORTED_DATATYPE: u16 = 1003;
    pub const NO_STATUS: u16 = 1005;
    pub const ABNORMAL: u16 = 1006;
    pub const INVALID_PAYLOAD: u16 = 1007;
    pub const POLICY_VIOLATION: u16 = 1008;
    pub const TOO_LARGE: u16 = 1009;
    pub const EXTENSION_MISSING: u16 = 1010;
    pub const INTERNAL_ERROR: u16 = 1011;
    pub const TLS_FAILED: u16 = 1015;
}

pub fn close_the_websocket_connection(address: Trusted<WebSocket>,
                                      task_source: &NetworkingTaskSource,
                                      wrapper: &RunnableWrapper,
                                      code: Option<u16>,
                                      reason: String) {
    let close_task = box CloseTask {
        address: address,
        failed: false,
        code: code,
        reason: Some(reason),
    };
    task_source.queue_with_wrapper(close_task, &wrapper).unwrap();
}

pub fn fail_the_websocket_connection(address: Trusted<WebSocket>,
                                     task_source: &NetworkingTaskSource,
                                     wrapper: &RunnableWrapper) {
    let close_task = box CloseTask {
        address: address,
        failed: true,
        code: Some(close_code::ABNORMAL),
        reason: None,
    };
    task_source.queue_with_wrapper(close_task, &wrapper).unwrap();
}

#[dom_struct]
pub struct WebSocket {
    eventtarget: EventTarget,
    url: Url,
    ready_state: Cell<WebSocketRequestState>,
    buffered_amount: Cell<u64>,
    clearing_buffer: Cell<bool>, //Flag to tell if there is a running thread to clear buffered_amount
    #[ignore_heap_size_of = "Defined in std"]
    sender: DOMRefCell<Option<IpcSender<WebSocketDomAction>>>,
    binary_type: Cell<BinaryType>,
    protocol: DOMRefCell<String>, //Subprotocol selected by server
}

impl WebSocket {
    fn new_inherited(url: Url) -> WebSocket {
        WebSocket {
            eventtarget: EventTarget::new_inherited(),
            url: url,
            ready_state: Cell::new(WebSocketRequestState::Connecting),
            buffered_amount: Cell::new(0),
            clearing_buffer: Cell::new(false),
            sender: DOMRefCell::new(None),
            binary_type: Cell::new(BinaryType::Blob),
            protocol: DOMRefCell::new("".to_owned()),
        }
    }

    fn new(global: &GlobalScope, url: Url) -> Root<WebSocket> {
        reflect_dom_object(box WebSocket::new_inherited(url),
                           global, WebSocketBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalScope,
                       url: DOMString,
                       protocols: Option<StringOrStringSequence>)
                       -> Fallible<Root<WebSocket>> {
        // Step 1.
        let resource_url = try!(Url::parse(&url).map_err(|_| Error::Syntax));
        // Although we do this replace and parse operation again in the resource thread,
        // we try here to be able to immediately throw a syntax error on failure.
        let _ = try!(parse_url(&replace_hosts(&resource_url)).map_err(|_| Error::Syntax));
        // Step 2: Disallow https -> ws connections.

        // Step 3: Potentially block access to some ports.
        let port: u16 = resource_url.port_or_known_default().unwrap();

        if BLOCKED_PORTS_LIST.iter().any(|&p| p == port) {
            return Err(Error::Security);
        }

        // Step 4.
        let protocols = match protocols {
            Some(StringOrStringSequence::String(string)) => vec![String::from(string)],
            Some(StringOrStringSequence::StringSequence(sequence)) => {
                sequence.into_iter().map(String::from).collect()
            },
            _ => Vec::new(),
        };

        // Step 5.
        for (i, protocol) in protocols.iter().enumerate() {
            // https://tools.ietf.org/html/rfc6455#section-4.1
            // Handshake requirements, step 10

            if protocols[i + 1..].iter().any(|p| p.eq_ignore_ascii_case(protocol)) {
                return Err(Error::Syntax);
            }

            // https://tools.ietf.org/html/rfc6455#section-4.1
            if !is_token(protocol.as_bytes()) {
                return Err(Error::Syntax);
            }
        }

        // Step 6: Origin.
        let origin = UrlHelper::Origin(&global.get_url()).0;

        // Step 7.
        let ws = WebSocket::new(global, resource_url.clone());
        let address = Trusted::new(&*ws);

        let connect_data = WebSocketConnectData {
            resource_url: resource_url.clone(),
            origin: origin,
            protocols: protocols,
        };

        // Create the interface for communication with the resource thread
        let (dom_action_sender, resource_action_receiver):
                (IpcSender<WebSocketDomAction>,
                IpcReceiver<WebSocketDomAction>) = ipc::channel().unwrap();
        let (resource_event_sender, dom_event_receiver):
                (IpcSender<WebSocketNetworkEvent>,
                IpcReceiver<WebSocketNetworkEvent>) = ipc::channel().unwrap();

        let connect = WebSocketCommunicate {
            event_sender: resource_event_sender,
            action_receiver: resource_action_receiver,
        };

        let _ = global.core_resource_thread().send(WebsocketConnect(connect, connect_data));

        *ws.sender.borrow_mut() = Some(dom_action_sender);

        let moved_address = address.clone();
        let task_source = global.networking_task_source();
        let wrapper = global.get_runnable_wrapper();
        thread::spawn(move || {
            while let Ok(event) = dom_event_receiver.recv() {
                match event {
                    WebSocketNetworkEvent::ConnectionEstablished(headers, protocols) => {
                        let open_thread = box ConnectionEstablishedTask {
                            address: moved_address.clone(),
                            headers: headers,
                            protocols: protocols,
                        };
                        task_source.queue_with_wrapper(open_thread, &wrapper).unwrap();
                    },
                    WebSocketNetworkEvent::MessageReceived(message) => {
                        let message_thread = box MessageReceivedTask {
                            address: moved_address.clone(),
                            message: message,
                        };
                        task_source.queue_with_wrapper(message_thread, &wrapper).unwrap();
                    },
                    WebSocketNetworkEvent::Fail => {
                        fail_the_websocket_connection(moved_address.clone(),
                            &task_source, &wrapper);
                    },
                    WebSocketNetworkEvent::Close(code, reason) => {
                        close_the_websocket_connection(moved_address.clone(),
                            &task_source, &wrapper, code, reason);
                    },
                }
            }
        });
        // Step 7.
        Ok(ws)
    }

    // https://html.spec.whatwg.org/multipage/#dom-websocket-send
    fn send_impl(&self, data_byte_len: u64) -> Fallible<bool> {
        let return_after_buffer = match self.ready_state.get() {
            WebSocketRequestState::Connecting => {
                return Err(Error::InvalidState);
            },
            WebSocketRequestState::Open => false,
            WebSocketRequestState::Closing | WebSocketRequestState::Closed => true,
        };

        let address = Trusted::new(self);

        match data_byte_len.checked_add(self.buffered_amount.get()) {
            None => panic!(),
            Some(new_amount) => self.buffered_amount.set(new_amount)
        };

        if return_after_buffer {
            return Ok(false);
        }

        if !self.clearing_buffer.get() && self.ready_state.get() == WebSocketRequestState::Open {
            self.clearing_buffer.set(true);

            let task = box BufferedAmountTask {
                address: address,
            };

            self.global()
                .script_chan()
                .send(CommonScriptMsg::RunnableMsg(WebSocketEvent, task))
                .unwrap();
        }

        Ok(true)
    }
}

impl WebSocketMethods for WebSocket {
    // https://html.spec.whatwg.org/multipage/#handler-websocket-onopen
    event_handler!(open, GetOnopen, SetOnopen);

    // https://html.spec.whatwg.org/multipage/#handler-websocket-onclose
    event_handler!(close, GetOnclose, SetOnclose);

    // https://html.spec.whatwg.org/multipage/#handler-websocket-onerror
    event_handler!(error, GetOnerror, SetOnerror);

    // https://html.spec.whatwg.org/multipage/#handler-websocket-onmessage
    event_handler!(message, GetOnmessage, SetOnmessage);

    // https://html.spec.whatwg.org/multipage/#dom-websocket-url
    fn Url(&self) -> DOMString {
        DOMString::from(self.url.as_str())
    }

    // https://html.spec.whatwg.org/multipage/#dom-websocket-readystate
    fn ReadyState(&self) -> u16 {
        self.ready_state.get() as u16
    }

    // https://html.spec.whatwg.org/multipage/#dom-websocket-bufferedamount
    fn BufferedAmount(&self) -> u64 {
        self.buffered_amount.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-websocket-binarytype
    fn BinaryType(&self) -> BinaryType {
        self.binary_type.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-websocket-binarytype
    fn SetBinaryType(&self, btype: BinaryType) {
        self.binary_type.set(btype)
    }

    // https://html.spec.whatwg.org/multipage/#dom-websocket-protocol
    fn Protocol(&self) -> DOMString {
         DOMString::from(self.protocol.borrow().clone())
    }

    // https://html.spec.whatwg.org/multipage/#dom-websocket-send
    fn Send(&self, data: USVString) -> ErrorResult {
        let data_byte_len = data.0.as_bytes().len() as u64;
        let send_data = try!(self.send_impl(data_byte_len));

        if send_data {
            let mut other_sender = self.sender.borrow_mut();
            let my_sender = other_sender.as_mut().unwrap();
            let _ = my_sender.send(WebSocketDomAction::SendMessage(MessageData::Text(data.0)));
        }

        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-websocket-send
    fn Send_(&self, blob: &Blob) -> ErrorResult {
        /* As per https://html.spec.whatwg.org/multipage/#websocket
           the buffered amount needs to be clamped to u32, even though Blob.Size() is u64
           If the buffer limit is reached in the first place, there are likely other major problems
        */
        let data_byte_len = blob.Size();
        let send_data = try!(self.send_impl(data_byte_len));

        if send_data {
            let mut other_sender = self.sender.borrow_mut();
            let my_sender = other_sender.as_mut().unwrap();
            let bytes = blob.get_bytes().unwrap_or(vec![]);
            let _ = my_sender.send(WebSocketDomAction::SendMessage(MessageData::Binary(bytes)));
        }

        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-websocket-close
    fn Close(&self, code: Option<u16>, reason: Option<USVString>) -> ErrorResult {
        if let Some(code) = code {
            //Fail if the supplied code isn't normal and isn't reserved for libraries, frameworks, and applications
            if code != close_code::NORMAL && (code < 3000 || code > 4999) {
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

                let address = Trusted::new(self);
                let task_source = self.global().networking_task_source();
                fail_the_websocket_connection(address, &task_source, &self.global().get_runnable_wrapper());
            }
            WebSocketRequestState::Open => {
                self.ready_state.set(WebSocketRequestState::Closing);

                // Kick off _Start the WebSocket Closing Handshake_
                // https://tools.ietf.org/html/rfc6455#section-7.1.2
                let reason = reason.map(|reason| reason.0);
                let mut other_sender = self.sender.borrow_mut();
                let my_sender = other_sender.as_mut().unwrap();
                let _ = my_sender.send(WebSocketDomAction::Close(code, reason));
            }
        }
        Ok(()) //Return Ok
    }
}


/// Task queued when *the WebSocket connection is established*.
struct ConnectionEstablishedTask {
    address: Trusted<WebSocket>,
    protocols: Vec<String>,
    headers: Headers,
}

impl Runnable for ConnectionEstablishedTask {
    fn name(&self) -> &'static str { "ConnectionEstablishedTask" }

    fn handler(self: Box<Self>) {
        let ws = self.address.root();

        // Step 1: Protocols.
        if !self.protocols.is_empty() && self.headers.get::<WebSocketProtocol>().is_none() {
            let task_source = ws.global().networking_task_source();
            fail_the_websocket_connection(self.address, &task_source, &ws.global().get_runnable_wrapper());
            return;
        }

        // Step 2.
        ws.ready_state.set(WebSocketRequestState::Open);

        // Step 3: Extensions.
        //TODO: Set extensions to extensions in use

        // Step 4: Protocols.
        let protocol_in_use = unwrap_websocket_protocol(self.headers.get::<WebSocketProtocol>());
        if let Some(protocol_name) = protocol_in_use {
            *ws.protocol.borrow_mut() = protocol_name.to_owned();
        };

        // Step 5: Cookies.
        if let Some(cookies) = self.headers.get_raw("set-cookie") {
            for cookie in cookies.iter() {
                if let Ok(cookie_value) = String::from_utf8(cookie.clone()) {
                    let _ = ws.global().core_resource_thread().send(
                        SetCookiesForUrl(ws.url.clone(), cookie_value, HTTP));
                }
            }
        }

        // Step 6.
        ws.upcast().fire_event(atom!("open"));
    }
}

struct BufferedAmountTask {
    address: Trusted<WebSocket>,
}

impl Runnable for BufferedAmountTask {
    // See https://html.spec.whatwg.org/multipage/#dom-websocket-bufferedamount
    //
    // To be compliant with standards, we need to reset bufferedAmount only when the event loop
    // reaches step 1.  In our implementation, the bytes will already have been sent on a background
    // thread.
    fn name(&self) -> &'static str { "BufferedAmountTask" }

    fn handler(self: Box<Self>) {
        let ws = self.address.root();

        ws.buffered_amount.set(0);
        ws.clearing_buffer.set(false);
    }
}

struct CloseTask {
    address: Trusted<WebSocket>,
    failed: bool,
    code: Option<u16>,
    reason: Option<String>,
}

impl Runnable for CloseTask {
    fn name(&self) -> &'static str { "CloseTask" }

    fn handler(self: Box<Self>) {
        let ws = self.address.root();

        if ws.ready_state.get() == WebSocketRequestState::Closed {
            // Do nothing if already closed.
            return;
        }

        // Perform _the WebSocket connection is closed_ steps.
        // https://html.spec.whatwg.org/multipage/#closeWebSocket

        // Step 1.
        ws.ready_state.set(WebSocketRequestState::Closed);

        // Step 2.
        if self.failed {
            ws.upcast().fire_event(atom!("error"));
        }

        // Step 3.
        let clean_close = !self.failed;
        let code = self.code.unwrap_or(close_code::NO_STATUS);
        let reason = DOMString::from(self.reason.unwrap_or("".to_owned()));
        let close_event = CloseEvent::new(&ws.global(),
                                          atom!("close"),
                                          EventBubbles::DoesNotBubble,
                                          EventCancelable::NotCancelable,
                                          clean_close,
                                          code,
                                          reason);
        close_event.upcast::<Event>().fire(ws.upcast());
    }
}

struct MessageReceivedTask {
    address: Trusted<WebSocket>,
    message: MessageData,
}

impl Runnable for MessageReceivedTask {
    fn name(&self) -> &'static str { "MessageReceivedTask" }

    #[allow(unsafe_code)]
    fn handler(self: Box<Self>) {
        let ws = self.address.root();
        debug!("MessageReceivedTask::handler({:p}): readyState={:?}", &*ws,
               ws.ready_state.get());

        // Step 1.
        if ws.ready_state.get() != WebSocketRequestState::Open {
            return;
        }

        // Step 2-5.
        let global = ws.global();
        // global.get_cx() returns a valid `JSContext` pointer, so this is safe.
        unsafe {
            let cx = global.get_cx();
            let _ac = JSAutoCompartment::new(cx, ws.reflector().get_jsobject().get());
            rooted!(in(cx) let mut message = UndefinedValue());
            match self.message {
                MessageData::Text(text) => text.to_jsval(cx, message.handle_mut()),
                MessageData::Binary(data) => {
                    match ws.binary_type.get() {
                        BinaryType::Blob => {
                            let blob = Blob::new(&global, BlobImpl::new_from_bytes(data), "".to_owned());
                            blob.to_jsval(cx, message.handle_mut());
                        }
                        BinaryType::Arraybuffer => {
                            let len = data.len() as uint32_t;
                            let buf = JS_NewArrayBuffer(cx, len);
                            let mut is_shared = false;
                            let buf_data: *mut uint8_t = JS_GetArrayBufferData(buf, &mut is_shared, ptr::null());
                            assert!(!is_shared);
                            ptr::copy_nonoverlapping(data.as_ptr(), buf_data, len as usize);
                            buf.to_jsval(cx, message.handle_mut());
                        }

                    }
                },
            }
            MessageEvent::dispatch_jsval(ws.upcast(), &global, message.handle());
        }
    }
}
