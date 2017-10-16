/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CustomEventBinding;
use dom::bindings::codegen::Bindings::CustomEventBinding::CustomEventMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::bindings::trace::RootedTraceableBox;
use dom::event::Event;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use js::jsapi::{Heap, HandleValue, JSContext};
use js::jsval::JSVal;
use servo_atoms::Atom;

// https://dom.spec.whatwg.org/#interface-customevent
#[dom_struct]
pub struct CustomEvent {
    event: Event,
    #[ignore_heap_size_of = "Defined in rust-mozjs"]
    detail: Heap<JSVal>,
}

impl CustomEvent {
    fn new_inherited() -> CustomEvent {
        CustomEvent {
            event: Event::new_inherited(),
            detail: Heap::default(),
        }
    }

    pub fn new_uninitialized(global: &GlobalScope) -> DomRoot<CustomEvent> {
        reflect_dom_object(Box::new(CustomEvent::new_inherited()),
                           global,
                           CustomEventBinding::Wrap)
    }
    pub fn new(global: &GlobalScope,
               type_: Atom,
               bubbles: bool,
               cancelable: bool,
               detail: HandleValue)
               -> DomRoot<CustomEvent> {
        let ev = CustomEvent::new_uninitialized(global);
        ev.init_custom_event(type_, bubbles, cancelable, detail);
        ev
    }

    #[allow(unsafe_code)]
    pub fn Constructor(global: &GlobalScope,
                       type_: DOMString,
                       init: RootedTraceableBox<CustomEventBinding::CustomEventInit>)
                       -> Fallible<DomRoot<CustomEvent>> {
        Ok(CustomEvent::new(global,
                            Atom::from(type_),
                            init.parent.bubbles,
                            init.parent.cancelable,
                            init.detail.handle()))
    }

    fn init_custom_event(&self,
                         type_: Atom,
                         can_bubble: bool,
                         cancelable: bool,
                         detail: HandleValue) {
        let event = self.upcast::<Event>();
        if event.dispatching() {
            return;
        }

        self.detail.set(detail.get());
        event.init_event(type_, can_bubble, cancelable);
    }
}

impl CustomEventMethods for CustomEvent {
    #[allow(unsafe_code)]
    // https://dom.spec.whatwg.org/#dom-customevent-detail
    unsafe fn Detail(&self, _cx: *mut JSContext) -> JSVal {
        self.detail.get()
    }

    #[allow(unsafe_code)]
    // https://dom.spec.whatwg.org/#dom-customevent-initcustomevent
    unsafe fn InitCustomEvent(&self,
                       _cx: *mut JSContext,
                       type_: DOMString,
                       can_bubble: bool,
                       cancelable: bool,
                       detail: HandleValue) {
        self.init_custom_event(Atom::from(type_), can_bubble, cancelable, detail)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
