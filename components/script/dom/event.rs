/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventBinding;
use dom::bindings::codegen::Bindings::EventBinding::{EventConstants, EventMethods};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{MutNullableJS, JSRef, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::eventtarget::EventTarget;
use servo_util::str::DOMString;

use std::borrow::ToOwned;
use std::cell::Cell;
use std::default::Default;

use time;

#[jstraceable]
#[derive(Copy)]
pub enum EventPhase {
    None      = EventConstants::NONE as int,
    Capturing = EventConstants::CAPTURING_PHASE as int,
    AtTarget  = EventConstants::AT_TARGET as int,
    Bubbling  = EventConstants::BUBBLING_PHASE as int,
}

#[derive(PartialEq)]
#[jstraceable]
pub enum EventTypeId {
    CustomEvent,
    HTMLEvent,
    KeyboardEvent,
    MessageEvent,
    MouseEvent,
    ProgressEvent,
    UIEvent,
    ErrorEvent
}

#[derive(PartialEq)]
pub enum EventBubbles {
    Bubbles,
    DoesNotBubble
}

#[derive(PartialEq)]
pub enum EventCancelable {
    Cancelable,
    NotCancelable
}

#[dom_struct]
pub struct Event {
    reflector_: Reflector,
    type_id: EventTypeId,
    current_target: MutNullableJS<EventTarget>,
    target: MutNullableJS<EventTarget>,
    type_: DOMRefCell<DOMString>,
    phase: Cell<EventPhase>,
    canceled: Cell<bool>,
    stop_propagation: Cell<bool>,
    stop_immediate: Cell<bool>,
    cancelable: Cell<bool>,
    bubbles: Cell<bool>,
    trusted: Cell<bool>,
    dispatching: Cell<bool>,
    initialized: Cell<bool>,
    timestamp: u64,
}

impl Event {
    pub fn new_inherited(type_id: EventTypeId) -> Event {
        Event {
            reflector_: Reflector::new(),
            type_id: type_id,
            current_target: Default::default(),
            target: Default::default(),
            phase: Cell::new(EventPhase::None),
            type_: DOMRefCell::new("".to_owned()),
            canceled: Cell::new(false),
            cancelable: Cell::new(false),
            bubbles: Cell::new(false),
            trusted: Cell::new(false),
            dispatching: Cell::new(false),
            stop_propagation: Cell::new(false),
            stop_immediate: Cell::new(false),
            initialized: Cell::new(false),
            timestamp: time::get_time().sec as u64,
        }
    }

    pub fn new_uninitialized(global: GlobalRef) -> Temporary<Event> {
        reflect_dom_object(box Event::new_inherited(EventTypeId::HTMLEvent),
                           global,
                           EventBinding::Wrap)
    }

    pub fn new(global: GlobalRef,
               type_: DOMString,
               bubbles: EventBubbles,
               cancelable: EventCancelable) -> Temporary<Event> {
        let event = Event::new_uninitialized(global).root();
        event.r().InitEvent(type_, bubbles == EventBubbles::Bubbles, cancelable == EventCancelable::Cancelable);
        Temporary::from_rooted(event.r())
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &EventBinding::EventInit) -> Fallible<Temporary<Event>> {
        let bubbles = if init.bubbles { EventBubbles::Bubbles } else { EventBubbles::DoesNotBubble };
        let cancelable = if init.cancelable { EventCancelable::Cancelable } else { EventCancelable::NotCancelable };
        Ok(Event::new(global, type_, bubbles, cancelable))
    }

    #[inline]
    pub fn type_id<'a>(&'a self) -> &'a EventTypeId {
        &self.type_id
    }

    #[inline]
    pub fn clear_current_target(&self) {
        self.current_target.clear();
    }

    #[inline]
    pub fn set_current_target(&self, val: JSRef<EventTarget>) {
        self.current_target.assign(Some(val));
    }

    #[inline]
    pub fn set_target(&self, val: JSRef<EventTarget>) {
        self.target.assign(Some(val));
    }

    #[inline]
    pub fn set_phase(&self, val: EventPhase) {
        self.phase.set(val)
    }

    #[inline]
    pub fn stop_propagation(&self) -> bool {
        self.stop_propagation.get()
    }

    #[inline]
    pub fn stop_immediate(&self) -> bool {
        self.stop_immediate.get()
    }

    #[inline]
    pub fn bubbles(&self) -> bool {
        self.bubbles.get()
    }

    #[inline]
    pub fn dispatching(&self) -> bool {
        self.dispatching.get()
    }

    #[inline]
    pub fn set_dispatching(&self, val: bool) {
        self.dispatching.set(val)
    }

    #[inline]
    pub fn initialized(&self) -> bool {
        self.initialized.get()
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
        if self.dispatching.get() {
            return;
        }

        self.initialized.set(true);
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

pub trait EventHelpers {
    fn set_trusted(self, trusted: bool);
}

impl<'a> EventHelpers for JSRef<'a, Event> {
    fn set_trusted(self, trusted: bool) {
        self.trusted.set(trusted);
    }
}
