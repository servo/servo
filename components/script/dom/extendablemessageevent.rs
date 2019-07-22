/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::ExtendableMessageEventBinding;
use crate::dom::bindings::codegen::Bindings::ExtendableMessageEventBinding::ExtendableMessageEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::extendableevent::ExtendableEvent;
use crate::dom::globalscope::GlobalScope;
use crate::dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use crate::script_runtime::JSContext;
use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::HandleValue;
use servo_atoms::Atom;

#[dom_struct]
pub struct ExtendableMessageEvent {
    event: ExtendableEvent,
    #[ignore_malloc_size_of = "mozjs"]
    data: Heap<JSVal>,
    origin: DOMString,
    lastEventId: DOMString,
}

impl ExtendableMessageEvent {
    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        data: HandleValue,
        origin: DOMString,
        lastEventId: DOMString,
    ) -> DomRoot<ExtendableMessageEvent> {
        let ev = Box::new(ExtendableMessageEvent {
            event: ExtendableEvent::new_inherited(),
            data: Heap::default(),
            origin: origin,
            lastEventId: lastEventId,
        });
        let ev = reflect_dom_object(ev, global, ExtendableMessageEventBinding::Wrap);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev.data.set(data.get());

        ev
    }

    pub fn Constructor(
        worker: &ServiceWorkerGlobalScope,
        type_: DOMString,
        init: RootedTraceableBox<ExtendableMessageEventBinding::ExtendableMessageEventInit>,
    ) -> Fallible<DomRoot<ExtendableMessageEvent>> {
        let global = worker.upcast::<GlobalScope>();
        let ev = ExtendableMessageEvent::new(
            global,
            Atom::from(type_),
            init.parent.parent.bubbles,
            init.parent.parent.cancelable,
            init.data.handle(),
            init.origin.clone().unwrap(),
            init.lastEventId.clone().unwrap(),
        );
        Ok(ev)
    }
}

impl ExtendableMessageEvent {
    pub fn dispatch_jsval(target: &EventTarget, scope: &GlobalScope, message: HandleValue) {
        let Extendablemessageevent = ExtendableMessageEvent::new(
            scope,
            atom!("message"),
            false,
            false,
            message,
            DOMString::new(),
            DOMString::new(),
        );
        Extendablemessageevent.upcast::<Event>().fire(target);
    }
}

impl ExtendableMessageEventMethods for ExtendableMessageEvent {
    // https://w3c.github.io/ServiceWorker/#extendablemessage-event-data-attribute
    fn Data(&self, _cx: JSContext) -> JSVal {
        self.data.get()
    }

    // https://w3c.github.io/ServiceWorker/#extendablemessage-event-origin-attribute
    fn Origin(&self) -> DOMString {
        self.origin.clone()
    }

    // https://w3c.github.io/ServiceWorker/#extendablemessage-event-lasteventid-attribute
    fn LastEventId(&self) -> DOMString {
        self.lastEventId.clone()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
