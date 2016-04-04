/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TransitionEventBinding;
use dom::bindings::codegen::Bindings::TransitionEventBinding::{TransitionEventMethods, TransitionEventInit};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::num::Finite;
use dom::bindings::reflector::reflect_dom_object;
use dom::event::{Event, EventBubbles, EventCancelable};
use string_cache::Atom;
use util::str::DOMString;

#[dom_struct]
pub struct TransitionEvent {
    event: Event,
    property_name: DOMString,
    elasped_time: Finite<f32>,
    pseudo_element: DOMString,
}

impl TransitionEvent {
    pub fn new_inherited(init: &TransitionEventInit) -> TransitionEvent {
        TransitionEvent {
            event: Event::new_inherited(),
            property_name: init.propertyName.clone(),
            elasped_time: Finite::new(*init.elapsedTime).unwrap(),
            pseudo_element: init.pseudoElement.clone(),
        }
    }

    pub fn new(global: GlobalRef,
               type_: DOMString,
               init: &TransitionEventInit) -> Root<TransitionEvent> {
        let transition_event = reflect_dom_object(box TransitionEvent::new_inherited(init), global, TransitionEventBinding::Wrap);

        {
            let event = transition_event.upcast::<Event>();
            let bubbles = EventBubbles::from(init.parent.bubbles);
            let cancelable = EventCancelable::from(init.parent.cancelable);
            event.init_event(Atom::from(type_), bool::from(bubbles), bool::from(cancelable));
        }

        transition_event
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &TransitionEventInit) -> Fallible<Root<TransitionEvent>> {
        Ok(TransitionEvent::new(global, type_, init))
    }
}

impl TransitionEventMethods for TransitionEvent {
    // https://drafts.csswg.org/css-transitions/#Events-TransitionEvent-propertyName
    fn PropertyName(&self) -> DOMString {
        self.property_name.clone()
    }

    // https://drafts.csswg.org/css-transitions/#Events-TransitionEvent-elapsedTime
    fn ElapsedTime(&self) -> Finite<f32> {
        self.elasped_time.clone()
    }

    // https://drafts.csswg.org/css-transitions/#Events-TransitionEvent-pseudoElement
    fn PseudoElement(&self) -> DOMString {
        self.pseudo_element.clone()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.trusted()
    }
}
