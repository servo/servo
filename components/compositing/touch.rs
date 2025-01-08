/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::{Point2D, Scale, Vector2D};
use log::{debug, warn};
use script_traits::{EventResult, TouchId};
use webrender_api::units::{DeviceIntPoint, DevicePixel, LayoutVector2D};

use self::TouchState::*;

// TODO: All `_SCREEN_PX` units below are currently actually used as `DevicePixel`
// without multiplying with the `hidpi_factor`. This should be fixed and the
// constants adjusted accordingly.
/// Minimum number of `DeviceIndependentPixel` to begin touch scrolling.
const TOUCH_PAN_MIN_SCREEN_PX: f32 = 20.0;
/// Factor by which the flinging velocity changes on each tick.
const FLING_SCALING_FACTOR: f32 = 0.95;
/// Minimum velocity required for transitioning to fling when panning ends.
const FLING_MIN_SCREEN_PX: f32 = 3.0;
/// Maximum velocity when flinging.
const FLING_MAX_SCREEN_PX: f32 = 4000.0;

pub struct TouchHandler {
    pub state: TouchState,
    pub active_touch_points: Vec<TouchPoint>,
}

#[derive(Clone, Copy, Debug)]
pub struct TouchPoint {
    pub id: TouchId,
    pub point: Point2D<f32, DevicePixel>,
}

impl TouchPoint {
    pub fn new(id: TouchId, point: Point2D<f32, DevicePixel>) -> Self {
        TouchPoint { id, point }
    }
}

/// The states of the touch input state machine.
#[derive(Clone, Copy, Debug, PartialEq)]
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
    Panning {
        velocity: Vector2D<f32, DevicePixel>,
    },
    /// No active touch points, but there is still scrolling velocity
    Flinging {
        velocity: Vector2D<f32, DevicePixel>,
        cursor: DeviceIntPoint,
    },
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
    Scroll(Vector2D<f32, DevicePixel>),
    /// Zoom by a magnification factor and scroll by the provided offset.
    Zoom(f32, Vector2D<f32, DevicePixel>),
    /// Send a JavaScript event to content.
    DispatchEvent,
    /// Don't do anything.
    NoAction,
}

pub(crate) struct FlingAction {
    pub delta: LayoutVector2D,
    pub cursor: DeviceIntPoint,
}

impl TouchHandler {
    pub fn new() -> Self {
        TouchHandler {
            state: Nothing,
            active_touch_points: Vec::new(),
        }
    }

    pub fn on_touch_down(&mut self, id: TouchId, point: Point2D<f32, DevicePixel>) {
        let point = TouchPoint::new(id, point);
        self.active_touch_points.push(point);

        self.state = match self.state {
            Nothing => WaitingForScript,
            Flinging { .. } => Touching,
            Touching | Panning { .. } => Pinching,
            WaitingForScript => WaitingForScript,
            DefaultPrevented => DefaultPrevented,
            Pinching | MultiTouch => MultiTouch,
        };
    }

    pub fn on_vsync(&mut self) -> Option<FlingAction> {
        let Flinging {
            velocity,
            ref cursor,
        } = &mut self.state
        else {
            return None;
        };
        if velocity.length().abs() < FLING_MIN_SCREEN_PX {
            self.state = Nothing;
            return None;
        }
        // TODO: Probably we should multiply with the current refresh rate (and divide on each frame)
        // or save a timestamp to account for a potentially changing display refresh rate.
        *velocity *= FLING_SCALING_FACTOR;
        debug_assert!(velocity.length() <= FLING_MAX_SCREEN_PX);
        Some(FlingAction {
            delta: LayoutVector2D::new(velocity.x, velocity.y),
            cursor: *cursor,
        })
    }

    pub fn on_touch_move(&mut self, id: TouchId, point: Point2D<f32, DevicePixel>) -> TouchAction {
        let idx = match self.active_touch_points.iter_mut().position(|t| t.id == id) {
            Some(i) => i,
            None => {
                warn!("Got a touchmove event for a non-active touch point");
                return TouchAction::NoAction;
            },
        };
        let old_point = self.active_touch_points[idx].point;

        let action = match self.state {
            Touching => {
                let delta = point - old_point;

                if delta.x.abs() > TOUCH_PAN_MIN_SCREEN_PX ||
                    delta.y.abs() > TOUCH_PAN_MIN_SCREEN_PX
                {
                    self.state = Panning {
                        velocity: Vector2D::new(delta.x, delta.y),
                    };
                    TouchAction::Scroll(delta)
                } else {
                    TouchAction::NoAction
                }
            },
            Panning { ref mut velocity } => {
                let delta = point - old_point;
                // TODO: Probably we should track 1-3 more points and use a smarter algorithm
                *velocity += delta;
                *velocity /= 2.0;
                TouchAction::Scroll(delta)
            },
            Flinging { .. } => {
                unreachable!("Touch Move event received without preceding down.")
            },
            DefaultPrevented => TouchAction::DispatchEvent,
            Pinching => {
                let (d0, c0) = self.pinch_distance_and_center();
                self.active_touch_points[idx].point = point;
                let (d1, c1) = self.pinch_distance_and_center();

                let magnification = d1 / d0;
                let scroll_delta = c1 - c0 * Scale::new(magnification);

                TouchAction::Zoom(magnification, scroll_delta)
            },
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

    pub fn on_touch_up(&mut self, id: TouchId, point: Point2D<f32, DevicePixel>) -> TouchAction {
        let old = match self.active_touch_points.iter().position(|t| t.id == id) {
            Some(i) => Some(self.active_touch_points.swap_remove(i).point),
            None => {
                warn!("Got a touch up event for a non-active touch point");
                None
            },
        };
        match self.state {
            Touching => {
                // FIXME: If the duration exceeds some threshold, send a contextmenu event instead.
                // FIXME: Don't send a click if preventDefault is called on the touchend event.
                self.state = Nothing;
                TouchAction::Click
            },
            Nothing => {
                self.state = Nothing;
                TouchAction::NoAction
            },
            Panning { velocity } if velocity.length().abs() >= FLING_MIN_SCREEN_PX => {
                // TODO: point != old. Not sure which one is better to take as cursor for flinging.
                debug!(
                    "Transitioning to Fling. Cursor is {point:?}. Old cursor was {old:?}. \
                    Raw velocity is {velocity:?}."
                );
                debug_assert!((point.x as i64) < (i32::MAX as i64));
                debug_assert!((point.y as i64) < (i32::MAX as i64));
                let cursor = DeviceIntPoint::new(point.x as i32, point.y as i32);
                // Multiplying the initial velocity gives the fling a much more snappy feel
                // and serves well as a poor-mans acceleration algorithm.
                let velocity = (velocity * 2.0).with_max_length(FLING_MAX_SCREEN_PX);
                self.state = Flinging { velocity, cursor };
                TouchAction::NoAction
            },
            Panning { .. } => {
                self.state = Nothing;
                TouchAction::NoAction
            },
            Pinching => {
                self.state = Panning {
                    velocity: Vector2D::new(0.0, 0.0),
                };
                TouchAction::NoAction
            },
            Flinging { .. } => {
                unreachable!("On touchup received, but already flinging.")
            },
            WaitingForScript => {
                if self.active_touch_points.is_empty() {
                    self.state = Nothing;
                    return TouchAction::Click;
                }
                TouchAction::NoAction
            },
            DefaultPrevented | MultiTouch => {
                if self.active_touch_points.is_empty() {
                    self.state = Nothing;
                }
                TouchAction::NoAction
            },
        }
    }

    pub fn on_touch_cancel(&mut self, id: TouchId, _point: Point2D<f32, DevicePixel>) {
        match self.active_touch_points.iter().position(|t| t.id == id) {
            Some(i) => {
                self.active_touch_points.swap_remove(i);
            },
            None => {
                warn!("Got a touchcancel event for a non-active touch point");
                return;
            },
        }
        match self.state {
            Nothing => {},
            Touching | Panning { .. } | Flinging { .. } => {
                self.state = Nothing;
            },
            Pinching => {
                self.state = Panning {
                    velocity: Vector2D::new(0.0, 0.0),
                };
            },
            WaitingForScript | DefaultPrevented | MultiTouch => {
                if self.active_touch_points.is_empty() {
                    self.state = Nothing;
                }
            },
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
                },
            }
        }
    }

    fn touch_count(&self) -> usize {
        self.active_touch_points.len()
    }

    fn pinch_distance_and_center(&self) -> (f32, Point2D<f32, DevicePixel>) {
        debug_assert_eq!(self.touch_count(), 2);
        let p0 = self.active_touch_points[0].point;
        let p1 = self.active_touch_points[1].point;
        let center = p0.lerp(p1, 0.5);
        let distance = (p0 - p1).length();

        (distance, center)
    }
}
