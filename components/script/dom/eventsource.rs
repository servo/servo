/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::mem;
use std::str::{Chars, FromStr};
use std::sync::{Arc, Mutex};

use dom_struct::dom_struct;
use euclid::Length;
use headers::ContentType;
use http::header::{self, HeaderName, HeaderValue};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::conversions::ToJSValConvertible;
use js::jsval::UndefinedValue;
use js::rust::HandleObject;
use mime::{self, Mime};
use net_traits::request::{CacheMode, CorsSettings, Destination, RequestBuilder};
use net_traits::{
    CoreResourceMsg, FetchChannels, FetchMetadata, FetchResponseListener, FetchResponseMsg,
    FilteredMetadata, NetworkError, ResourceFetchTiming, ResourceTimingType,
};
use servo_atoms::Atom;
use servo_url::ServoUrl;
use utf8;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::EventSourceBinding::{
    EventSourceInit, EventSourceMethods,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageevent::MessageEvent;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::fetch::{create_a_potential_cors_request, FetchCanceller};
use crate::network_listener::{self, NetworkListener, PreInvoke, ResourceTimingListener};
use crate::realms::enter_realm;
use crate::task_source::{TaskSource, TaskSourceName};
use crate::timers::OneshotTimerCallback;

const DEFAULT_RECONNECTION_TIME: u64 = 5000;

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
struct GenerationId(u32);

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
/// <https://html.spec.whatwg.org/multipage/#dom-eventsource-readystate>
enum ReadyState {
    Connecting = 0,
    Open = 1,
    Closed = 2,
}

#[dom_struct]
pub struct EventSource {
    eventtarget: EventTarget,
    #[no_trace]
    url: ServoUrl,
    #[no_trace]
    request: DomRefCell<Option<RequestBuilder>>,
    last_event_id: DomRefCell<DOMString>,
    reconnection_time: Cell<u64>,
    generation_id: Cell<GenerationId>,

    ready_state: Cell<ReadyState>,
    with_credentials: bool,
    canceller: DomRefCell<FetchCanceller>,
}

enum ParserState {
    Field,
    Comment,
    Value,
    Eol,
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

    resource_timing: ResourceFetchTiming,
}

impl EventSourceContext {
    /// <https://html.spec.whatwg.org/multipage/#announce-the-connection>
    fn announce_the_connection(&self) {
        let event_source = self.event_source.root();
        if self.gen_id != event_source.generation_id.get() {
            return;
        }
        let global = event_source.global();
        let event_source = self.event_source.clone();
        // FIXME(nox): Why are errors silenced here?
        let _ = global.remote_event_task_source().queue(
            task!(announce_the_event_source_connection: move || {
                let event_source = event_source.root();
                if event_source.ready_state.get() != ReadyState::Closed {
                    event_source.ready_state.set(ReadyState::Open);
                    event_source.upcast::<EventTarget>().fire_event(atom!("open"));
                }
            }),
            &global,
        );
    }

    /// <https://html.spec.whatwg.org/multipage/#fail-the-connection>
    fn fail_the_connection(&self) {
        let event_source = self.event_source.root();
        if self.gen_id != event_source.generation_id.get() {
            return;
        }
        event_source.fail_the_connection();
    }

    // https://html.spec.whatwg.org/multipage/#reestablish-the-connection
    fn reestablish_the_connection(&self) {
        let event_source = self.event_source.root();

        if self.gen_id != event_source.generation_id.get() {
            return;
        }

        let trusted_event_source = self.event_source.clone();
        let action_sender = self.action_sender.clone();
        let global = event_source.global();
        // FIXME(nox): Why are errors silenced here?
        let _ = global.remote_event_task_source().queue(
            task!(reestablish_the_event_source_onnection: move || {
                let event_source = trusted_event_source.root();

                // Step 1.1.
                if event_source.ready_state.get() == ReadyState::Closed {
                    return;
                }

                // Step 1.2.
                event_source.ready_state.set(ReadyState::Connecting);

                // Step 1.3.
                event_source.upcast::<EventTarget>().fire_event(atom!("error"));

                // Step 2.
                let duration = Length::new(event_source.reconnection_time.get());

                // Step 3.
                // TODO: Optionally wait some more.

                // Steps 4-5.
                let callback = OneshotTimerCallback::EventSourceTimeout(
                    EventSourceTimeoutCallback {
                        event_source: trusted_event_source,
                        action_sender,
                    }
                );
                // FIXME(nox): Why are errors silenced here?
                let _ = event_source.global().schedule_callback(callback, duration);
            }),
            &global,
        );
    }

    // https://html.spec.whatwg.org/multipage/#processField
    fn process_field(&mut self) {
        match &*self.field {
            "event" => mem::swap(&mut self.event_type, &mut self.value),
            "data" => {
                self.data.push_str(&self.value);
                self.data.push('\n');
            },
            "id" => mem::swap(&mut self.last_event_id, &mut self.value),
            "retry" => {
                if let Ok(time) = u64::from_str(&self.value) {
                    self.event_source.root().reconnection_time.set(time);
                }
            },
            _ => (),
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
            let _ac = enter_realm(&*event_source);
            rooted!(in(*GlobalScope::get_cx()) let mut data = UndefinedValue());
            unsafe {
                self.data
                    .to_jsval(*GlobalScope::get_cx(), data.handle_mut())
            };
            MessageEvent::new(
                &event_source.global(),
                type_,
                false,
                false,
                data.handle(),
                DOMString::from(self.origin.clone()),
                None,
                event_source.last_event_id.borrow().clone(),
                Vec::with_capacity(0),
            )
        };
        // Step 7
        self.event_type.clear();
        self.data.clear();

        // Step 8.
        let global = event_source.global();
        let event_source = self.event_source.clone();
        let event = Trusted::new(&*event);
        // FIXME(nox): Why are errors silenced here?
        let _ = global.remote_event_task_source().queue(
            task!(dispatch_the_event_source_event: move || {
                let event_source = event_source.root();
                if event_source.ready_state.get() != ReadyState::Closed {
                    event.root().upcast::<Event>().fire(event_source.upcast());
                }
            }),
            &global,
        );
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
                },

                ('\n', &ParserState::Value) => {
                    self.parser_state = ParserState::Eol;
                    self.process_field();
                },
                ('\r', &ParserState::Value) => {
                    if let Some(&'\n') = stream.peek() {
                        continue;
                    }
                    self.parser_state = ParserState::Eol;
                    self.process_field();
                },

                ('\n', &ParserState::Field) => {
                    self.parser_state = ParserState::Eol;
                    self.process_field();
                },
                ('\r', &ParserState::Field) => {
                    if let Some(&'\n') = stream.peek() {
                        continue;
                    }
                    self.parser_state = ParserState::Eol;
                    self.process_field();
                },

                ('\n', &ParserState::Eol) => self.dispatch_event(),
                ('\r', &ParserState::Eol) => {
                    if let Some(&'\n') = stream.peek() {
                        continue;
                    }
                    self.dispatch_event();
                },

                ('\n', &ParserState::Comment) => self.parser_state = ParserState::Eol,
                ('\r', &ParserState::Comment) => {
                    if let Some(&'\n') = stream.peek() {
                        continue;
                    }
                    self.parser_state = ParserState::Eol;
                },

                (_, &ParserState::Field) => self.field.push(ch),
                (_, &ParserState::Value) => self.value.push(ch),
                (_, &ParserState::Eol) => {
                    self.parser_state = ParserState::Field;
                    self.field.push(ch);
                },
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
                    FetchMetadata::Filtered { unsafe_, filtered } => match filtered {
                        FilteredMetadata::Opaque | FilteredMetadata::OpaqueRedirect(_) => {
                            return self.fail_the_connection()
                        },
                        _ => unsafe_,
                    },
                };
                let mime = match meta.content_type {
                    None => return self.fail_the_connection(),
                    Some(ct) => <ContentType as Into<Mime>>::into(ct.into_inner()),
                };
                if (mime.type_(), mime.subtype()) != (mime::TEXT, mime::EVENT_STREAM) {
                    return self.fail_the_connection();
                }
                self.origin = meta.final_url.origin().ascii_serialization();
                self.announce_the_connection();
            },
            Err(_) => {
                // The spec advises failing here if reconnecting would be
                // "futile", with no more specific advice; WPT tests
                // consider a non-http(s) scheme to be futile.
                match self.event_source.root().url.scheme() {
                    "http" | "https" => self.reestablish_the_connection(),
                    _ => self.fail_the_connection(),
                }
            },
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
                },
            }
        }

        while !input.is_empty() {
            match utf8::decode(input) {
                Ok(s) => {
                    self.parse(s.chars());
                    return;
                },
                Err(utf8::DecodeError::Invalid {
                    valid_prefix,
                    remaining_input,
                    ..
                }) => {
                    self.parse(valid_prefix.chars());
                    self.parse("\u{FFFD}".chars());
                    input = remaining_input;
                },
                Err(utf8::DecodeError::Incomplete {
                    valid_prefix,
                    incomplete_suffix,
                }) => {
                    self.parse(valid_prefix.chars());
                    self.incomplete_utf8 = Some(incomplete_suffix);
                    return;
                },
            }
        }
    }

    fn process_response_eof(&mut self, _response: Result<ResourceFetchTiming, NetworkError>) {
        if self.incomplete_utf8.take().is_some() {
            self.parse("\u{FFFD}".chars());
        }
        self.reestablish_the_connection();
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        network_listener::submit_timing(self)
    }
}

impl ResourceTimingListener for EventSourceContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (InitiatorType::Other, self.event_source.root().url().clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.event_source.root().global()
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
            url,
            request: DomRefCell::new(None),
            last_event_id: DomRefCell::new(DOMString::from("")),
            reconnection_time: Cell::new(DEFAULT_RECONNECTION_TIME),
            generation_id: Cell::new(GenerationId(0)),

            ready_state: Cell::new(ReadyState::Connecting),
            with_credentials,
            canceller: DomRefCell::new(Default::default()),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        url: ServoUrl,
        with_credentials: bool,
    ) -> DomRoot<EventSource> {
        reflect_dom_object_with_proto(
            Box::new(EventSource::new_inherited(url, with_credentials)),
            global,
            proto,
        )
    }

    // https://html.spec.whatwg.org/multipage/#sse-processing-model:fail-the-connection-3
    pub fn cancel(&self) {
        self.canceller.borrow_mut().cancel();
        self.fail_the_connection();
    }

    /// <https://html.spec.whatwg.org/multipage/#fail-the-connection>
    pub fn fail_the_connection(&self) {
        let global = self.global();
        let event_source = Trusted::new(self);
        // FIXME(nox): Why are errors silenced here?
        let _ = global.remote_event_task_source().queue(
            task!(fail_the_event_source_connection: move || {
                let event_source = event_source.root();
                if event_source.ready_state.get() != ReadyState::Closed {
                    event_source.ready_state.set(ReadyState::Closed);
                    event_source.upcast::<EventTarget>().fire_event(atom!("error"));
                }
            }),
            &global,
        );
    }

    pub fn request(&self) -> RequestBuilder {
        self.request.borrow().clone().unwrap()
    }

    pub fn url(&self) -> &ServoUrl {
        &self.url
    }

    // https://html.spec.whatwg.org/multipage/#dom-eventsource
    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        url: DOMString,
        event_source_init: &EventSourceInit,
    ) -> Fallible<DomRoot<EventSource>> {
        // TODO: Step 2 relevant settings object
        // Step 3
        let base_url = global.api_base_url();
        let url_record = match base_url.join(&url) {
            Ok(u) => u,
            //  Step 4
            Err(_) => return Err(Error::Syntax),
        };
        // Step 1, 5
        let ev = EventSource::new(
            global,
            proto,
            url_record.clone(),
            event_source_init.withCredentials,
        );
        global.track_event_source(&ev);
        // Steps 6-7
        let cors_attribute_state = if event_source_init.withCredentials {
            CorsSettings::UseCredentials
        } else {
            CorsSettings::Anonymous
        };
        // Step 8
        // TODO: Step 9 set request's client settings
        let mut request = create_a_potential_cors_request(
            url_record,
            Destination::None,
            Some(cors_attribute_state),
            Some(true),
            global.get_referrer(),
        )
        .origin(global.origin().immutable().clone())
        .pipeline_id(Some(global.pipeline_id()));

        // Step 10
        // TODO(eijebong): Replace once typed headers allow it
        request.headers.insert(
            header::ACCEPT,
            HeaderValue::from_static("text/event-stream"),
        );
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
            resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
        };
        let listener = NetworkListener {
            context: Arc::new(Mutex::new(context)),
            task_source: global.networking_task_source(),
            canceller: Some(global.task_canceller(TaskSourceName::Networking)),
        };
        ROUTER.add_route(
            action_receiver.to_opaque(),
            Box::new(move |message| {
                listener.notify_fetch(message.to().unwrap());
            }),
        );
        let cancel_receiver = ev.canceller.borrow_mut().initialize();
        global
            .core_resource_thread()
            .send(CoreResourceMsg::Fetch(
                request,
                FetchChannels::ResponseMsg(action_sender, Some(cancel_receiver)),
            ))
            .unwrap();
        // Step 13
        Ok(ev)
    }
}

// https://html.spec.whatwg.org/multipage/#garbage-collection-2
impl Drop for EventSource {
    fn drop(&mut self) {
        // If an EventSource object is garbage collected while its connection is still open,
        // the user agent must abort any instance of the fetch algorithm opened by this EventSource.
        self.canceller.borrow_mut().cancel();
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
        self.canceller.borrow_mut().cancel();
        self.ready_state.set(ReadyState::Closed);
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct EventSourceTimeoutCallback {
    #[ignore_malloc_size_of = "Because it is non-owning"]
    event_source: Trusted<EventSource>,
    #[ignore_malloc_size_of = "Because it is non-owning"]
    #[no_trace]
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
            //TODO(eijebong): Change this once typed header support custom values
            request.headers.insert(
                HeaderName::from_static("last-event-id"),
                HeaderValue::from_str(&String::from(event_source.last_event_id.borrow().clone()))
                    .unwrap(),
            );
        }
        // Step 5.4
        global
            .core_resource_thread()
            .send(CoreResourceMsg::Fetch(
                request,
                FetchChannels::ResponseMsg(self.action_sender, None),
            ))
            .unwrap();
    }
}
