/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::CustomEventBinding;
use dom::bindings::codegen::InheritTypes::{EventCast, CustomEventDerived};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::Fallible;
use dom::bindings::trace::Traceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::event::{Event, EventMethods, EventTypeId, CustomEventTypeId};
use dom::window::Window;
use js::jsapi::JSContext;
use js::jsval::{JSVal, NullValue};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct CustomEvent {
    event: Event,
    detail: Traceable<JSVal>
}

impl CustomEventDerived for Event {
    fn is_customevent(&self) -> bool {
        self.type_id == CustomEventTypeId
    }
}

pub trait CustomEventMethods {
    fn Detail(&self, _cx: *mut JSContext) -> JSVal;
    fn InitCustomEvent(&mut self, _cx: *mut JSContext,
                       type_: DOMString, can_bubble: bool,
                       cancelable: bool, detail: JSVal);
}

impl CustomEvent {
    pub fn new_inherited(type_id: EventTypeId) -> CustomEvent {
        CustomEvent {
            event: Event::new_inherited(type_id),
            detail: Traceable::new(NullValue())
        }
    }

    pub fn new_uninitialized(window: &JSRef<Window>) -> Temporary<CustomEvent> {
        reflect_dom_object(box CustomEvent::new_inherited(CustomEventTypeId),
                           window,
                           CustomEventBinding::Wrap)
    }
    pub fn new(window: &JSRef<Window>, type_: DOMString, bubbles: bool, cancelable: bool, detail: JSVal) -> Temporary<CustomEvent> {
        let mut ev = CustomEvent::new_uninitialized(window).root();
        ev.InitCustomEvent(window.deref().get_cx(), type_, bubbles, cancelable, detail);
        Temporary::from_rooted(&*ev)
    }
    pub fn Constructor(owner: &JSRef<Window>,
                       type_: DOMString,
                       init: &CustomEventBinding::CustomEventInit) -> Fallible<Temporary<CustomEvent>>{
        Ok(CustomEvent::new(owner, type_, init.parent.bubbles, init.parent.cancelable, init.detail))
    }
}

impl<'a> CustomEventMethods for JSRef<'a, CustomEvent> {
    fn Detail(&self, _cx: *mut JSContext) -> JSVal {
        self.detail.deref().clone()
    }

    fn InitCustomEvent(&mut self,
                        _cx: *mut JSContext,
                       type_: DOMString,
                       can_bubble: bool,
                       cancelable: bool,
                       detail: JSVal) {
        self.detail = Traceable::new(detail);
        let event: &mut JSRef<Event> = EventCast::from_mut_ref(self);
        event.InitEvent(type_, can_bubble, cancelable);
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
