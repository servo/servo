/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::ExtendableEventBinding;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::event::Event;
use js::jsapi::{HandleValue, JSContext};
use string_cache::Atom;

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
    pub fn new_uninitialized(global: GlobalRef) -> Root<ExtendableEvent> {
        reflect_dom_object(box ExtendableEvent::new_inherited(),
                           global,
                           ExtendableEventBinding::Wrap)
    }
    pub fn new(global: GlobalRef,
               type_: Atom,
               bubbles: bool,
               cancelable: bool)
               -> Root<ExtendableEvent> {
        let ev = ExtendableEvent::new_uninitialized(global);
        ev.init_extendable_event(type_, bubbles, cancelable);
        ev
    }

    pub fn Constructor(global: GlobalRef,
                   type_: DOMString,
                   init: &ExtendableEventBinding::ExtendableEventInit) -> Fallible<Root<ExtendableEvent>> {
        Ok(ExtendableEvent::new(global,
                            Atom::from(type_),
                            init.parent.bubbles,
                            init.parent.cancelable))
    }

    fn init_extendable_event(&self,
                         type_: Atom,
                         can_bubble: bool,
                         cancelable: bool) {
        let event = self.upcast::<Event>();
        if event.dispatching() {
            return;
        }
        event.init_event(type_, can_bubble, cancelable);
    }

    // https://w3c.github.io/ServiceWorker/#wait-until-method
    pub fn WaitUntil(&self, _cx: *mut JSContext, val: HandleValue) {
        // Step 1
        if !self.extensions_allowed {
            // TODO throw invalid state error, but this does not return a `Fallible` ?
        }
        // Step 2
        // TODO add a extended_promises array to enqueue the `val`
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    pub fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
