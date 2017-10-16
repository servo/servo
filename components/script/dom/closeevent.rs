/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CloseEventBinding;
use dom::bindings::codegen::Bindings::CloseEventBinding::CloseEventMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use servo_atoms::Atom;

#[dom_struct]
pub struct CloseEvent {
    event: Event,
    was_clean: bool,
    code: u16,
    reason: DOMString,
}

impl CloseEvent {
    pub fn new_inherited(was_clean: bool, code: u16, reason: DOMString) -> CloseEvent {
        CloseEvent {
            event: Event::new_inherited(),
            was_clean: was_clean,
            code: code,
            reason: reason,
        }
    }

    pub fn new_uninitialized(global: &GlobalScope) -> DomRoot<CloseEvent> {
        reflect_dom_object(Box::new(CloseEvent::new_inherited(false, 0, DOMString::new())),
                           global,
                           CloseEventBinding::Wrap)
    }

    pub fn new(global: &GlobalScope,
               type_: Atom,
               bubbles: EventBubbles,
               cancelable: EventCancelable,
               wasClean: bool,
               code: u16,
               reason: DOMString)
               -> DomRoot<CloseEvent> {
        let event = Box::new(CloseEvent::new_inherited(wasClean, code, reason));
        let ev = reflect_dom_object(event, global, CloseEventBinding::Wrap);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_,
                             bool::from(bubbles),
                             bool::from(cancelable));
        }
        ev
    }

    pub fn Constructor(global: &GlobalScope,
                       type_: DOMString,
                       init: &CloseEventBinding::CloseEventInit)
                       -> Fallible<DomRoot<CloseEvent>> {
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
        self.was_clean
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
