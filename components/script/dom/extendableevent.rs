/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::ExtendableEventBinding;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::event::Event;
use dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use dom_struct::dom_struct;
use js::jsapi::{HandleValue, JSContext};
use servo_atoms::Atom;

// https://w3c.github.io/ServiceWorker/#extendable-event
#[dom_struct]
pub struct ExtendableEvent {
    event: Event,
    extensions_allowed: bool
}

impl ExtendableEvent {
    pub fn new_inherited() -> ExtendableEvent {
        ExtendableEvent {
            event: Event::new_inherited(),
            extensions_allowed: true
        }
    }
    pub fn new(worker: &ServiceWorkerGlobalScope,
               type_: Atom,
               bubbles: bool,
               cancelable: bool)
               -> Root<ExtendableEvent> {
        let ev = reflect_dom_object(box ExtendableEvent::new_inherited(), worker, ExtendableEventBinding::Wrap);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    pub fn Constructor(worker: &ServiceWorkerGlobalScope,
                       type_: DOMString,
                       init: &ExtendableEventBinding::ExtendableEventInit) -> Fallible<Root<ExtendableEvent>> {
        Ok(ExtendableEvent::new(worker,
                                Atom::from(type_),
                                init.parent.bubbles,
                                init.parent.cancelable))
    }

    // https://w3c.github.io/ServiceWorker/#wait-until-method
    pub fn WaitUntil(&self, _cx: *mut JSContext, _val: HandleValue) -> ErrorResult {
        // Step 1
        if !self.extensions_allowed {
            return Err(Error::InvalidState);
        }
        // Step 2
        // TODO add a extended_promises array to enqueue the `val`
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    pub fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
