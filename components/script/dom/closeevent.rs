/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CloseEventBinding;
use dom::bindings::codegen::Bindings::CloseEventBinding::CloseEventMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::InheritTypes::EventCast;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventTypeId, EventBubbles, EventCancelable};
use script_task::ScriptChan;

use util::str::DOMString;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct CloseEvent {
    event: Event,
    wasClean: bool,
    code: u16,
    reason: DOMString,
}

impl CloseEvent {
    pub fn new_inherited(type_id: EventTypeId, wasClean: bool, code: u16,
                         reason: DOMString) -> CloseEvent {
        CloseEvent {
            event: Event::new_inherited(type_id),
            wasClean: wasClean,
            code: code,
            reason: reason,
        }
    }

    pub fn new(global: GlobalRef,
               type_: DOMString,
               bubbles: EventBubbles,
               cancelable: EventCancelable,
               wasClean: bool,
               code: u16,
               reason: DOMString) -> Root<CloseEvent> {
        let event = box CloseEvent::new_inherited(EventTypeId::CloseEvent,
                                                  wasClean, code, reason);
        let ev = reflect_dom_object(event, global, CloseEventBinding::Wrap);
        {
            let event = EventCast::from_ref(ev.r());
            event.InitEvent(type_,
                            bubbles == EventBubbles::Bubbles,
                            cancelable == EventCancelable::Cancelable);
        }
        ev
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &CloseEventBinding::CloseEventInit)
                       -> Fallible<Root<CloseEvent>> {
        let bubbles = if init.parent.bubbles { EventBubbles::Bubbles } else { EventBubbles::DoesNotBubble };
        let cancelable = if init.parent.cancelable {
            EventCancelable::Cancelable
        } else {
            EventCancelable::NotCancelable
        };
        Ok(CloseEvent::new(global, type_, bubbles, cancelable, init.wasClean,
                           init.code, init.reason.clone()))
    }
}

impl<'a> CloseEventMethods for &'a CloseEvent {
    // https://html.spec.whatwg.org/multipage/#dom-closeevent-wasclean
    fn WasClean(self) -> bool {
        self.wasClean
    }

    // https://html.spec.whatwg.org/multipage/#dom-closeevent-code
    fn Code(self) -> u16 {
        self.code
    }

    // https://html.spec.whatwg.org/multipage/#dom-closeevent-reason
    fn Reason(self) -> DOMString {
        self.reason.clone()
    }
}
