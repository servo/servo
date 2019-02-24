/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use crate::dom::bindings::codegen::Bindings::WheelEventBinding;
use crate::dom::bindings::codegen::Bindings::WheelEventBinding::WheelEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::mouseevent::MouseEvent;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use std::cell::Cell;

#[dom_struct]
pub struct WheelEvent {
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

    pub fn new_unintialized(window: &Window) -> DomRoot<WheelEvent> {
        reflect_dom_object(
            Box::new(WheelEvent::new_inherited()),
            window,
            WheelEventBinding::Wrap,
        )
    }

    pub fn new(
        window: &Window,
        type_: DOMString,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        view: Option<&Window>,
        detail: i32,
        delta_x: Finite<f64>,
        delta_y: Finite<f64>,
        delta_z: Finite<f64>,
        delta_mode: u32,
    ) -> DomRoot<WheelEvent> {
        let ev = WheelEvent::new_unintialized(window);
        ev.InitWheelEvent(
            type_,
            bool::from(can_bubble),
            bool::from(cancelable),
            view,
            detail,
            delta_x,
            delta_y,
            delta_z,
            delta_mode,
        );

        ev
    }

    pub fn Constructor(
        window: &Window,
        type_: DOMString,
        init: &WheelEventBinding::WheelEventInit,
    ) -> Fallible<DomRoot<WheelEvent>> {
        let event = WheelEvent::new(
            window,
            type_,
            EventBubbles::from(init.parent.parent.parent.parent.bubbles),
            EventCancelable::from(init.parent.parent.parent.parent.cancelable),
            init.parent.parent.parent.view.deref(),
            init.parent.parent.parent.detail,
            init.deltaX,
            init.deltaY,
            init.deltaZ,
            init.deltaMode,
        );

        Ok(event)
    }
}

impl WheelEventMethods for WheelEvent {
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
            self.mouseevent.GetRelatedTarget().deref(),
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
