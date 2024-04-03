/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::InputEventBinding::{self, InputEventMethods};
use crate::dom::bindings::codegen::Bindings::UIEventBinding::UIEvent_Binding::UIEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::uievent::UIEvent;
use crate::dom::window::Window;

#[dom_struct]
pub struct InputEvent {
    uievent: UIEvent,
    data: Option<DOMString>,
    is_composing: bool,
}

impl InputEvent {
    #[allow(clippy::too_many_arguments)]
    fn new(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        can_bubble: bool,
        cancelable: bool,
        view: Option<&Window>,
        detail: i32,
        data: Option<DOMString>,
        is_composing: bool,
    ) -> DomRoot<InputEvent> {
        let ev = reflect_dom_object_with_proto(
            Box::new(InputEvent {
                uievent: UIEvent::new_inherited(),
                data,
                is_composing,
            }),
            window,
            proto,
        );
        ev.uievent
            .InitUIEvent(type_, can_bubble, cancelable, view, detail);
        ev
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &InputEventBinding::InputEventInit,
    ) -> Fallible<DomRoot<InputEvent>> {
        let event = InputEvent::new(
            window,
            proto,
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
