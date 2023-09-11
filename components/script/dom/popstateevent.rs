/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::{HandleObject, HandleValue};
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::PopStateEventBinding;
use crate::dom::bindings::codegen::Bindings::PopStateEventBinding::PopStateEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::window::Window;
use crate::script_runtime::JSContext;

// https://html.spec.whatwg.org/multipage/#the-popstateevent-interface
#[dom_struct]
pub struct PopStateEvent {
    event: Event,
    #[ignore_malloc_size_of = "Defined in rust-mozjs"]
    state: Heap<JSVal>,
}

impl PopStateEvent {
    fn new_inherited() -> PopStateEvent {
        PopStateEvent {
            event: Event::new_inherited(),
            state: Heap::default(),
        }
    }

    fn new_uninitialized(window: &Window, proto: Option<HandleObject>) -> DomRoot<PopStateEvent> {
        reflect_dom_object_with_proto(Box::new(PopStateEvent::new_inherited()), window, proto)
    }

    fn new(
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        state: HandleValue,
    ) -> DomRoot<PopStateEvent> {
        let ev = PopStateEvent::new_uninitialized(window, proto);
        ev.state.set(state.get());
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: RootedTraceableBox<PopStateEventBinding::PopStateEventInit>,
    ) -> Fallible<DomRoot<PopStateEvent>> {
        Ok(PopStateEvent::new(
            window,
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            init.state.handle(),
        ))
    }

    pub fn dispatch_jsval(target: &EventTarget, window: &Window, state: HandleValue) {
        let event = PopStateEvent::new(window, None, atom!("popstate"), false, false, state);
        event.upcast::<Event>().fire(target);
    }
}

impl PopStateEventMethods for PopStateEvent {
    // https://html.spec.whatwg.org/multipage/#dom-popstateevent-state
    fn State(&self, _cx: JSContext) -> JSVal {
        self.state.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
