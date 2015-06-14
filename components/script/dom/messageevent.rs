/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::MessageEventBinding;
use dom::bindings::codegen::Bindings::MessageEventBinding::MessageEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, MessageEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventHelpers, EventTypeId};
use dom::eventtarget::EventTarget;

use util::str::DOMString;

use js::jsapi::{JSContext, Heap, HandleValue};
use js::jsval::{JSVal, UndefinedValue};

use std::borrow::ToOwned;
use std::default::Default;

#[dom_struct]
pub struct MessageEvent {
    event: Event,
    data: Heap<JSVal>,
    origin: DOMString,
    lastEventId: DOMString,
}

impl MessageEventDerived for Event {
    fn is_messageevent(&self) -> bool {
        *self.type_id() == EventTypeId::MessageEvent
    }
}

impl MessageEvent {
    fn new_inherited(data: HandleValue, origin: DOMString, lastEventId: DOMString)
                         -> Box<MessageEvent> {
        let mut ret = box MessageEvent {
            event: Event::new_inherited(EventTypeId::MessageEvent),
            data: Heap::default(),
            origin: origin,
            lastEventId: lastEventId,
        };
        ret.data.set(data.get());
        ret
    }

    pub fn new_uninitialized(global: GlobalRef) -> Root<MessageEvent> {
        MessageEvent::new_initialized(global, HandleValue::undefined(), "".to_owned(), "".to_owned())
    }

    pub fn new_initialized(global: GlobalRef,
                           data: HandleValue,
                           origin: DOMString,
                           lastEventId: DOMString) -> Root<MessageEvent> {
        reflect_dom_object(MessageEvent::new_inherited(data, origin, lastEventId),
        global,
        MessageEventBinding::Wrap)
    }

    pub fn new(global: GlobalRef, type_: DOMString,
               bubbles: bool, cancelable: bool,
               data: HandleValue, origin: DOMString, lastEventId: DOMString)
               -> Root<MessageEvent> {
        let ev = MessageEvent::new_initialized(global, data, origin, lastEventId);
        {
            let event = EventCast::from_ref(ev.r());
            event.InitEvent(type_, bubbles, cancelable);
        }
        ev
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &MessageEventBinding::MessageEventInit)
                       -> Fallible<Root<MessageEvent>> {
        let ev = MessageEvent::new(global, type_, init.parent.bubbles, init.parent.cancelable,
                                   init.data, init.origin.clone(), init.lastEventId.clone());
        Ok(ev)
    }
}

impl MessageEvent {
    pub fn dispatch_jsval(target: &EventTarget,
                          scope: GlobalRef,
                          message: HandleValue) {
        let messageevent = MessageEvent::new(
            scope, "message".to_owned(), false, false, message,
            "".to_owned(), "".to_owned());
        let event = EventCast::from_ref(messageevent.r());
        event.fire(target);
    }
}

impl<'a> MessageEventMethods for &'a MessageEvent {
    fn Data(self, _cx: *mut JSContext) -> JSVal {
        self.data.get()
    }

    fn Origin(self) -> DOMString {
        self.origin.clone()
    }

    fn LastEventId(self) -> DOMString {
        self.lastEventId.clone()
    }
}

