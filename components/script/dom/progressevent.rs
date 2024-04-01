/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::ProgressEventBinding;
use crate::dom::bindings::codegen::Bindings::ProgressEventBinding::ProgressEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct ProgressEvent {
    event: Event,
    length_computable: bool,
    loaded: u64,
    total: u64,
}

impl ProgressEvent {
    fn new_inherited(length_computable: bool, loaded: u64, total: u64) -> ProgressEvent {
        ProgressEvent {
            event: Event::new_inherited(),
            length_computable,
            loaded,
            total,
        }
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        length_computable: bool,
        loaded: u64,
        total: u64,
    ) -> DomRoot<ProgressEvent> {
        Self::new_with_proto(
            global,
            None,
            type_,
            can_bubble,
            cancelable,
            length_computable,
            loaded,
            total,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        length_computable: bool,
        loaded: u64,
        total: u64,
    ) -> DomRoot<ProgressEvent> {
        let ev = reflect_dom_object_with_proto(
            Box::new(ProgressEvent::new_inherited(
                length_computable,
                loaded,
                total,
            )),
            global,
            proto,
        );
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(can_bubble), bool::from(cancelable));
        }
        ev
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &ProgressEventBinding::ProgressEventInit,
    ) -> Fallible<DomRoot<ProgressEvent>> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        let ev = ProgressEvent::new_with_proto(
            global,
            proto,
            Atom::from(type_),
            bubbles,
            cancelable,
            init.lengthComputable,
            init.loaded,
            init.total,
        );
        Ok(ev)
    }
}

impl ProgressEventMethods for ProgressEvent {
    // https://xhr.spec.whatwg.org/#dom-progressevent-lengthcomputable
    fn LengthComputable(&self) -> bool {
        self.length_computable
    }

    // https://xhr.spec.whatwg.org/#dom-progressevent-loaded
    fn Loaded(&self) -> u64 {
        self.loaded
    }

    // https://xhr.spec.whatwg.org/#dom-progressevent-total
    fn Total(&self) -> u64 {
        self.total
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
