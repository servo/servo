/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CustomEventBinding;
use dom::bindings::codegen::Bindings::CustomEventBinding::CustomEventMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, CustomEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Rootable, Temporary};
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventTypeId};
use js::jsapi::{JSContext, HandleValue, Heap};
use js::jsval::{JSVal, NullValue};
use util::str::DOMString;
use std::cell::Cell;

// https://dom.spec.whatwg.org/#interface-customevent
#[dom_struct]
pub struct CustomEvent {
    event: Event,
    detail: Cell<Heap<JSVal>>,
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
            detail: Cell::new(Heap::new(NullValue())),
        }
    }

    pub fn new_uninitialized(global: GlobalRef) -> Temporary<CustomEvent> {
        reflect_dom_object(box CustomEvent::new_inherited(EventTypeId::CustomEvent),
                           global,
                           CustomEventBinding::Wrap)
    }
    pub fn new(global: GlobalRef, type_: DOMString, bubbles: bool, cancelable: bool, detail: HandleValue) -> Temporary<CustomEvent> {
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
    // https://dom.spec.whatwg.org/#dom-customevent-detail
    fn Detail(self, _cx: *mut JSContext) -> JSVal {
        self.detail.get().get()
    }

    // https://dom.spec.whatwg.org/#dom-customevent-initcustomevent
    #[allow(unsafe_code)]
    fn InitCustomEvent(self,
                       _cx: *mut JSContext,
                       type_: DOMString,
                       can_bubble: bool,
                       cancelable: bool,
                       detail: HandleValue) {
        let event: JSRef<Event> = EventCast::from_ref(self);
        if event.dispatching() {
            return;
        }

        unsafe {
            let cell = self.detail.as_unsafe_cell().get();
            (*cell).set(detail.get());
        }
        event.InitEvent(type_, can_bubble, cancelable);
    }
}

