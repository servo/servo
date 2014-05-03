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
use js::jsval::{JSVal, NullValue};
use js::jsapi::JSContext;

#[deriving(Encodable)]
pub struct CustomEvent {
    event: Event
}

impl CustomEventDerived for Event {
    fn is_customevent(&self) -> bool {
        self.type_id == CustomEventTypeId
    }
}

impl CustomEvent {
    pub fn new_inherited(type_id: EventTypeId) -> CustomEvent {
        CustomEvent {
            event: Event::new_inherited(type_id)
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
        ev.get_mut().InitCustomEvent(owner.get().get_cx(), type_, init.parent.bubbles, init.parent.cancelable, NullValue());
        Ok(ev)
    }


    pub fn Detail(&self, _cx: *JSContext) -> JSVal {
        // FIXME store this properly
        NullValue()
    }

    pub fn InitCustomEvent(&mut self,
                       _cx: *JSContext,
                       type_: DOMString,
                       can_bubble: bool,
                       cancelable: bool,
                       _detail: JSVal) {
        self.event.InitEvent(type_, can_bubble, cancelable);
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
