/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::ProgressEventBinding;
use dom::bindings::codegen::Bindings::ProgressEventBinding::ProgressEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, ProgressEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventTypeId, EventBubbles, EventCancelable};
use util::str::DOMString;

#[dom_struct]
pub struct ProgressEvent {
    event: Event,
    length_computable: bool,
    loaded: u64,
    total: u64
}

impl ProgressEventDerived for Event {
    fn is_progressevent(&self) -> bool {
        *self.type_id() == EventTypeId::ProgressEvent
    }
}

impl ProgressEvent {
    fn new_inherited(length_computable: bool, loaded: u64, total: u64) -> ProgressEvent {
        ProgressEvent {
            event: Event::new_inherited(EventTypeId::ProgressEvent),
            length_computable: length_computable,
            loaded: loaded,
            total: total
        }
    }
    pub fn new(global: GlobalRef, type_: DOMString,
               can_bubble: EventBubbles, cancelable: EventCancelable,
               length_computable: bool, loaded: u64, total: u64) -> Root<ProgressEvent> {
        let ev = reflect_dom_object(box ProgressEvent::new_inherited(length_computable, loaded, total),
                                    global,
                                    ProgressEventBinding::Wrap);
        {
            let event = EventCast::from_ref(ev.r());
            event.InitEvent(type_, can_bubble == EventBubbles::Bubbles, cancelable == EventCancelable::Cancelable);
        }
        ev
    }
    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &ProgressEventBinding::ProgressEventInit)
                       -> Fallible<Root<ProgressEvent>> {
        let bubbles = if init.parent.bubbles {EventBubbles::Bubbles} else {EventBubbles::DoesNotBubble};
        let cancelable = if init.parent.cancelable {EventCancelable::Cancelable}
                         else {EventCancelable::NotCancelable};
        let ev = ProgressEvent::new(global, type_, bubbles, cancelable,
                                    init.lengthComputable, init.loaded, init.total);
        Ok(ev)
    }
}

impl<'a> ProgressEventMethods for &'a ProgressEvent {
    // https://xhr.spec.whatwg.org/#dom-progressevent-lengthcomputable
    fn LengthComputable(self) -> bool {
        self.length_computable
    }

    // https://xhr.spec.whatwg.org/#dom-progressevent-loaded
    fn Loaded(self) -> u64 {
        self.loaded
    }

    // https://xhr.spec.whatwg.org/#dom-progressevent-total
    fn Total(self) -> u64 {
        self.total
    }
}
