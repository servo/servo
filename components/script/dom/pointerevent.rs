/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PointerEventBinding;
use dom::bindings::codegen::Bindings::PointerEventBinding::PointerEventMethods;
use dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::error::Fallible;
use dom::mouseevent::MouseEvent;
use dom::bindings::num::Finite;
use dom_struct::dom_struct;
use dom::bindings::js::{Root, RootedReference};
use dom::bindings::reflector::reflect_dom_object;
use dom::window::Window;
use dom::bindings::str::DOMString;
use std::cell::Cell;

// Webidl matching struct
// QUESTION: Are webidl long's i32 or i64?
// Currently pointer types are set by string, but it will be safer to create an enum for each
// pub enum PointerType {
//     Unknown,
//     Mouse,
//     Pen,
//     Touch,
//     LastEntry
// }

// https://www.w3.org/TR/pointerevents/#the-pointerover-event
#[dom_struct]
pub struct PointerEvent {
    mouse_event: MouseEvent,
    pointer_id: Cell<i32>,      // unique identifier for pointer causing the event, MUST be Unique at the time
    width: Cell<f64>,           // width (magnitutde on the x axis) in css pixels of the contact geometry of the pointer
    height: Cell<f64>,          // height
    pressure: Cell<f32>,        // Normalized pressure of the pointer input from [0,1]
    tilt_x: Cell<i32>,          // Plane angle [-90,90] between y-Z Plane
    tilt_y: Cell<i32>,
    pointer_type: DOMString,     // Device type that caused the event (e.g mouse, pen, touch)
    is_primary: Cell<bool>,
}

impl PointerEvent {
    // Called by new_unitialized
    pub fn new_inherited() -> PointerEvent {
        PointerEvent {
            mouse_event: MouseEvent::new_inherited(),
            pointer_id: Cell::new(0),
            width: Cell::new(0.0),
            height: Cell::new(0.0),
            pressure: Cell::new(0.0),
            tilt_x: Cell::new(0),
            tilt_y: Cell::new(0),
            pointer_type: DOMString::new(),
            is_primary: Cell::new(false)
        }
    }

    // Called by new
    pub fn new_uninitialized(window: &Window) -> Root<PointerEvent> {
        reflect_dom_object(box Self::new_inherited(),
            window,
            PointerEventBinding::Wrap)
    }

    // Called by Constructor
    pub fn new(window: &Window,
               type_: DOMString) -> Root<PointerEvent> {
        let ev = PointerEvent::new_uninitialized(window);

        ev
    }

    /// Called by JS
    pub fn Constructor(window: &Window,
                       type_: DOMString,
                       init: &PointerEventBinding::PointerEventInit) -> Fallible<Root<PointerEvent>> {
        let event = PointerEvent::new(window, type_);

        Ok(event)
    }
}

impl PointerEventMethods for PointerEvent {
    fn PointerId(&self) -> i32 {
        self.pointer_id.get()
    }

    fn Width(&self) -> Finite<f64> {
        Finite::wrap(self.width.get())
    }

    fn Height(&self) -> Finite<f64> {
        Finite::wrap(self.height.get())
    }

    fn Pressure(&self) -> Finite<f32> {
        Finite::wrap(self.pressure.get())
    }

    fn TiltX(&self) -> i32 {
        self.tilt_x.get()
    }

    fn TiltY(&self) -> i32 {
        self.tilt_y.get()
    }

    fn PointerType(&self) -> DOMString {
        self.pointer_type.clone()
    }

    fn IsPrimary(&self) -> bool {
        self.is_primary.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.mouse_event.IsTrusted()
    }
}
