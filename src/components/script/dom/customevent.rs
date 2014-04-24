/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::CustomEventBinding;
use dom::bindings::codegen::InheritTypes::CustomEventDerived;
use dom::bindings::js::JS;
use dom::bindings::error::Fallible;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::event::{Event, EventTypeId, CustomEventTypeId};
use dom::window::Window;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct CustomEvent {
    event: Event,
    detail: i32
}

impl CustomEventDerived for Event {
    fn is_customevent(&self) -> bool {
        self.type_id == CustomEventTypeId
    }
}

impl CustomEvent {
    pub fn new_inherited(type_id: EventTypeId) -> CustomEvent {
        CustomEvent {
            event: Event::new_inherited(type_id),
            detail: 0
        }
    }

    pub fn new(window: &JS<Window>) -> JS<CustomEvent> {
        reflect_dom_object(~CustomEvent::new_inherited(CustomEventTypeId),
                           window,
                           CustomEventBinding::Wrap)
    }

    pub fn Constructor(owner: &JS<Window>,
                       type_: DOMString,
                       init: &CustomEventBinding::CustomEventInit) -> Fallible<JS<CustomEvent>> {
        let mut ev = CustomEvent::new(owner);
        ev.get_mut().InitCustomEvent(type_, init.parent.bubbles, init.parent.cancelable, init.detail);
        Ok(ev)
    }


    pub fn Detail(&self) -> i32 {
        self.detail
    }

    pub fn InitCustomEvent(&mut self,
                       type_: DOMString,
                       can_bubble: bool,
                       cancelable: bool,
                       detail: i32) {
        self.event.InitEvent(type_, can_bubble, cancelable);
        self.detail = detail;
    }
}

impl Reflectable for CustomEvent {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.event.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.event.mut_reflector()
    }
}
