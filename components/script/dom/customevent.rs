/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CustomEventBinding;
use dom::bindings::codegen::Bindings::CustomEventBinding::CustomEventMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, CustomEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary, MutHeap};
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventTypeId};
use js::jsapi::JSContext;
use js::jsval::{JSVal, NullValue};
use servo_util::str::DOMString;

#[dom_struct]
pub struct CustomEvent {
    event: Event,
    detail: MutHeap<JSVal>,
}

impl CustomEventDerived for Event {
    fn is_customevent(&self) -> bool {
        *self.type_id() == EventTypeId::CustomEvent
    }
}

impl CustomEvent {
    fn new_inherited(type_id: EventTypeId) -> CustomEvent {
        CustomEvent {
            event: Event::new_inherited(type_id),
            detail: MutHeap::new(NullValue()),
        }
    }

    pub fn new_uninitialized(global: GlobalRef) -> Temporary<CustomEvent> {
        reflect_dom_object(box CustomEvent::new_inherited(EventTypeId::CustomEvent),
                           global,
                           CustomEventBinding::Wrap)
    }
    pub fn new(global: GlobalRef, type_: DOMString, bubbles: bool, cancelable: bool, detail: JSVal) -> Temporary<CustomEvent> {
        let ev = CustomEvent::new_uninitialized(global).root();
        ev.r().InitCustomEvent(global.get_cx(), type_, bubbles, cancelable, detail);
        Temporary::from_rooted(ev.r())
    }
    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &CustomEventBinding::CustomEventInit) -> Fallible<Temporary<CustomEvent>>{
        Ok(CustomEvent::new(global, type_, init.parent.bubbles, init.parent.cancelable, init.detail))
    }
}

impl<'a> CustomEventMethods for JSRef<'a, CustomEvent> {
    fn Detail(self, _cx: *mut JSContext) -> JSVal {
        self.detail.get()
    }

    fn InitCustomEvent(self,
                       _cx: *mut JSContext,
                       type_: DOMString,
                       can_bubble: bool,
                       cancelable: bool,
                       detail: JSVal) {
        let event: JSRef<Event> = EventCast::from_ref(self);
        if event.dispatching() {
            return;
        }

        self.detail.set(detail);
        event.InitEvent(type_, can_bubble, cancelable);
    }
}

