/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::PopStateEventBinding;
use dom::bindings::codegen::Bindings::PopStateEventBinding::PopStateEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutHeapJSVal, Root};
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::event::Event;
use dom::globalscope::GlobalScope;
use js::jsapi::{HandleValue, JSContext};
use js::jsval::JSVal;
use servo_atoms::Atom;

// https://html.spec.whatwg.org/multipage/#the-popstateevent-interface
#[dom_struct]
pub struct PopStateEvent {
    event: Event,
    #[ignore_heap_size_of = "Defined in rust-mozjs"]
    state: MutHeapJSVal,
}

impl PopStateEvent {
    fn new_inherited() -> PopStateEvent {
        PopStateEvent {
            event: Event::new_inherited(),
            state: MutHeapJSVal::new(),
        }
    }

    pub fn new_uninitialized(global: &GlobalScope) -> Root<PopStateEvent> {
        reflect_dom_object(box PopStateEvent::new_inherited(),
                           global,
                           PopStateEventBinding::Wrap)
    }

    pub fn new(global: &GlobalScope,
               type_: Atom,
               bubbles: bool,
               cancelable: bool,
               state: HandleValue)
               -> Root<PopStateEvent> {
        let ev = PopStateEvent::new_uninitialized(global);
        ev.state.set(state.get());
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    #[allow(unsafe_code)]
    pub fn Constructor(global: &GlobalScope,
                       type_: DOMString,
                       init: &PopStateEventBinding::PopStateEventInit)
                       -> Fallible<Root<PopStateEvent>> {
        Ok(PopStateEvent::new(global,
                              Atom::from(type_),
                              init.parent.bubbles,
                              init.parent.cancelable,
                              unsafe { HandleValue::from_marked_location(&init.state) }))
    }
}

impl PopStateEventMethods for PopStateEvent {
    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-popstateevent-state
    unsafe fn State(&self, _cx: *mut JSContext) -> JSVal {
        self.state.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
