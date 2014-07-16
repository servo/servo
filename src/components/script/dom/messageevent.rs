/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MessageEventBinding;
use dom::bindings::codegen::InheritTypes::{EventCast, MessageEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::trace::Traceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::event::{Event, EventMethods, MessageEventTypeId};

use servo_util::str::DOMString;

use js::jsapi::JSContext;
use js::jsval::JSVal;

#[deriving(Encodable)]
pub struct MessageEvent {
    event: Event,
    data: Traceable<JSVal>,
    origin: DOMString,
    lastEventId: DOMString,
}

impl MessageEventDerived for Event {
    fn is_messageevent(&self) -> bool {
        self.type_id == MessageEventTypeId
    }
}

impl MessageEvent {
    pub fn new_inherited(data: JSVal, origin: DOMString, lastEventId: DOMString)
                         -> MessageEvent {
        MessageEvent {
            event: Event::new_inherited(MessageEventTypeId),
            data: Traceable::new(data),
            origin: origin,
            lastEventId: lastEventId,
        }
    }

    pub fn new(global: &GlobalRef, type_: DOMString,
               bubbles: bool, cancelable: bool,
               data: JSVal, origin: DOMString, lastEventId: DOMString)
               -> Temporary<MessageEvent> {
        let ev = reflect_dom_object(box MessageEvent::new_inherited(data, origin, lastEventId),
                                    global,
                                    MessageEventBinding::Wrap).root();
        let event: &JSRef<Event> = EventCast::from_ref(&*ev);
        event.InitEvent(type_, bubbles, cancelable);
        Temporary::from_rooted(&*ev)
    }

    pub fn Constructor(global: &GlobalRef,
                       type_: DOMString,
                       init: &MessageEventBinding::MessageEventInit)
                       -> Fallible<Temporary<MessageEvent>> {
        let ev = MessageEvent::new(global, type_, init.parent.bubbles, init.parent.cancelable,
                                   init.data, init.origin.clone(), init.lastEventId.clone());
        Ok(ev)
    }
}

pub trait MessageEventMethods {
    fn Data(&self, cx: *mut JSContext) -> JSVal;
    fn Origin(&self) -> DOMString;
    fn LastEventId(&self) -> DOMString;
}

impl<'a> MessageEventMethods for JSRef<'a, MessageEvent> {
    fn Data(&self, _cx: *mut JSContext) -> JSVal {
        *self.data
    }

    fn Origin(&self) -> DOMString {
        self.origin.clone()
    }

    fn LastEventId(&self) -> DOMString {
        self.lastEventId.clone()
    }
}

impl Reflectable for MessageEvent {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.event.reflector()
    }
}
