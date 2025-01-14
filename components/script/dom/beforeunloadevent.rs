/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(dead_code)]

use dom_struct::dom_struct;
use servo_atoms::Atom;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::BeforeUnloadEventBinding::BeforeUnloadEventMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

// https://html.spec.whatwg.org/multipage/#beforeunloadevent
#[dom_struct]
pub(crate) struct BeforeUnloadEvent {
    event: Event,
    return_value: DomRefCell<DOMString>,
}

impl BeforeUnloadEvent {
    fn new_inherited() -> BeforeUnloadEvent {
        BeforeUnloadEvent {
            event: Event::new_inherited(),
            return_value: DomRefCell::new(DOMString::new()),
        }
    }

    pub(crate) fn new_uninitialized(window: &Window) -> DomRoot<BeforeUnloadEvent> {
        reflect_dom_object(
            Box::new(BeforeUnloadEvent::new_inherited()),
            window,
            CanGc::note(),
        )
    }

    pub(crate) fn new(
        window: &Window,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
    ) -> DomRoot<BeforeUnloadEvent> {
        let ev = BeforeUnloadEvent::new_uninitialized(window);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        }
        ev
    }
}

impl BeforeUnloadEventMethods<crate::DomTypeHolder> for BeforeUnloadEvent {
    // https://html.spec.whatwg.org/multipage/#dom-beforeunloadevent-returnvalue
    fn ReturnValue(&self) -> DOMString {
        self.return_value.borrow().clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-beforeunloadevent-returnvalue
    fn SetReturnValue(&self, value: DOMString) {
        *self.return_value.borrow_mut() = value;
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
