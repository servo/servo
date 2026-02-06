/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use base::id::WebViewId;
use embedder_traits::{InputEventId, PaintHitTestResult, Scroll, TouchEventType, TouchId};
use euclid::{Point2D, Scale, Vector2D};
use log::{debug, error, warn};
use rustc_hash::FxHashMap;
use style_traits::CSSPixel;
use webrender_api::units::{DevicePixel, DevicePoint, DeviceVector2D};

use self::TouchSequenceState::*;
use crate::paint::RepaintReason;
use crate::painter::Painter;
use crate::refresh_driver::{BaseRefreshDriver, RefreshDriverObserver};
use crate::webview_renderer::{ScrollEvent, ScrollZoomEvent, WebViewRenderer};

/// An ID for a sequence of touch events between a `Down` and the `Up` or `Cancel` event.
/// The ID is the same for all events between `Down` and `Up` or `Cancel`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) struct TouchSequenceId(u32);

impl TouchSequenceId {
    const fn new() -> Self {
        Self(0)
    }

    /// Increments the ID for the next touch sequence.
    ///
    /// The increment is wrapping, since we can assume that the touch handler
    /// script for touch sequence N will have finished processing by the time
    /// we have wrapped around.
    fn next(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

/// Minimum number of `DeviceIndependentPixel` to begin touch scrolling/Pinching.
const TOUCH_PAN_MIN_SCREEN_PX: f32 = 20.0;
/// Factor by which the flinging velocity changes on each tick.
const FLING_SCALING_FACTOR: f32 = 0.95;
/// Minimum velocity required for transitioning to fling when panning ends.
const FLING_MIN_SCREEN_PX: f32 = 3.0;
/// Maximum velocity when flinging.
const FLING_MAX_SCREEN_PX: f32 = 4000.0;

pub struct TouchHandler {
    /// The [`WebViewId`] of the `WebView` this [`TouchHandler`] is associated with.
    webview_id: WebViewId,
    pub current_sequence_id: TouchSequenceId,
    // todo: VecDeque + modulo arithmetic would be more efficient.
    touch_sequence_map: FxHashMap<TouchSequenceId, TouchSequenceInfo>,
    /// A set of [`InputEventId`]s for touch events that have been sent to the Constellation
    /// and have not been handled yet.
    pub(crate) pending_touch_input_events: RefCell<FxHashMap<InputEventId, PendingTouchInputEvent>>,
    /// Whether or not the [`FlingRefreshDriverObserver`] is currently observing frames for fling.
    observing_frames_for_fling: Cell<bool>,
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

/// A cached [`PaintHitTestResult`] to use during a touch sequence. This
/// is kept so that the renderer doesn't have to constantly keep making hit tests
/// while during panning and flinging actions.
struct HitTestResultCache {
    value: PaintHitTestResult,
    device_pixels_per_page: Scale<f32, CSSPixel, DevicePixel>,
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
    pending_touch_move_actions: Vec<ScrollZoomEvent>,
    /// Cache for the last touch hit test result.
    hit_test_result_cache: Option<HitTestResultCache>,
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

    fn add_pending_touch_move_action(&mut self, action: ScrollZoomEvent) {
        debug_assert!(self.prevent_move == TouchMoveAllowed::Pending);
        self.pending_touch_move_actions.push(action);
    }

    /// Returns true when all touch events of a sequence have been received.
    /// This does not mean that all event handlers have finished yet.
    fn is_finished(&self) -> bool {
        matches!(
            self.state,
            Finished | Flinging { .. } | PendingFling { .. } | PendingClick(_)
        )
    }

    fn update_hit_test_result_cache_pointer(&mut self, delta: Vector2D<f32, DevicePixel>) {
        if let Some(ref mut hit_test_result_cache) = self.hit_test_result_cache {
            let scaled_delta = delta / hit_test_result_cache.device_pixels_per_page;
            // Update the point of the hit test result to match the current touch point.
            hit_test_result_cache.value.point_in_viewport += scaled_delta;
        }
    }
}

/// An action that can be immediately performed in response to a touch move event
/// without waiting for script.
#[derive(Clone, Copy, Debug, PartialEq)]

pub struct TouchPoint {
    pub id: TouchId,
    pub point: Point2D<f32, DevicePixel>,
}

impl TouchPoint {
    fn new(id: TouchId, point: Point2D<f32, DevicePixel>) -> Self {
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
        point: DevicePoint,
    },
    /// No active touch points, but there is still scrolling velocity
    Flinging {
        velocity: Vector2D<f32, DevicePixel>,
        point: DevicePoint,
    },
    /// The touch sequence is finished, but a click is still pending, waiting on script.
    PendingClick(DevicePoint),
    /// touch sequence finished.
    Finished,
}

pub(crate) struct FlingAction {
    pub delta: DeviceVector2D,
    pub cursor: DevicePoint,
}

impl TouchHandler {
    pub(crate) fn new(webview_id: WebViewId) -> Self {
        let finished_info = TouchSequenceInfo {
            state: TouchSequenceState::Finished,
            active_touch_points: vec![],
            handling_touch_move: false,
            prevent_click: false,
            prevent_move: TouchMoveAllowed::Pending,
            pending_touch_move_actions: vec![],
            hit_test_result_cache: None,
        };
        // We insert a simulated initial touch sequence, which is already finished,
        // so that we always have one element in the map, which simplifies creating
        // a new touch sequence on touch_down.
        let mut touch_sequence_map = FxHashMap::default();
        touch_sequence_map.insert(TouchSequenceId::new(), finished_info);
        TouchHandler {
            webview_id,
            current_sequence_id: TouchSequenceId::new(),
            touch_sequence_map,
            pending_touch_input_events: Default::default(),
            observing_frames_for_fling: Default::default(),
        }
    }

    pub(crate) fn set_handling_touch_move(&mut self, sequence_id: TouchSequenceId, flag: bool) {
        if let Some(sequence) = self.touch_sequence_map.get_mut(&sequence_id) {
            sequence.handling_touch_move = flag;
        }
    }

    pub(crate) fn is_handling_touch_move(&self, sequence_id: TouchSequenceId) -> bool {
        self.touch_sequence_map
            .get(&sequence_id)
            .is_some_and(|seq| seq.handling_touch_move)
    }

    pub(crate) fn prevent_click(&mut self, sequence_id: TouchSequenceId) {
        if let Some(sequence) = self.touch_sequence_map.get_mut(&sequence_id) {
            sequence.prevent_click = true;
        } else {
            warn!("TouchSequenceInfo corresponding to the sequence number has been deleted.");
        }
    }

    pub(crate) fn prevent_move(&mut self, sequence_id: TouchSequenceId) {
        if let Some(sequence) = self.touch_sequence_map.get_mut(&sequence_id) {
            sequence.prevent_move = TouchMoveAllowed::Prevented;
        } else {
            warn!("TouchSequenceInfo corresponding to the sequence number has been deleted.");
        }
    }

    /// Returns true if default move actions are allowed, false if prevented or the result
    /// is still pending.,
    pub(crate) fn move_allowed(&self, sequence_id: TouchSequenceId) -> bool {
        self.touch_sequence_map
            .get(&sequence_id)
            .is_none_or(|sequence| sequence.prevent_move == TouchMoveAllowed::Allowed)
    }

    pub(crate) fn take_pending_touch_move_actions(
        &mut self,
        sequence_id: TouchSequenceId,
    ) -> Vec<ScrollZoomEvent> {
        self.touch_sequence_map
            .get_mut(&sequence_id)
            .map(|sequence| std::mem::take(&mut sequence.pending_touch_move_actions))
            .unwrap_or_default()
    }

    pub(crate) fn remove_pending_touch_move_actions(&mut self, sequence_id: TouchSequenceId) {
        if let Some(sequence) = self.touch_sequence_map.get_mut(&sequence_id) {
            sequence.pending_touch_move_actions.clear();
        }
    }

    // try to remove touch sequence, if touch sequence end and not has pending action.
    pub(crate) fn try_remove_touch_sequence(&mut self, sequence_id: TouchSequenceId) {
        if let Some(sequence) = self.touch_sequence_map.get(&sequence_id) {
            if sequence.pending_touch_move_actions.is_empty() && sequence.state == Finished {
                self.touch_sequence_map.remove(&sequence_id);
            }
        }
    }

    pub(crate) fn remove_touch_sequence(&mut self, sequence_id: TouchSequenceId) {
        let old = self.touch_sequence_map.remove(&sequence_id);
        debug_assert!(old.is_some(), "Sequence already removed?");
    }

    fn get_current_touch_sequence_mut(&mut self) -> &mut TouchSequenceInfo {
        self.touch_sequence_map
            .get_mut(&self.current_sequence_id)
            .expect("Current Touch sequence does not exist")
    }

    fn try_get_current_touch_sequence(&self) -> Option<&TouchSequenceInfo> {
        self.touch_sequence_map.get(&self.current_sequence_id)
    }

    fn try_get_current_touch_sequence_mut(&mut self) -> Option<&mut TouchSequenceInfo> {
        self.touch_sequence_map.get_mut(&self.current_sequence_id)
    }

    fn get_touch_sequence(&self, sequence_id: TouchSequenceId) -> &TouchSequenceInfo {
        self.touch_sequence_map
            .get(&sequence_id)
            .expect("Touch sequence not found.")
    }

    pub(crate) fn get_touch_sequence_mut(
        &mut self,
        sequence_id: TouchSequenceId,
    ) -> Option<&mut TouchSequenceInfo> {
        self.touch_sequence_map.get_mut(&sequence_id)
    }

    pub(crate) fn on_touch_down(&mut self, id: TouchId, point: Point2D<f32, DevicePixel>) {
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
                    pending_touch_move_actions: vec![],
                    hit_test_result_cache: None,
                },
            );
        } else {
            debug!("Touch down in sequence {:?}.", self.current_sequence_id);
            let touch_sequence = self.get_current_touch_sequence_mut();
            touch_sequence
                .active_touch_points
                .push(TouchPoint::new(id, point));
            match touch_sequence.active_touch_points.len() {
                2.. => {
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

    pub(crate) fn notify_new_frame_start(&mut self) -> Option<FlingAction> {
        let touch_sequence = self.touch_sequence_map.get_mut(&self.current_sequence_id)?;

        let Flinging {
            velocity,
            point: cursor,
        } = &mut touch_sequence.state
        else {
            self.observing_frames_for_fling.set(false);
            return None;
        };

        if velocity.length().abs() < FLING_MIN_SCREEN_PX {
            self.stop_fling_if_needed();
            None
        } else {
            // TODO: Probably we should multiply with the current refresh rate (and divide on each frame)
            // or save a timestamp to account for a potentially changing display refresh rate.
            *velocity *= FLING_SCALING_FACTOR;
            let _span = profile_traits::info_span!(
                "TouchHandler::Flinging",
                velocity = ?velocity,
            )
            .entered();
            debug_assert!(velocity.length() <= FLING_MAX_SCREEN_PX);
            Some(FlingAction {
                delta: DeviceVector2D::new(velocity.x, velocity.y),
                cursor: *cursor,
            })
        }
    }

    pub(crate) fn stop_fling_if_needed(&mut self) {
        let current_sequence_id = self.current_sequence_id;
        let Some(touch_sequence) = self.try_get_current_touch_sequence_mut() else {
            debug!(
                "Touch sequence already removed before stoping potential flinging during Paint update"
            );
            return;
        };
        let Flinging { .. } = touch_sequence.state else {
            return;
        };
        let _span = profile_traits::info_span!("TouchHandler::FlingEnd").entered();
        debug!("Stopping flinging in touch sequence {current_sequence_id:?}");
        touch_sequence.state = Finished;
        // If we were flinging previously, there could still be a touch_up event result
        // coming in after we stopped flinging
        self.try_remove_touch_sequence(current_sequence_id);
        self.observing_frames_for_fling.set(false);
    }

    pub(crate) fn on_touch_move(
        &mut self,
        id: TouchId,
        point: Point2D<f32, DevicePixel>,
        scale: f32,
    ) -> Option<ScrollZoomEvent> {
        // As `TouchHandler` is per `WebViewRenderer` which is per `WebView` we might get a Touch Sequence Move that
        // started with a down on a different webview. As the touch_sequence id is only changed on touch_down this
        // move event gets a touch id which is already cleaned up.
        let touch_sequence = self.try_get_current_touch_sequence_mut()?;
        let idx = match touch_sequence
            .active_touch_points
            .iter_mut()
            .position(|t| t.id == id)
        {
            Some(i) => i,
            None => {
                error!("Got a touchmove event for a non-active touch point");
                return None;
            },
        };
        let old_point = touch_sequence.active_touch_points[idx].point;
        let delta = point - old_point;
        touch_sequence.update_hit_test_result_cache_pointer(delta);

        let action = match touch_sequence.touch_count() {
            1 => {
                if let Panning { ref mut velocity } = touch_sequence.state {
                    // TODO: Probably we should track 1-3 more points and use a smarter algorithm
                    *velocity += delta;
                    *velocity /= 2.0;
                    // update the touch point every time when panning.
                    touch_sequence.active_touch_points[idx].point = point;

                    // Scroll offsets are opposite to the direction of finger motion.
                    Some(ScrollZoomEvent::Scroll(ScrollEvent {
                        scroll: Scroll::Delta((-delta).into()),
                        point,
                        event_count: 1,
                    }))
                } else if delta.x.abs() > TOUCH_PAN_MIN_SCREEN_PX * scale ||
                    delta.y.abs() > TOUCH_PAN_MIN_SCREEN_PX * scale
                {
                    let _span = profile_traits::info_span!(
                        "TouchHandler::ScrollBegin",
                        delta = ?delta,
                    )
                    .entered();
                    touch_sequence.state = Panning {
                        velocity: Vector2D::new(delta.x, delta.y),
                    };
                    // No clicks should be issued after we transitioned to move.
                    touch_sequence.prevent_click = true;
                    // update the touch point
                    touch_sequence.active_touch_points[idx].point = point;

                    // Scroll offsets are opposite to the direction of finger motion.
                    Some(ScrollZoomEvent::Scroll(ScrollEvent {
                        scroll: Scroll::Delta((-delta).into()),
                        point,
                        event_count: 1,
                    }))
                } else {
                    // We don't update the touchpoint, so multiple small moves can
                    // accumulate and merge into a larger move.
                    None
                }
            },
            2 => {
                if touch_sequence.state == Pinching ||
                    delta.x.abs() > TOUCH_PAN_MIN_SCREEN_PX * scale ||
                    delta.y.abs() > TOUCH_PAN_MIN_SCREEN_PX * scale
                {
                    touch_sequence.state = Pinching;
                    let (d0, _) = touch_sequence.pinch_distance_and_center();

                    // update the touch point with the enough distance or pinching.
                    touch_sequence.active_touch_points[idx].point = point;
                    let (d1, c1) = touch_sequence.pinch_distance_and_center();

                    Some(ScrollZoomEvent::PinchZoom(d1 / d0, c1))
                } else {
                    // We don't update the touchpoint, so multiple small moves can
                    // accumulate and merge into a larger move.
                    None
                }
            },
            _ => {
                touch_sequence.active_touch_points[idx].point = point;
                touch_sequence.state = MultiTouch;
                None
            },
        };
        // If the touch action is not `NoAction` and the first move has not been processed,
        //  set pending_touch_move_action.
        if let Some(action) = action {
            if touch_sequence.prevent_move == TouchMoveAllowed::Pending {
                touch_sequence.add_pending_touch_move_action(action);
            }
        }

        action
    }

    pub(crate) fn on_touch_up(&mut self, id: TouchId, point: Point2D<f32, DevicePixel>) {
        let Some(touch_sequence) = self.try_get_current_touch_sequence_mut() else {
            warn!("Current touch sequence not found");
            return;
        };
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
                    let _span = profile_traits::info_span!(
                        "TouchHandler::FlingStart",
                        velocity = ?velocity,
                    )
                    .entered();
                    // TODO: point != old. Not sure which one is better to take as cursor for flinging.
                    debug!(
                        "Transitioning to Fling. Cursor is {point:?}. Old cursor was {old:?}. \
                            Raw velocity is {velocity:?}."
                    );

                    // Multiplying the initial velocity gives the fling a much more snappy feel
                    // and serves well as a poor-mans acceleration algorithm.
                    let velocity = (velocity * 2.0).with_max_length(FLING_MAX_SCREEN_PX);
                    match touch_sequence.prevent_move {
                        TouchMoveAllowed::Allowed => {
                            touch_sequence.state = Flinging { velocity, point }
                            // todo: return Touchaction here, or is it sufficient to just
                            // wait for the next vsync?
                        },
                        TouchMoveAllowed::Pending => {
                            touch_sequence.state = PendingFling { velocity, point }
                        },
                        TouchMoveAllowed::Prevented => touch_sequence.state = Finished,
                    }
                } else {
                    let _span = profile_traits::info_span!("TouchHandler::ScrollEnd").entered();
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

    pub(crate) fn on_touch_cancel(&mut self, id: TouchId, _point: Point2D<f32, DevicePixel>) {
        // A similar thing with touch move can happen here where the event is coming from a different webview.
        let Some(touch_sequence) = self.try_get_current_touch_sequence_mut() else {
            return;
        };
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
        if touch_sequence.active_touch_points.is_empty() {
            touch_sequence.state = Finished;
        }
    }

    pub(crate) fn get_hit_test_result_cache_value(&self) -> Option<PaintHitTestResult> {
        let sequence = self.touch_sequence_map.get(&self.current_sequence_id)?;
        if sequence.state == Finished {
            return None;
        }
        sequence
            .hit_test_result_cache
            .as_ref()
            .map(|cache| Some(cache.value.clone()))?
    }

    pub(crate) fn set_hit_test_result_cache_value(
        &mut self,
        value: PaintHitTestResult,
        device_pixels_per_page: Scale<f32, CSSPixel, DevicePixel>,
    ) {
        if let Some(sequence) = self.touch_sequence_map.get_mut(&self.current_sequence_id) {
            if sequence.hit_test_result_cache.is_none() {
                sequence.hit_test_result_cache = Some(HitTestResultCache {
                    value,
                    device_pixels_per_page,
                });
            }
        }
    }

    pub(crate) fn add_pending_touch_input_event(
        &self,
        id: InputEventId,
        event_type: TouchEventType,
    ) {
        self.pending_touch_input_events.borrow_mut().insert(
            id,
            PendingTouchInputEvent {
                event_type,
                sequence_id: self.current_sequence_id,
            },
        );
    }

    pub(crate) fn take_pending_touch_input_event(
        &self,
        id: InputEventId,
    ) -> Option<PendingTouchInputEvent> {
        self.pending_touch_input_events.borrow_mut().remove(&id)
    }

    pub(crate) fn add_touch_move_refresh_observer_if_necessary(
        &self,
        refresh_driver: Rc<BaseRefreshDriver>,
        repaint_reason: &Cell<RepaintReason>,
    ) {
        if self.observing_frames_for_fling.get() {
            return;
        }

        let Some(current_touch_sequence) = self.try_get_current_touch_sequence() else {
            return;
        };

        if !matches!(
            current_touch_sequence.state,
            TouchSequenceState::Flinging { .. },
        ) {
            return;
        }

        refresh_driver.add_observer(Rc::new(FlingRefreshDriverObserver {
            webview_id: self.webview_id,
        }));
        self.observing_frames_for_fling.set(true);
        repaint_reason.set(repaint_reason.get().union(RepaintReason::StartedFlinging));
    }
}

/// This data structure is used to store information about touch events that are
/// sent from the Renderer to the Constellation, so that they can finish processing
/// once their DOM events are fired.
pub(crate) struct PendingTouchInputEvent {
    pub event_type: TouchEventType,
    pub sequence_id: TouchSequenceId,
}

pub(crate) struct FlingRefreshDriverObserver {
    pub webview_id: WebViewId,
}

impl RefreshDriverObserver for FlingRefreshDriverObserver {
    fn frame_started(&self, painter: &mut Painter) -> bool {
        painter
            .webview_renderer_mut(self.webview_id)
            .is_some_and(WebViewRenderer::update_touch_handling_at_new_frame_start)
    }
}
