/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use embedder_traits::TouchId;
use euclid::{Point2D, Scale, Vector2D};
use log::{debug, warn};
use webrender_api::units::{DeviceIntPoint, DevicePixel, DevicePoint, LayoutVector2D};

use self::TouchSequenceState::*;

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
    pub current_sequence_id: u32,
    touch_sequence_map: HashMap<u32, TouchSequenceInfo>,
}

struct TouchSequenceInfo {
    // touch sequence state
    state: TouchSequenceState,
    // touch sequence active touch points
    active_touch_points: Vec<TouchPoint>,
    // The script thread is processing the flag for the touchmove operation.
    handling_touch_move: bool,
    // Cancel click when touch move occurs or multiple touches on current thread.
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

impl TouchSequenceInfo {
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

    fn set_pending_touch_move_action(&mut self, action: TouchAction) {
        if self.prevent_default {
            return;
        }
        if let Some(pre_action) = self.pending_touch_move_action {
            let combine_action = match (pre_action, action) {
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
                        self.pending_touch_move_action
                    );
                },
            };
            self.pending_touch_move_action = Some(combine_action);
        } else {
            self.pending_touch_move_action = Some(action);
        }
    }

    fn set_pending_touch_up_action(&mut self, action: TouchAction) {
        if self.prevent_default {
            return;
        }
        self.pending_touch_up_action = Some(action);
    }

    fn is_finished(&self) -> bool {
        matches!(self.state, Finished | Flinging { .. })
    }
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
enum TouchSequenceState {
    /// touch point is active but does not start moving
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
    /// touch sequence finished.
    Finished,
}

pub(crate) struct FlingAction {
    pub delta: LayoutVector2D,
    pub cursor: DeviceIntPoint,
}

impl TouchHandler {
    pub fn new() -> Self {
        TouchHandler {
            current_sequence_id: 1,
            touch_sequence_map: Default::default(),
        }
    }

    pub(crate) fn set_handling_touch_move(&mut self, sequence_id: u32, flag: bool) {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .unwrap()
            .handling_touch_move = flag;
    }

    pub(crate) fn is_handling_touch_move(&mut self, sequence_id: u32) -> bool {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .unwrap()
            .handling_touch_move
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

    pub(crate) fn remove_pending_touch_move_action(&mut self, sequence_id: u32) {
        if let Some(sequence) = self.touch_sequence_map.get_mut(&sequence_id) {
            sequence.pending_touch_move_action = None;
        }
    }

    pub(crate) fn pending_touch_up_action(&mut self, sequence_id: u32) -> Option<TouchAction> {
        match self.touch_sequence_map.get(&sequence_id) {
            Some(sequence) => sequence.pending_touch_up_action,
            // touch_sequence_info has been deleted in advance without pending_action.
            None => None,
        }
    }

    pub(crate) fn remove_pending_touch_up_action(&mut self, sequence_id: u32) {
        if let Some(sequence) = self.touch_sequence_map.get_mut(&sequence_id) {
            sequence.pending_touch_up_action = None;
        }
    }

    // try to remove touch sequence, if touch sequence end and not has pending action.
    pub(crate) fn remove_touch_sequence(&mut self, sequence_id: u32) {
        if let Some(sequence) = self.touch_sequence_map.get(&sequence_id) {
            if sequence.pending_touch_move_action.is_none() &&
                sequence.pending_touch_up_action.is_none() &&
                sequence.state == Finished
            {
                self.touch_sequence_map.remove(&sequence_id);
            }
        }
    }

    fn get_touch_sequence(&mut self, sequence_id: u32) -> &mut TouchSequenceInfo {
        assert!(
            self.touch_sequence_map
                .get_mut(&self.current_sequence_id)
                .is_some(),
            "Touch event must be start with touch down."
        );
        self.touch_sequence_map.get_mut(&sequence_id).unwrap()
    }

    pub fn on_touch_down(&mut self, id: TouchId, point: Point2D<f32, DevicePixel>) {
        // if touch_sequence_map not contains self.current_sequence_id, create and insert it.
        if !self
            .touch_sequence_map
            .contains_key(&self.current_sequence_id) ||
            self.get_touch_sequence(self.current_sequence_id)
                .is_finished()
        {
            self.current_sequence_id = self.current_sequence_id.wrapping_add(1);
            let active_touch_points = vec![TouchPoint::new(id, point)];
            self.touch_sequence_map.insert(
                self.current_sequence_id,
                TouchSequenceInfo {
                    state: Touching,
                    active_touch_points,
                    handling_touch_move: false,
                    cancel_click: false,
                    prevent_default: false,
                    first_move_processed: false,
                    pending_touch_move_action: None,
                    pending_touch_up_action: None,
                },
            );
        } else {
            let touch_sequence = self.get_touch_sequence(self.current_sequence_id);
            touch_sequence
                .active_touch_points
                .push(TouchPoint::new(id, point));
            touch_sequence.state = MultiTouch;
            // cancel click by cancel_click property
            touch_sequence.cancel_click = true;
        }
    }

    pub fn on_vsync(&mut self) -> Option<FlingAction> {
        match self.touch_sequence_map.get_mut(&self.current_sequence_id) {
            None => None,
            Some(touch_sequence) => {
                if let Flinging {
                    velocity,
                    ref cursor,
                } = &mut touch_sequence.state
                {
                    if velocity.length().abs() < FLING_MIN_SCREEN_PX {
                        touch_sequence.state = Finished;
                        self.remove_pending_touch_up_action(self.current_sequence_id);
                        self.remove_touch_sequence(self.current_sequence_id);
                        None
                    } else {
                        // TODO: Probably we should multiply with the current refresh rate (and divide on each frame)
                        // or save a timestamp to account for a potentially changing display refresh rate.
                        *velocity *= FLING_SCALING_FACTOR;
                        debug_assert!(velocity.length() <= FLING_MAX_SCREEN_PX);
                        Some(FlingAction {
                            delta: LayoutVector2D::new(velocity.x, velocity.y),
                            cursor: *cursor,
                        })
                    }
                } else {
                    None
                }
            },
        }
    }

    pub fn on_touch_move(&mut self, id: TouchId, point: Point2D<f32, DevicePixel>) -> TouchAction {
        let touch_sequence = self.get_touch_sequence(self.current_sequence_id);
        let idx = match touch_sequence
            .active_touch_points
            .iter_mut()
            .position(|t| t.id == id)
        {
            Some(i) => i,
            None => {
                unreachable!("Got a touchmove event for a non-active touch point");
            },
        };
        let old_point = touch_sequence.active_touch_points[idx].point;
        let delta = point - old_point;

        let action = match touch_sequence.touch_count() {
            1 => {
                if let Panning { ref mut velocity } = touch_sequence.state {
                    // TODO: Probably we should track 1-3 more points and use a smarter algorithm
                    *velocity += delta;
                    *velocity /= 2.0;
                    // update the touch point every time when panning.
                    touch_sequence.active_touch_points[idx].point = point;
                    TouchAction::Scroll(delta, point)
                } else if delta.x.abs() > TOUCH_PAN_MIN_SCREEN_PX ||
                    delta.y.abs() > TOUCH_PAN_MIN_SCREEN_PX
                {
                    touch_sequence.state = Panning {
                        velocity: Vector2D::new(delta.x, delta.y),
                    };
                    // If first touchmove occurs, click does not occur.
                    touch_sequence.cancel_click = true;
                    // update the touch point with the enough distance.
                    touch_sequence.active_touch_points[idx].point = point;
                    TouchAction::Scroll(delta, point)
                } else {
                    TouchAction::NoAction
                }
            },
            2 => {
                if touch_sequence.state == Pinching ||
                    delta.x.abs() > TOUCH_PAN_MIN_SCREEN_PX ||
                    delta.y.abs() > TOUCH_PAN_MIN_SCREEN_PX
                {
                    touch_sequence.state = Pinching;
                    let (d0, c0) = touch_sequence.pinch_distance_and_center();
                    // update the touch point with the enough distance or pinching.
                    touch_sequence.active_touch_points[idx].point = point;
                    let (d1, c1) = touch_sequence.pinch_distance_and_center();
                    let magnification = d1 / d0;
                    let scroll_delta = c1 - c0 * Scale::new(magnification);
                    TouchAction::Zoom(magnification, scroll_delta)
                } else {
                    TouchAction::NoAction
                }
            },
            _ => {
                touch_sequence.state = MultiTouch;
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
        if TouchAction::NoAction != action && !touch_sequence.first_move_processed {
            touch_sequence.set_pending_touch_move_action(action);
        }

        action
    }

    pub fn on_touch_up(&mut self, id: TouchId, point: Point2D<f32, DevicePixel>) {
        let touch_sequence = self.get_touch_sequence(self.current_sequence_id);
        let old = match touch_sequence
            .active_touch_points
            .iter()
            .position(|t| t.id == id)
        {
            Some(i) => Some(touch_sequence.active_touch_points.swap_remove(i).point),
            None => {
                warn!("Got a touch up event for a non-active touch point");
                None
            },
        };
        match touch_sequence.touch_count() {
            0 => {
                // if touch up and touch count is zero mark touch sequence end.
                if let Panning { velocity } = touch_sequence.state {
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
                        touch_sequence
                            .set_pending_touch_up_action(TouchAction::Flinging(velocity, cursor));
                    }
                } else if !touch_sequence.cancel_click {
                    touch_sequence.set_pending_touch_up_action(TouchAction::Click(point));
                }
                touch_sequence.state = Finished;
            },
            _ => {
                touch_sequence.state = Touching;
            },
        }
    }

    pub fn on_touch_cancel(&mut self, id: TouchId, _point: Point2D<f32, DevicePixel>) {
        let touch_sequence = self.get_touch_sequence(self.current_sequence_id);
        match touch_sequence
            .active_touch_points
            .iter()
            .position(|t| t.id == id)
        {
            Some(i) => {
                touch_sequence.active_touch_points.swap_remove(i);
            },
            None => {
                warn!("Got a touchcancel event for a non-active touch point");
                return;
            },
        }
        touch_sequence.state = Finished;
    }

    pub fn on_fling(&mut self, velocity: Vector2D<f32, DevicePixel>, cursor: DeviceIntPoint) {
        let touch_sequence = self.get_touch_sequence(self.current_sequence_id);
        touch_sequence.state = Flinging { velocity, cursor };
    }
}
