/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::InputEventBinding::{self, InputEventMethods};
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventBinding::UIEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::js::{Root, RootedReference};
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::uievent::UIEvent;
use dom::window::Window;
use dom_struct::dom_str;
use std::cell::Cell;

#[dom_struct]
pub struct InputEvent {
    uievent: UIEvent,
    data: DOMRefCell<Option<DOMString>>,
    is_composing: Cell<bool>,
}

impl InputEvent {
    fn new_inherited() -> InputEvent {
        InputEvent {
            uievent: UIEvent::new_inherited(),
            data: DOMRefCell::new(Some(DOMString::new())),
            is_composing: Cell::new(false),
        }
    }

    pub fn new_uninitialized(window: &Window) -> Root<InputEvent> {
        reflect_dom_object(box InputEvent::new_inherited(),
                           window,
                           InputEventBinding::Wrap)
    }

    pub fn new(window: &Window,
               type_: DOMString,
               can_bubble: bool,
               cancelable: bool,
               view: Option<&Window>,
               detail: i32,
               data: Option<DOMString>,
               is_composing: bool) -> Root<InputEvent> {
        let ev = InputEvent::new_uninitialized(window);
        ev.uievent.InitUIEvent(type_, can_bubble, cancelable, view, detail);
        *ev.data.borrow_mut() = data;
        ev.is_composing.set(is_composing);
        ev
    }

    pub fn Constructor(window: &Window,
                       type_: DOMString,
                       init: &InputEventBinding::InputEventInit)
                       -> Fallible<Root<InputEvent>> {
        let event = InputEvent::new(window,
                                    type_,
                                    init.parent.parent.bubbles,
                                    init.parent.parent.cancelable,
                                    init.parent.view.r(),
                                    init.parent.detail,
                                    init.data.clone(),
                                    init.isComposing);
        Ok(event)
    }
}

impl InputEventMethods for InputEvent {
    // https://w3c.github.io/uievents/#dom-inputevent-data
    fn GetData(&self) -> Option<DOMString> {
        self.data.borrow().clone()
    }

    // https://w3c.github.io/uievents/#dom-inputevent-iscomposing
    fn IsComposing(&self) -> bool {
        self.is_composing.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
