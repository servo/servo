/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::CloseEventBinding;
use crate::dom::bindings::codegen::Bindings::CloseEventBinding::CloseEventMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct CloseEvent {
    event: Event,
    was_clean: bool,
    code: u16,
    reason: DOMString,
}

#[allow(non_snake_case)]
impl CloseEvent {
    pub fn new_inherited(was_clean: bool, code: u16, reason: DOMString) -> CloseEvent {
        CloseEvent {
            event: Event::new_inherited(),
            was_clean,
            code,
            reason,
        }
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        wasClean: bool,
        code: u16,
        reason: DOMString,
    ) -> DomRoot<CloseEvent> {
        Self::new_with_proto(
            global, None, type_, bubbles, cancelable, wasClean, code, reason,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        wasClean: bool,
        code: u16,
        reason: DOMString,
    ) -> DomRoot<CloseEvent> {
        let event = Box::new(CloseEvent::new_inherited(wasClean, code, reason));
        let ev = reflect_dom_object_with_proto(event, global, proto);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        }
        ev
    }

    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &CloseEventBinding::CloseEventInit,
    ) -> Fallible<DomRoot<CloseEvent>> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        Ok(CloseEvent::new_with_proto(
            global,
            proto,
            Atom::from(type_),
            bubbles,
            cancelable,
            init.wasClean,
            init.code,
            init.reason.clone(),
        ))
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
