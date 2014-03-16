/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::EventBinding;
use dom::bindings::codegen::EventBinding::EventConstants;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::{Fallible, ErrorResult};
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
    Phase_None      = EventConstants::NONE,
    Phase_Capturing = EventConstants::CAPTURING_PHASE,
    Phase_At_Target = EventConstants::AT_TARGET,
    Phase_Bubbling  = EventConstants::BUBBLING_PHASE,
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
    type_id: EventTypeId,
    reflector_: Reflector,
    current_target: Option<JS<EventTarget>>,
    target: Option<JS<EventTarget>>,
    type_: DOMString,
    phase: EventPhase,
    default_prevented: bool,
    stop_propagation: bool,
    stop_immediate: bool,
    cancelable: bool,
    bubbles: bool,
    trusted: bool,
    dispatching: bool,
    initialized: bool,
}

impl Event {
    pub fn new_inherited(type_id: EventTypeId) -> Event {
        Event {
            type_id: type_id,
            reflector_: Reflector::new(),
            current_target: None,
            target: None,
            phase: Phase_None,
            type_: ~"",
            default_prevented: false,
            cancelable: true,
            bubbles: true,
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
        self.default_prevented
    }

    pub fn PreventDefault(&mut self) {
        if self.cancelable {
            self.default_prevented = true
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
                     cancelable: bool) -> ErrorResult {
        self.type_ = type_;
        self.cancelable = cancelable;
        self.bubbles = bubbles;
        self.initialized = true;
        Ok(())
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
