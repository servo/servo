/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::EventBinding;
use dom::bindings::codegen::EventBinding::EventConstants;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::Fallible;
use dom::eventtarget::EventTarget;
use dom::window::Window;
use servo_util::str::DOMString;

use geom::point::Point2D;

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
}

impl Event {
    pub fn new_inherited(type_id: EventTypeId) -> Event {
        Event {
            type_id: type_id,
            reflector_: Reflector::new(),
            current_target: None,
            target: None,
            phase: PhaseNone,
            type_: ~"",
            canceled: false,
            cancelable: true,
            bubbles: false,
            trusted: false,
            dispatching: false,
            stop_propagation: false,
            stop_immediate: false,
            initialized: false,
        }
    }

    pub fn new(window: &JS<Window>) -> JS<Event> {
        reflect_dom_object(~Event::new_inherited(HTMLEventTypeId),
                           window,
                           EventBinding::Wrap)
    }

    pub fn EventPhase(&self) -> u16 {
        self.phase as u16
    }

    pub fn Type(&self) -> DOMString {
        self.type_.clone()
    }

    pub fn GetTarget(&self) -> Option<JS<EventTarget>> {
        self.target.clone()
    }

    pub fn GetCurrentTarget(&self) -> Option<JS<EventTarget>> {
        self.current_target.clone()
    }

    pub fn DefaultPrevented(&self) -> bool {
        self.canceled
    }

    pub fn PreventDefault(&mut self) {
        if self.cancelable {
            self.canceled = true
        }
    }

    pub fn StopPropagation(&mut self) {
        self.stop_propagation = true;
    }

    pub fn StopImmediatePropagation(&mut self) {
        self.stop_immediate = true;
        self.stop_propagation = true;
    }

    pub fn Bubbles(&self) -> bool {
        self.bubbles
    }

    pub fn Cancelable(&self) -> bool {
        self.cancelable
    }

    pub fn TimeStamp(&self) -> u64 {
        0
    }

    pub fn InitEvent(&mut self,
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

    pub fn IsTrusted(&self) -> bool {
        self.trusted
    }

    pub fn Constructor(global: &JS<Window>,
                       type_: DOMString,
                       init: &EventBinding::EventInit) -> Fallible<JS<Event>> {
        let mut ev = Event::new(global);
        ev.get_mut().InitEvent(type_, init.bubbles, init.cancelable);
        Ok(ev)
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
