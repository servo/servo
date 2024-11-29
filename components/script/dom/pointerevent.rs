/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Borrow;
use std::cell::Cell;
use std::default::Default;

use dom_struct::dom_struct;
use euclid::default::Point2D;
use js::rust::HandleObject;
use servo_config::pref;

use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::PointerEventBinding;
use crate::dom::bindings::codegen::Bindings::PointerEventBinding::PointerEventInit;
use crate::dom::bindings::codegen::Bindings::PointerEventBinding::PointerEventMethods;
use crate::dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::node::Node;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

use super::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;

#[dom_struct]
pub struct PointerEvent {
    mouseevent: MouseEvent,
    pointer_id: Cell<i32>,
    width: Cell<i32>,
    height: Cell<i32>,
    pressure: Cell<f32>,
    tangential_pressure: Cell<f32>,
    tilt_x: Cell<i32>,
    tilt_y: Cell<i32>,
    twist: Cell<i32>,
    altitude_angle: Cell<f64>,
    azimuth_angle: Cell<f64>,
    pointer_type: DomRefCell<DOMString>,
    is_primary: Cell<bool>,
    coalesced_events: DomRefCell<Vec<DomRoot<PointerEvent>>>,
    predicted_events: DomRefCell<Vec<DomRoot<PointerEvent>>>,
}

impl PointerEvent {
    pub fn new_inherited() -> PointerEvent {
        PointerEvent {
            mouseevent: MouseEvent::new_inherited(),
            pointer_id: Cell::new(0),
            width: Cell::new(0),
            height: Cell::new(0),
            pressure: Cell::new(0.),
            tangential_pressure: Cell::new(0.),
            tilt_x: Cell::new(0),
            tilt_y: Cell::new(0),
            twist: Cell::new(0),
            altitude_angle: Cell::new(0.),
            azimuth_angle: Cell::new(0.),
            pointer_type: DomRefCell::new(DOMString::new()),
            is_primary: Cell::new(false),
            coalesced_events: DomRefCell::new(Vec::new()),
            predicted_events: DomRefCell::new(Vec::new()),
        }
    }

    pub fn new_uninitialized(window: &Window, can_gc: CanGc) -> DomRoot<PointerEvent> {
        Self::new_uninitialized_with_proto(window, None, can_gc)
    }

    fn new_uninitialized_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<PointerEvent> {
        reflect_dom_object_with_proto(
            Box::new(PointerEvent::new_inherited()),
            window,
            proto,
            can_gc,
        )
    }
    //
    // #[allow(clippy::too_many_arguments)]
    // pub fn new(
    //     window: &Window,
    //     type_: DOMString,
    //     can_bubble: EventBubbles,
    //     cancelable: EventCancelable,
    //     view: Option<&Window>,
    //     detail: i32,
    //     screen_x: i32,
    //     screen_y: i32,
    //     client_x: i32,
    //     client_y: i32,
    //     ctrl_key: bool,
    //     alt_key: bool,
    //     shift_key: bool,
    //     meta_key: bool,
    //     button: i16,
    //     buttons: u16,
    //     related_target: Option<&EventTarget>,
    //     point_in_target: Option<Point2D<f32>>,
    //     can_gc: CanGc,
    // ) -> DomRoot<MouseEvent> {
    //     Self::new_with_proto(
    //         window,
    //         None,
    //         type_,
    //         can_bubble,
    //         cancelable,
    //         view,
    //         detail,
    //         screen_x,
    //         screen_y,
    //         client_x,
    //         client_y,
    //         ctrl_key,
    //         alt_key,
    //         shift_key,
    //         meta_key,
    //         button,
    //         buttons,
    //         related_target,
    //         point_in_target,
    //         can_gc,
    //     )
    // }
    //
    // #[allow(clippy::too_many_arguments)]
    // fn new_with_proto(
    //     window: &Window,
    //     proto: Option<HandleObject>,
    //     type_: DOMString,
    //     can_bubble: EventBubbles,
    //     cancelable: EventCancelable,
    //     view: Option<&Window>,
    //     detail: i32,
    //     screen_x: i32,
    //     screen_y: i32,
    //     client_x: i32,
    //     client_y: i32,
    //     ctrl_key: bool,
    //     alt_key: bool,
    //     shift_key: bool,
    //     meta_key: bool,
    //     button: i16,
    //     buttons: u16,
    //     related_target: Option<&EventTarget>,
    //     point_in_target: Option<Point2D<f32>>,
    //     can_gc: CanGc,
    // ) -> DomRoot<MouseEvent> {
    //     let ev = MouseEvent::new_uninitialized_with_proto(window, proto, can_gc);
    //     ev.InitMouseEvent(
    //         type_,
    //         bool::from(can_bubble),
    //         bool::from(cancelable),
    //         view,
    //         detail,
    //         screen_x,
    //         screen_y,
    //         client_x,
    //         client_y,
    //         ctrl_key,
    //         alt_key,
    //         shift_key,
    //         meta_key,
    //         button,
    //         related_target,
    //     );
    //     ev.buttons.set(buttons);
    //     ev.point_in_target.set(point_in_target);
    //     // TODO: Set proper values in https://github.com/servo/servo/issues/24415
    //     ev.page_x.set(client_x);
    //     ev.page_y.set(client_y);
    //     ev
    // }
}

impl PointerEventMethods<crate::DomTypeHolder> for PointerEvent {
    fn PointerId(&self) -> i32 {
        self.pointer_id.get()
    }

    fn Width(&self) -> i32 {
        self.width.get()
    }

    fn Height(&self) -> i32 {
        self.height.get()
    }

    fn Pressure(&self) -> Finite<f32> {
        Finite::wrap(self.pressure.get())
    }

    fn TangentialPressure(&self) -> Finite<f32> {
        Finite::wrap(self.tangential_pressure.get())
    }

    fn TiltX(&self) -> i32 {
        self.tilt_x.get()
    }

    fn TiltY(&self) -> i32 {
        self.tilt_y.get()
    }

    fn Twist(&self) -> i32 {
        self.twist.get()
    }

    fn AltitudeAngle(&self) -> Finite<f64> {
        Finite::wrap(self.altitude_angle.get())
    }

    fn AzimuthAngle(&self) -> Finite<f64> {
        Finite::wrap(self.azimuth_angle.get())
    }

    fn PointerType(&self) -> DOMString {
        self.pointer_type.borrow().clone()
    }

    fn IsPrimary(&self) -> bool {
        self.is_primary.get()
    }

    fn GetCoalescedEvents(&self) -> Vec<DomRoot<PointerEvent>> {
        self.coalesced_events.borrow().clone()
    }

    fn GetPredictedEvents(&self) -> Vec<DomRoot<PointerEvent>> {
        self.predicted_events.borrow().clone()
    }

    fn IsTrusted(&self) -> bool {
        self.mouseevent.IsTrusted()
    }

    fn Constructor(
        global: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &PointerEventInit,
    ) -> DomRoot<PointerEvent> {
        todo!()
    }
}
