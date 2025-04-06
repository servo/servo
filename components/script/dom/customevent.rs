/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::{HandleObject, HandleValue, MutableHandleValue};
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::CustomEventBinding;
use crate::dom::bindings::codegen::Bindings::CustomEventBinding::CustomEventMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::{CanGc, JSContext};

// https://dom.spec.whatwg.org/#interface-customevent
#[dom_struct]
pub(crate) struct CustomEvent {
    event: Event,
    #[ignore_malloc_size_of = "Defined in rust-mozjs"]
    detail: Heap<JSVal>,
}

impl CustomEvent {
    fn new_inherited() -> CustomEvent {
        CustomEvent {
            event: Event::new_inherited(),
            detail: Heap::default(),
        }
    }

    pub(crate) fn new_uninitialized(global: &GlobalScope, can_gc: CanGc) -> DomRoot<CustomEvent> {
        Self::new_uninitialized_with_proto(global, None, can_gc)
    }

    fn new_uninitialized_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<CustomEvent> {
        reflect_dom_object_with_proto(
            Box::new(CustomEvent::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        detail: HandleValue,
        can_gc: CanGc,
    ) -> DomRoot<CustomEvent> {
        let ev = CustomEvent::new_uninitialized_with_proto(global, proto, can_gc);
        ev.init_custom_event(type_, bubbles, cancelable, detail);
        ev
    }

    fn init_custom_event(
        &self,
        type_: Atom,
        can_bubble: bool,
        cancelable: bool,
        detail: HandleValue,
    ) {
        let event = self.upcast::<Event>();
        if event.dispatching() {
            return;
        }

        self.detail.set(detail.get());
        event.init_event(type_, can_bubble, cancelable);
    }
}

impl CustomEventMethods<crate::DomTypeHolder> for CustomEvent {
    // https://dom.spec.whatwg.org/#dom-customevent-customevent
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: RootedTraceableBox<CustomEventBinding::CustomEventInit>,
    ) -> DomRoot<CustomEvent> {
        let event = CustomEvent::new(
            global,
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            init.detail.handle(),
            can_gc,
        );
        event.upcast::<Event>().set_composed(init.parent.composed);
        event
    }

    // https://dom.spec.whatwg.org/#dom-customevent-detail
    fn Detail(&self, _cx: JSContext, mut retval: MutableHandleValue) {
        retval.set(self.detail.get())
    }

    // https://dom.spec.whatwg.org/#dom-customevent-initcustomevent
    fn InitCustomEvent(
        &self,
        _cx: JSContext,
        type_: DOMString,
        can_bubble: bool,
        cancelable: bool,
        detail: HandleValue,
    ) {
        self.init_custom_event(Atom::from(type_), can_bubble, cancelable, detail)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
