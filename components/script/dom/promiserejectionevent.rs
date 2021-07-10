/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::PromiseRejectionEventBinding;
use crate::dom::bindings::codegen::Bindings::PromiseRejectionEventBinding::PromiseRejectionEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_runtime::JSContext;
use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::HandleValue;
use servo_atoms::Atom;
use std::rc::Rc;

#[dom_struct]
pub struct PromiseRejectionEvent {
    event: Event,
    #[ignore_malloc_size_of = "Rc"]
    promise: Rc<Promise>,
    #[ignore_malloc_size_of = "Defined in rust-mozjs"]
    reason: Heap<JSVal>,
}

impl PromiseRejectionEvent {
    #[allow(unrooted_must_root)]
    fn new_inherited(promise: Rc<Promise>) -> Self {
        PromiseRejectionEvent {
            event: Event::new_inherited(),
            promise,
            reason: Heap::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        promise: Rc<Promise>,
        reason: HandleValue,
    ) -> DomRoot<Self> {
        let ev = reflect_dom_object(
            Box::new(PromiseRejectionEvent::new_inherited(promise)),
            global,
        );

        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));

            ev.reason.set(reason.get());
        }
        ev
    }

    #[allow(unrooted_must_root, non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        type_: DOMString,
        init: RootedTraceableBox<PromiseRejectionEventBinding::PromiseRejectionEventInit>,
    ) -> Fallible<DomRoot<Self>> {
        let reason = init.reason.handle();
        let promise = init.promise.clone();
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);

        let event = PromiseRejectionEvent::new(
            global,
            Atom::from(type_),
            bubbles,
            cancelable,
            promise,
            reason,
        );
        Ok(event)
    }
}

impl PromiseRejectionEventMethods for PromiseRejectionEvent {
    // https://html.spec.whatwg.org/multipage/#dom-promiserejectionevent-promise
    fn Promise(&self) -> Rc<Promise> {
        self.promise.clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-promiserejectionevent-reason
    fn Reason(&self, _cx: JSContext) -> JSVal {
        self.reason.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
