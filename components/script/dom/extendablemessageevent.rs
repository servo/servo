/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ExtendableMessageEventBinding;
use dom::bindings::codegen::Bindings::ExtendableMessageEventBinding::ExtendableMessageEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::bindings::trace::RootedTraceableBox;
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::extendableevent::ExtendableEvent;
use dom::globalscope::GlobalScope;
use dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use dom_struct::dom_struct;
use js::jsapi::{HandleValue, Heap, JSContext};
use js::jsval::JSVal;
use servo_atoms::Atom;

#[dom_struct]
pub struct ExtendableMessageEvent {
    event: ExtendableEvent,
    data: Heap<JSVal>,
    origin: DOMString,
    lastEventId: DOMString,
}

impl ExtendableMessageEvent {
    pub fn new(global: &GlobalScope, type_: Atom,
               bubbles: bool, cancelable: bool,
               data: HandleValue, origin: DOMString, lastEventId: DOMString)
               -> DomRoot<ExtendableMessageEvent> {
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

    pub fn Constructor(worker: &ServiceWorkerGlobalScope,
                       type_: DOMString,
                       init: RootedTraceableBox<ExtendableMessageEventBinding::ExtendableMessageEventInit>)
                       -> Fallible<DomRoot<ExtendableMessageEvent>> {
        let global = worker.upcast::<GlobalScope>();
        let ev = ExtendableMessageEvent::new(global,
                                             Atom::from(type_),
                                             init.parent.parent.bubbles,
                                             init.parent.parent.cancelable,
                                             init.data.handle(),
                                             init.origin.clone().unwrap(),
                                             init.lastEventId.clone().unwrap());
        Ok(ev)
    }
}

impl ExtendableMessageEvent {
    pub fn dispatch_jsval(target: &EventTarget,
                          scope: &GlobalScope,
                          message: HandleValue) {
        let Extendablemessageevent = ExtendableMessageEvent::new(
            scope, atom!("message"), false, false, message,
            DOMString::new(), DOMString::new());
        Extendablemessageevent.upcast::<Event>().fire(target);
    }
}

impl ExtendableMessageEventMethods for ExtendableMessageEvent {
    #[allow(unsafe_code)]
    // https://w3c.github.io/ServiceWorker/#extendablemessage-event-data-attribute
    unsafe fn Data(&self, _cx: *mut JSContext) -> JSVal {
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
