/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use euclid::Point2D;
use js::rust::HandleObject;
use keyboard_types::Modifiers;
use style_traits::CSSPixel;

use crate::dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use crate::dom::bindings::codegen::Bindings::WheelEventBinding;
use crate::dom::bindings::codegen::Bindings::WheelEventBinding::WheelEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct WheelEvent {
    mouseevent: MouseEvent,
    delta_x: Cell<Finite<f64>>,
    delta_y: Cell<Finite<f64>>,
    delta_z: Cell<Finite<f64>>,
    delta_mode: Cell<u32>,
}

impl WheelEvent {
    fn new_inherited() -> WheelEvent {
        WheelEvent {
            mouseevent: MouseEvent::new_inherited(),
            delta_x: Cell::new(Finite::wrap(0.0)),
            delta_y: Cell::new(Finite::wrap(0.0)),
            delta_z: Cell::new(Finite::wrap(0.0)),
            delta_mode: Cell::new(0),
        }
    }

    fn new_unintialized(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<WheelEvent> {
        reflect_dom_object_with_proto(Box::new(WheelEvent::new_inherited()), window, proto, can_gc)
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
        delta_x: Finite<f64>,
        delta_y: Finite<f64>,
        delta_z: Finite<f64>,
        delta_mode: u32,
        can_gc: CanGc,
    ) -> DomRoot<WheelEvent> {
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
            delta_x,
            delta_y,
            delta_z,
            delta_mode,
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
        delta_x: Finite<f64>,
        delta_y: Finite<f64>,
        delta_z: Finite<f64>,
        delta_mode: u32,
        can_gc: CanGc,
    ) -> DomRoot<WheelEvent> {
        let ev = WheelEvent::new_unintialized(window, proto, can_gc);
        ev.intitialize_wheel_event(
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
            delta_x,
            delta_y,
            delta_z,
            delta_mode,
        );

        ev
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn intitialize_wheel_event(
        &self,
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
        delta_x: Finite<f64>,
        delta_y: Finite<f64>,
        delta_z: Finite<f64>,
        delta_mode: u32,
    ) {
        if self.upcast::<Event>().dispatching() {
            return;
        }

        self.upcast::<MouseEvent>().initialize_mouse_event(
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
        self.delta_x.set(delta_x);
        self.delta_y.set(delta_y);
        self.delta_z.set(delta_z);
        self.delta_mode.set(delta_mode);
    }
}

impl WheelEventMethods<crate::DomTypeHolder> for WheelEvent {
    // https://w3c.github.io/uievents/#dom-wheelevent-wheelevent
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &WheelEventBinding::WheelEventInit,
    ) -> Fallible<DomRoot<WheelEvent>> {
        let bubbles = EventBubbles::from(init.parent.parent.parent.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.parent.parent.parent.cancelable);
        let scroll_offset = window.scroll_offset(can_gc);

        let page_point = Point2D::<i32, CSSPixel>::new(
            scroll_offset.x as i32 + init.parent.clientX,
            scroll_offset.y as i32 + init.parent.clientY,
        );

        let event = WheelEvent::new_with_proto(
            window,
            proto,
            type_,
            bubbles,
            cancelable,
            init.parent.parent.parent.view.as_deref(),
            init.parent.parent.parent.detail,
            Point2D::new(init.parent.screenX, init.parent.screenY),
            Point2D::new(init.parent.clientX, init.parent.clientY),
            Point2D::new(page_point.x, page_point.y),
            init.parent.parent.modifiers(),
            init.parent.button,
            init.parent.buttons,
            init.parent.relatedTarget.as_deref(),
            None,
            init.deltaX,
            init.deltaY,
            init.deltaZ,
            init.deltaMode,
            can_gc,
        );

        Ok(event)
    }

    // https://w3c.github.io/uievents/#widl-WheelEvent-deltaX
    fn DeltaX(&self) -> Finite<f64> {
        self.delta_x.get()
    }

    // https://w3c.github.io/uievents/#widl-WheelEvent-deltaY
    fn DeltaY(&self) -> Finite<f64> {
        self.delta_y.get()
    }

    // https://w3c.github.io/uievents/#widl-WheelEvent-deltaZ
    fn DeltaZ(&self) -> Finite<f64> {
        self.delta_z.get()
    }

    // https://w3c.github.io/uievents/#widl-WheelEvent-deltaMode
    fn DeltaMode(&self) -> u32 {
        self.delta_mode.get()
    }

    // https://w3c.github.io/uievents/#widl-WheelEvent-initWheelEvent
    fn InitWheelEvent(
        &self,
        type_arg: DOMString,
        can_bubble_arg: bool,
        cancelable_arg: bool,
        view_arg: Option<&Window>,
        detail_arg: i32,
        delta_x_arg: Finite<f64>,
        delta_y_arg: Finite<f64>,
        delta_z_arg: Finite<f64>,
        delta_mode_arg: u32,
        can_gc: CanGc,
    ) {
        if self.upcast::<Event>().dispatching() {
            return;
        }

        self.upcast::<MouseEvent>().InitMouseEvent(
            type_arg,
            can_bubble_arg,
            cancelable_arg,
            view_arg,
            detail_arg,
            self.mouseevent.ScreenX(),
            self.mouseevent.ScreenY(),
            self.mouseevent.ClientX(),
            self.mouseevent.ClientY(),
            self.mouseevent.CtrlKey(),
            self.mouseevent.AltKey(),
            self.mouseevent.ShiftKey(),
            self.mouseevent.MetaKey(),
            self.mouseevent.Button(),
            self.mouseevent.GetRelatedTarget().as_deref(),
            can_gc,
        );
        self.delta_x.set(delta_x_arg);
        self.delta_y.set(delta_y_arg);
        self.delta_z.set(delta_z_arg);
        self.delta_mode.set(delta_mode_arg);
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.mouseevent.IsTrusted()
    }
}
