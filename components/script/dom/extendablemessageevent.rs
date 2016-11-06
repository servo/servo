/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ExtendableMessageEventBinding;
use dom::bindings::codegen::Bindings::ExtendableMessageEventBinding::ExtendableMessageEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::extendableevent::ExtendableEvent;
use dom::globalscope::GlobalScope;
use js::jsapi::{HandleValue, Heap, JSContext};
use js::jsval::JSVal;
use servo_atoms::Atom;
use std::default::Default;

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
               -> Root<ExtendableMessageEvent> {
        let mut ev = box ExtendableMessageEvent {
            event: ExtendableEvent::new_inherited(),
            data: Heap::default(),
            origin: origin,
            lastEventId: lastEventId,
        };
        ev.data.set(data.get());
        let ev = reflect_dom_object(ev, global, ExtendableMessageEventBinding::Wrap);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    pub fn Constructor(global: &GlobalScope,
                       type_: DOMString,
                       init: &ExtendableMessageEventBinding::ExtendableMessageEventInit)
                       -> Fallible<Root<ExtendableMessageEvent>> {
        rooted!(in(global.get_cx()) let data = init.data);
        let ev = ExtendableMessageEvent::new(global,
                                             Atom::from(type_),
                                             init.parent.parent.bubbles,
                                             init.parent.parent.cancelable,
                                             data.handle(),
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
