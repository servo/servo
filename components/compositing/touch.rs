/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use embedder_traits::TouchId;
use euclid::{Point2D, Scale, Vector2D};
use log::{debug, warn};
use webrender_api::units::{DeviceIntPoint, DevicePixel, DevicePoint, LayoutVector2D};

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
    pub handling_touch_move: bool,
    pub current_sequence_id: u32,
    touch_sequence_map: HashMap<u32, TouchSequenceInfo>,
}

struct TouchSequenceInfo {
    // Mark the end of the touch sequence
    sequence_end: bool,
    // Cancels clicking when touch move occurs or multiple touches.
    cancel_click: bool,
    // prevent default action in the touch sequence
    prevent_default: bool,
    // Once the first move has been processed by script, we can transition to
    // non-cancellable events, and directly perform the pan without waiting for script.
    first_move_processed: bool,
    // Move operation waiting to be processed in the touch sequence
    pending_touch_move_action: Option<TouchAction>,
    // Up operation waiting to be processed in the touch sequence
    pending_touch_up_action: Option<TouchAction>,
}

/// The action to take in response to a touch event
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TouchAction {
    /// Simulate a mouse click.
    Click(DevicePoint),
    /// Fling by the provided offset
    Flinging(Vector2D<f32, DevicePixel>, DeviceIntPoint),
    /// Scroll by the provided offset.
    Scroll(Vector2D<f32, DevicePixel>, DevicePoint),
    /// Zoom by a magnification factor and scroll by the provided offset.
    Zoom(f32, Vector2D<f32, DevicePixel>),
    /// Don't do anything.
    NoAction,
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

pub(crate) struct FlingAction {
    pub delta: LayoutVector2D,
    pub cursor: DeviceIntPoint,
}

impl TouchHandler {
    pub fn new() -> Self {
        TouchHandler {
            state: Nothing,
            active_touch_points: Vec::new(),
            handling_touch_move: false,
            current_sequence_id: 1,
            touch_sequence_map: Default::default(),
        }
    }

    fn sequence_end(&mut self, sequence_id: u32) {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .unwrap()
            .sequence_end = true;
    }

    fn is_sequence_end(&mut self, sequence_id: u32) -> bool {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .unwrap()
            .sequence_end
    }

    fn cancel_click(&mut self, sequence_id: u32) {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .unwrap()
            .cancel_click = true;
    }

    fn click_allowed(&mut self, sequence_id: u32) -> bool {
        !self
            .touch_sequence_map
            .get(&sequence_id)
            .unwrap()
            .cancel_click
    }

    pub(crate) fn prevent_default(&mut self, sequence_id: u32) {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .unwrap()
            .prevent_default = true;
    }

    pub(crate) fn is_prevent_default(&mut self, sequence_id: u32) -> bool {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .unwrap()
            .prevent_default
    }

    pub(crate) fn first_move_processed(&mut self, sequence_id: u32) {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .unwrap()
            .first_move_processed = true;
    }

    fn is_first_move_processed(&mut self, sequence_id: u32) -> bool {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .unwrap()
            .first_move_processed
    }

    pub(crate) fn first_move_allowed(&mut self, sequence_id: u32) -> bool {
        self.is_first_move_processed(sequence_id) && !self.is_prevent_default(sequence_id)
    }

    pub(crate) fn pending_touch_move_action(&mut self, sequence_id: u32) -> Option<TouchAction> {
        match self.touch_sequence_map.get(&sequence_id) {
            Some(sequence) => sequence.pending_touch_move_action,
            None => None,
        }
    }

    fn set_pending_touch_move_action(&mut self, sequence_id: u32, action: TouchAction) {
        if self.is_prevent_default(sequence_id) {
            return;
        }
        if let Some(pre_action) = self
            .touch_sequence_map
            .get(&sequence_id)
            .unwrap()
            .pending_touch_move_action
        {
            let new_action = match (pre_action, action) {
                (TouchAction::NoAction, _) | (_, TouchAction::NoAction) => action,
                // Combine touch move action.
                (TouchAction::Scroll(delta, point), TouchAction::Scroll(delta_new, _)) => {
                    TouchAction::Scroll(delta + delta_new, point)
                },
                (TouchAction::Scroll(delta, _), TouchAction::Zoom(magnification, scroll_delta)) |
                (TouchAction::Zoom(magnification, scroll_delta), TouchAction::Scroll(delta, _)) => {
                    TouchAction::Zoom(magnification, delta + scroll_delta)
                },
                (
                    TouchAction::Zoom(magnification, scroll_delta),
                    TouchAction::Zoom(magnification_new, scroll_delta_new),
                ) => TouchAction::Zoom(
                    magnification * magnification_new,
                    scroll_delta + scroll_delta_new,
                ),
                _ => {
                    unreachable!(
                        "pending_touch_move_action cannot be `TouchAction::Click` or `TouchAction::Flinging` \
                        pre-action:{:?} new action:{action:?}.",
                        self.touch_sequence_map.get(&sequence_id).unwrap().pending_touch_move_action
                    );
                },
            };
            self.touch_sequence_map
                .get_mut(&sequence_id)
                .unwrap()
                .pending_touch_move_action = Some(new_action);
        } else {
            self.touch_sequence_map
                .get_mut(&sequence_id)
                .unwrap()
                .pending_touch_move_action = Some(action);
        }
    }

    pub(crate) fn remove_pending_touch_move_action(&mut self, sequence_id: u32) {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .unwrap()
            .pending_touch_move_action = None;
    }

    pub(crate) fn pending_touch_up_action(&mut self, sequence_id: u32) -> Option<TouchAction> {
        match self.touch_sequence_map.get(&sequence_id) {
            Some(sequence) => sequence.pending_touch_up_action,
            // touch_sequence_info has been deleted in advance without pending_action.
            None => None,
        }
    }

    fn set_pending_touch_up_action(&mut self, sequence_id: u32, action: TouchAction) {
        if self.is_prevent_default(sequence_id) {
            return;
        }
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .unwrap()
            .pending_touch_up_action = Some(action);
    }

    pub(crate) fn remove_pending_touch_up_action(&mut self, sequence_id: u32) {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .unwrap()
            .pending_touch_up_action = None;
    }

    // try to remove touch sequence, if touch sequence end and not has pending action.
    pub(crate) fn remove_touch_sequence(&mut self, sequence_id: u32) {
        if let Some(sequence) = self.touch_sequence_map.get(&sequence_id) {
            if sequence.pending_touch_move_action.is_none() &&
                sequence.pending_touch_up_action.is_none() &&
                self.is_sequence_end(sequence_id)
            {
                self.touch_sequence_map.remove(&sequence_id);
            }
        }
    }

    pub fn on_touch_down(&mut self, id: TouchId, point: Point2D<f32, DevicePixel>) {
        let point = TouchPoint::new(id, point);
        self.active_touch_points.push(point);
        if self.touch_count() == 1 {
            self.current_sequence_id = self.current_sequence_id.wrapping_add(1);
            let old = self.touch_sequence_map.insert(
                self.current_sequence_id,
                TouchSequenceInfo {
                    sequence_end: false,
                    cancel_click: false,
                    prevent_default: false,
                    first_move_processed: false,
                    pending_touch_move_action: None,
                    pending_touch_up_action: None,
                },
            );
            assert!(old.is_none(), "touch_sequence_map has same sequence id.");
        } else {
            self.cancel_click(self.current_sequence_id);
        }
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
        let delta = point - old_point;

        let action = match self.touch_count() {
            1 => {
                if let Panning { ref mut velocity } = self.state {
                    // TODO: Probably we should track 1-3 more points and use a smarter algorithm
                    *velocity += delta;
                    *velocity /= 2.0;
                    // update the touch point every time when panning.
                    self.active_touch_points[idx].point = point;
                    TouchAction::Scroll(delta, point)
                } else if delta.x.abs() > TOUCH_PAN_MIN_SCREEN_PX ||
                    delta.y.abs() > TOUCH_PAN_MIN_SCREEN_PX
                {
                    self.state = Panning {
                        velocity: Vector2D::new(delta.x, delta.y),
                    };
                    // If first touchmove occurs, click does not occur.
                    self.cancel_click(self.current_sequence_id);
                    // update the touch point with the enough distance.
                    self.active_touch_points[idx].point = point;
                    TouchAction::Scroll(delta, point)
                } else {
                    TouchAction::NoAction
                }
            },
            2 => {
                if self.state == Pinching ||
                    delta.x.abs() > TOUCH_PAN_MIN_SCREEN_PX ||
                    delta.y.abs() > TOUCH_PAN_MIN_SCREEN_PX
                {
                    self.state = Pinching;
                    let (d0, c0) = self.pinch_distance_and_center();
                    // update the touch point with the enough distance or pinching.
                    self.active_touch_points[idx].point = point;
                    let (d1, c1) = self.pinch_distance_and_center();
                    let magnification = d1 / d0;
                    let scroll_delta = c1 - c0 * Scale::new(magnification);
                    TouchAction::Zoom(magnification, scroll_delta)
                } else {
                    TouchAction::NoAction
                }
            },
            _ => {
                self.state = MultiTouch;
                TouchAction::NoAction
            },
        };
        if let TouchAction::Click(_) | TouchAction::Flinging(_, _) = action {
            unreachable!(
                "touch move action cannot be `TouchAction::Click` or `TouchAction::Flinging`."
            )
        }
        // If the touch action is not `NoAction` and the first move has not been processed,
        //  set pending_touch_move_action.
        if TouchAction::NoAction != action &&
            !self.is_first_move_processed(self.current_sequence_id)
        {
            self.set_pending_touch_move_action(self.current_sequence_id, action);
        }

        action
    }

    pub fn on_touch_up(&mut self, id: TouchId, point: Point2D<f32, DevicePixel>) {
        let old = match self.active_touch_points.iter().position(|t| t.id == id) {
            Some(i) => Some(self.active_touch_points.swap_remove(i).point),
            None => {
                warn!("Got a touch up event for a non-active touch point");
                None
            },
        };
        match self.touch_count() {
            0 => {
                // if touch up and touch count is zero mark touch sequence end.
                self.sequence_end(self.current_sequence_id);
                if let Panning { velocity } = self.state {
                    if velocity.length().abs() >= FLING_MIN_SCREEN_PX {
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
                        self.set_pending_touch_up_action(
                            self.current_sequence_id,
                            TouchAction::Flinging(velocity, cursor),
                        );
                    } else {
                        self.state = Nothing;
                    }
                } else {
                    self.state = Nothing;
                    if self.click_allowed(self.current_sequence_id) {
                        self.set_pending_touch_up_action(
                            self.current_sequence_id,
                            TouchAction::Click(point),
                        );
                    }
                }
            },
            _ => {
                self.state = Touching;
            },
        }
    }

    pub fn on_touch_cancel(&mut self, id: TouchId, _point: Point2D<f32, DevicePixel>) {
        self.sequence_end(self.current_sequence_id);
        match self.active_touch_points.iter().position(|t| t.id == id) {
            Some(i) => {
                self.active_touch_points.swap_remove(i);
            },
            None => {
                warn!("Got a touchcancel event for a non-active touch point");
                return;
            },
        }
        self.state = Nothing;
    }

    pub fn on_fling(&mut self, velocity: Vector2D<f32, DevicePixel>, cursor: DeviceIntPoint) {
        self.state = Flinging { velocity, cursor };
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
