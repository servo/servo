/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::InputEventBinding::{self, InputEventMethods};
use crate::dom::bindings::codegen::Bindings::UIEventBinding::UIEventBinding::UIEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::uievent::UIEvent;
use crate::dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct InputEvent {
    uievent: UIEvent,
    data: Option<DOMString>,
    is_composing: bool,
}

impl InputEvent {
    pub fn new(
        window: &Window,
        type_: DOMString,
        can_bubble: bool,
        cancelable: bool,
        view: Option<&Window>,
        detail: i32,
        data: Option<DOMString>,
        is_composing: bool,
    ) -> DomRoot<InputEvent> {
        let ev = reflect_dom_object(
            Box::new(InputEvent {
                uievent: UIEvent::new_inherited(),
                data: data,
                is_composing: is_composing,
            }),
            window,
        );
        ev.uievent
            .InitUIEvent(type_, can_bubble, cancelable, view, detail);
        ev
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        type_: DOMString,
        init: &InputEventBinding::InputEventInit,
    ) -> Fallible<DomRoot<InputEvent>> {
        let event = InputEvent::new(
            window,
            type_,
            init.parent.parent.bubbles,
            init.parent.parent.cancelable,
            init.parent.view.as_deref(),
            init.parent.detail,
            init.data.clone(),
            init.isComposing,
        );
        Ok(event)
    }
}

impl InputEventMethods for InputEvent {
    // https://w3c.github.io/uievents/#dom-inputevent-data
    fn GetData(&self) -> Option<DOMString> {
        self.data.clone()
    }

    // https://w3c.github.io/uievents/#dom-inputevent-iscomposing
    fn IsComposing(&self) -> bool {
        self.is_composing
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
