/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::EventBinding;
use dom::bindings::codegen::BindingDeclarations::EventBinding::EventConstants;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::Fallible;
use dom::eventtarget::EventTarget;
use dom::window::Window;
use servo_util::str::DOMString;

use geom::point::Point2D;

use time;

pub enum Event_ {
    ResizeEvent(uint, uint),
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

#[deriving(Eq, Encodable)]
pub enum EventTypeId {
    HTMLEventTypeId,
    UIEventTypeId,
    MouseEventTypeId,
    KeyEventTypeId
}

#[deriving(Encodable)]
pub struct Event {
    pub type_id: EventTypeId,
    pub reflector_: Reflector,
    pub current_target: Option<JS<EventTarget>>,
    pub target: Option<JS<EventTarget>>,
    pub type_: DOMString,
    pub phase: EventPhase,
    pub canceled: bool,
    pub stop_propagation: bool,
    pub stop_immediate: bool,
    pub cancelable: bool,
    pub bubbles: bool,
    pub trusted: bool,
    pub dispatching: bool,
    pub initialized: bool,
    pub timestamp: u64,
}

impl Event {
    pub fn new_inherited(type_id: EventTypeId) -> Event {
        Event {
            type_id: type_id,
            reflector_: Reflector::new(),
            current_target: None,
            target: None,
            phase: PhaseNone,
            type_: "".to_owned(),
            canceled: false,
            cancelable: true,
            bubbles: false,
            trusted: false,
            dispatching: false,
            stop_propagation: false,
            stop_immediate: false,
            initialized: false,
            timestamp: time::get_time().sec as u64,
        }
    }

    pub fn new(window: &JSRef<Window>) -> Temporary<Event> {
        reflect_dom_object(~Event::new_inherited(HTMLEventTypeId),
                           window,
                           EventBinding::Wrap)
    }

    pub fn Constructor(global: &JSRef<Window>,
                       type_: DOMString,
                       init: &EventBinding::EventInit) -> Fallible<Temporary<Event>> {
        let mut ev = Event::new(global).root();
        ev.InitEvent(type_, init.bubbles, init.cancelable);
        Ok(Temporary::from_rooted(&*ev))
    }
}

pub trait EventMethods {
    fn EventPhase(&self) -> u16;
    fn Type(&self) -> DOMString;
    fn GetTarget(&self) -> Option<Temporary<EventTarget>>;
    fn GetCurrentTarget(&self) -> Option<Temporary<EventTarget>>;
    fn DefaultPrevented(&self) -> bool;
    fn PreventDefault(&mut self);
    fn StopPropagation(&mut self);
    fn StopImmediatePropagation(&mut self);
    fn Bubbles(&self) -> bool;
    fn Cancelable(&self) -> bool;
    fn TimeStamp(&self) -> u64;
    fn InitEvent(&mut self, type_: DOMString, bubbles: bool, cancelable: bool);
    fn IsTrusted(&self) -> bool;
}

impl<'a> EventMethods for JSRef<'a, Event> {
    fn EventPhase(&self) -> u16 {
        self.phase as u16
    }

    fn Type(&self) -> DOMString {
        self.type_.clone()
    }

    fn GetTarget(&self) -> Option<Temporary<EventTarget>> {
        self.target.as_ref().map(|target| Temporary::new(target.clone()))
    }

    fn GetCurrentTarget(&self) -> Option<Temporary<EventTarget>> {
        self.current_target.as_ref().map(|target| Temporary::new(target.clone()))
    }

    fn DefaultPrevented(&self) -> bool {
        self.canceled
    }

    fn PreventDefault(&mut self) {
        if self.cancelable {
            self.canceled = true
        }
    }

    fn StopPropagation(&mut self) {
        self.stop_propagation = true;
    }

    fn StopImmediatePropagation(&mut self) {
        self.stop_immediate = true;
        self.stop_propagation = true;
    }

    fn Bubbles(&self) -> bool {
        self.bubbles
    }

    fn Cancelable(&self) -> bool {
        self.cancelable
    }

    fn TimeStamp(&self) -> u64 {
        self.timestamp
    }

    fn InitEvent(&mut self,
                 type_: DOMString,
                 bubbles: bool,
                 cancelable: bool) {
        self.initialized = true;
        if self.dispatching {
            return;
        }
        self.stop_propagation = false;
        self.stop_immediate = false;
        self.canceled = false;
        self.trusted = false;
        self.target = None;
        self.type_ = type_;
        self.bubbles = bubbles;
        self.cancelable = cancelable;
    }

    fn IsTrusted(&self) -> bool {
        self.trusted
    }
}

impl Reflectable for Event {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
