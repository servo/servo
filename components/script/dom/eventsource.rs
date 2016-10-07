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
use std::cell::Cell;
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
    url: Url,
    ready_state: Cell<EventSourceReadyState>,
    with_credentials: bool,
    last_event_id: DOMRefCell<DOMString>
}

impl EventSource {
    fn new_inherited(url: Url, with_credentials: bool) -> EventSource {
        EventSource {
            eventtarget: EventTarget::new_inherited(),
            url: url,
            ready_state: Cell::new(EventSourceReadyState::Connecting),
            with_credentials: with_credentials,
            last_event_id: DOMRefCell::new(DOMString::from(""))
        }
    }

    fn new(global: &GlobalScope, url: Url, with_credentials: bool) -> Root<EventSource> {
        reflect_dom_object(box EventSource::new_inherited(url, with_credentials),
                           global,
                           Wrap)
    }

    pub fn Constructor(global: &GlobalScope,
                       url_str: DOMString,
                       event_source_init: &EventSourceInit) -> Fallible<Root<EventSource>> {
        // Steps 1-2
        let base_url = global.get_url();
        let url = match base_url.join(&*url_str) {
            Ok(u) => u,
            Err(_) => return Err(Error::Syntax)
        };
        // Step 3
        let event_source = EventSource::new(global, url, event_source_init.withCredentials);
        // Step 4
        // Step 5
        // Step 6
        // Step 7
        // Step 8
        // Step 9
        // Step 10
        // Step 11
        Ok(event_source)
        // Step 12
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
        self.ready_state.set(EventSourceReadyState::Closed);
        // TODO: Terminate ongoing fetch
    }
}
