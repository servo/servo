/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::mem;
use std::str::{Chars, FromStr};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use dom_struct::dom_struct;
use headers::ContentType;
use http::StatusCode;
use http::header::{self, HeaderName, HeaderValue};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::conversions::ToJSValConvertible;
use js::jsval::UndefinedValue;
use js::rust::HandleObject;
use mime::{self, Mime};
use net_traits::request::{CacheMode, CorsSettings, Destination, RequestBuilder, RequestId};
use net_traits::{
    CoreResourceMsg, FetchChannels, FetchMetadata, FetchResponseListener, FetchResponseMsg,
    FilteredMetadata, NetworkError, ResourceFetchTiming, ResourceTimingType,
};
use servo_url::ServoUrl;
use stylo_atoms::Atom;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::EventSourceBinding::{
    EventSourceInit, EventSourceMethods,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageevent::MessageEvent;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::fetch::{FetchCanceller, create_a_potential_cors_request};
use crate::network_listener::{self, NetworkListener, PreInvoke, ResourceTimingListener};
use crate::realms::enter_realm;
use crate::script_runtime::CanGc;
use crate::timers::OneshotTimerCallback;

const DEFAULT_RECONNECTION_TIME: Duration = Duration::from_millis(5000);

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
struct GenerationId(u32);

#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
/// <https://html.spec.whatwg.org/multipage/#dom-eventsource-readystate>
enum ReadyState {
    Connecting = 0,
    Open = 1,
    Closed = 2,
}

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableEventSource {
    canceller: DomRefCell<FetchCanceller>,
}

impl DroppableEventSource {
    pub(crate) fn new(canceller: DomRefCell<FetchCanceller>) -> Self {
        DroppableEventSource { canceller }
    }

    pub(crate) fn cancel(&self) {
        self.canceller.borrow_mut().cancel();
    }

    pub(crate) fn set_canceller(&self, data: FetchCanceller) {
        *self.canceller.borrow_mut() = data;
    }
}

// https://html.spec.whatwg.org/multipage/#garbage-collection-2
impl Drop for DroppableEventSource {
    fn drop(&mut self) {
        // If an EventSource object is garbage collected while its connection is still open,
        // the user agent must abort any instance of the fetch algorithm opened by this EventSource.
        self.cancel();
    }
}

#[dom_struct]
pub(crate) struct EventSource {
    eventtarget: EventTarget,
    #[no_trace]
    url: ServoUrl,
    #[no_trace]
    request: DomRefCell<Option<RequestBuilder>>,
    last_event_id: DomRefCell<DOMString>,
    reconnection_time: Cell<Duration>,
    generation_id: Cell<GenerationId>,

    ready_state: Cell<ReadyState>,
    with_credentials: bool,
    droppable: DroppableEventSource,
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
        global.task_manager().remote_event_task_source().queue(
            task!(announce_the_event_source_connection: move || {
                let event_source = event_source.root();
                if event_source.ready_state.get() != ReadyState::Closed {
                    event_source.ready_state.set(ReadyState::Open);
                    event_source.upcast::<EventTarget>().fire_event(atom!("open"), CanGc::note());
                }
            }),
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
        global.task_manager().remote_event_task_source().queue(
            task!(reestablish_the_event_source_onnection: move || {
                let event_source = trusted_event_source.root();

                // Step 1.1.
                if event_source.ready_state.get() == ReadyState::Closed {
                    return;
                }

                // Step 1.2.
                event_source.ready_state.set(ReadyState::Connecting);

                // Step 1.3.
                event_source.upcast::<EventTarget>().fire_event(atom!("error"), CanGc::note());

                // Step 2.
                let duration = event_source.reconnection_time.get();

                // Step 3.
                // TODO: Optionally wait some more.

                // Steps 4-5.
                let callback = OneshotTimerCallback::EventSourceTimeout(
                    EventSourceTimeoutCallback {
                        event_source: trusted_event_source,
                        action_sender,
                    }
                );
                event_source.global().schedule_callback(callback, duration);
            }),
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
            "id" if !self.value.contains('\0') => {
                mem::swap(&mut self.last_event_id, &mut self.value);
            },
            "retry" => {
                if let Ok(time) = u64::from_str(&self.value) {
                    self.event_source
                        .root()
                        .reconnection_time
                        .set(Duration::from_millis(time));
                }
            },
            _ => (),
        }

        self.field.clear();
        self.value.clear();
    }

    // https://html.spec.whatwg.org/multipage/#dispatchMessage
    #[allow(unsafe_code)]
    fn dispatch_event(&mut self, can_gc: CanGc) {
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
                can_gc,
            )
        };
        // Step 7
        self.event_type.clear();
        self.data.clear();

        // Step 8.
        let global = event_source.global();
        let event_source = self.event_source.clone();
        let event = Trusted::new(&*event);
        global.task_manager().remote_event_task_source().queue(
            task!(dispatch_the_event_source_event: move || {
                let event_source = event_source.root();
                if event_source.ready_state.get() != ReadyState::Closed {
                    event.root().upcast::<Event>().fire(event_source.upcast(), CanGc::note());
                }
            }),
        );
    }

    // https://html.spec.whatwg.org/multipage/#event-stream-interpretation
    fn parse(&mut self, stream: Chars, can_gc: CanGc) {
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

                ('\n', &ParserState::Eol) => self.dispatch_event(can_gc),
                ('\r', &ParserState::Eol) => {
                    if let Some(&'\n') = stream.peek() {
                        continue;
                    }
                    self.dispatch_event(can_gc);
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
    fn process_request_body(&mut self, _: RequestId) {
        // TODO
    }

    fn process_request_eof(&mut self, _: RequestId) {
        // TODO
    }

    fn process_response(&mut self, _: RequestId, metadata: Result<FetchMetadata, NetworkError>) {
        match metadata {
            Ok(fm) => {
                let meta = match fm {
                    FetchMetadata::Unfiltered(m) => m,
                    FetchMetadata::Filtered { unsafe_, filtered } => match filtered {
                        FilteredMetadata::Opaque | FilteredMetadata::OpaqueRedirect(_) => {
                            return self.fail_the_connection();
                        },
                        _ => unsafe_,
                    },
                };
                // Step 15.3 if res's status is not 200, or if res's `Content-Type` is not
                // `text/event-stream`, then fail the connection.
                if meta.status.code() != StatusCode::OK {
                    return self.fail_the_connection();
                }
                let mime = match meta.content_type {
                    None => return self.fail_the_connection(),
                    Some(ct) => <ContentType as Into<Mime>>::into(ct.into_inner()),
                };
                if (mime.type_(), mime.subtype()) != (mime::TEXT, mime::EVENT_STREAM) {
                    return self.fail_the_connection();
                }
                self.origin = meta.final_url.origin().ascii_serialization();
                // Step 15.4 announce the connection and interpret res's body line by line.
                self.announce_the_connection();
            },
            Err(_) => {
                // Step 15.2 if res is a network error, then reestablish the connection, unless
                // the user agent knows that to be futile, in which case the user agent may
                // fail the connection.

                // WPT tests consider a non-http(s) scheme to be futile.
                match self.event_source.root().url.scheme() {
                    "http" | "https" => self.reestablish_the_connection(),
                    _ => self.fail_the_connection(),
                }
            },
        }
    }

    fn process_response_chunk(&mut self, _: RequestId, chunk: Vec<u8>) {
        let mut input = &*chunk;
        if let Some(mut incomplete) = self.incomplete_utf8.take() {
            match incomplete.try_complete(input) {
                None => return,
                Some((result, remaining_input)) => {
                    self.parse(result.unwrap_or("\u{FFFD}").chars(), CanGc::note());
                    input = remaining_input;
                },
            }
        }

        while !input.is_empty() {
            match utf8::decode(input) {
                Ok(s) => {
                    self.parse(s.chars(), CanGc::note());
                    return;
                },
                Err(utf8::DecodeError::Invalid {
                    valid_prefix,
                    remaining_input,
                    ..
                }) => {
                    self.parse(valid_prefix.chars(), CanGc::note());
                    self.parse("\u{FFFD}".chars(), CanGc::note());
                    input = remaining_input;
                },
                Err(utf8::DecodeError::Incomplete {
                    valid_prefix,
                    incomplete_suffix,
                }) => {
                    self.parse(valid_prefix.chars(), CanGc::note());
                    self.incomplete_utf8 = Some(incomplete_suffix);
                    return;
                },
            }
        }
    }

    fn process_response_eof(
        &mut self,
        _: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        if self.incomplete_utf8.take().is_some() {
            self.parse("\u{FFFD}".chars(), CanGc::note());
        }
        if response.is_ok() {
            self.reestablish_the_connection();
        }
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        network_listener::submit_timing(self, CanGc::note())
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(violations, None);
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
            droppable: DroppableEventSource::new(DomRefCell::new(Default::default())),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        url: ServoUrl,
        with_credentials: bool,
        can_gc: CanGc,
    ) -> DomRoot<EventSource> {
        reflect_dom_object_with_proto(
            Box::new(EventSource::new_inherited(url, with_credentials)),
            global,
            proto,
            can_gc,
        )
    }

    // https://html.spec.whatwg.org/multipage/#sse-processing-model:fail-the-connection-3
    pub(crate) fn cancel(&self) {
        self.droppable.cancel();
        self.fail_the_connection();
    }

    /// <https://html.spec.whatwg.org/multipage/#fail-the-connection>
    pub(crate) fn fail_the_connection(&self) {
        let global = self.global();
        let event_source = Trusted::new(self);
        global.task_manager().remote_event_task_source().queue(
            task!(fail_the_event_source_connection: move || {
                let event_source = event_source.root();
                if event_source.ready_state.get() != ReadyState::Closed {
                    event_source.ready_state.set(ReadyState::Closed);
                    event_source.upcast::<EventTarget>().fire_event(atom!("error"), CanGc::note());
                }
            }),
        );
    }

    pub(crate) fn request(&self) -> RequestBuilder {
        self.request.borrow().clone().unwrap()
    }

    pub(crate) fn url(&self) -> &ServoUrl {
        &self.url
    }
}

impl EventSourceMethods<crate::DomTypeHolder> for EventSource {
    // https://html.spec.whatwg.org/multipage/#dom-eventsource
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        url: DOMString,
        event_source_init: &EventSourceInit,
    ) -> Fallible<DomRoot<EventSource>> {
        // TODO: Step 2 relevant settings object
        // Step 3 Let urlRecord be the result of encoding-parsing a URL given url,
        // relative to settings.
        let base_url = global.api_base_url();
        let url_record = match base_url.join(&url) {
            Ok(u) => u,
            // Step 4 If urlRecord is failure, then throw a "SyntaxError" DOMException.
            Err(_) => return Err(Error::Syntax),
        };
        // Step 1 Let ev be a new EventSource object.
        let ev = EventSource::new(
            global,
            proto,
            // Step 5 Set ev's url to urlRecord.
            url_record.clone(),
            event_source_init.withCredentials,
            can_gc,
        );
        global.track_event_source(&ev);
        let cors_attribute_state = if event_source_init.withCredentials {
            // Step 7 If the value of eventSourceInitDict's withCredentials member is true,
            // then set corsAttributeState to Use Credentials and set ev's withCredentials
            // attribute to true.
            CorsSettings::UseCredentials
        } else {
            // Step 6 Let corsAttributeState be Anonymous.
            CorsSettings::Anonymous
        };
        // Step 8 Let request be the result of creating a potential-CORS request
        // given urlRecord, the empty string, and corsAttributeState.
        // TODO: Step 9 set request's client settings
        let mut request = create_a_potential_cors_request(
            global.webview_id(),
            url_record,
            Destination::None,
            Some(cors_attribute_state),
            Some(true),
            global.get_referrer(),
            global.insecure_requests_policy(),
            global.has_trustworthy_ancestor_or_current_origin(),
            global.policy_container(),
        )
        .origin(global.origin().immutable().clone())
        .pipeline_id(Some(global.pipeline_id()));

        // Step 10 User agents may set (`Accept`, `text/event-stream`) in request's header list.
        // TODO(eijebong): Replace once typed headers allow it
        request.headers.insert(
            header::ACCEPT,
            HeaderValue::from_static("text/event-stream"),
        );
        // Step 11 Set request's cache mode to "no-store".
        request.cache_mode = CacheMode::NoStore;
        // Step 13 Set ev's request to request.
        *ev.request.borrow_mut() = Some(request.clone());
        // Step 14 Let processEventSourceEndOfBody given response res be the following step:
        // if res is not a network error, then reestablish the connection.
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
        let mut listener = NetworkListener {
            context: Arc::new(Mutex::new(context)),
            task_source: global.task_manager().networking_task_source().into(),
        };
        ROUTER.add_typed_route(
            action_receiver,
            Box::new(move |message| {
                listener.notify_fetch(message.unwrap());
            }),
        );
        ev.droppable.set_canceller(FetchCanceller::new(request.id));
        global
            .core_resource_thread()
            .send(CoreResourceMsg::Fetch(
                request,
                FetchChannels::ResponseMsg(action_sender),
            ))
            .unwrap();
        // Step 16 Return ev.
        Ok(ev)
    }

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
        self.droppable.cancel();
        self.ready_state.set(ReadyState::Closed);
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct EventSourceTimeoutCallback {
    #[ignore_malloc_size_of = "Because it is non-owning"]
    event_source: Trusted<EventSource>,
    #[ignore_malloc_size_of = "Because it is non-owning"]
    #[no_trace]
    action_sender: ipc::IpcSender<FetchResponseMsg>,
}

impl EventSourceTimeoutCallback {
    // https://html.spec.whatwg.org/multipage/#reestablish-the-connection
    pub(crate) fn invoke(self) {
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
                FetchChannels::ResponseMsg(self.action_sender),
            ))
            .unwrap();
    }
}
