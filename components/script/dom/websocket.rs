/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::cell::Cell;
use std::ptr;

use constellation_traits::BlobImpl;
use dom_struct::dom_struct;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::{JSAutoRealm, JSObject};
use js::jsval::UndefinedValue;
use js::rust::{CustomAutoRooterGuard, HandleObject};
use js::typedarray::{ArrayBuffer, ArrayBufferView, CreateWith};
use net_traits::request::{
    CacheMode, CredentialsMode, RedirectMode, Referrer, RequestBuilder, RequestMode,
    ServiceWorkersMode,
};
use net_traits::{
    CoreResourceMsg, FetchChannels, MessageData, WebSocketDomAction, WebSocketNetworkEvent,
};
use profile_traits::ipc as ProfiledIpc;
use servo_url::{ImmutableOrigin, ServoUrl};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use crate::dom::bindings::codegen::Bindings::WebSocketBinding::{BinaryType, WebSocketMethods};
use crate::dom::bindings::codegen::UnionTypes::StringOrStringSequence;
use crate::dom::bindings::conversions::ToJSValConvertible;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, DomObject, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString, is_token};
use crate::dom::blob::Blob;
use crate::dom::closeevent::CloseEvent;
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageevent::MessageEvent;
use crate::script_runtime::CanGc;
use crate::task::TaskOnce;
use crate::task_source::SendableTaskSource;

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
enum WebSocketRequestState {
    Connecting = 0,
    Open = 1,
    Closing = 2,
    Closed = 3,
}

// Close codes defined in https://tools.ietf.org/html/rfc6455#section-7.4.1
// Names are from https://github.com/mozilla/gecko-dev/blob/master/netwerk/protocol/websocket/nsIWebSocketChannel.idl
#[allow(dead_code)]
mod close_code {
    pub(crate) const NORMAL: u16 = 1000;
    pub(crate) const GOING_AWAY: u16 = 1001;
    pub(crate) const PROTOCOL_ERROR: u16 = 1002;
    pub(crate) const UNSUPPORTED_DATATYPE: u16 = 1003;
    pub(crate) const NO_STATUS: u16 = 1005;
    pub(crate) const ABNORMAL: u16 = 1006;
    pub(crate) const INVALID_PAYLOAD: u16 = 1007;
    pub(crate) const POLICY_VIOLATION: u16 = 1008;
    pub(crate) const TOO_LARGE: u16 = 1009;
    pub(crate) const EXTENSION_MISSING: u16 = 1010;
    pub(crate) const INTERNAL_ERROR: u16 = 1011;
    pub(crate) const TLS_FAILED: u16 = 1015;
}

fn close_the_websocket_connection(
    address: Trusted<WebSocket>,
    task_source: &SendableTaskSource,
    code: Option<u16>,
    reason: String,
) {
    task_source.queue(CloseTask {
        address,
        failed: false,
        code,
        reason: Some(reason),
    });
}

fn fail_the_websocket_connection(address: Trusted<WebSocket>, task_source: &SendableTaskSource) {
    task_source.queue(CloseTask {
        address,
        failed: true,
        code: Some(close_code::ABNORMAL),
        reason: None,
    });
}

#[dom_struct]
pub(crate) struct WebSocket {
    eventtarget: EventTarget,
    #[no_trace]
    url: ServoUrl,
    ready_state: Cell<WebSocketRequestState>,
    buffered_amount: Cell<u64>,
    clearing_buffer: Cell<bool>, //Flag to tell if there is a running thread to clear buffered_amount
    #[ignore_malloc_size_of = "Defined in std"]
    #[no_trace]
    sender: IpcSender<WebSocketDomAction>,
    binary_type: Cell<BinaryType>,
    protocol: DomRefCell<String>, //Subprotocol selected by server
}

impl WebSocket {
    fn new_inherited(url: ServoUrl, sender: IpcSender<WebSocketDomAction>) -> WebSocket {
        WebSocket {
            eventtarget: EventTarget::new_inherited(),
            url,
            ready_state: Cell::new(WebSocketRequestState::Connecting),
            buffered_amount: Cell::new(0),
            clearing_buffer: Cell::new(false),
            sender,
            binary_type: Cell::new(BinaryType::Blob),
            protocol: DomRefCell::new("".to_owned()),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        url: ServoUrl,
        sender: IpcSender<WebSocketDomAction>,
        can_gc: CanGc,
    ) -> DomRoot<WebSocket> {
        reflect_dom_object_with_proto(
            Box::new(WebSocket::new_inherited(url, sender)),
            global,
            proto,
            can_gc,
        )
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
            Some(new_amount) => self.buffered_amount.set(new_amount),
        };

        if return_after_buffer {
            return Ok(false);
        }

        if !self.clearing_buffer.get() && self.ready_state.get() == WebSocketRequestState::Open {
            self.clearing_buffer.set(true);

            // TODO(mrobinson): Should this task be cancellable?
            self.global()
                .task_manager()
                .websocket_task_source()
                .queue_unconditionally(BufferedAmountTask { address });
        }

        Ok(true)
    }

    pub(crate) fn origin(&self) -> ImmutableOrigin {
        self.url.origin()
    }
}

impl WebSocketMethods<crate::DomTypeHolder> for WebSocket {
    /// <https://html.spec.whatwg.org/multipage/#dom-websocket>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        url: DOMString,
        protocols: Option<StringOrStringSequence>,
    ) -> Fallible<DomRoot<WebSocket>> {
        // Step 1. Let baseURL be this's relevant settings object's API base URL.
        // Step 2. Let urlRecord be the result of applying the URL parser to url with baseURL.
        // Step 3. If urlRecord is failure, then throw a "SyntaxError" DOMException.
        let mut url_record = ServoUrl::parse(&url).or(Err(Error::Syntax))?;

        // Step 4. If urlRecord’s scheme is "http", then set urlRecord’s scheme to "ws".
        // Step 5. Otherwise, if urlRecord’s scheme is "https", set urlRecord’s scheme to "wss".
        // Step 6. If urlRecord’s scheme is not "ws" or "wss", then throw a "SyntaxError" DOMException.
        match url_record.scheme() {
            "http" => {
                url_record
                    .as_mut_url()
                    .set_scheme("ws")
                    .expect("Can't set scheme from http to ws");
            },
            "https" => {
                url_record
                    .as_mut_url()
                    .set_scheme("wss")
                    .expect("Can't set scheme from https to wss");
            },
            "ws" | "wss" => {},
            _ => return Err(Error::Syntax),
        }

        // Step 7. If urlRecord’s fragment is non-null, then throw a "SyntaxError" DOMException.
        if url_record.fragment().is_some() {
            return Err(Error::Syntax);
        }

        // Step 8. If protocols is a string, set protocols to a sequence consisting of just that string.
        let protocols = protocols.map_or(vec![], |p| match p {
            StringOrStringSequence::String(string) => vec![string.into()],
            StringOrStringSequence::StringSequence(seq) => {
                seq.into_iter().map(String::from).collect()
            },
        });

        // Step 9. If any of the values in protocols occur more than once or otherwise fail to match the requirements
        // for elements that comprise the value of `Sec-WebSocket-Protocol` fields as defined by The WebSocket protocol,
        // then throw a "SyntaxError" DOMException.
        for (i, protocol) in protocols.iter().enumerate() {
            // https://tools.ietf.org/html/rfc6455#section-4.1
            // Handshake requirements, step 10

            if protocols[i + 1..]
                .iter()
                .any(|p| p.eq_ignore_ascii_case(protocol))
            {
                return Err(Error::Syntax);
            }

            // https://tools.ietf.org/html/rfc6455#section-4.1
            if !is_token(protocol.as_bytes()) {
                return Err(Error::Syntax);
            }
        }

        // Create the interface for communication with the resource thread
        let (dom_action_sender, resource_action_receiver): (
            IpcSender<WebSocketDomAction>,
            IpcReceiver<WebSocketDomAction>,
        ) = ipc::channel().unwrap();
        let (resource_event_sender, dom_event_receiver): (
            IpcSender<WebSocketNetworkEvent>,
            ProfiledIpc::IpcReceiver<WebSocketNetworkEvent>,
        ) = ProfiledIpc::channel(global.time_profiler_chan().clone()).unwrap();

        // Step 12. Establish a WebSocket connection given urlRecord, protocols, and client.
        let ws = WebSocket::new(global, proto, url_record.clone(), dom_action_sender, can_gc);
        let address = Trusted::new(&*ws);

        let request = RequestBuilder::new(global.webview_id(), url_record, Referrer::NoReferrer)
            .origin(global.origin().immutable().clone())
            .insecure_requests_policy(global.insecure_requests_policy())
            .has_trustworthy_ancestor_origin(global.has_trustworthy_ancestor_or_current_origin())
            .mode(RequestMode::WebSocket { protocols })
            .service_workers_mode(ServiceWorkersMode::None)
            .credentials_mode(CredentialsMode::Include)
            .cache_mode(CacheMode::NoCache)
            .policy_container(global.policy_container())
            .redirect_mode(RedirectMode::Error);

        let channels = FetchChannels::WebSocket {
            event_sender: resource_event_sender,
            action_receiver: resource_action_receiver,
        };
        let _ = global
            .core_resource_thread()
            .send(CoreResourceMsg::Fetch(request, channels));

        let task_source = global.task_manager().websocket_task_source().to_sendable();
        ROUTER.add_typed_route(
            dom_event_receiver.to_ipc_receiver(),
            Box::new(move |message| match message.unwrap() {
                WebSocketNetworkEvent::ReportCSPViolations(violations) => {
                    let task = ReportCSPViolationTask {
                        websocket: address.clone(),
                        violations,
                    };
                    task_source.queue(task);
                },
                WebSocketNetworkEvent::ConnectionEstablished { protocol_in_use } => {
                    let open_thread = ConnectionEstablishedTask {
                        address: address.clone(),
                        protocol_in_use,
                    };
                    task_source.queue(open_thread);
                },
                WebSocketNetworkEvent::MessageReceived(message) => {
                    let message_thread = MessageReceivedTask {
                        address: address.clone(),
                        message,
                    };
                    task_source.queue(message_thread);
                },
                WebSocketNetworkEvent::Fail => {
                    fail_the_websocket_connection(address.clone(), &task_source);
                },
                WebSocketNetworkEvent::Close(code, reason) => {
                    close_the_websocket_connection(address.clone(), &task_source, code, reason);
                },
            }),
        );

        Ok(ws)
    }

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
        let data_byte_len = data.0.len() as u64;
        let send_data = self.send_impl(data_byte_len)?;

        if send_data {
            let _ = self
                .sender
                .send(WebSocketDomAction::SendMessage(MessageData::Text(data.0)));
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
        let send_data = self.send_impl(data_byte_len)?;

        if send_data {
            let bytes = blob.get_bytes().unwrap_or_default();
            let _ = self
                .sender
                .send(WebSocketDomAction::SendMessage(MessageData::Binary(bytes)));
        }

        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-websocket-send
    fn Send__(&self, array: CustomAutoRooterGuard<ArrayBuffer>) -> ErrorResult {
        let bytes = array.to_vec();
        let data_byte_len = bytes.len();
        let send_data = self.send_impl(data_byte_len as u64)?;

        if send_data {
            let _ = self
                .sender
                .send(WebSocketDomAction::SendMessage(MessageData::Binary(bytes)));
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-websocket-send
    fn Send___(&self, array: CustomAutoRooterGuard<ArrayBufferView>) -> ErrorResult {
        let bytes = array.to_vec();
        let data_byte_len = bytes.len();
        let send_data = self.send_impl(data_byte_len as u64)?;

        if send_data {
            let _ = self
                .sender
                .send(WebSocketDomAction::SendMessage(MessageData::Binary(bytes)));
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-websocket-close
    fn Close(&self, code: Option<u16>, reason: Option<USVString>) -> ErrorResult {
        if let Some(code) = code {
            //Fail if the supplied code isn't normal and isn't reserved for libraries, frameworks, and applications
            if code != close_code::NORMAL && !(3000..=4999).contains(&code) {
                return Err(Error::InvalidAccess);
            }
        }
        if let Some(ref reason) = reason {
            if reason.0.len() > 123 {
                //reason cannot be larger than 123 bytes
                return Err(Error::Syntax);
            }
        }

        match self.ready_state.get() {
            WebSocketRequestState::Closing | WebSocketRequestState::Closed => {}, //Do nothing
            WebSocketRequestState::Connecting => {
                //Connection is not yet established
                /*By setting the state to closing, the open function
                will abort connecting the websocket*/
                self.ready_state.set(WebSocketRequestState::Closing);

                fail_the_websocket_connection(
                    Trusted::new(self),
                    &self
                        .global()
                        .task_manager()
                        .websocket_task_source()
                        .to_sendable(),
                );
            },
            WebSocketRequestState::Open => {
                self.ready_state.set(WebSocketRequestState::Closing);

                // Kick off _Start the WebSocket Closing Handshake_
                // https://tools.ietf.org/html/rfc6455#section-7.1.2
                let reason = reason.map(|reason| reason.0);
                let _ = self.sender.send(WebSocketDomAction::Close(code, reason));
            },
        }
        Ok(()) //Return Ok
    }
}

struct ReportCSPViolationTask {
    websocket: Trusted<WebSocket>,
    violations: Vec<Violation>,
}

impl TaskOnce for ReportCSPViolationTask {
    fn run_once(self) {
        let global = self.websocket.root().global();
        global.report_csp_violations(self.violations, None);
    }
}

/// Task queued when *the WebSocket connection is established*.
/// <https://html.spec.whatwg.org/multipage/#feedback-from-the-protocol:concept-websocket-established>
struct ConnectionEstablishedTask {
    address: Trusted<WebSocket>,
    protocol_in_use: Option<String>,
}

impl TaskOnce for ConnectionEstablishedTask {
    /// <https://html.spec.whatwg.org/multipage/#feedback-from-the-protocol:concept-websocket-established>
    fn run_once(self) {
        let ws = self.address.root();

        // Step 1.
        ws.ready_state.set(WebSocketRequestState::Open);

        // Step 2: Extensions.
        // TODO: Set extensions to extensions in use.

        // Step 3.
        if let Some(protocol_name) = self.protocol_in_use {
            *ws.protocol.borrow_mut() = protocol_name;
        };

        // Step 4.
        ws.upcast().fire_event(atom!("open"), CanGc::note());
    }
}

struct BufferedAmountTask {
    address: Trusted<WebSocket>,
}

impl TaskOnce for BufferedAmountTask {
    // See https://html.spec.whatwg.org/multipage/#dom-websocket-bufferedamount
    //
    // To be compliant with standards, we need to reset bufferedAmount only when the event loop
    // reaches step 1.  In our implementation, the bytes will already have been sent on a background
    // thread.
    fn run_once(self) {
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

impl TaskOnce for CloseTask {
    fn run_once(self) {
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
            ws.upcast().fire_event(atom!("error"), CanGc::note());
        }

        // Step 3.
        let clean_close = !self.failed;
        let code = self.code.unwrap_or(close_code::NO_STATUS);
        let reason = DOMString::from(self.reason.unwrap_or("".to_owned()));
        let close_event = CloseEvent::new(
            &ws.global(),
            atom!("close"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            clean_close,
            code,
            reason,
            CanGc::note(),
        );
        close_event
            .upcast::<Event>()
            .fire(ws.upcast(), CanGc::note());
    }
}

struct MessageReceivedTask {
    address: Trusted<WebSocket>,
    message: MessageData,
}

impl TaskOnce for MessageReceivedTask {
    #[allow(unsafe_code)]
    fn run_once(self) {
        let ws = self.address.root();
        debug!(
            "MessageReceivedTask::handler({:p}): readyState={:?}",
            &*ws,
            ws.ready_state.get()
        );

        // Step 1.
        if ws.ready_state.get() != WebSocketRequestState::Open {
            return;
        }

        // Step 2-5.
        let global = ws.global();
        // GlobalScope::get_cx() returns a valid `JSContext` pointer, so this is safe.
        unsafe {
            let cx = GlobalScope::get_cx();
            let _ac = JSAutoRealm::new(*cx, ws.reflector().get_jsobject().get());
            rooted!(in(*cx) let mut message = UndefinedValue());
            match self.message {
                MessageData::Text(text) => text.to_jsval(*cx, message.handle_mut()),
                MessageData::Binary(data) => match ws.binary_type.get() {
                    BinaryType::Blob => {
                        let blob = Blob::new(
                            &global,
                            BlobImpl::new_from_bytes(data, "".to_owned()),
                            CanGc::note(),
                        );
                        blob.to_jsval(*cx, message.handle_mut());
                    },
                    BinaryType::Arraybuffer => {
                        rooted!(in(*cx) let mut array_buffer = ptr::null_mut::<JSObject>());
                        assert!(
                            ArrayBuffer::create(
                                *cx,
                                CreateWith::Slice(&data),
                                array_buffer.handle_mut()
                            )
                            .is_ok()
                        );

                        (*array_buffer).to_jsval(*cx, message.handle_mut());
                    },
                },
            }
            MessageEvent::dispatch_jsval(
                ws.upcast(),
                &global,
                message.handle(),
                Some(&ws.origin().ascii_serialization()),
                None,
                vec![],
                CanGc::note(),
            );
        }
    }
}
