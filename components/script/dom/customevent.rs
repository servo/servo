/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CustomEventBinding;
use dom::bindings::codegen::Bindings::CustomEventBinding::CustomEventMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutHeapJSVal, Root};
use dom::bindings::reflector::reflect_dom_object;
use dom::event::Event;
use js::jsapi::{HandleValue, JSContext};
use js::jsval::JSVal;
use util::str::DOMString;

// https://dom.spec.whatwg.org/#interface-customevent
#[dom_struct]
pub struct CustomEvent {
    event: Event,
    #[ignore_heap_size_of = "Defined in rust-mozjs"]
    detail: MutHeapJSVal,
}

impl CustomEvent {
    fn new_inherited() -> CustomEvent {
        CustomEvent {
            event: Event::new_inherited(),
            detail: MutHeapJSVal::new(),
        }
    }

    pub fn new_uninitialized(global: GlobalRef) -> Root<CustomEvent> {
        reflect_dom_object(box CustomEvent::new_inherited(),
                           global,
                           CustomEventBinding::Wrap)
    }
    pub fn new(global: GlobalRef,
               type_: DOMString,
               bubbles: bool,
               cancelable: bool,
               detail: HandleValue)
               -> Root<CustomEvent> {
        let ev = CustomEvent::new_uninitialized(global);
        ev.InitCustomEvent(global.get_cx(), type_, bubbles, cancelable, detail);
        ev
    }
    #[allow(unsafe_code)]
    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &CustomEventBinding::CustomEventInit)
                       -> Fallible<Root<CustomEvent>> {
        Ok(CustomEvent::new(global,
                            type_,
                            init.parent.bubbles,
                            init.parent.cancelable,
                            unsafe { HandleValue::from_marked_location(&init.detail) }))
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
        let event = self.upcast::<Event>();
        if event.dispatching() {
            return;
        }

        self.detail.set(detail.get());
        event.InitEvent(type_, can_bubble, cancelable);
    }
}
