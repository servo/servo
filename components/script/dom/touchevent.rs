/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TouchEventBinding;
use dom::bindings::codegen::Bindings::TouchEventBinding::TouchEventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutJS, Root};
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::event::{EventBubbles, EventCancelable};
use dom::touchlist::TouchList;
use dom::uievent::UIEvent;
use dom::window::Window;
use dom_struct::dom_struct;
use std::cell::Cell;

#[dom_struct]
pub struct TouchEvent {
    uievent: UIEvent,
    touches: MutJS<TouchList>,
    target_touches: MutJS<TouchList>,
    changed_touches: MutJS<TouchList>,
    alt_key: Cell<bool>,
    meta_key: Cell<bool>,
    ctrl_key: Cell<bool>,
    shift_key: Cell<bool>,
}

impl TouchEvent {
    fn new_inherited(touches: &TouchList,
                     changed_touches: &TouchList,
                     target_touches: &TouchList) -> TouchEvent {
        TouchEvent {
            uievent: UIEvent::new_inherited(),
            touches: MutJS::new(touches),
            target_touches: MutJS::new(target_touches),
            changed_touches: MutJS::new(changed_touches),
            ctrl_key: Cell::new(false),
            shift_key: Cell::new(false),
            alt_key: Cell::new(false),
            meta_key: Cell::new(false),
        }
    }

    pub fn new_uninitialized(window: &Window,
                     touches: &TouchList,
                     changed_touches: &TouchList,
                     target_touches: &TouchList) -> Root<TouchEvent> {
        reflect_dom_object(box TouchEvent::new_inherited(touches, changed_touches, target_touches),
                           window,
                           TouchEventBinding::Wrap)
    }

    pub fn new(window: &Window,
               type_: DOMString,
               can_bubble: EventBubbles,
               cancelable: EventCancelable,
               view: Option<&Window>,
               detail: i32,
               touches: &TouchList,
               changed_touches: &TouchList,
               target_touches: &TouchList,
               ctrl_key: bool,
               alt_key: bool,
               shift_key: bool,
               meta_key: bool) -> Root<TouchEvent> {
        let ev = TouchEvent::new_uninitialized(window, touches, changed_touches, target_touches);
        ev.upcast::<UIEvent>().InitUIEvent(type_,
                                           bool::from(can_bubble),
                                           bool::from(cancelable),
                                           view, detail);
        ev.ctrl_key.set(ctrl_key);
        ev.alt_key.set(alt_key);
        ev.shift_key.set(shift_key);
        ev.meta_key.set(meta_key);
        ev
    }
}

impl<'a> TouchEventMethods for &'a TouchEvent {
    /// https://w3c.github.io/touch-events/#widl-TouchEvent-ctrlKey
    fn CtrlKey(&self) -> bool {
        self.ctrl_key.get()
    }

    /// https://w3c.github.io/touch-events/#widl-TouchEvent-shiftKey
    fn ShiftKey(&self) -> bool {
        self.shift_key.get()
    }

    /// https://w3c.github.io/touch-events/#widl-TouchEvent-altKey
    fn AltKey(&self) -> bool {
        self.alt_key.get()
    }

    /// https://w3c.github.io/touch-events/#widl-TouchEvent-metaKey
    fn MetaKey(&self) -> bool {
        self.meta_key.get()
    }

    /// https://w3c.github.io/touch-events/#widl-TouchEventInit-touches
    fn Touches(&self) -> Root<TouchList> {
        self.touches.get()
    }

    /// https://w3c.github.io/touch-events/#widl-TouchEvent-targetTouches
    fn TargetTouches(&self) -> Root<TouchList> {
        self.target_touches.get()
    }

    /// https://w3c.github.io/touch-events/#widl-TouchEvent-changedTouches
    fn ChangedTouches(&self) -> Root<TouchList> {
        self.changed_touches.get()
    }

    /// https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
