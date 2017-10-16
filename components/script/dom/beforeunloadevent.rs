/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(dead_code)]

use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::BeforeUnloadEventBinding;
use dom::bindings::codegen::Bindings::BeforeUnloadEventBinding::BeforeUnloadEventMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::window::Window;
use dom_struct::dom_struct;
use servo_atoms::Atom;

// https://html.spec.whatwg.org/multipage/#beforeunloadevent
#[dom_struct]
pub struct BeforeUnloadEvent {
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

    pub fn new_uninitialized(window: &Window) -> DomRoot<BeforeUnloadEvent> {
        reflect_dom_object(Box::new(BeforeUnloadEvent::new_inherited()),
                           window,
                           BeforeUnloadEventBinding::Wrap)
    }

    pub fn new(window: &Window,
               type_: Atom,
               bubbles: EventBubbles,
               cancelable: EventCancelable) -> DomRoot<BeforeUnloadEvent> {
        let ev = BeforeUnloadEvent::new_uninitialized(window);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles),
                             bool::from(cancelable));
        }
        ev
    }
}

impl BeforeUnloadEventMethods for BeforeUnloadEvent {
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
