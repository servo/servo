/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::UIEventBinding::UIEventMethods;
use script_bindings::inheritance::Castable;
use script_bindings::reflector::reflect_dom_object_with_cx;
use script_bindings::str::DOMString;

use crate::dom::bindings::codegen::Bindings::TextEventBinding::TextEventMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::event::Event;
use crate::dom::uievent::UIEvent;
use crate::dom::window::Window;

#[dom_struct]
/// <https://w3c.github.io/uievents/#textevent>
pub(crate) struct TextEvent {
    uievent: UIEvent,
    data: DomRefCell<DOMString>,
}

impl TextEvent {
    pub(crate) fn new_inherited() -> TextEvent {
        TextEvent {
            uievent: UIEvent::new_inherited(),
            data: DomRefCell::new(DOMString::new()),
        }
    }

    pub(crate) fn new_uninitialized(cx: &mut JSContext, window: &Window) -> DomRoot<TextEvent> {
        reflect_dom_object_with_cx(Box::new(TextEvent::new_inherited()), window, cx)
    }
}

impl TextEventMethods<crate::DomTypeHolder> for TextEvent {
    /// <https://w3c.github.io/uievents/event-algo.html#dom-textevent-inittextevent>
    fn InitTextEvent(
        &self,
        type_: DOMString,
        bubbles: bool,
        cancelable: bool,
        view: Option<&Window>,
        data: DOMString,
    ) {
        // 1. If this’s dispatch flag is set, then return.
        if self.upcast::<Event>().dispatching() {
            return;
        }

        // 2. Initialize a UIEvent with this, type and eventTarget
        // 3. Set this.bubbles = bubbles
        // 4. Set this.cancelable = cancelable
        // 5. Set this.view = view
        // note: The bubbles/cancelable/view should be parameters to "Initialize a UIEvent" instead of being set twice.
        self.uievent
            .init_event(type_.into(), bubbles, cancelable, view, 0);

        // 6. Set this.data = data
        *self.data.borrow_mut() = data;
    }

    fn Data(&self) -> DOMString {
        self.data.borrow().clone()
    }

    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
