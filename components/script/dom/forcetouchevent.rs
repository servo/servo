/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ForceTouchEventBinding;
use dom::bindings::codegen::Bindings::ForceTouchEventBinding::ForceTouchEventMethods;
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::uievent::UIEvent;
use dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct ForceTouchEvent {
    uievent: UIEvent,
    force: f32,
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
               force: f32) -> DomRoot<ForceTouchEvent> {
        let event = Box::new(ForceTouchEvent::new_inherited(force));
        let ev = reflect_dom_object(event, window, ForceTouchEventBinding::Wrap);
        ev.upcast::<UIEvent>().InitUIEvent(type_, true, true, Some(window), 0);
        ev
    }
}

impl<'a> ForceTouchEventMethods for &'a ForceTouchEvent {
    fn ServoForce(&self) -> Finite<f32> {
        Finite::wrap(self.force)
    }

    fn SERVO_FORCE_AT_MOUSE_DOWN(&self) -> Finite<f32> {
        Finite::wrap(1.0)
    }

    fn SERVO_FORCE_AT_FORCE_MOUSE_DOWN(&self) -> Finite<f32> {
        Finite::wrap(2.0)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
