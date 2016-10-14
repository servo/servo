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
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use hyper::header::{Accept, qitem};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use mime::{Mime, TopLevel, SubLevel};
use net_traits::{CoreResourceMsg, FetchMetadata, FetchResponseListener, NetworkError};
use net_traits::request::{CacheMode, CorsSettings, CredentialsMode};
use net_traits::request::{RequestInit, RequestMode};
use network_listener::{NetworkListener, PreInvoke};
use script_thread::{Runnable, RunnableWrapper};
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use task_source::TaskSource;
use task_source::networking::NetworkingTaskSource;
use url::Url;

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
    url: DOMRefCell<Option<Url>>,
    request: DOMRefCell<Option<RequestInit>>,
    ready_state: Cell<ReadyState>,
    with_credentials: bool,
    last_event_id: DOMRefCell<DOMString>
}

struct EventSourceContext {
    event_source: Trusted<EventSource>,
    networking_task_source: NetworkingTaskSource,
    wrapper: RunnableWrapper
}

impl EventSourceContext {
    fn announce_the_connection(&self) {
        let runnable = box AnnounceConnectionRunnable {
            event_source: self.event_source.clone()
        };
        let _ = self.networking_task_source.queue_with_wrapper(runnable, &self.wrapper);
    }

    fn fail_the_connection(&self) {
        let runnable = box FailConnectionRunnable {
            event_source: self.event_source.clone()
        };
        let _ = self.networking_task_source.queue_with_wrapper(runnable, &self.wrapper);
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
                        Mime(TopLevel::Text, SubLevel::EventStream, _) =>
                            self.announce_the_connection(),
                        _ => self.fail_the_connection()
                    }
                }
            }
            Err(_) => {
                // FIXME: Fail the connection for now, but it should really attempt to re-establish
                self.fail_the_connection();
            }
        }
    }

    fn process_response_chunk(&mut self, mut _chunk: Vec<u8>) {
        // TODO
    }

    fn process_response_eof(&mut self, _response: Result<(), NetworkError>) {
        // TODO
    }
}

impl PreInvoke for EventSourceContext {}

impl EventSource {
    fn new_inherited(with_credentials: bool) -> EventSource {
        EventSource {
            eventtarget: EventTarget::new_inherited(),
            url: DOMRefCell::new(None),
            request: DOMRefCell::new(None),
            ready_state: Cell::new(ReadyState::Connecting),
            with_credentials: with_credentials,
            last_event_id: DOMRefCell::new(DOMString::from(""))
        }
    }

    fn new(global: &GlobalScope, with_credentials: bool) -> Root<EventSource> {
        reflect_dom_object(box EventSource::new_inherited(with_credentials),
                           global,
                           Wrap)
    }

    pub fn Constructor(global: &GlobalScope,
                       url: DOMString,
                       event_source_init: &EventSourceInit) -> Fallible<Root<EventSource>> {
        // Step 1
        let ev = EventSource::new(global, event_source_init.withCredentials);
        // TODO: Step 2 relevant settings object
        // Step 3
        let base_url = global.api_base_url();
        let url_record = match base_url.join(&*url) {
            Ok(u) => u,
            //  Step 4
            Err(_) => return Err(Error::Syntax)
        };
        // Step 5
        *ev.url.borrow_mut() = Some(url_record.clone());
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
        let context = EventSourceContext {
            event_source: Trusted::new(&ev),
            networking_task_source: global.networking_task_source(),
            wrapper: global.get_runnable_wrapper()
        };
        let listener = NetworkListener {
            context: Arc::new(Mutex::new(context)),
            task_source: global.networking_task_source(),
            wrapper: Some(global.get_runnable_wrapper())
        };
        let (action_sender, action_receiver) = ipc::channel().unwrap();
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
        DOMString::from(self.url.borrow().clone().map_or("".to_owned(), Url::into_string))
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
        self.ready_state.set(ReadyState::Closed);
        // TODO: Terminate ongoing fetch
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
