/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::f64::consts::PI;

use dom_struct::dom_struct;
use euclid::Point2D;

use crate::dom::bindings::codegen::Bindings::TouchBinding::TouchMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutDom};
use crate::dom::event::{EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::pointerevent::PointerEvent;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct Touch {
    reflector_: Reflector,
    identifier: i32,
    target: MutDom<EventTarget>,
    screen_x: f64,
    screen_y: f64,
    client_x: f64,
    client_y: f64,
    page_x: f64,
    page_y: f64,
}

impl Touch {
    #[allow(clippy::too_many_arguments)]
    fn new_inherited(
        identifier: i32,
        target: &EventTarget,
        screen_x: Finite<f64>,
        screen_y: Finite<f64>,
        client_x: Finite<f64>,
        client_y: Finite<f64>,
        page_x: Finite<f64>,
        page_y: Finite<f64>,
    ) -> Touch {
        Touch {
            reflector_: Reflector::new(),
            identifier,
            target: MutDom::new(target),
            screen_x: *screen_x,
            screen_y: *screen_y,
            client_x: *client_x,
            client_y: *client_y,
            page_x: *page_x,
            page_y: *page_y,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        window: &Window,
        identifier: i32,
        target: &EventTarget,
        screen_x: Finite<f64>,
        screen_y: Finite<f64>,
        client_x: Finite<f64>,
        client_y: Finite<f64>,
        page_x: Finite<f64>,
        page_y: Finite<f64>,
        can_gc: CanGc,
    ) -> DomRoot<Touch> {
        reflect_dom_object(
            Box::new(Touch::new_inherited(
                identifier, target, screen_x, screen_y, client_x, client_y, page_x, page_y,
            )),
            window,
            can_gc,
        )
    }

    /// Create a PointerEvent from this Touch.
    /// <https://w3c.github.io/pointerevents/#the-primary-pointer>
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn to_pointer_event(
        &self,
        window: &Window,
        event_type: &str,
        pointer_id: i32,
        is_primary: bool,
        modifiers: keyboard_types::Modifiers,
        is_cancelable: bool,
        point_in_node: Option<euclid::Point2D<f32, style_traits::CSSPixel>>,
        can_gc: CanGc,
    ) -> DomRoot<PointerEvent> {
        // Pressure is 0.5 for active touches, 0.0 for up/cancel/out/leave
        // <https://w3c.github.io/pointerevents/#dom-pointerevent-pressure>
        // TODO: add proper force support.
        let pressure = if event_type == "pointerup" ||
            event_type == "pointercancel" ||
            event_type == "pointerout" ||
            event_type == "pointerleave"
        {
            0.0
        } else {
            0.5
        };

        // <https://w3c.github.io/pointerevents/#the-button-property>
        // For pointermove, pointerover, pointerenter, pointerout, pointerleave: button is -1
        // For pointerdown, pointerup, pointercancel: button is 0 (primary button)
        let button = if event_type == "pointermove" ||
            event_type == "pointerover" ||
            event_type == "pointerenter" ||
            event_type == "pointerout" ||
            event_type == "pointerleave"
        {
            -1
        } else {
            0
        };

        // Buttons: 1 if a button is pressed during the event, 0 otherwise
        // For touch: button is pressed during over/enter/down/move, not during up/cancel/out/leave
        let buttons = if event_type == "pointermove" ||
            event_type == "pointerover" ||
            event_type == "pointerenter" ||
            event_type == "pointerdown"
        {
            1
        } else {
            0
        };

        // For enter/leave events, they don't bubble and are not cancelable
        let (bubbles, cancelable) = match event_type {
            "pointerenter" | "pointerleave" => {
                (EventBubbles::DoesNotBubble, EventCancelable::NotCancelable)
            },
            _ => (EventBubbles::Bubbles, EventCancelable::from(is_cancelable)),
        };

        PointerEvent::new(
            window,
            event_type.into(),
            bubbles,
            cancelable,
            Some(window),
            0, // detail
            Point2D::new(*self.ScreenX() as i32, *self.ScreenY() as i32),
            Point2D::new(*self.ClientX() as i32, *self.ClientY() as i32),
            Point2D::new(*self.PageX() as i32, *self.PageY() as i32),
            modifiers,
            button,
            buttons,
            None, // related_target
            point_in_node,
            pointer_id,
            1, // width (TODO: could get from touch if available)
            1, // height (TODO: could get from touch if available)
            pressure,
            0.0,      // tangential_pressure
            0,        // tilt_x
            0,        // tilt_y
            0,        // twist
            PI / 2.0, // altitude_angle (perpendicular to surface)
            0.0,      // azimuth_angle
            "touch".into(),
            is_primary,
            vec![], // coalesced_events
            vec![], // predicted_events
            can_gc,
        )
    }
}

impl TouchMethods<crate::DomTypeHolder> for Touch {
    /// <https://w3c.github.io/touch-events/#widl-Touch-identifier>
    fn Identifier(&self) -> i32 {
        self.identifier
    }

    /// <https://w3c.github.io/touch-events/#widl-Touch-target>
    fn Target(&self) -> DomRoot<EventTarget> {
        self.target.get()
    }

    /// <https://w3c.github.io/touch-events/#widl-Touch-screenX>
    fn ScreenX(&self) -> Finite<f64> {
        Finite::wrap(self.screen_x)
    }

    /// <https://w3c.github.io/touch-events/#widl-Touch-screenY>
    fn ScreenY(&self) -> Finite<f64> {
        Finite::wrap(self.screen_y)
    }

    /// <https://w3c.github.io/touch-events/#widl-Touch-clientX>
    fn ClientX(&self) -> Finite<f64> {
        Finite::wrap(self.client_x)
    }

    /// <https://w3c.github.io/touch-events/#widl-Touch-clientY>
    fn ClientY(&self) -> Finite<f64> {
        Finite::wrap(self.client_y)
    }

    /// <https://w3c.github.io/touch-events/#widl-Touch-clientX>
    fn PageX(&self) -> Finite<f64> {
        Finite::wrap(self.page_x)
    }

    /// <https://w3c.github.io/touch-events/#widl-Touch-clientY>
    fn PageY(&self) -> Finite<f64> {
        Finite::wrap(self.page_y)
    }
}
