/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::{HandleObject, HandleValue, MutableHandleValue};
use script_bindings::reflector::reflect_dom_object_with_proto;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::PopStateEventBinding;
use crate::dom::bindings::codegen::Bindings::PopStateEventBinding::PopStateEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::window::Window;

// https://html.spec.whatwg.org/multipage/#the-popstateevent-interface
#[dom_struct]
pub(crate) struct PopStateEvent {
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

    fn new_uninitialized(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
    ) -> DomRoot<PopStateEvent> {
        reflect_dom_object_with_proto(cx, Box::new(PopStateEvent::new_inherited()), window, proto)
    }

    fn new(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        state: HandleValue,
    ) -> DomRoot<PopStateEvent> {
        let ev = PopStateEvent::new_uninitialized(cx, window, proto);
        ev.state.set(state.get());
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    pub(crate) fn dispatch_jsval(
        cx: &mut js::context::JSContext,
        target: &EventTarget,
        window: &Window,
        state: HandleValue,
    ) {
        let event = PopStateEvent::new(cx, window, None, atom!("popstate"), false, false, state);
        event.upcast::<Event>().fire(cx, target);
    }
}

impl PopStateEventMethods<crate::DomTypeHolder> for PopStateEvent {
    /// <https://html.spec.whatwg.org/multipage/#popstateevent>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: RootedTraceableBox<PopStateEventBinding::PopStateEventInit>,
    ) -> Fallible<DomRoot<PopStateEvent>> {
        Ok(PopStateEvent::new(
            cx,
            window,
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            init.state.handle(),
        ))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-popstateevent-state>
    fn State(&self, mut retval: MutableHandleValue) {
        retval.set(self.state.get())
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
