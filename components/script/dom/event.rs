/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding;
use dom::bindings::codegen::Bindings::EventBinding::{EventConstants, EventMethods};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{MutNullableJS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::eventtarget::EventTarget;
use servo_util::str::DOMString;
use std::cell::{Cell, RefCell};
use std::default::Default;

use time;

#[jstraceable]
pub enum EventPhase {
    PhaseNone      = EventConstants::NONE as int,
    PhaseCapturing = EventConstants::CAPTURING_PHASE as int,
    PhaseAtTarget  = EventConstants::AT_TARGET as int,
    PhaseBubbling  = EventConstants::BUBBLING_PHASE as int,
}

#[deriving(PartialEq)]
#[jstraceable]
pub enum EventTypeId {
    CustomEventTypeId,
    HTMLEventTypeId,
    KeyEventTypeId,
    MessageEventTypeId,
    MouseEventTypeId,
    ProgressEventTypeId,
    UIEventTypeId
}

#[jstraceable]
#[must_root]
pub struct Event {
    pub type_id: EventTypeId,
    reflector_: Reflector,
    pub current_target: MutNullableJS<EventTarget>,
    pub target: MutNullableJS<EventTarget>,
    type_: RefCell<DOMString>,
    pub phase: Cell<EventPhase>,
    pub canceled: Cell<bool>,
    pub stop_propagation: Cell<bool>,
    pub stop_immediate: Cell<bool>,
    pub cancelable: Cell<bool>,
    pub bubbles: Cell<bool>,
    pub trusted: Cell<bool>,
    pub dispatching: Cell<bool>,
    pub initialized: Cell<bool>,
    timestamp: u64,
}

impl Event {
    pub fn new_inherited(type_id: EventTypeId) -> Event {
        Event {
            type_id: type_id,
            reflector_: Reflector::new(),
            current_target: Default::default(),
            target: Default::default(),
            phase: Cell::new(PhaseNone),
            type_: RefCell::new("".to_string()),
            canceled: Cell::new(false),
            cancelable: Cell::new(true),
            bubbles: Cell::new(false),
            trusted: Cell::new(false),
            dispatching: Cell::new(false),
            stop_propagation: Cell::new(false),
            stop_immediate: Cell::new(false),
            initialized: Cell::new(false),
            timestamp: time::get_time().sec as u64,
        }
    }

    pub fn new_uninitialized(global: &GlobalRef) -> Temporary<Event> {
        reflect_dom_object(box Event::new_inherited(HTMLEventTypeId),
                           global,
                           EventBinding::Wrap)
    }

    pub fn new(global: &GlobalRef,
               type_: DOMString,
               can_bubble: bool,
               cancelable: bool) -> Temporary<Event> {
        let event = Event::new_uninitialized(global).root();
        event.InitEvent(type_, can_bubble, cancelable);
        Temporary::from_rooted(*event)
    }

    pub fn Constructor(global: &GlobalRef,
                       type_: DOMString,
                       init: &EventBinding::EventInit) -> Fallible<Temporary<Event>> {
        Ok(Event::new(global, type_, init.bubbles, init.cancelable))
    }
}

impl<'a> EventMethods for JSRef<'a, Event> {
    fn EventPhase(self) -> u16 {
        self.phase.get() as u16
    }

    fn Type(self) -> DOMString {
        self.type_.borrow().clone()
    }

    fn GetTarget(self) -> Option<Temporary<EventTarget>> {
        self.target.get()
    }

    fn GetCurrentTarget(self) -> Option<Temporary<EventTarget>> {
        self.current_target.get()
    }

    fn DefaultPrevented(self) -> bool {
        self.canceled.get()
    }

    fn PreventDefault(self) {
        if self.cancelable.get() {
            self.canceled.set(true)
        }
    }

    fn StopPropagation(self) {
        self.stop_propagation.set(true);
    }

    fn StopImmediatePropagation(self) {
        self.stop_immediate.set(true);
        self.stop_propagation.set(true);
    }

    fn Bubbles(self) -> bool {
        self.bubbles.get()
    }

    fn Cancelable(self) -> bool {
        self.cancelable.get()
    }

    fn TimeStamp(self) -> u64 {
        self.timestamp
    }

    fn InitEvent(self,
                 type_: DOMString,
                 bubbles: bool,
                 cancelable: bool) {
        self.initialized.set(true);
        if self.dispatching.get() {
            return;
        }
        self.stop_propagation.set(false);
        self.stop_immediate.set(false);
        self.canceled.set(false);
        self.trusted.set(false);
        self.target.clear();
        *self.type_.borrow_mut() = type_;
        self.bubbles.set(bubbles);
        self.cancelable.set(cancelable);
    }

    fn IsTrusted(self) -> bool {
        self.trusted.get()
    }
}

impl Reflectable for Event {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
