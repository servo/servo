/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::IDBVersionChangeEventBinding::{
    IDBVersionChangeEventInit, IDBVersionChangeEventMethods,
};
use crate::dom::bindings::import::module::HandleObject;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct IDBVersionChangeEvent {
    event: Event,
    old_version: u64,
    new_version: Option<u64>,
}

impl IDBVersionChangeEvent {
    pub fn new_inherited(old_version: u64, new_version: Option<u64>) -> IDBVersionChangeEvent {
        IDBVersionChangeEvent {
            event: Event::new_inherited(),
            old_version,
            new_version,
        }
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        old_version: u64,
        new_version: Option<u64>,
        can_gc: CanGc,
    ) -> DomRoot<IDBVersionChangeEvent> {
        Self::new_with_proto(
            global,
            None,
            type_,
            bool::from(bubbles),
            bool::from(cancelable),
            old_version,
            new_version,
            can_gc,
        )
    }

    #[expect(clippy::too_many_arguments)]
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        old_version: u64,
        new_version: Option<u64>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let ev = reflect_dom_object_with_proto(
            Box::new(IDBVersionChangeEvent::new_inherited(
                old_version,
                new_version,
            )),
            global,
            proto,
            can_gc,
        );
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    /// <https://w3c.github.io/IndexedDB/#fire-a-version-change-event>
    pub(crate) fn fire_version_change_event(
        global: &GlobalScope,
        target: &EventTarget,
        event_type: Atom,
        old_version: u64,
        new_version: Option<u64>,
        can_gc: CanGc,
    ) -> bool {
        // Step 1: Let event be the result of creating an event using IDBVersionChangeEvent.
        // Step 2: Set event’s type attribute to e.
        // Step 3: Set event’s bubbles and cancelable attributes to false.
        // Step 4: Set event’s oldVersion attribute to oldVersion.
        // Step 5: Set event’s newVersion attribute to newVersion.
        let event = IDBVersionChangeEvent::new(
            global,
            event_type,
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            old_version,
            new_version,
            can_gc,
        );

        // Step 6: Let legacyOutputDidListenersThrowFlag be false.
        let legacy_output_did_listeners_throw = Cell::new(false);
        // Step 7: Dispatch event at target with legacyOutputDidListenersThrowFlag.
        let _ = event
            .upcast::<Event>()
            .fire_with_legacy_output_did_listeners_throw(
                target,
                &legacy_output_did_listeners_throw,
                can_gc,
            );
        // Step 8: Return legacyOutputDidListenersThrowFlag.
        legacy_output_did_listeners_throw.get()
    }
}

impl IDBVersionChangeEventMethods<crate::DomTypeHolder> for IDBVersionChangeEvent {
    /// <https://w3c.github.io/IndexedDB/#dom-idbversionchangeevent-idbversionchangeevent>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &IDBVersionChangeEventInit,
    ) -> DomRoot<Self> {
        Self::new_with_proto(
            global,
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            init.oldVersion,
            init.newVersion,
            can_gc,
        )
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbversionchangeevent-oldversion>
    fn OldVersion(&self) -> u64 {
        self.old_version
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbversionchangeevent-newversion>
    fn GetNewVersion(&self) -> Option<u64> {
        self.new_version
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
