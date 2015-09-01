/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CustomEventBinding;
use dom::bindings::codegen::Bindings::CustomEventBinding::CustomEventMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, CustomEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Root, MutHeapJSVal};
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventTypeId};
use js::jsapi::{JSContext, HandleValue};
use js::jsval::JSVal;
use util::str::DOMString;

// https://dom.spec.whatwg.org/#interface-customevent
#[dom_struct]
pub struct CustomEvent {
    event: Event,
    #[ignore_heap_size_of = "Defined in rust-mozjs"]
    detail: MutHeapJSVal,
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
            detail: MutHeapJSVal::new(),
        }
    }

    pub fn new_uninitialized(global: GlobalRef) -> Root<CustomEvent> {
        reflect_dom_object(box CustomEvent::new_inherited(EventTypeId::CustomEvent),
                           global,
                           CustomEventBinding::Wrap)
    }
    pub fn new(global: GlobalRef,
               type_: DOMString,
               bubbles: bool,
               cancelable: bool,
               detail: HandleValue) -> Root<CustomEvent> {
        let ev = CustomEvent::new_uninitialized(global);
        ev.r().InitCustomEvent(global.get_cx(), type_, bubbles, cancelable, detail);
        ev
    }
    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &CustomEventBinding::CustomEventInit) -> Fallible<Root<CustomEvent>>{
        Ok(CustomEvent::new(global,
                            type_,
                            init.parent.bubbles,
                            init.parent.cancelable,
                            HandleValue { ptr: &init.detail }))
    }
}

impl CustomEventMethods for CustomEvent {
    // https://dom.spec.whatwg.org/#dom-customevent-detail
    fn Detail(&self, _cx: *mut JSContext) -> JSVal {
        self.detail.get()
    }

    // https://dom.spec.whatwg.org/#dom-customevent-initcustomevent
    fn InitCustomEvent(&self,
                       _cx: *mut JSContext,
                       type_: DOMString,
                       can_bubble: bool,
                       cancelable: bool,
                       detail: HandleValue) {
        let event = EventCast::from_ref(self);
        if event.dispatching() {
            return;
        }

        self.detail.set(detail.get());
        event.InitEvent(type_, can_bubble, cancelable);
    }
}
