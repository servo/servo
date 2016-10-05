/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventSourceBinding::{EventSourceInit, EventSourceMethods, Wrap};
use dom::bindings::error::{Error, Fallible};
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use hyper::header::{Accept, qitem};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::{CoreResourceMsg, FetchMetadata, FetchResponseListener, NetworkError};
use net_traits::request::{CacheMode, CORSSettings, CredentialsMode};
use net_traits::request::{RequestInit, RequestMode};
use network_listener::{NetworkListener, PreInvoke};
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use url::Url;

#[derive(JSTraceable, PartialEq, Copy, Clone, Debug, HeapSizeOf)]
enum EventSourceReadyState {
    Connecting = 0,
    #[allow(dead_code)]
    Open = 1,
    Closed = 2
}

#[dom_struct]
pub struct EventSource {
    eventtarget: EventTarget,
    url: DOMRefCell<Option<Url>>,
    request: DOMRefCell<Option<RequestInit>>,
    ready_state: Cell<EventSourceReadyState>,
    with_credentials: bool,
    last_event_id: DOMRefCell<DOMString>
}

struct EventSourceContext;

impl FetchResponseListener for EventSourceContext {
    fn process_request_body(&mut self) {
        // TODO
    }

    fn process_request_eof(&mut self) {
        // TODO
    }

    fn process_response(&mut self, _metadata: Result<FetchMetadata, NetworkError>) {
        // TODO
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
            ready_state: Cell::new(EventSourceReadyState::Connecting),
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
            CORSSettings::UseCredentials
        } else {
            CORSSettings::Anonymous
        };
        // Step 8
        // TODO: Step 9 set request's client settings
        let mut request = RequestInit {
            url: url_record,
            origin: global.get_url(),
            pipeline_id: Some(global.pipeline_id()),
            // https://html.spec.whatwg.org/multipage/#create-a-potential-cors-request
            use_url_credentials: true,
            mode: RequestMode::CORSMode,
            credentials_mode: if cors_attribute_state == CORSSettings::Anonymous {
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
        let context = EventSourceContext;
        let listener = NetworkListener {
            context: Arc::new(Mutex::new(context)),
            script_chan: global.script_chan(),
            wrapper: None
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
        self.ready_state.set(EventSourceReadyState::Closed);
        // TODO: Terminate ongoing fetch
    }
}
