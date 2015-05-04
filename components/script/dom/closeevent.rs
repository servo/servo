/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::CloseEventBinding;
use dom::bindings::codegen::Bindings::CloseEventBinding::CloseEventMethods;
use dom::bindings::codegen::InheritTypes::EventCast;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef,Temporary, Rootable};
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventTypeId, EventBubbles, EventCancelable};
use script_task::ScriptChan;
use std::borrow::ToOwned;
use std::cell::Cell;
use util::str::DOMString;


#[dom_struct]
pub struct CloseEvent{
    event: Event,
    wasClean: Cell<bool>,
    code: Cell<u16>,
    reason: DOMRefCell<DOMString>
}

impl CloseEvent{
    pub fn new_inherited(type_id: EventTypeId) -> CloseEvent{
        CloseEvent{
            event: Event::new_inherited(type_id),
            wasClean: Cell::new(true),
            code: Cell::new(0),
            reason: DOMRefCell::new("".to_owned())
        }
    }

    pub fn new(global: GlobalRef,
               type_: DOMString,
               bubbles: EventBubbles,
               cancelable: EventCancelable,
               wasClean: bool,
               code: u16,
               reason: DOMString) -> Temporary<CloseEvent> {
        let ev = reflect_dom_object(box CloseEvent::new_inherited(EventTypeId::CloseEvent),
                                    global,
                                    CloseEventBinding::Wrap);
        let ev = ev.root();
        let event: JSRef<Event> = EventCast::from_ref(ev.r());
        event.InitEvent(type_,
                        bubbles == EventBubbles::Bubbles,
                        cancelable == EventCancelable::Cancelable);
        let ev = ev.r();
        ev.wasClean.set(wasClean);
        ev.code.set(code);
        *ev.reason.borrow_mut() = reason;
        Temporary::from_rooted(ev)
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &CloseEventBinding::CloseEventInit) -> Fallible<Temporary<CloseEvent>> {
        let clean_status = init.wasClean.unwrap_or(true);
        let cd = init.code.unwrap_or(0);
        let rsn = match init.reason.as_ref() {
            Some(reason) => reason.clone(),
            None => "".to_owned(),
        };
        let bubbles = if init.parent.bubbles { EventBubbles::Bubbles } else { EventBubbles::DoesNotBubble };
        let cancelable = if init.parent.cancelable { EventCancelable::Cancelable } else { EventCancelable::NotCancelable };
        Ok(CloseEvent::new(global, type_, bubbles, cancelable, clean_status, cd, rsn))
    }
}

impl<'a> CloseEventMethods for JSRef<'a, CloseEvent>{
    fn WasClean(self) -> bool {
        self.wasClean.get()
    }

    fn Code(self) -> u16 {
        self.code.get()
    }

    fn Reason(self) -> DOMString {
        let reason = self.reason.borrow();
        reason.clone()
    }
}
