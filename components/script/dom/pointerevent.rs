/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use euclid::Point2D;
use js::rust::HandleObject;
use keyboard_types::Modifiers;
use style_traits::CSSPixel;

use super::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::PointerEventBinding::{
    PointerEventInit, PointerEventMethods,
};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/pointerevents/#dom-pointerevent-pointerid>
#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub(crate) enum PointerId {
    NonPointerDevice = -1,
    Mouse,
}

#[dom_struct]
pub(crate) struct PointerEvent {
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
    pub(crate) fn new_inherited() -> PointerEvent {
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

    pub(crate) fn new_uninitialized(window: &Window, can_gc: CanGc) -> DomRoot<PointerEvent> {
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

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        window: &Window,
        type_: DOMString,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        view: Option<&Window>,
        detail: i32,
        screen_point: Point2D<i32, CSSPixel>,
        client_point: Point2D<i32, CSSPixel>,
        page_point: Point2D<i32, CSSPixel>,
        modifiers: Modifiers,
        button: i16,
        buttons: u16,
        related_target: Option<&EventTarget>,
        point_in_target: Option<Point2D<f32, CSSPixel>>,
        pointer_id: i32,
        width: i32,
        height: i32,
        pressure: f32,
        tangential_pressure: f32,
        tilt_x: i32,
        tilt_y: i32,
        twist: i32,
        altitude_angle: f64,
        azimuth_angle: f64,
        pointer_type: DOMString,
        is_primary: bool,
        coalesced_events: Vec<DomRoot<PointerEvent>>,
        predicted_events: Vec<DomRoot<PointerEvent>>,
        can_gc: CanGc,
    ) -> DomRoot<PointerEvent> {
        Self::new_with_proto(
            window,
            None,
            type_,
            can_bubble,
            cancelable,
            view,
            detail,
            screen_point,
            client_point,
            page_point,
            modifiers,
            button,
            buttons,
            related_target,
            point_in_target,
            pointer_id,
            width,
            height,
            pressure,
            tangential_pressure,
            tilt_x,
            tilt_y,
            twist,
            altitude_angle,
            azimuth_angle,
            pointer_type,
            is_primary,
            coalesced_events,
            predicted_events,
            can_gc,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        view: Option<&Window>,
        detail: i32,
        screen_point: Point2D<i32, CSSPixel>,
        client_point: Point2D<i32, CSSPixel>,
        page_point: Point2D<i32, CSSPixel>,
        modifiers: Modifiers,
        button: i16,
        buttons: u16,
        related_target: Option<&EventTarget>,
        point_in_target: Option<Point2D<f32, CSSPixel>>,
        pointer_id: i32,
        width: i32,
        height: i32,
        pressure: f32,
        tangential_pressure: f32,
        tilt_x: i32,
        tilt_y: i32,
        twist: i32,
        altitude_angle: f64,
        azimuth_angle: f64,
        pointer_type: DOMString,
        is_primary: bool,
        coalesced_events: Vec<DomRoot<PointerEvent>>,
        predicted_events: Vec<DomRoot<PointerEvent>>,
        can_gc: CanGc,
    ) -> DomRoot<PointerEvent> {
        let ev = PointerEvent::new_uninitialized_with_proto(window, proto, can_gc);
        ev.mouseevent.initialize_mouse_event(
            type_,
            can_bubble,
            cancelable,
            view,
            detail,
            screen_point,
            client_point,
            page_point,
            modifiers,
            button,
            buttons,
            related_target,
            point_in_target,
        );
        ev.pointer_id.set(pointer_id);
        ev.width.set(width);
        ev.height.set(height);
        ev.pressure.set(pressure);
        ev.tangential_pressure.set(tangential_pressure);
        ev.tilt_x.set(tilt_x);
        ev.tilt_y.set(tilt_y);
        ev.twist.set(twist);
        ev.altitude_angle.set(altitude_angle);
        ev.azimuth_angle.set(azimuth_angle);
        *ev.pointer_type.borrow_mut() = pointer_type;
        ev.is_primary.set(is_primary);
        *ev.coalesced_events.borrow_mut() = coalesced_events;
        *ev.predicted_events.borrow_mut() = predicted_events;
        ev
    }
}

impl PointerEventMethods<crate::DomTypeHolder> for PointerEvent {
    /// <https://w3c.github.io/pointerevents/#dom-pointerevent-constructor>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &PointerEventInit,
    ) -> DomRoot<PointerEvent> {
        let bubbles = EventBubbles::from(init.parent.parent.parent.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.parent.parent.parent.cancelable);
        let scroll_offset = window.scroll_offset(can_gc);
        let page_point = Point2D::new(
            scroll_offset.x as i32 + init.parent.clientX,
            scroll_offset.y as i32 + init.parent.clientY,
        );
        PointerEvent::new_with_proto(
            window,
            proto,
            type_,
            bubbles,
            cancelable,
            init.parent.parent.parent.view.as_deref(),
            init.parent.parent.parent.detail,
            Point2D::new(init.parent.screenX, init.parent.screenY),
            Point2D::new(init.parent.clientX, init.parent.clientY),
            page_point,
            init.parent.parent.modifiers(),
            init.parent.button,
            init.parent.buttons,
            init.parent.relatedTarget.as_deref(),
            None,
            init.pointerId,
            init.width,
            init.height,
            *init.pressure,
            *init.tangentialPressure,
            init.tiltX.unwrap_or_default(),
            init.tiltY.unwrap_or_default(),
            init.twist,
            *init.altitudeAngle.unwrap_or_default(),
            *init.azimuthAngle.unwrap_or_default(),
            init.pointerType.clone(),
            init.isPrimary,
            init.coalescedEvents.clone(),
            init.predictedEvents.clone(),
            can_gc,
        )
    }

    /// <https://w3c.github.io/pointerevents/#dom-pointerevent-pointerid>
    fn PointerId(&self) -> i32 {
        self.pointer_id.get()
    }

    /// <https://w3c.github.io/pointerevents/#dom-pointerevent-width>
    fn Width(&self) -> i32 {
        self.width.get()
    }

    /// <https://w3c.github.io/pointerevents/#dom-pointerevent-height>
    fn Height(&self) -> i32 {
        self.height.get()
    }

    /// <https://w3c.github.io/pointerevents/#dom-pointerevent-pressure>
    fn Pressure(&self) -> Finite<f32> {
        Finite::wrap(self.pressure.get())
    }

    /// <https://w3c.github.io/pointerevents/#dom-pointerevent-tangentialpressure>
    fn TangentialPressure(&self) -> Finite<f32> {
        Finite::wrap(self.tangential_pressure.get())
    }

    /// <https://w3c.github.io/pointerevents/#dom-pointerevent-tiltx>
    fn TiltX(&self) -> i32 {
        self.tilt_x.get()
    }

    /// <https://w3c.github.io/pointerevents/#dom-pointerevent-tilty>
    fn TiltY(&self) -> i32 {
        self.tilt_y.get()
    }

    /// <https://w3c.github.io/pointerevents/#dom-pointerevent-twist>
    fn Twist(&self) -> i32 {
        self.twist.get()
    }

    /// <https://w3c.github.io/pointerevents/#dom-pointerevent-altitudeangle>
    fn AltitudeAngle(&self) -> Finite<f64> {
        Finite::wrap(self.altitude_angle.get())
    }

    /// <https://w3c.github.io/pointerevents/#dom-pointerevent-azimuthangle>
    fn AzimuthAngle(&self) -> Finite<f64> {
        Finite::wrap(self.azimuth_angle.get())
    }

    /// <https://w3c.github.io/pointerevents/#dom-pointerevent-pointertype>
    fn PointerType(&self) -> DOMString {
        self.pointer_type.borrow().clone()
    }

    /// <https://w3c.github.io/pointerevents/#dom-pointerevent-isprimary>
    fn IsPrimary(&self) -> bool {
        self.is_primary.get()
    }

    /// <https://w3c.github.io/pointerevents/#dom-pointerevent-getcoalescedevents>
    fn GetCoalescedEvents(&self) -> Vec<DomRoot<PointerEvent>> {
        self.coalesced_events.borrow().clone()
    }

    /// <https://w3c.github.io/pointerevents/#dom-pointerevent-getpredictedevents>
    fn GetPredictedEvents(&self) -> Vec<DomRoot<PointerEvent>> {
        self.predicted_events.borrow().clone()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.mouseevent.IsTrusted()
    }
}
