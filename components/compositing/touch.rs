/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use embedder_traits::{TouchId, TouchSequenceId};
use euclid::{Point2D, Scale, Vector2D};
use log::{debug, error, warn};
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
    pub current_sequence_id: TouchSequenceId,
    // todo: VecDeque + modulo arithmetic would be more efficient.
    pub touch_sequence_map: HashMap<TouchSequenceId, TouchSequenceInfo>,
}

/// Whether the default move action is allowed or not.
#[derive(Debug, Eq, PartialEq)]
pub enum TouchMoveAllowed {
    /// The default move action is prevented by script
    Prevented,
    /// The default move action is allowed
    Allowed,
    /// The initial move handler result is still pending
    Pending,
}

pub struct TouchSequenceInfo {
    /// touch sequence state
    pub(crate) state: TouchSequenceState,
    /// touch sequence active touch points
    active_touch_points: Vec<TouchPoint>,
    /// The script thread is already processing a touchmove operation.
    ///
    /// We use this to skip sending the event to the script thread,
    /// to prevent overloading script.
    handling_touch_move: bool,
    /// Do not perform a click action.
    ///
    /// This happens when
    /// - We had a touch move larger than the minimum distance OR
    /// - We had multiple active touchpoints OR
    /// - `preventDefault()` was called in a touch_down or touch_up handler
    pub prevent_click: bool,
    /// Whether move is allowed, prevented or the result is still pending.
    /// Once the first move has been processed by script, we can transition to
    /// non-cancellable events, and directly perform the pan without waiting for script.
    pub prevent_move: TouchMoveAllowed,
    /// Move operation waiting to be processed in the touch sequence.
    ///
    /// This is only used while the first touch move is processed in script.
    /// Todo: It would be nice to merge this into the TouchSequenceState, but
    /// this requires some additional work to handle the merging of pending
    /// touch move events. Presumably if we keep a history of previous touch points,
    /// this would allow a better fling algorithm and easier merging of zoom events.
    pending_touch_move_action: Option<TouchMoveAction>,
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

    fn update_pending_touch_move_action(&mut self, action: TouchMoveAction) {
        debug_assert!(self.prevent_move == TouchMoveAllowed::Pending);

        if let Some(pre_action) = self.pending_touch_move_action {
            let combine_action = match (pre_action, action) {
                (TouchMoveAction::NoAction, _) | (_, TouchMoveAction::NoAction) => action,
                // Combine touch move action.
                (TouchMoveAction::Scroll(delta, point), TouchMoveAction::Scroll(delta_new, _)) => {
                    TouchMoveAction::Scroll(delta + delta_new, point)
                },
                (
                    TouchMoveAction::Scroll(delta, _),
                    TouchMoveAction::Zoom(magnification, scroll_delta),
                ) |
                (
                    TouchMoveAction::Zoom(magnification, scroll_delta),
                    TouchMoveAction::Scroll(delta, _),
                ) => {
                    // Todo: It's unclear what the best action would be. Should we keep both
                    // scroll and zoom?
                    TouchMoveAction::Zoom(magnification, delta + scroll_delta)
                },
                (
                    TouchMoveAction::Zoom(magnification, scroll_delta),
                    TouchMoveAction::Zoom(magnification_new, scroll_delta_new),
                ) => TouchMoveAction::Zoom(
                    magnification * magnification_new,
                    scroll_delta + scroll_delta_new,
                ),
            };
            self.pending_touch_move_action = Some(combine_action);
        } else {
            self.pending_touch_move_action = Some(action);
        }
    }

    /// Returns true when all touch events of a sequence have been received.
    /// This does not mean that all event handlers have finished yet.
    fn is_finished(&self) -> bool {
        matches!(
            self.state,
            Finished | Flinging { .. } | PendingFling { .. } | PendingClick(_)
        )
    }
}

/// An action that can be immediately performed in response to a touch move event
/// without waiting for script.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TouchMoveAction {
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
pub(crate) enum TouchSequenceState {
    /// touch point is active but does not start moving
    Touching,
    /// A single touch point is active and has started panning.
    Panning {
        velocity: Vector2D<f32, DevicePixel>,
    },
    /// A two-finger pinch zoom gesture is active.
    Pinching,
    /// A multi-touch gesture is in progress.
    MultiTouch,
    // All states below here are reached after a touch-up, i.e. all events of the sequence
    // have already been received.
    /// The initial touch move handler has not finished processing yet, so we need to wait
    /// for the result in order to transition to fling.
    PendingFling {
        velocity: Vector2D<f32, DevicePixel>,
        cursor: DeviceIntPoint,
    },
    /// No active touch points, but there is still scrolling velocity
    Flinging {
        velocity: Vector2D<f32, DevicePixel>,
        cursor: DeviceIntPoint,
    },
    /// The touch sequence is finished, but a click is still pending, waiting on script.
    PendingClick(DevicePoint),
    /// touch sequence finished.
    Finished,
}

pub(crate) struct FlingAction {
    pub delta: LayoutVector2D,
    pub cursor: DeviceIntPoint,
}

impl TouchHandler {
    pub fn new() -> Self {
        let finished_info = TouchSequenceInfo {
            state: TouchSequenceState::Finished,
            active_touch_points: vec![],
            handling_touch_move: false,
            prevent_click: false,
            prevent_move: TouchMoveAllowed::Pending,
            pending_touch_move_action: None,
        };
        TouchHandler {
            current_sequence_id: TouchSequenceId::new(),
            // We insert a simulated initial touch sequence, which is already finished,
            // so that we always have one element in the map, which simplifies creating
            // a new touch sequence on touch_down.
            touch_sequence_map: HashMap::from([(TouchSequenceId::new(), finished_info)]),
        }
    }

    pub(crate) fn set_handling_touch_move(&mut self, sequence_id: TouchSequenceId, flag: bool) {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .unwrap()
            .handling_touch_move = flag;
    }

    pub(crate) fn is_handling_touch_move(&self, sequence_id: TouchSequenceId) -> bool {
        self.touch_sequence_map
            .get(&sequence_id)
            .unwrap()
            .handling_touch_move
    }

    pub(crate) fn prevent_click(&mut self, sequence_id: TouchSequenceId) {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .unwrap()
            .prevent_click = true;
    }

    pub(crate) fn prevent_move(&mut self, sequence_id: TouchSequenceId) {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .unwrap()
            .prevent_move = TouchMoveAllowed::Prevented;
    }

    /// Returns true if default move actions are allowed, false if prevented or the result
    /// is still pending.,
    pub(crate) fn move_allowed(&mut self, sequence_id: TouchSequenceId) -> bool {
        self.touch_sequence_map
            .get(&sequence_id)
            .unwrap()
            .prevent_move ==
            TouchMoveAllowed::Allowed
    }

    pub(crate) fn pending_touch_move_action(
        &mut self,
        sequence_id: TouchSequenceId,
    ) -> Option<TouchMoveAction> {
        match self.touch_sequence_map.get(&sequence_id) {
            Some(sequence) => sequence.pending_touch_move_action,
            None => None,
        }
    }

    pub(crate) fn remove_pending_touch_move_action(&mut self, sequence_id: TouchSequenceId) {
        if let Some(sequence) = self.touch_sequence_map.get_mut(&sequence_id) {
            sequence.pending_touch_move_action = None;
        }
    }

    // try to remove touch sequence, if touch sequence end and not has pending action.
    pub(crate) fn try_remove_touch_sequence(&mut self, sequence_id: TouchSequenceId) {
        if let Some(sequence) = self.touch_sequence_map.get(&sequence_id) {
            if sequence.pending_touch_move_action.is_none() && sequence.state == Finished {
                self.touch_sequence_map.remove(&sequence_id);
            }
        }
    }

    pub(crate) fn remove_touch_sequence(&mut self, sequence_id: TouchSequenceId) {
        let old = self.touch_sequence_map.remove(&sequence_id);
        debug_assert!(old.is_some(), "Sequence already removed?");
    }

    pub fn get_current_touch_sequence_mut(&mut self) -> &mut TouchSequenceInfo {
        self.touch_sequence_map
            .get_mut(&self.current_sequence_id)
            .expect("Current Touch sequence does not exist")
    }

    pub(crate) fn get_touch_sequence(&self, sequence_id: TouchSequenceId) -> &TouchSequenceInfo {
        self.touch_sequence_map
            .get(&sequence_id)
            .expect("Touch sequence not found.")
    }
    pub(crate) fn get_touch_sequence_mut(
        &mut self,
        sequence_id: TouchSequenceId,
    ) -> &mut TouchSequenceInfo {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .expect("Touch sequence not found.")
    }

    pub fn on_touch_down(&mut self, id: TouchId, point: Point2D<f32, DevicePixel>) {
        // if the current sequence ID does not exist in the map, then it was already handled
        if !self
            .touch_sequence_map
            .contains_key(&self.current_sequence_id) ||
            self.get_touch_sequence(self.current_sequence_id)
                .is_finished()
        {
            self.current_sequence_id.next();
            debug!("Entered new touch sequence: {:?}", self.current_sequence_id);
            let active_touch_points = vec![TouchPoint::new(id, point)];
            self.touch_sequence_map.insert(
                self.current_sequence_id,
                TouchSequenceInfo {
                    state: Touching,
                    active_touch_points,
                    handling_touch_move: false,
                    prevent_click: false,
                    prevent_move: TouchMoveAllowed::Pending,
                    pending_touch_move_action: None,
                },
            );
        } else {
            debug!("Touch down in sequence {:?}.", self.current_sequence_id);
            let touch_sequence = self.get_current_touch_sequence_mut();
            touch_sequence
                .active_touch_points
                .push(TouchPoint::new(id, point));
            match touch_sequence.active_touch_points.len() {
                2 => {
                    touch_sequence.state = Pinching;
                },
                3.. => {
                    touch_sequence.state = MultiTouch;
                },
                0..2 => {
                    unreachable!("Secondary touch_down event with less than 2 fingers active?");
                },
            }
            // Multiple fingers prevent a click.
            touch_sequence.prevent_click = true;
        }
    }

    pub fn on_vsync(&mut self) -> Option<FlingAction> {
        let touch_sequence = self.touch_sequence_map.get_mut(&self.current_sequence_id)?;

        let Flinging {
            velocity,
            ref cursor,
        } = &mut touch_sequence.state
        else {
            return None;
        };
        if velocity.length().abs() < FLING_MIN_SCREEN_PX {
            touch_sequence.state = Finished;
            // If we were flinging previously, there could still be a touch_up event result
            // coming in after we stopped flinging
            self.try_remove_touch_sequence(self.current_sequence_id);
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
    }

    pub fn on_touch_move(
        &mut self,
        id: TouchId,
        point: Point2D<f32, DevicePixel>,
    ) -> TouchMoveAction {
        let touch_sequence = self.get_touch_sequence_mut(self.current_sequence_id);
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
                    TouchMoveAction::Scroll(delta, point)
                } else if delta.x.abs() > TOUCH_PAN_MIN_SCREEN_PX ||
                    delta.y.abs() > TOUCH_PAN_MIN_SCREEN_PX
                {
                    touch_sequence.state = Panning {
                        velocity: Vector2D::new(delta.x, delta.y),
                    };
                    // No clicks should be issued after we transitioned to move.
                    touch_sequence.prevent_click = true;
                    // update the touch point
                    touch_sequence.active_touch_points[idx].point = point;
                    TouchMoveAction::Scroll(delta, point)
                } else {
                    // We don't update the touchpoint, so multiple small moves can
                    // accumulate and merge into a larger move.
                    TouchMoveAction::NoAction
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
                    TouchMoveAction::Zoom(magnification, scroll_delta)
                } else {
                    // We don't update the touchpoint, so multiple small moves can
                    // accumulate and merge into a larger move.
                    TouchMoveAction::NoAction
                }
            },
            _ => {
                touch_sequence.active_touch_points[idx].point = point;
                touch_sequence.state = MultiTouch;
                TouchMoveAction::NoAction
            },
        };
        // If the touch action is not `NoAction` and the first move has not been processed,
        //  set pending_touch_move_action.
        if TouchMoveAction::NoAction != action &&
            touch_sequence.prevent_move == TouchMoveAllowed::Pending
        {
            touch_sequence.update_pending_touch_move_action(action);
        }

        action
    }

    pub fn on_touch_up(&mut self, id: TouchId, point: Point2D<f32, DevicePixel>) {
        let touch_sequence = self.get_touch_sequence_mut(self.current_sequence_id);
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
        match touch_sequence.state {
            Touching => {
                if touch_sequence.prevent_click {
                    touch_sequence.state = Finished;
                } else {
                    touch_sequence.state = PendingClick(point);
                }
            },
            Panning { velocity } => {
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
                    match touch_sequence.prevent_move {
                        TouchMoveAllowed::Allowed => {
                            touch_sequence.state = Flinging { velocity, cursor }
                            // todo: return Touchaction here, or is it sufficient to just
                            // wait for the next vsync?
                        },
                        TouchMoveAllowed::Pending => {
                            touch_sequence.state = PendingFling { velocity, cursor }
                        },
                        TouchMoveAllowed::Prevented => touch_sequence.state = Finished,
                    }
                } else {
                    touch_sequence.state = Finished;
                }
            },
            Pinching => {
                touch_sequence.state = Touching;
            },
            MultiTouch => {
                // We stay in multi-touch mode once we entered it until all fingers are lifted.
                if touch_sequence.active_touch_points.is_empty() {
                    touch_sequence.state = Finished;
                }
            },
            PendingFling { .. } | Flinging { .. } | PendingClick(_) | Finished => {
                error!("Touch-up received, but touch handler already in post-touchup state.")
            },
        }
        #[cfg(debug_assertions)]
        if touch_sequence.active_touch_points.is_empty() {
            debug_assert!(
                touch_sequence.is_finished(),
                "Did not transition to a finished state: {:?}",
                touch_sequence.state
            );
        }
        debug!(
            "Touch up with remaining active touchpoints: {:?}, in sequence {:?}",
            touch_sequence.active_touch_points.len(),
            self.current_sequence_id
        );
    }

    pub fn on_touch_cancel(&mut self, id: TouchId, _point: Point2D<f32, DevicePixel>) {
        let touch_sequence = self.get_touch_sequence_mut(self.current_sequence_id);
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
}
