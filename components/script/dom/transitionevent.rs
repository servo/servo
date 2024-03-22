/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::TransitionEventBinding::{
    TransitionEventInit, TransitionEventMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::window::Window;

#[dom_struct]
pub struct TransitionEvent {
    event: Event,
    #[no_trace]
    property_name: Atom,
    elapsed_time: Finite<f32>,
    pseudo_element: DOMString,
}

impl TransitionEvent {
    fn new_inherited(init: &TransitionEventInit) -> TransitionEvent {
        TransitionEvent {
            event: Event::new_inherited(),
            property_name: Atom::from(init.propertyName.clone()),
            elapsed_time: init.elapsedTime,
            pseudo_element: init.pseudoElement.clone(),
        }
    }

    pub fn new(
        window: &Window,
        type_: Atom,
        init: &TransitionEventInit,
    ) -> DomRoot<TransitionEvent> {
        Self::new_with_proto(window, None, type_, init)
    }

    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        init: &TransitionEventInit,
    ) -> DomRoot<TransitionEvent> {
        let ev = reflect_dom_object_with_proto(
            Box::new(TransitionEvent::new_inherited(init)),
            window,
            proto,
        );
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, init.parent.bubbles, init.parent.cancelable);
        }
        ev
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &TransitionEventInit,
    ) -> Fallible<DomRoot<TransitionEvent>> {
        Ok(TransitionEvent::new_with_proto(
            window,
            proto,
            Atom::from(type_),
            init,
        ))
    }
}

impl TransitionEventMethods for TransitionEvent {
    // https://drafts.csswg.org/css-transitions/#Events-TransitionEvent-propertyName
    fn PropertyName(&self) -> DOMString {
        DOMString::from(&*self.property_name)
    }

    // https://drafts.csswg.org/css-transitions/#Events-TransitionEvent-elapsedTime
    fn ElapsedTime(&self) -> Finite<f32> {
        self.elapsed_time
    }

    // https://drafts.csswg.org/css-transitions/#Events-TransitionEvent-pseudoElement
    fn PseudoElement(&self) -> DOMString {
        self.pseudo_element.clone()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.upcast::<Event>().IsTrusted()
    }
}
