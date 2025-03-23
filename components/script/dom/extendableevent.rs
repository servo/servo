/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::{HandleObject, HandleValue};
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::ExtendableEventBinding::{
    ExtendableEventInit, ExtendableEventMethods,
};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use crate::script_runtime::{CanGc, JSContext};

// https://w3c.github.io/ServiceWorker/#extendable-event
#[dom_struct]
pub(crate) struct ExtendableEvent {
    event: Event,
    extensions_allowed: bool,
}

#[allow(non_snake_case)]
impl ExtendableEvent {
    pub(crate) fn new_inherited() -> ExtendableEvent {
        ExtendableEvent {
            event: Event::new_inherited(),
            extensions_allowed: true,
        }
    }

    pub(crate) fn new(
        worker: &ServiceWorkerGlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        can_gc: CanGc,
    ) -> DomRoot<ExtendableEvent> {
        Self::new_with_proto(worker, None, type_, bubbles, cancelable, can_gc)
    }

    fn new_with_proto(
        worker: &ServiceWorkerGlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        can_gc: CanGc,
    ) -> DomRoot<ExtendableEvent> {
        let ev = reflect_dom_object_with_proto(
            Box::new(ExtendableEvent::new_inherited()),
            worker,
            proto,
            can_gc,
        );
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }
}

impl ExtendableEventMethods<crate::DomTypeHolder> for ExtendableEvent {
    // https://w3c.github.io/ServiceWorker/#dom-extendableevent-extendableevent
    fn Constructor(
        worker: &ServiceWorkerGlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &ExtendableEventInit,
    ) -> Fallible<DomRoot<ExtendableEvent>> {
        Ok(ExtendableEvent::new_with_proto(
            worker,
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            can_gc,
        ))
    }

    // https://w3c.github.io/ServiceWorker/#wait-until-method
    fn WaitUntil(&self, _cx: JSContext, _val: HandleValue) -> ErrorResult {
        // Step 1
        if !self.extensions_allowed {
            return Err(Error::InvalidState);
        }
        // Step 2
        // TODO add a extended_promises array to enqueue the `val`
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
