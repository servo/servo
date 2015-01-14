/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::ErrorEventBinding;
use dom::bindings::codegen::Bindings::ErrorEventBinding::ErrorEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, ErrorEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary, MutHeap};
use js::jsapi::JSContext;
use dom::bindings::trace::JSTraceable;

use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventTypeId, EventBubbles, EventCancelable};
use servo_util::str::DOMString;

use dom::bindings::cell::DOMRefCell;
use std::cell::{Cell};
use js::jsval::{JSVal, NullValue};

#[dom_struct]
pub struct ErrorEvent {
    event: Event,
    message: DOMRefCell<DOMString>,
    filename: DOMRefCell<DOMString>,
    lineno: Cell<u32>,
    colno: Cell<u32>,
    error: MutHeap<JSVal>,
}

impl ErrorEventDerived for Event {
    fn is_errorevent(&self) -> bool {
        *self.type_id() == EventTypeId::ErrorEvent
    }
}

impl ErrorEvent {
    fn new_inherited(type_id: EventTypeId) -> ErrorEvent {
        ErrorEvent {
            event: Event::new_inherited(type_id),
            message: DOMRefCell::new("".into_string()),
            filename: DOMRefCell::new("".into_string()),
            lineno: Cell::new(0),
            colno: Cell::new(0),
            error: MutHeap::new(NullValue())
        }
    }

    pub fn new_uninitialized(global: GlobalRef) -> Temporary<ErrorEvent> {
        reflect_dom_object(box ErrorEvent::new_inherited(EventTypeId::ErrorEvent),
                           global,
                           ErrorEventBinding::Wrap)
    }

    pub fn new(global: GlobalRef,
               type_: DOMString,
               bubbles: EventBubbles,
               cancelable: EventCancelable,
               message: DOMString,
               filename: DOMString,
               lineno: u32,
               colno: u32,
               error: JSVal) -> Temporary<ErrorEvent> {
        let ev = ErrorEvent::new_uninitialized(global).root();
        let event: JSRef<Event> = EventCast::from_ref(ev.r());
        event.InitEvent(type_, bubbles == EventBubbles::Bubbles,
                        cancelable == EventCancelable::Cancelable);
        *ev.r().message.borrow_mut() = message;
        *ev.r().filename.borrow_mut() = filename;
        ev.r().lineno.set(lineno);
        ev.r().colno.set(colno);
        ev.r().error.set(error);
        Temporary::from_rooted(ev.r())
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &ErrorEventBinding::ErrorEventInit) -> Fallible<Temporary<ErrorEvent>>{
        let msg = match init.message.as_ref() {
            Some(message) => message.clone(),
            None => "".into_string(),
        };

        let file_name = match init.filename.as_ref() {
            None => "".into_string(),
            Some(filename) => filename.clone(),
        };

        let line_num = init.lineno.unwrap_or(0);

        let col_num = init.colno.unwrap_or(0);

        let bubbles = if init.parent.bubbles { EventBubbles::Bubbles } else { EventBubbles::DoesNotBubble };

        let cancelable = if init.parent.cancelable { EventCancelable::Cancelable } else { EventCancelable::NotCancelable };

        let event = ErrorEvent::new(global, type_,
                                bubbles, cancelable,
                                msg, file_name,
                                line_num, col_num, init.error);
        Ok(event)
    }

}

impl<'a> ErrorEventMethods for JSRef<'a, ErrorEvent> {
    fn Lineno(self) -> u32 {
        self.lineno.get()
    }

    fn Colno(self) -> u32 {
        self.colno.get()
    }

    fn Message(self) -> DOMString {
        self.message.borrow().clone()
    }

    fn Filename(self) -> DOMString {
        self.filename.borrow().clone()
    }

    fn Error(self, _cx: *mut JSContext) -> JSVal {
        self.error.get()
    }

}
