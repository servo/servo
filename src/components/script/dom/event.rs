/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::eventtarget::AbstractEventTarget;
use dom::window::Window;
use dom::bindings::codegen::EventBinding;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::utils::{DOMString, ErrorResult, Fallible};
use dom::mouseevent::MouseEvent;
use dom::uievent::UIEvent;

use geom::point::Point2D;

use std::cast;
use std::unstable::raw::Box;

pub enum Event_ {
    ResizeEvent(uint, uint), 
    ReflowEvent,
    ClickEvent(uint, Point2D<f32>),
    MouseDownEvent(uint, Point2D<f32>),
    MouseUpEvent(uint, Point2D<f32>),
}

pub struct AbstractEvent {
    event: *mut Box<Event>
}

pub enum EventPhase {
    Phase_None = 0,
    Phase_Capturing,
    Phase_At_Target,
    Phase_Bubbling
}

impl AbstractEvent {
    pub fn from_box(box_: *mut Box<Event>) -> AbstractEvent {
        AbstractEvent {
            event: box_
        }
    }

    //
    // Downcasting borrows
    //

    fn transmute<'a, T>(&'a self) -> &'a T {
        unsafe {
            let box_: *Box<T> = self.event as *Box<T>;
            &(*box_).data
        }
    }

    fn transmute_mut<'a, T>(&'a self) -> &'a mut T {
        unsafe {
            let box_: *mut Box<T> = self.event as *mut Box<T>;
            &mut (*box_).data
        }
    }

    pub fn type_id(&self) -> EventTypeId {
        self.event().type_id
    }

    pub fn event<'a>(&'a self) -> &'a Event {
        self.transmute()
    }

    pub fn mut_event<'a>(&'a self) -> &'a mut Event {
        self.transmute_mut()
    }

    pub fn is_uievent(&self) -> bool {
        self.type_id() == UIEventTypeId
    }

    pub fn uievent<'a>(&'a self) -> &'a UIEvent {
        assert!(self.is_uievent());
        self.transmute()
    }

    pub fn mut_uievent<'a>(&'a self) -> &'a mut UIEvent {
        assert!(self.is_uievent());
        self.transmute_mut()
    }

    pub fn is_mouseevent(&self) -> bool {
        self.type_id() == MouseEventTypeId
    }

    pub fn mouseevent<'a>(&'a self) -> &'a MouseEvent {
        assert!(self.is_mouseevent());
        self.transmute()
    }

    pub fn mut_mouseevent<'a>(&'a self) -> &'a mut MouseEvent {
        assert!(self.is_mouseevent());
        self.transmute_mut()
    }

    pub fn propagation_stopped(&self) -> bool {
        self.event().stop_propagation
    }

    pub fn bubbles(&self) -> bool {
        self.event().bubbles
    }
}

impl Reflectable for AbstractEvent {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.event().reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.mut_event().mut_reflector()
    }
}

#[deriving(Eq)]
pub enum EventTypeId {
    HTMLEventTypeId,
    UIEventTypeId,
    MouseEventTypeId,
    KeyEventTypeId
}

pub struct Event {
    type_id: EventTypeId,
    reflector_: Reflector,
    current_target: Option<AbstractEventTarget>,
    target: Option<AbstractEventTarget>,
    type_: ~str,
    phase: EventPhase,
    default_prevented: bool,
    stop_propagation: bool,
    stop_immediate: bool,
    cancelable: bool,
    bubbles: bool,
    trusted: bool,
    dispatching: bool,
    initialized: bool
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

    //FIXME: E should be bounded by some trait that is only implemented for Event types
    pub fn as_abstract<E>(event: @mut E) -> AbstractEvent {
        // This surrenders memory management of the event!
        AbstractEvent {
            event: unsafe { cast::transmute(event) },
        }
    }

    pub fn new(window: @mut Window) -> AbstractEvent {
        let ev = reflect_dom_object(@mut Event::new_inherited(HTMLEventTypeId),
                                    window, EventBinding::Wrap);
        Event::as_abstract(ev)
    }

    pub fn EventPhase(&self) -> u16 {
        self.phase as u16
    }

    pub fn Type(&self) -> DOMString {
        self.type_.clone()
    }

    pub fn GetTarget(&self) -> Option<AbstractEventTarget> {
        self.target
    }

    pub fn GetCurrentTarget(&self) -> Option<AbstractEventTarget> {
        self.current_target
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

    pub fn Constructor(global: @mut Window,
                       type_: DOMString,
                       init: &EventBinding::EventInit) -> Fallible<AbstractEvent> {
        let ev = Event::new(global);
        ev.mut_event().InitEvent(type_, init.bubbles, init.cancelable);
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
