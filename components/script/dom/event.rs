/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventBinding;
use dom::bindings::codegen::Bindings::EventBinding::{EventConstants, EventMethods};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::eventtarget::EventTarget;
use std::cell::Cell;
use std::default::Default;
use string_cache::Atom;
use time;
use util::str::DOMString;

#[derive(JSTraceable, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u16)]
#[derive(HeapSizeOf)]
pub enum EventPhase {
    None      = EventConstants::NONE,
    Capturing = EventConstants::CAPTURING_PHASE,
    AtTarget  = EventConstants::AT_TARGET,
    Bubbling  = EventConstants::BUBBLING_PHASE,
}

#[derive(PartialEq, HeapSizeOf)]
pub enum EventBubbles {
    Bubbles,
    DoesNotBubble
}

impl From<EventBubbles> for bool {
    fn from(bubbles: EventBubbles) -> Self {
        match bubbles {
            EventBubbles::Bubbles => true,
            EventBubbles::DoesNotBubble => false
        }
    }
}

impl From<bool> for EventBubbles {
    fn from(boolean: bool) -> Self {
        match boolean {
            true => EventBubbles::Bubbles,
            false => EventBubbles::DoesNotBubble
        }
    }
}

#[derive(PartialEq, HeapSizeOf)]
pub enum EventCancelable {
    Cancelable,
    NotCancelable
}

impl From<EventCancelable> for bool {
    fn from(bubbles: EventCancelable) -> Self {
        match bubbles {
            EventCancelable::Cancelable => true,
            EventCancelable::NotCancelable => false
        }
    }
}

impl From<bool> for EventCancelable {
    fn from(boolean: bool) -> Self {
        match boolean {
            true => EventCancelable::Cancelable,
            false => EventCancelable::NotCancelable
        }
    }
}

#[dom_struct]
pub struct Event {
    reflector_: Reflector,
    current_target: MutNullableHeap<JS<EventTarget>>,
    target: MutNullableHeap<JS<EventTarget>>,
    type_: DOMRefCell<Atom>,
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
    pub fn new_inherited() -> Event {
        Event {
            reflector_: Reflector::new(),
            current_target: Default::default(),
            target: Default::default(),
            type_: DOMRefCell::new(atom!("")),
            phase: Cell::new(EventPhase::None),
            canceled: Cell::new(false),
            stop_propagation: Cell::new(false),
            stop_immediate: Cell::new(false),
            cancelable: Cell::new(false),
            bubbles: Cell::new(false),
            trusted: Cell::new(false),
            dispatching: Cell::new(false),
            initialized: Cell::new(false),
            timestamp: time::get_time().sec as u64,
        }
    }

    pub fn new_uninitialized(global: GlobalRef) -> Root<Event> {
        reflect_dom_object(box Event::new_inherited(),
                           global,
                           EventBinding::Wrap)
    }

    pub fn new(global: GlobalRef,
               type_: Atom,
               bubbles: EventBubbles,
               cancelable: EventCancelable) -> Root<Event> {
        let event = Event::new_uninitialized(global);
        event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        event
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &EventBinding::EventInit) -> Fallible<Root<Event>> {
        let bubbles = EventBubbles::from(init.bubbles);
        let cancelable = EventCancelable::from(init.cancelable);
        Ok(Event::new(global, Atom::from(type_), bubbles, cancelable))
    }

    pub fn init_event(&self, type_: Atom, bubbles: bool, cancelable: bool) {
        if self.dispatching.get() {
            return;
        }

        self.initialized.set(true);
        self.stop_propagation.set(false);
        self.stop_immediate.set(false);
        self.canceled.set(false);
        self.trusted.set(false);
        self.target.set(None);
        *self.type_.borrow_mut() = type_;
        self.bubbles.set(bubbles);
        self.cancelable.set(cancelable);
    }

    #[inline]
    pub fn clear_current_target(&self) {
        self.current_target.set(None);
    }

    #[inline]
    pub fn set_current_target(&self, val: &EventTarget) {
        self.current_target.set(Some(val));
    }

    #[inline]
    pub fn set_target(&self, val: &EventTarget) {
        self.target.set(Some(val));
    }

    #[inline]
    pub fn phase(&self) -> EventPhase {
        self.phase.get()
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

    #[inline]
    pub fn type_(&self) -> Atom {
        self.type_.borrow().clone()
    }
}

impl EventMethods for Event {
    // https://dom.spec.whatwg.org/#dom-event-eventphase
    fn EventPhase(&self) -> u16 {
        self.phase.get() as u16
    }

    // https://dom.spec.whatwg.org/#dom-event-type
    fn Type(&self) -> DOMString {
        DOMString::from(&*self.type_()) // FIXME(ajeffrey): Directly convert from Atom to DOMString
    }

    // https://dom.spec.whatwg.org/#dom-event-target
    fn GetTarget(&self) -> Option<Root<EventTarget>> {
        self.target.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-currenttarget
    fn GetCurrentTarget(&self) -> Option<Root<EventTarget>> {
        self.current_target.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-defaultprevented
    fn DefaultPrevented(&self) -> bool {
        self.canceled.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-preventdefault
    fn PreventDefault(&self) {
        if self.cancelable.get() {
            self.canceled.set(true)
        }
    }

    // https://dom.spec.whatwg.org/#dom-event-stoppropagation
    fn StopPropagation(&self) {
        self.stop_propagation.set(true);
    }

    // https://dom.spec.whatwg.org/#dom-event-stopimmediatepropagation
    fn StopImmediatePropagation(&self) {
        self.stop_immediate.set(true);
        self.stop_propagation.set(true);
    }

    // https://dom.spec.whatwg.org/#dom-event-bubbles
    fn Bubbles(&self) -> bool {
        self.bubbles.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-cancelable
    fn Cancelable(&self) -> bool {
        self.cancelable.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-timestamp
    fn TimeStamp(&self) -> u64 {
        self.timestamp
    }

    // https://dom.spec.whatwg.org/#dom-event-initevent
    fn InitEvent(&self,
                 type_: DOMString,
                 bubbles: bool,
                 cancelable: bool) {
         self.init_event(Atom::from(type_), bubbles, cancelable)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.trusted.get()
    }
}


impl Event {
    pub fn set_trusted(&self, trusted: bool) {
        self.trusted.set(trusted);
    }

    // https://html.spec.whatwg.org/multipage/#fire-a-simple-event
    pub fn fire(&self, target: &EventTarget) -> bool {
        self.set_trusted(true);
        target.dispatch_event(self)
    }
}
