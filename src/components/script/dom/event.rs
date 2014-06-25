/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding;
use dom::bindings::codegen::Bindings::EventBinding::EventConstants;
use dom::bindings::error::Fallible;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::trace::Traceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::eventtarget::EventTarget;
use dom::window::Window;
use servo_msg::constellation_msg::WindowSizeData;
use servo_util::str::DOMString;
use std::cell::{Cell, RefCell};

use geom::point::Point2D;

use time;

pub enum Event_ {
    ResizeEvent(WindowSizeData),
    ReflowEvent,
    ClickEvent(uint, Point2D<f32>),
    MouseDownEvent(uint, Point2D<f32>),
    MouseUpEvent(uint, Point2D<f32>),
    MouseMoveEvent(Point2D<f32>)
}

#[deriving(Encodable)]
pub enum EventPhase {
    PhaseNone      = EventConstants::NONE,
    PhaseCapturing = EventConstants::CAPTURING_PHASE,
    PhaseAtTarget  = EventConstants::AT_TARGET,
    PhaseBubbling  = EventConstants::BUBBLING_PHASE,
}

#[deriving(PartialEq, Encodable)]
pub enum EventTypeId {
    CustomEventTypeId,
    HTMLEventTypeId,
    KeyEventTypeId,
    MouseEventTypeId,
    ProgressEventTypeId,
    UIEventTypeId
}

#[deriving(Encodable)]
pub struct Event {
    pub type_id: EventTypeId,
    pub reflector_: Reflector,
    pub current_target: Cell<Option<JS<EventTarget>>>,
    pub target: Cell<Option<JS<EventTarget>>>,
    type_: Traceable<RefCell<DOMString>>,
    pub phase: Traceable<Cell<EventPhase>>,
    pub canceled: Traceable<Cell<bool>>,
    pub stop_propagation: Traceable<Cell<bool>>,
    pub stop_immediate: Traceable<Cell<bool>>,
    pub cancelable: Traceable<Cell<bool>>,
    pub bubbles: Traceable<Cell<bool>>,
    pub trusted: Traceable<Cell<bool>>,
    pub dispatching: Traceable<Cell<bool>>,
    pub initialized: Traceable<Cell<bool>>,
    pub timestamp: u64,
}

impl Event {
    pub fn new_inherited(type_id: EventTypeId) -> Event {
        Event {
            type_id: type_id,
            reflector_: Reflector::new(),
            current_target: Cell::new(None),
            target: Cell::new(None),
            phase: Traceable::new(Cell::new(PhaseNone)),
            type_: Traceable::new(RefCell::new("".to_string())),
            canceled: Traceable::new(Cell::new(false)),
            cancelable: Traceable::new(Cell::new(true)),
            bubbles: Traceable::new(Cell::new(false)),
            trusted: Traceable::new(Cell::new(false)),
            dispatching: Traceable::new(Cell::new(false)),
            stop_propagation: Traceable::new(Cell::new(false)),
            stop_immediate: Traceable::new(Cell::new(false)),
            initialized: Traceable::new(Cell::new(false)),
            timestamp: time::get_time().sec as u64,
        }
    }

    pub fn new_uninitialized(window: &JSRef<Window>) -> Temporary<Event> {
        reflect_dom_object(box Event::new_inherited(HTMLEventTypeId),
                           window,
                           EventBinding::Wrap)
    }

    pub fn new(window: &JSRef<Window>,
               type_: DOMString,
               can_bubble: bool,
               cancelable: bool) -> Temporary<Event> {
        let event = Event::new_uninitialized(window).root();
        event.deref().InitEvent(type_, can_bubble, cancelable);
        Temporary::from_rooted(&*event)
    }

    pub fn Constructor(global: &JSRef<Window>,
                       type_: DOMString,
                       init: &EventBinding::EventInit) -> Fallible<Temporary<Event>> {
        Ok(Event::new(global, type_, init.bubbles, init.cancelable))
    }
}

pub trait EventMethods {
    fn EventPhase(&self) -> u16;
    fn Type(&self) -> DOMString;
    fn GetTarget(&self) -> Option<Temporary<EventTarget>>;
    fn GetCurrentTarget(&self) -> Option<Temporary<EventTarget>>;
    fn DefaultPrevented(&self) -> bool;
    fn PreventDefault(&self);
    fn StopPropagation(&self);
    fn StopImmediatePropagation(&self);
    fn Bubbles(&self) -> bool;
    fn Cancelable(&self) -> bool;
    fn TimeStamp(&self) -> u64;
    fn InitEvent(&self, type_: DOMString, bubbles: bool, cancelable: bool);
    fn IsTrusted(&self) -> bool;
}

impl<'a> EventMethods for JSRef<'a, Event> {
    fn EventPhase(&self) -> u16 {
        self.phase.deref().get() as u16
    }

    fn Type(&self) -> DOMString {
        self.type_.deref().borrow().clone()
    }

    fn GetTarget(&self) -> Option<Temporary<EventTarget>> {
        self.target.get().as_ref().map(|target| Temporary::new(target.clone()))
    }

    fn GetCurrentTarget(&self) -> Option<Temporary<EventTarget>> {
        self.current_target.get().as_ref().map(|target| Temporary::new(target.clone()))
    }

    fn DefaultPrevented(&self) -> bool {
        self.canceled.deref().get()
    }

    fn PreventDefault(&self) {
        if self.cancelable.deref().get() {
            self.canceled.deref().set(true)
        }
    }

    fn StopPropagation(&self) {
        self.stop_propagation.deref().set(true);
    }

    fn StopImmediatePropagation(&self) {
        self.stop_immediate.deref().set(true);
        self.stop_propagation.deref().set(true);
    }

    fn Bubbles(&self) -> bool {
        self.bubbles.deref().get()
    }

    fn Cancelable(&self) -> bool {
        self.cancelable.deref().get()
    }

    fn TimeStamp(&self) -> u64 {
        self.timestamp
    }

    fn InitEvent(&self,
                 type_: DOMString,
                 bubbles: bool,
                 cancelable: bool) {
        self.initialized.deref().set(true);
        if self.dispatching.deref().get() {
            return;
        }
        self.stop_propagation.deref().set(false);
        self.stop_immediate.deref().set(false);
        self.canceled.deref().set(false);
        self.trusted.deref().set(false);
        self.target.set(None);
        *self.type_.deref().borrow_mut() = type_;
        self.bubbles.deref().set(bubbles);
        self.cancelable.deref().set(cancelable);
    }

    fn IsTrusted(&self) -> bool {
        self.trusted.deref().get()
    }
}

impl Reflectable for Event {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
