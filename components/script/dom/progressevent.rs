/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::ProgressEventBinding;
use dom::bindings::codegen::Bindings::ProgressEventBinding::ProgressEventMethods;
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
pub struct ProgressEvent {
    event: Event,
    length_computable: bool,
    loaded: u64,
    total: u64
}

impl ProgressEvent {
    fn new_inherited(length_computable: bool, loaded: u64, total: u64) -> ProgressEvent {
        ProgressEvent {
            event: Event::new_inherited(),
            length_computable: length_computable,
            loaded: loaded,
            total: total
        }
    }
    pub fn new_uninitialized(global: &GlobalScope) -> DomRoot<ProgressEvent> {
        reflect_dom_object(Box::new(ProgressEvent::new_inherited(false, 0, 0)),
                           global,
                           ProgressEventBinding::Wrap)
    }
    pub fn new(global: &GlobalScope, type_: Atom,
               can_bubble: EventBubbles, cancelable: EventCancelable,
               length_computable: bool, loaded: u64, total: u64) -> DomRoot<ProgressEvent> {
        let ev = reflect_dom_object(Box::new(ProgressEvent::new_inherited(length_computable, loaded, total)),
                                    global,
                                    ProgressEventBinding::Wrap);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(can_bubble), bool::from(cancelable));
        }
        ev
    }
    pub fn Constructor(global: &GlobalScope,
                       type_: DOMString,
                       init: &ProgressEventBinding::ProgressEventInit)
                       -> Fallible<DomRoot<ProgressEvent>> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        let ev = ProgressEvent::new(global, Atom::from(type_), bubbles, cancelable,
                                    init.lengthComputable, init.loaded, init.total);
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
