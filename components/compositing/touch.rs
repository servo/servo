/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::{TypedPoint2D, TypedVector2D};
use euclid::ScaleFactor;
use script_traits::{DevicePixel, EventResult, TouchId};
use self::TouchState::*;

/// Minimum number of `DeviceIndependentPixel` to begin touch scrolling.
const TOUCH_PAN_MIN_SCREEN_PX: f32 = 20.0;

pub struct TouchHandler {
    pub state: TouchState,
    pub active_touch_points: Vec<TouchPoint>,
}

#[derive(Clone, Copy, Debug)]
pub struct TouchPoint {
    pub id: TouchId,
    pub point: TypedPoint2D<f32, DevicePixel>
}

impl TouchPoint {
    pub fn new(id: TouchId, point: TypedPoint2D<f32, DevicePixel>) -> Self {
        TouchPoint { id: id, point: point }
    }
}

/// The states of the touch input state machine.
///
/// TODO: Add support for "flinging" (scrolling inertia)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TouchState {
    /// Not tracking any touch point
    Nothing,
    /// A touchstart event was dispatched to the page, but the response wasn't received yet.
    /// Contains the initial touch point.
    WaitingForScript,
    /// Script is consuming the current touch sequence; don't perform default actions.
    DefaultPrevented,
    /// A single touch point is active and may perform click or pan default actions.
    /// Contains the initial touch location.
    Touching,
    /// A single touch point is active and has started panning.
    Panning,
    /// A two-finger pinch zoom gesture is active.
    Pinching,
    /// A multi-touch gesture is in progress. Contains the number of active touch points.
    MultiTouch,
}

/// The action to take in response to a touch event
#[derive(Clone, Copy, Debug)]
pub enum TouchAction {
    /// Simulate a mouse click.
    Click,
    /// Scroll by the provided offset.
    Scroll(TypedVector2D<f32, DevicePixel>),
    /// Zoom by a magnification factor and scroll by the provided offset.
    Zoom(f32, TypedVector2D<f32, DevicePixel>),
    /// Send a JavaScript event to content.
    DispatchEvent,
    /// Don't do anything.
    NoAction,
}

impl TouchHandler {
    pub fn new() -> Self {
        TouchHandler {
            state: Nothing,
            active_touch_points: Vec::new(),
        }
    }

    pub fn on_touch_down(&mut self, id: TouchId, point: TypedPoint2D<f32, DevicePixel>) {
        let point = TouchPoint::new(id, point);
        self.active_touch_points.push(point);

        self.state = match self.state {
            Nothing               => WaitingForScript,
            Touching | Panning    => Pinching,
            WaitingForScript      => WaitingForScript,
            DefaultPrevented      => DefaultPrevented,
            Pinching | MultiTouch => MultiTouch,
        };
    }

    pub fn on_touch_move(&mut self, id: TouchId, point: TypedPoint2D<f32, DevicePixel>)
                         -> TouchAction {
        let idx = match self.active_touch_points.iter_mut().position(|t| t.id == id) {
            Some(i) => i,
            None => {
                warn!("Got a touchmove event for a non-active touch point");
                return TouchAction::NoAction;
            }
        };
        let old_point = self.active_touch_points[idx].point;

        let action = match self.state {
            Touching => {
                let delta = point - old_point;

                if delta.x.abs() > TOUCH_PAN_MIN_SCREEN_PX ||
                   delta.y.abs() > TOUCH_PAN_MIN_SCREEN_PX
                {
                    self.state = Panning;
                    TouchAction::Scroll(delta)
                } else {
                    TouchAction::NoAction
                }
            }
            Panning => {
                let delta = point - old_point;
                TouchAction::Scroll(delta)
            }
            DefaultPrevented => {
                TouchAction::DispatchEvent
            }
            Pinching => {
                let (d0, c0) = self.pinch_distance_and_center();
                self.active_touch_points[idx].point = point;
                let (d1, c1) = self.pinch_distance_and_center();

                let magnification = d1 / d0;
                let scroll_delta = c1 - c0 * ScaleFactor::new(magnification);

                TouchAction::Zoom(magnification, scroll_delta)
            }
            WaitingForScript => TouchAction::NoAction,
            MultiTouch => TouchAction::NoAction,
            Nothing => unreachable!(),
        };

        // If we're still waiting to see whether this is a click or pan, remember the original
        // location.  Otherwise, update the touch point with the latest location.
        if self.state != Touching && self.state != WaitingForScript {
            self.active_touch_points[idx].point = point;
        }
        action
    }

    pub fn on_touch_up(&mut self, id: TouchId, _point: TypedPoint2D<f32, DevicePixel>)
                       -> TouchAction {
        match self.active_touch_points.iter().position(|t| t.id == id) {
            Some(i) => {
                self.active_touch_points.swap_remove(i);
            }
            None => {
                warn!("Got a touch up event for a non-active touch point");
            }
        }
        match self.state {
            Touching => {
                // FIXME: If the duration exceeds some threshold, send a contextmenu event instead.
                // FIXME: Don't send a click if preventDefault is called on the touchend event.
                self.state = Nothing;
                TouchAction::Click
            }
            Nothing | Panning => {
                self.state = Nothing;
                TouchAction::NoAction
            }
            Pinching => {
                self.state = Panning;
                TouchAction::NoAction
            }
            WaitingForScript | DefaultPrevented | MultiTouch => {
                if self.active_touch_points.is_empty() {
                    self.state = Nothing;
                }
                TouchAction::NoAction
            }
        }
    }

    pub fn on_touch_cancel(&mut self, id: TouchId, _point: TypedPoint2D<f32, DevicePixel>) {
        match self.active_touch_points.iter().position(|t| t.id == id) {
            Some(i) => {
                self.active_touch_points.swap_remove(i);
            }
            None => {
                warn!("Got a touchcancel event for a non-active touch point");
                return;
            }
        }
        match self.state {
            Nothing => {}
            Touching | Panning => {
                self.state = Nothing;
            }
            Pinching => {
                self.state = Panning;
            }
            WaitingForScript | DefaultPrevented | MultiTouch => {
                if self.active_touch_points.is_empty() {
                    self.state = Nothing;
                }
            }
        }
    }

    pub fn on_event_processed(&mut self, result: EventResult) {
        if let WaitingForScript = self.state {
            self.state = match result {
                EventResult::DefaultPrevented => DefaultPrevented,
                EventResult::DefaultAllowed => match self.touch_count() {
                    1 => Touching,
                    2 => Pinching,
                    _ => MultiTouch,
                }
            }
        }
    }

    fn touch_count(&self) -> usize {
        self.active_touch_points.len()
    }

    fn pinch_distance_and_center(&self) -> (f32, TypedPoint2D<f32, DevicePixel>) {
        debug_assert!(self.touch_count() == 2);
        let p0 = self.active_touch_points[0].point;
        let p1 = self.active_touch_points[1].point;
        let center = p0.lerp(p1, 0.5);
        let distance = (p0 - p1).length();

        (distance, center)
    }
}
