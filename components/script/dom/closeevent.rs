/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CloseEventBinding;
use dom::bindings::codegen::Bindings::CloseEventBinding::CloseEventMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::event::{Event, EventBubbles, EventCancelable};
use string_cache::Atom;

#[dom_struct]
pub struct CloseEvent {
    event: Event,
    wasClean: bool,
    code: u16,
    reason: DOMString,
}

impl CloseEvent {
    pub fn new_inherited(wasClean: bool, code: u16, reason: DOMString) -> CloseEvent {
        CloseEvent {
            event: Event::new_inherited(),
            wasClean: wasClean,
            code: code,
            reason: reason,
        }
    }

    pub fn new_uninitialized(global: GlobalRef) -> Root<CloseEvent> {
        reflect_dom_object(box CloseEvent::new_inherited(false, 0, DOMString::new()),
                           global,
                           CloseEventBinding::Wrap)
    }

    pub fn new(global: GlobalRef,
               type_: Atom,
               bubbles: EventBubbles,
               cancelable: EventCancelable,
               wasClean: bool,
               code: u16,
               reason: DOMString)
               -> Root<CloseEvent> {
        let event = box CloseEvent::new_inherited(wasClean, code, reason);
        let ev = reflect_dom_object(event, global, CloseEventBinding::Wrap);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_,
                             bool::from(bubbles),
                             bool::from(cancelable));
        }
        ev
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &CloseEventBinding::CloseEventInit)
                       -> Fallible<Root<CloseEvent>> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        Ok(CloseEvent::new(global,
                           Atom::from(type_),
                           bubbles,
                           cancelable,
                           init.wasClean,
                           init.code,
                           init.reason.clone()))
    }

}

impl CloseEventMethods for CloseEvent {
    // https://html.spec.whatwg.org/multipage/#dom-closeevent-wasclean
    fn WasClean(&self) -> bool {
        self.wasClean
    }

    // https://html.spec.whatwg.org/multipage/#dom-closeevent-code
    fn Code(&self) -> u16 {
        self.code
    }

    // https://html.spec.whatwg.org/multipage/#dom-closeevent-reason
    fn Reason(&self) -> DOMString {
        self.reason.clone()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
