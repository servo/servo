/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ForceTouchEventBinding;
use dom::bindings::codegen::Bindings::ForceTouchEventBinding::ForceTouchEventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{Root};
use dom::bindings::num::Finite;
use dom::bindings::reflector::reflect_dom_object;
use dom::event::{EventBubbles, EventCancelable};
use dom::uievent::UIEvent;
use dom::window::Window;
use util::str::DOMString;

#[dom_struct]
pub struct ForceTouchEvent {
    uievent: UIEvent,
    force: f32, // FIXME f64
}

impl ForceTouchEvent {
    fn new_inherited(force: f32) -> ForceTouchEvent {
        ForceTouchEvent {
            uievent: UIEvent::new_inherited(),
            force: force,
        }
    }

    pub fn new(window: &Window,
               type_: DOMString,
               can_bubble: EventBubbles,
               cancelable: EventCancelable,
               view: Option<&Window>,
               detail: i32,
               force: f32) -> Root<ForceTouchEvent> {

        let event = box ForceTouchEvent::new_inherited(force);
        let ev = reflect_dom_object(event, GlobalRef::Window(window), ForceTouchEventBinding::Wrap);
        // FIXME: is the following line really useful? See touch.rs
        ev.upcast::<UIEvent>().InitUIEvent(type_,
                                           can_bubble == EventBubbles::Bubbles,
                                           cancelable == EventCancelable::Cancelable,
                                           view, detail);
        ev
    }
}

impl<'a> ForceTouchEventMethods for &'a ForceTouchEvent {

    fn WebkitForce(&self) -> Finite<f32> {
        Finite::wrap(self.force)
    }

    fn WEBKIT_FORCE_AT_MOUSE_DOWN(&self) -> Finite<f32> {
        Finite::wrap(1.0)
    }

    fn WEBKIT_FORCE_AT_FORCE_MOUSE_DOWN(&self) -> Finite<f32> {
        Finite::wrap(2.0)
    }

    // FIXME: is that really useful???
    // FIXME: probably doesn't need to be a UIEvent
    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
