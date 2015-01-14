/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::MessageEventBinding;
use dom::bindings::codegen::Bindings::MessageEventBinding::MessageEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, MessageEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventTypeId};
use dom::eventtarget::{EventTarget, EventTargetHelpers};

use servo_util::str::DOMString;

use js::jsapi::JSContext;
use js::jsval::{JSVal, UndefinedValue};

#[dom_struct]
pub struct MessageEvent {
    event: Event,
    data: JSVal,
    origin: DOMString,
    lastEventId: DOMString,
}

impl MessageEventDerived for Event {
    fn is_messageevent(&self) -> bool {
        *self.type_id() == EventTypeId::MessageEvent
    }
}

impl MessageEvent {
    fn new_inherited(data: JSVal, origin: DOMString, lastEventId: DOMString)
                         -> MessageEvent {
        MessageEvent {
            event: Event::new_inherited(EventTypeId::MessageEvent),
            data: data,
            origin: origin,
            lastEventId: lastEventId,
        }
    }

    pub fn new_uninitialized(global: GlobalRef) -> Temporary<MessageEvent> {
        MessageEvent::new_initialized(global, UndefinedValue(), "".into_string(), "".into_string())
    }

    pub fn new_initialized(global: GlobalRef, data: JSVal, origin: DOMString, lastEventId: DOMString) -> Temporary<MessageEvent> {
        reflect_dom_object(box MessageEvent::new_inherited(data, origin, lastEventId),
        global,
        MessageEventBinding::Wrap)
    }

    pub fn new(global: GlobalRef, type_: DOMString,
               bubbles: bool, cancelable: bool,
               data: JSVal, origin: DOMString, lastEventId: DOMString)
               -> Temporary<MessageEvent> {
        let ev = MessageEvent::new_initialized(global, data, origin, lastEventId).root();
        let event: JSRef<Event> = EventCast::from_ref(ev.r());
        event.InitEvent(type_, bubbles, cancelable);
        Temporary::from_rooted(ev.r())
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &MessageEventBinding::MessageEventInit)
                       -> Fallible<Temporary<MessageEvent>> {
        let ev = MessageEvent::new(global, type_, init.parent.bubbles, init.parent.cancelable,
                                   init.data, init.origin.clone(), init.lastEventId.clone());
        Ok(ev)
    }
}

impl MessageEvent {
    pub fn dispatch_jsval(target: JSRef<EventTarget>,
                          scope: GlobalRef,
                          message: JSVal) {
        let messageevent = MessageEvent::new(
            scope, "message".into_string(), false, false, message,
            "".into_string(), "".into_string()).root();
        let event: JSRef<Event> = EventCast::from_ref(messageevent.r());
        target.dispatch_event(event);
    }
}

impl<'a> MessageEventMethods for JSRef<'a, MessageEvent> {
    fn Data(self, _cx: *mut JSContext) -> JSVal {
        self.data
    }

    fn Origin(self) -> DOMString {
        self.origin.clone()
    }

    fn LastEventId(self) -> DOMString {
        self.lastEventId.clone()
    }
}

