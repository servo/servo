/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::eventtarget::EventTarget;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::PopStateEventBinding;
use dom::bindings::codegen::Bindings::PopStateEventBinding::PopStateEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::event::Event;
use js::jsapi::{HandleValue, Heap, JSContext, RootedValue};
use js::jsval::JSVal;
use string_cache::Atom;
use util::str::DOMString;

// https://html.spec.whatwg.org/multipage/#the-popstateevent-interface
#[dom_struct]
pub struct PopStateEvent {
    event: Event,
    state: Heap<JSVal>,
}

impl PopStateEvent {
    pub fn new_uninitialized(global: GlobalRef) -> Root<PopStateEvent> {
        PopStateEvent::new_initialized(global,
                                       HandleValue::undefined())
    }

    pub fn new_initialized(global: GlobalRef,
                           state: HandleValue)
                           -> Root <PopStateEvent> {
        let mut ev = box PopStateEvent {
            event: Event::new_inherited(),
            state: Heap::default(),
        };
        ev.state.set(state.get());
        reflect_dom_object(ev, global, PopStateEventBinding::Wrap)
    }

    pub fn new(global: GlobalRef,
               type_: Atom,
               bubbles: bool,
               cancelable: bool,
               state: HandleValue)
               -> Root<PopStateEvent> {
        let ev = PopStateEvent::new_initialized(global, state);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &PopStateEventBinding::PopStateEventInit)
                       -> Fallible<Root<PopStateEvent>> {
        let data = RootedValue::new(global.get_cx(), init.state);
        let ev = PopStateEvent::new(global,
                                    Atom::from(type_),
                                    init.parent.bubbles,
                                    init.parent.cancelable,
                                    data.handle());
        Ok(ev)
    }
}

impl PopStateEvent {
    pub fn dispatch_jsval(target: &EventTarget,
                          scope: GlobalRef,
                          state: HandleValue) {
        let popstateevent = PopStateEvent::new(scope,
                                               Atom::from("popstateevent"),
                                               true,
                                               false,
                                               state);
        popstateevent.upcast::<Event>().fire(target);
    }
}

impl PopStateEventMethods for PopStateEvent {
    // https://html.spec.whatwg.org/multipage/#dom-popstateevent-state
    fn State(&self, _cx: *mut JSContext) -> JSVal {
        self.state.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
