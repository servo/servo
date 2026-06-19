/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object_with_cx;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::BeforeUnloadEventBinding::BeforeUnloadEventMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::window::Window;

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

    pub(crate) fn new_uninitialized(
        cx: &mut JSContext,
        window: &Window,
    ) -> DomRoot<BeforeUnloadEvent> {
        reflect_dom_object_with_cx(Box::new(BeforeUnloadEvent::new_inherited()), window, cx)
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
    ) -> DomRoot<BeforeUnloadEvent> {
        let ev = BeforeUnloadEvent::new_uninitialized(cx, window);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        }
        ev
    }
}

impl BeforeUnloadEventMethods<crate::DomTypeHolder> for BeforeUnloadEvent {
    /// <https://html.spec.whatwg.org/multipage/#dom-beforeunloadevent-returnvalue>
    fn ReturnValue(&self) -> DOMString {
        self.return_value.borrow().clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-beforeunloadevent-returnvalue>
    fn SetReturnValue(&self, value: DOMString) {
        *self.return_value.borrow_mut() = value;
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
