/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::ProgressEventBinding;
use dom::bindings::codegen::Bindings::ProgressEventBinding::ProgressEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, ProgressEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::event::{Event, ProgressEventTypeId};
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct ProgressEvent {
    event: Event,
    length_computable: bool,
    loaded: u64,
    total: u64
}

impl ProgressEventDerived for Event {
    fn is_progressevent(&self) -> bool {
        self.type_id == ProgressEventTypeId
    }
}

impl ProgressEvent {
    pub fn new_inherited(length_computable: bool, loaded: u64, total: u64) -> ProgressEvent {
        ProgressEvent {
            event: Event::new_inherited(ProgressEventTypeId),
            length_computable: length_computable,
            loaded: loaded,
            total: total
        }
    }
    pub fn new(global: &GlobalRef, type_: DOMString,
               can_bubble: bool, cancelable: bool,
               length_computable: bool, loaded: u64, total: u64) -> Temporary<ProgressEvent> {
        let ev = reflect_dom_object(box ProgressEvent::new_inherited(length_computable, loaded, total),
                                    global,
                                    ProgressEventBinding::Wrap).root();
        let event: JSRef<Event> = EventCast::from_ref(*ev);
        event.InitEvent(type_, can_bubble, cancelable);
        Temporary::from_rooted(*ev)
    }
    pub fn Constructor(global: &GlobalRef,
                       type_: DOMString,
                       init: &ProgressEventBinding::ProgressEventInit)
                       -> Fallible<Temporary<ProgressEvent>> {
        let ev = ProgressEvent::new(global, type_, init.parent.bubbles, init.parent.cancelable,
                                    init.lengthComputable, init.loaded, init.total);
        Ok(ev)
    }
}

impl<'a> ProgressEventMethods for JSRef<'a, ProgressEvent> {
    fn LengthComputable(self) -> bool {
        self.length_computable
    }
    fn Loaded(self) -> u64{
        self.loaded
    }
    fn Total(self) -> u64 {
        self.total
    }
}

impl Reflectable for ProgressEvent {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.event.reflector()
    }
}
