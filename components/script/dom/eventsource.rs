/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventSourceBinding::{EventSourceInit, EventSourceMethods, Wrap};
use dom::bindings::error::{Error, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::messageevent::MessageEvent;
use dom_struct::dom_struct;
use euclid::length::Length;
use hyper::header::{Accept, qitem};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::conversions::ToJSValConvertible;
use js::jsapi::JSAutoCompartment;
use js::jsval::UndefinedValue;
use mime::{Mime, TopLevel, SubLevel};
use net_traits::{CoreResourceMsg, FetchMetadata, FetchResponseMsg, FetchResponseListener, NetworkError};
use net_traits::request::{CacheMode, CorsSettings, CredentialsMode};
use net_traits::request::{RequestInit, RequestMode};
use network_listener::{NetworkListener, PreInvoke};
use script_thread::Runnable;
use servo_atoms::Atom;
use servo_url::ServoUrl;
use std::cell::Cell;
use std::mem;
use std::str::{Chars, FromStr};
use std::sync::{Arc, Mutex};
use task_source::TaskSource;
use timers::OneshotTimerCallback;
use utf8;

header! { (LastEventId, "Last-Event-ID") => [String] }

const DEFAULT_RECONNECTION_TIME: u64 = 5000;

#[derive(JSTraceable, PartialEq, Copy, Clone, Debug, HeapSizeOf)]
struct GenerationId(u32);

#[derive(JSTraceable, PartialEq, Copy, Clone, Debug, HeapSizeOf)]
/// https://html.spec.whatwg.org/multipage/#dom-eventsource-readystate
enum ReadyState {
    Connecting = 0,
    Open = 1,
    Closed = 2
}

#[dom_struct]
pub struct EventSource {
    eventtarget: EventTarget,
    url: ServoUrl,
    request: DOMRefCell<Option<RequestInit>>,
    last_event_id: DOMRefCell<DOMString>,
    reconnection_time: Cell<u64>,
    generation_id: Cell<GenerationId>,

    ready_state: Cell<ReadyState>,
    with_credentials: bool,
}

enum ParserState {
    Field,
    Comment,
    Value,
    Eol
}

struct EventSourceContext {
    incomplete_utf8: Option<utf8::Incomplete>,

    event_source: Trusted<EventSource>,
    gen_id: GenerationId,
    action_sender: ipc::IpcSender<FetchResponseMsg>,

    parser_state: ParserState,
    field: String,
    value: String,
    origin: String,

    event_type: String,
    data: String,
    last_event_id: String,
}

impl EventSourceContext {
    fn announce_the_connection(&self) {
        let event_source = self.event_source.root();
        if self.gen_id != event_source.generation_id.get() {
            return;
        }
        let runnable = box AnnounceConnectionRunnable {
            event_source: self.event_source.clone()
        };
        let _ = event_source.global().networking_task_source().queue(runnable, &*event_source.global());
    }

    fn fail_the_connection(&self) {
        let event_source = self.event_source.root();
        if self.gen_id != event_source.generation_id.get() {
            return;
        }
        let runnable = box FailConnectionRunnable {
            event_source: self.event_source.clone()
        };
        let _ = event_source.global().networking_task_source().queue(runnable, &*event_source.global());
    }

    // https://html.spec.whatwg.org/multipage/#reestablish-the-connection
    fn reestablish_the_connection(&self) {
        let event_source = self.event_source.root();

        if self.gen_id != event_source.generation_id.get() {
            return;
        }

        // Step 1
        let runnable = box ReestablishConnectionRunnable {
            event_source: self.event_source.clone(),
            action_sender: self.action_sender.clone()
        };
        let _ = event_source.global().networking_task_source().queue(runnable, &*event_source.global());
    }

    // https://html.spec.whatwg.org/multipage/#processField
    fn process_field(&mut self) {
        match &*self.field {
            "event" => mem::swap(&mut self.event_type, &mut self.value),
            "data" => {
                self.data.push_str(&self.value);
                self.data.push('\n');
            }
            "id" => mem::swap(&mut self.last_event_id, &mut self.value),
            "retry" => if let Ok(time) = u64::from_str(&self.value) {
                self.event_source.root().reconnection_time.set(time);
            },
            _ => ()
        }

        self.field.clear();
        self.value.clear();
    }

    // https://html.spec.whatwg.org/multipage/#dispatchMessage
    #[allow(unsafe_code)]
    fn dispatch_event(&mut self) {
        let event_source = self.event_source.root();
        // Step 1
        *event_source.last_event_id.borrow_mut() = DOMString::from(self.last_event_id.clone());
        // Step 2
        if self.data.is_empty() {
            self.data.clear();
            self.event_type.clear();
            return;
        }
        // Step 3
        if let Some(last) = self.data.pop() {
            if last != '\n' {
                self.data.push(last);
            }
        }
        // Step 6
        let type_ = if !self.event_type.is_empty() {
            Atom::from(self.event_type.clone())
        } else {
            atom!("message")
        };
        // Steps 4-5
        let event = {
            let _ac = JSAutoCompartment::new(event_source.global().get_cx(),
                                             event_source.reflector().get_jsobject().get());
            rooted!(in(event_source.global().get_cx()) let mut data = UndefinedValue());
            unsafe { self.data.to_jsval(event_source.global().get_cx(), data.handle_mut()) };
            MessageEvent::new(&*event_source.global(), type_, false, false, data.handle(),
                              DOMString::from(self.origin.clone()),
                              event_source.last_event_id.borrow().clone())
        };
        // Step 7
        self.event_type.clear();
        self.data.clear();
        // Step 8
        let runnable = box DispatchEventRunnable {
            event_source: self.event_source.clone(),
            event: Trusted::new(&event)
        };
        let _ = event_source.global().networking_task_source().queue(runnable, &*event_source.global());
    }

    // https://html.spec.whatwg.org/multipage/#event-stream-interpretation
    fn parse(&mut self, stream: Chars) {
        let mut stream = stream.peekable();

        while let Some(ch) = stream.next() {
            match (ch, &self.parser_state) {
                (':', &ParserState::Eol) => self.parser_state = ParserState::Comment,
                (':', &ParserState::Field) => {
                    self.parser_state = ParserState::Value;
                    if let Some(&' ') = stream.peek() {
                        stream.next();
                    }
                }

                ('\n', &ParserState::Value) => {
                    self.parser_state = ParserState::Eol;
                    self.process_field();
                }
                ('\r', &ParserState::Value) => {
                    if let Some(&'\n') = stream.peek() {
                        continue;
                    }
                    self.parser_state = ParserState::Eol;
                    self.process_field();
                }

                ('\n', &ParserState::Field) => {
                    self.parser_state = ParserState::Eol;
                    self.process_field();
                }
                ('\r', &ParserState::Field) => {
                    if let Some(&'\n') = stream.peek() {
                        continue;
                    }
                    self.parser_state = ParserState::Eol;
                    self.process_field();
                }

                ('\n', &ParserState::Eol) => self.dispatch_event(),
                ('\r', &ParserState::Eol) => {
                    if let Some(&'\n') = stream.peek() {
                        continue;
                    }
                    self.dispatch_event();
                }

                ('\n', &ParserState::Comment) => self.parser_state = ParserState::Eol,
                ('\r', &ParserState::Comment) => {
                    if let Some(&'\n') = stream.peek() {
                        continue;
                    }
                    self.parser_state = ParserState::Eol;
                }

                (_, &ParserState::Field) => self.field.push(ch),
                (_, &ParserState::Value) => self.value.push(ch),
                (_, &ParserState::Eol) => {
                    self.parser_state = ParserState::Field;
                    self.field.push(ch);
                }
                (_, &ParserState::Comment) => (),
            }
        }
    }
}

impl FetchResponseListener for EventSourceContext {
    fn process_request_body(&mut self) {
        // TODO
    }

    fn process_request_eof(&mut self) {
        // TODO
    }

    fn process_response(&mut self, metadata: Result<FetchMetadata, NetworkError>) {
        match metadata {
            Ok(fm) => {
                let meta = match fm {
                    FetchMetadata::Unfiltered(m) => m,
                    FetchMetadata::Filtered { unsafe_, .. } => unsafe_
                };
                match meta.content_type {
                    None => self.fail_the_connection(),
                    Some(ct) => match ct.into_inner().0 {
                        Mime(TopLevel::Text, SubLevel::EventStream, _) => {
                            self.origin = meta.final_url.origin().unicode_serialization();
                            self.announce_the_connection();
                        }
                        _ => self.fail_the_connection()
                    }
                }
            }
            Err(_) => {
                self.reestablish_the_connection();
            }
        }
    }

    fn process_response_chunk(&mut self, chunk: Vec<u8>) {
        let mut input = &*chunk;
        if let Some(mut incomplete) = self.incomplete_utf8.take() {
            match incomplete.try_complete(input) {
                None => return,
                Some((result, remaining_input)) => {
                    self.parse(result.unwrap_or("\u{FFFD}").chars());
                    input = remaining_input;
                }
            }
        }

        while !input.is_empty() {
            match utf8::decode(&input) {
                Ok(s) => {
                    self.parse(s.chars());
                    return
                }
                Err(utf8::DecodeError::Invalid { valid_prefix, remaining_input, .. }) => {
                    self.parse(valid_prefix.chars());
                    self.parse("\u{FFFD}".chars());
                    input = remaining_input;
                }
                Err(utf8::DecodeError::Incomplete { valid_prefix, incomplete_suffix }) => {
                    self.parse(valid_prefix.chars());
                    self.incomplete_utf8 = Some(incomplete_suffix);
                    return
                }
            }
        }
    }

    fn process_response_eof(&mut self, _response: Result<(), NetworkError>) {
        if let Some(_) = self.incomplete_utf8.take() {
            self.parse("\u{FFFD}".chars());
        }
        self.reestablish_the_connection();
    }
}

impl PreInvoke for EventSourceContext {
    fn should_invoke(&self) -> bool {
        self.event_source.root().generation_id.get() == self.gen_id
    }
}

impl EventSource {
    fn new_inherited(url: ServoUrl, with_credentials: bool) -> EventSource {
        EventSource {
            eventtarget: EventTarget::new_inherited(),
            url: url,
            request: DOMRefCell::new(None),
            last_event_id: DOMRefCell::new(DOMString::from("")),
            reconnection_time: Cell::new(DEFAULT_RECONNECTION_TIME),
            generation_id: Cell::new(GenerationId(0)),

            ready_state: Cell::new(ReadyState::Connecting),
            with_credentials: with_credentials,
        }
    }

    fn new(global: &GlobalScope, url: ServoUrl, with_credentials: bool) -> Root<EventSource> {
        reflect_dom_object(box EventSource::new_inherited(url, with_credentials),
                           global,
                           Wrap)
    }

    pub fn request(&self) -> RequestInit {
        self.request.borrow().clone().unwrap()
    }

    pub fn Constructor(global: &GlobalScope,
                       url: DOMString,
                       event_source_init: &EventSourceInit) -> Fallible<Root<EventSource>> {
        // TODO: Step 2 relevant settings object
        // Step 3
        let base_url = global.api_base_url();
        let url_record = match base_url.join(&*url) {
            Ok(u) => u,
            //  Step 4
            Err(_) => return Err(Error::Syntax)
        };
        // Step 1, 5
        let ev = EventSource::new(global, url_record.clone(), event_source_init.withCredentials);
        // Steps 6-7
        let cors_attribute_state = if event_source_init.withCredentials {
            CorsSettings::UseCredentials
        } else {
            CorsSettings::Anonymous
        };
        // Step 8
        // TODO: Step 9 set request's client settings
        let mut request = RequestInit {
            url: url_record,
            origin: global.get_url(),
            pipeline_id: Some(global.pipeline_id()),
            // https://html.spec.whatwg.org/multipage/#create-a-potential-cors-request
            use_url_credentials: true,
            mode: RequestMode::CorsMode,
            credentials_mode: if cors_attribute_state == CorsSettings::Anonymous {
                CredentialsMode::CredentialsSameOrigin
            } else {
                CredentialsMode::Include
            },
            ..RequestInit::default()
        };
        // Step 10
        request.headers.set(Accept(vec![qitem(mime!(Text / EventStream))]));
        // Step 11
        request.cache_mode = CacheMode::NoStore;
        // Step 12
        *ev.request.borrow_mut() = Some(request.clone());
        // Step 14
        let (action_sender, action_receiver) = ipc::channel().unwrap();
        let context = EventSourceContext {
            incomplete_utf8: None,

            event_source: Trusted::new(&ev),
            gen_id: ev.generation_id.get(),
            action_sender: action_sender.clone(),

            parser_state: ParserState::Eol,
            field: String::new(),
            value: String::new(),
            origin: String::new(),

            event_type: String::new(),
            data: String::new(),
            last_event_id: String::new(),
        };
        let listener = NetworkListener {
            context: Arc::new(Mutex::new(context)),
            task_source: global.networking_task_source(),
            wrapper: Some(global.get_runnable_wrapper())
        };
        ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
            listener.notify_fetch(message.to().unwrap());
        });
        global.core_resource_thread().send(CoreResourceMsg::Fetch(request, action_sender)).unwrap();
        // Step 13
        Ok(ev)
    }
}

impl EventSourceMethods for EventSource {
    // https://html.spec.whatwg.org/multipage/#handler-eventsource-onopen
    event_handler!(open, GetOnopen, SetOnopen);

    // https://html.spec.whatwg.org/multipage/#handler-eventsource-onmessage
    event_handler!(message, GetOnmessage, SetOnmessage);

    // https://html.spec.whatwg.org/multipage/#handler-eventsource-onerror
    event_handler!(error, GetOnerror, SetOnerror);

    // https://html.spec.whatwg.org/multipage/#dom-eventsource-url
    fn Url(&self) -> DOMString {
        DOMString::from(self.url.as_str())
    }

    // https://html.spec.whatwg.org/multipage/#dom-eventsource-withcredentials
    fn WithCredentials(&self) -> bool {
        self.with_credentials
    }

    // https://html.spec.whatwg.org/multipage/#dom-eventsource-readystate
    fn ReadyState(&self) -> u16 {
        self.ready_state.get() as u16
    }

    // https://html.spec.whatwg.org/multipage/#dom-eventsource-close
    fn Close(&self) {
        let GenerationId(prev_id) = self.generation_id.get();
        self.generation_id.set(GenerationId(prev_id + 1));
        self.ready_state.set(ReadyState::Closed);
    }
}

pub struct AnnounceConnectionRunnable {
    event_source: Trusted<EventSource>,
}

impl Runnable for AnnounceConnectionRunnable {
    fn name(&self) -> &'static str { "EventSource AnnounceConnectionRunnable" }

    // https://html.spec.whatwg.org/multipage/#announce-the-connection
    fn handler(self: Box<AnnounceConnectionRunnable>) {
        let event_source = self.event_source.root();
        if event_source.ready_state.get() != ReadyState::Closed {
            event_source.ready_state.set(ReadyState::Open);
            event_source.upcast::<EventTarget>().fire_event(atom!("open"));
        }
    }
}

pub struct FailConnectionRunnable {
    event_source: Trusted<EventSource>,
}

impl Runnable for FailConnectionRunnable {
    fn name(&self) -> &'static str { "EventSource FailConnectionRunnable" }

    // https://html.spec.whatwg.org/multipage/#fail-the-connection
    fn handler(self: Box<FailConnectionRunnable>) {
        let event_source = self.event_source.root();
        if event_source.ready_state.get() != ReadyState::Closed {
            event_source.ready_state.set(ReadyState::Closed);
            event_source.upcast::<EventTarget>().fire_event(atom!("error"));
        }
    }
}

pub struct ReestablishConnectionRunnable {
    event_source: Trusted<EventSource>,
    action_sender: ipc::IpcSender<FetchResponseMsg>,
}

impl Runnable for ReestablishConnectionRunnable {
    fn name(&self) -> &'static str { "EventSource ReestablishConnectionRunnable" }

    // https://html.spec.whatwg.org/multipage/#reestablish-the-connection
    fn handler(self: Box<ReestablishConnectionRunnable>) {
        let event_source = self.event_source.root();
        // Step 1.1
        if event_source.ready_state.get() == ReadyState::Closed {
            return;
        }
        // Step 1.2
        event_source.ready_state.set(ReadyState::Connecting);
        // Step 1.3
        event_source.upcast::<EventTarget>().fire_event(atom!("error"));
        // Step 2
        let duration = Length::new(event_source.reconnection_time.get());
        // TODO Step 3: Optionally wait some more
        // Steps 4-5
        let callback = OneshotTimerCallback::EventSourceTimeout(EventSourceTimeoutCallback {
            event_source: self.event_source.clone(),
            action_sender: self.action_sender.clone()
        });
        let _ = event_source.global().schedule_callback(callback, duration);
    }
}

#[derive(JSTraceable, HeapSizeOf)]
pub struct EventSourceTimeoutCallback {
    #[ignore_heap_size_of = "Because it is non-owning"]
    event_source: Trusted<EventSource>,
    #[ignore_heap_size_of = "Because it is non-owning"]
    action_sender: ipc::IpcSender<FetchResponseMsg>,
}

impl EventSourceTimeoutCallback {
    // https://html.spec.whatwg.org/multipage/#reestablish-the-connection
    pub fn invoke(self) {
        let event_source = self.event_source.root();
        let global = event_source.global();
        // Step 5.1
        if event_source.ready_state.get() != ReadyState::Connecting {
            return;
        }
        // Step 5.2
        let mut request = event_source.request();
        // Step 5.3
        if !event_source.last_event_id.borrow().is_empty() {
            request.headers.set(LastEventId(String::from(event_source.last_event_id.borrow().clone())));
        }
        // Step 5.4
        global.core_resource_thread().send(CoreResourceMsg::Fetch(request, self.action_sender)).unwrap();
    }
}

pub struct DispatchEventRunnable {
    event_source: Trusted<EventSource>,
    event: Trusted<MessageEvent>,
}

impl Runnable for DispatchEventRunnable {
    fn name(&self) -> &'static str { "EventSource DispatchEventRunnable" }

    // https://html.spec.whatwg.org/multipage/#dispatchMessage
    fn handler(self: Box<DispatchEventRunnable>) {
        let event_source = self.event_source.root();
        // Step 8
        if event_source.ready_state.get() != ReadyState::Closed {
            self.event.root().upcast::<Event>().fire(&event_source.upcast());
        }
    }
}
