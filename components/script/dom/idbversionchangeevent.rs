/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::IDBVersionChangeEventBinding::{
    IDBVersionChangeEventInit, IDBVersionChangeEventMethods,
};
use crate::dom::bindings::import::module::HandleObject;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{
    reflect_dom_object, reflect_dom_object_with_proto, DomObject,
};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;

#[dom_struct]
pub struct IDBVersionChangeEvent {
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
    ) -> DomRoot<IDBVersionChangeEvent> {
        Self::new_with_proto(global, type_, bool::from(bubbles), bool::from(cancelable), old_version, new_version)
    }

    fn new_with_proto(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        old_version: u64,
        new_version: Option<u64>,
    ) -> DomRoot<Self> {
        let ev = reflect_dom_object(
            Box::new(IDBVersionChangeEvent::new_inherited(
                old_version,
                new_version,
            )),
            global,
        );
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &IDBVersionChangeEventInit,
    ) -> DomRoot<Self> {
        Self::new_with_proto(&window.global(), Atom::from(type_), init.parent.bubbles, init.parent.cancelable, init.oldVersion, init.newVersion)
    }
}

impl IDBVersionChangeEventMethods for IDBVersionChangeEvent {
    // https://www.w3.org/TR/IndexedDB-2/#dom-idbversionchangeevent-oldversion
    fn OldVersion(&self) -> u64 {
        self.old_version
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbversionchangeevent-newversion
    fn GetNewVersion(&self) -> Option<u64> {
        self.new_version
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
