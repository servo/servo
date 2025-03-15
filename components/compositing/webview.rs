/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::{Entry, Keys, Values, ValuesMut};
use std::rc::Rc;

use base::id::{PipelineId, WebViewId};
use compositing_traits::{ConstellationMsg, SendableFrameTree};
use embedder_traits::{
    InputEvent, MouseButton, MouseButtonAction, MouseButtonEvent, MouseMoveEvent, ShutdownState,
    TouchEvent, TouchEventType, TouchId,
};
use euclid::{Point2D, Scale, Vector2D};
use fnv::FnvHashSet;
use log::{debug, warn};
use script_traits::{AnimationState, ScriptThreadMessage, TouchEventResult};
use webrender::Transaction;
use webrender_api::units::{DeviceIntPoint, DevicePoint, DeviceRect, LayoutVector2D};
use webrender_api::{
    ExternalScrollId, HitTestFlags, RenderReasons, SampledScrollOffset, ScrollLocation,
};
use webrender_traits::{CompositorHitTestResult, ScrollState};

use crate::IOCompositor;
use crate::compositor::{PipelineDetails, ServoRenderer};
use crate::touch::{TouchHandler, TouchMoveAction, TouchMoveAllowed, TouchSequenceState};

#[derive(Clone, Copy)]
struct ScrollEvent {
    /// Scroll by this offset, or to Start or End
    scroll_location: ScrollLocation,
    /// Apply changes to the frame at this location
    cursor: DeviceIntPoint,
    /// The number of OS events that have been coalesced together into this one event.
    event_count: u32,
}

#[derive(Clone, Copy)]
enum ScrollZoomEvent {
    /// An pinch zoom event that magnifies the view by the given factor.
    PinchZoom(f32),
    /// A scroll event that scrolls the scroll node at the given location by the
    /// given amount.
    Scroll(ScrollEvent),
}

pub(crate) struct WebView {
    /// The [`WebViewId`] of the `WebView` associated with this [`WebViewDetails`].
    pub id: WebViewId,
    /// The root [`PipelineId`] of the currently displayed page in this WebView.
    pub root_pipeline_id: Option<PipelineId>,
    pub rect: DeviceRect,
    /// Tracks details about each active pipeline that the compositor knows about.
    pub pipelines: HashMap<PipelineId, PipelineDetails>,
    /// Data that is shared by all WebView renderers.
    pub(crate) global: Rc<RefCell<ServoRenderer>>,
    /// Pending scroll/zoom events.
    pending_scroll_zoom_events: Vec<ScrollZoomEvent>,
    /// Touch input state machine
    touch_handler: TouchHandler,
}

impl Drop for WebView {
    fn drop(&mut self) {
        self.global
            .borrow_mut()
            .pipeline_to_webview_map
            .retain(|_, webview_id| self.id != *webview_id);
    }
}

impl WebView {
    pub(crate) fn new(id: WebViewId, rect: DeviceRect, global: Rc<RefCell<ServoRenderer>>) -> Self {
        Self {
            id,
            root_pipeline_id: None,
            rect,
            pipelines: Default::default(),
            touch_handler: TouchHandler::new(),
            global,
            pending_scroll_zoom_events: Default::default(),
        }
    }

    pub(crate) fn animations_or_animation_callbacks_running(&self) -> bool {
        self.pipelines
            .values()
            .any(PipelineDetails::animations_or_animation_callbacks_running)
    }

    pub(crate) fn animation_callbacks_running(&self) -> bool {
        self.pipelines
            .values()
            .any(PipelineDetails::animation_callbacks_running)
    }

    pub(crate) fn pipeline_ids(&self) -> Keys<'_, PipelineId, PipelineDetails> {
        self.pipelines.keys()
    }

    /// Returns the [`PipelineDetails`] for the given [`PipelineId`], creating it if needed.
    pub(crate) fn ensure_pipeline_details(
        &mut self,
        pipeline_id: PipelineId,
    ) -> &mut PipelineDetails {
        self.pipelines.entry(pipeline_id).or_insert_with(|| {
            self.global
                .borrow_mut()
                .pipeline_to_webview_map
                .insert(pipeline_id, self.id);
            PipelineDetails::new(pipeline_id)
        })
    }

    pub(crate) fn set_throttled(&mut self, pipeline_id: PipelineId, throttled: bool) {
        self.ensure_pipeline_details(pipeline_id).throttled = throttled;
    }

    pub(crate) fn remove_pipeline(&mut self, pipeline_id: PipelineId) {
        self.global
            .borrow_mut()
            .pipeline_to_webview_map
            .remove(&pipeline_id);
        self.pipelines.remove(&pipeline_id);
    }

    pub(crate) fn set_frame_tree(&mut self, frame_tree: &SendableFrameTree) {
        let pipeline_id = frame_tree.pipeline.id;
        let old_pipeline_id = std::mem::replace(&mut self.root_pipeline_id, Some(pipeline_id));

        if old_pipeline_id != self.root_pipeline_id {
            debug!(
                "Updating webview ({:?}) from pipeline {:?} to {:?}",
                3, old_pipeline_id, self.root_pipeline_id
            );
        }

        self.set_frame_tree_on_pipeline_details(frame_tree, None);
        self.reset_scroll_tree_for_unattached_pipelines(frame_tree);
        self.send_scroll_positions_to_layout_for_pipeline(pipeline_id);
    }

    pub(crate) fn send_scroll_positions_to_layout_for_pipeline(&self, pipeline_id: PipelineId) {
        let Some(details) = self.pipelines.get(&pipeline_id) else {
            return;
        };

        let mut scroll_states = Vec::new();
        details.scroll_tree.nodes.iter().for_each(|node| {
            if let (Some(scroll_id), Some(scroll_offset)) = (node.external_id(), node.offset()) {
                scroll_states.push(ScrollState {
                    scroll_id,
                    scroll_offset,
                });
            }
        });

        if let Some(pipeline) = details.pipeline.as_ref() {
            let message = ScriptThreadMessage::SetScrollStates(pipeline_id, scroll_states);
            let _ = pipeline.script_chan.send(message);
        }
    }

    pub(crate) fn set_frame_tree_on_pipeline_details(
        &mut self,
        frame_tree: &SendableFrameTree,
        parent_pipeline_id: Option<PipelineId>,
    ) {
        let pipeline_id = frame_tree.pipeline.id;
        let pipeline_details = self.ensure_pipeline_details(pipeline_id);
        pipeline_details.pipeline = Some(frame_tree.pipeline.clone());
        pipeline_details.parent_pipeline_id = parent_pipeline_id;

        for kid in &frame_tree.children {
            self.set_frame_tree_on_pipeline_details(kid, Some(pipeline_id));
        }
    }

    pub(crate) fn reset_scroll_tree_for_unattached_pipelines(
        &mut self,
        frame_tree: &SendableFrameTree,
    ) {
        // TODO(mrobinson): Eventually this can selectively preserve the scroll trees
        // state for some unattached pipelines in order to preserve scroll position when
        // navigating backward and forward.
        fn collect_pipelines(
            pipelines: &mut FnvHashSet<PipelineId>,
            frame_tree: &SendableFrameTree,
        ) {
            pipelines.insert(frame_tree.pipeline.id);
            for kid in &frame_tree.children {
                collect_pipelines(pipelines, kid);
            }
        }

        let mut attached_pipelines: FnvHashSet<PipelineId> = FnvHashSet::default();
        collect_pipelines(&mut attached_pipelines, frame_tree);

        self.pipelines
            .iter_mut()
            .filter(|(id, _)| !attached_pipelines.contains(id))
            .for_each(|(_, details)| {
                details.scroll_tree.nodes.iter_mut().for_each(|node| {
                    node.set_offset(LayoutVector2D::zero());
                })
            })
    }

    /// Sets or unsets the animations-running flag for the given pipeline, and schedules a
    /// recomposite if necessary. Returns true if the pipeline is throttled.
    pub(crate) fn change_running_animations_state(
        &mut self,
        pipeline_id: PipelineId,
        animation_state: AnimationState,
    ) -> bool {
        let pipeline_details = self.ensure_pipeline_details(pipeline_id);
        match animation_state {
            AnimationState::AnimationsPresent => {
                pipeline_details.animations_running = true;
            },
            AnimationState::AnimationCallbacksPresent => {
                pipeline_details.animation_callbacks_running = true;
            },
            AnimationState::NoAnimationsPresent => {
                pipeline_details.animations_running = false;
            },
            AnimationState::NoAnimationCallbacksPresent => {
                pipeline_details.animation_callbacks_running = false;
            },
        }
        pipeline_details.throttled
    }

    pub(crate) fn tick_all_animations(&self, compositor: &IOCompositor) -> bool {
        let mut ticked_any = false;
        for pipeline_details in self.pipelines.values() {
            ticked_any = pipeline_details.tick_animations(compositor) || ticked_any;
        }
        ticked_any
    }

    pub(crate) fn tick_animations_for_pipeline(
        &self,
        pipeline_id: PipelineId,
        compositor: &IOCompositor,
    ) {
        if let Some(pipeline_details) = self.pipelines.get(&pipeline_id) {
            pipeline_details.tick_animations(compositor);
        }
    }

    /// On a Window refresh tick (e.g. vsync)
    pub fn on_vsync(&mut self) {
        if let Some(fling_action) = self.touch_handler.on_vsync() {
            self.on_scroll_window_event(
                ScrollLocation::Delta(fling_action.delta),
                fling_action.cursor,
            );
        }
    }

    pub(crate) fn dispatch_input_event(&mut self, event: InputEvent) {
        // Events that do not need to do hit testing are sent directly to the
        // constellation to filter down.
        let Some(point) = event.point() else {
            return;
        };

        // If we can't find a pipeline to send this event to, we cannot continue.
        let get_pipeline_details = |pipeline_id| self.pipelines.get(&pipeline_id);
        let Some(result) = self
            .global
            .borrow()
            .hit_test_at_point(point, get_pipeline_details)
        else {
            return;
        };

        self.global.borrow_mut().update_cursor(point, &result);

        if let Err(error) =
            self.global
                .borrow()
                .constellation_sender
                .send(ConstellationMsg::ForwardInputEvent(
                    self.id,
                    event,
                    Some(result),
                ))
        {
            warn!("Sending event to constellation failed ({error:?}).");
        }
    }

    pub fn notify_input_event(&mut self, event: InputEvent) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }

        if let InputEvent::Touch(event) = event {
            self.on_touch_event(event);
            return;
        }

        if self.global.borrow().convert_mouse_to_touch {
            match event {
                InputEvent::MouseButton(event) => {
                    match event.action {
                        MouseButtonAction::Click => {},
                        MouseButtonAction::Down => self.on_touch_down(TouchEvent::new(
                            TouchEventType::Down,
                            TouchId(0),
                            event.point,
                        )),
                        MouseButtonAction::Up => self.on_touch_up(TouchEvent::new(
                            TouchEventType::Up,
                            TouchId(0),
                            event.point,
                        )),
                    }
                    return;
                },
                InputEvent::MouseMove(event) => {
                    self.on_touch_move(TouchEvent::new(
                        TouchEventType::Move,
                        TouchId(0),
                        event.point,
                    ));
                    return;
                },
                _ => {},
            }
        }

        self.dispatch_input_event(event);
    }

    fn send_touch_event(&self, mut event: TouchEvent) -> bool {
        let get_pipeline_details = |pipeline_id| self.pipelines.get(&pipeline_id);
        let Some(result) = self
            .global
            .borrow()
            .hit_test_at_point(event.point, get_pipeline_details)
        else {
            return false;
        };

        event.init_sequence_id(self.touch_handler.current_sequence_id);
        let event = InputEvent::Touch(event);
        if let Err(e) =
            self.global
                .borrow()
                .constellation_sender
                .send(ConstellationMsg::ForwardInputEvent(
                    self.id,
                    event,
                    Some(result),
                ))
        {
            warn!("Sending event to constellation failed ({:?}).", e);
            false
        } else {
            true
        }
    }

    pub fn on_touch_event(&mut self, event: TouchEvent) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }

        match event.event_type {
            TouchEventType::Down => self.on_touch_down(event),
            TouchEventType::Move => self.on_touch_move(event),
            TouchEventType::Up => self.on_touch_up(event),
            TouchEventType::Cancel => self.on_touch_cancel(event),
        }
    }

    fn on_touch_down(&mut self, event: TouchEvent) {
        self.touch_handler.on_touch_down(event.id, event.point);
        self.send_touch_event(event);
    }

    fn on_touch_move(&mut self, mut event: TouchEvent) {
        let action: TouchMoveAction = self.touch_handler.on_touch_move(event.id, event.point);
        if TouchMoveAction::NoAction != action {
            // if first move processed and allowed, we directly process the move event,
            // without waiting for the script handler.
            if self
                .touch_handler
                .move_allowed(self.touch_handler.current_sequence_id)
            {
                // https://w3c.github.io/touch-events/#cancelability
                event.disable_cancelable();
                match action {
                    TouchMoveAction::Scroll(delta, point) => self.on_scroll_window_event(
                        ScrollLocation::Delta(LayoutVector2D::from_untyped(delta.to_untyped())),
                        point.cast(),
                    ),
                    TouchMoveAction::Zoom(magnification, scroll_delta) => {
                        let cursor = Point2D::new(-1, -1); // Make sure this hits the base layer.

                        // The order of these events doesn't matter, because zoom is handled by
                        // a root display list and the scroll event here is handled by the scroll
                        // applied to the content display list.
                        self.pending_scroll_zoom_events
                            .push(ScrollZoomEvent::PinchZoom(magnification));
                        self.pending_scroll_zoom_events
                            .push(ScrollZoomEvent::Scroll(ScrollEvent {
                                scroll_location: ScrollLocation::Delta(
                                    LayoutVector2D::from_untyped(scroll_delta.to_untyped()),
                                ),
                                cursor,
                                event_count: 1,
                            }));
                    },
                    _ => {},
                }
            }
            // When the event is touchmove, if the script thread is processing the touch
            // move event, we skip sending the event to the script thread.
            // This prevents the script thread from stacking up for a large amount of time.
            if !self
                .touch_handler
                .is_handling_touch_move(self.touch_handler.current_sequence_id) &&
                self.send_touch_event(event) &&
                event.is_cancelable()
            {
                self.touch_handler
                    .set_handling_touch_move(self.touch_handler.current_sequence_id, true);
            }
        }
    }

    fn on_touch_up(&mut self, event: TouchEvent) {
        self.touch_handler.on_touch_up(event.id, event.point);
        self.send_touch_event(event);
    }

    fn on_touch_cancel(&mut self, event: TouchEvent) {
        // Send the event to script.
        self.touch_handler.on_touch_cancel(event.id, event.point);
        self.send_touch_event(event);
    }

    pub(crate) fn on_touch_event_processed(&mut self, result: TouchEventResult) {
        match result {
            TouchEventResult::DefaultPrevented(sequence_id, event_type) => {
                debug!(
                    "Touch event {:?} in sequence {:?} prevented!",
                    event_type, sequence_id
                );
                match event_type {
                    TouchEventType::Down => {
                        // prevents both click and move
                        self.touch_handler.prevent_click(sequence_id);
                        self.touch_handler.prevent_move(sequence_id);
                        self.touch_handler
                            .remove_pending_touch_move_action(sequence_id);
                    },
                    TouchEventType::Move => {
                        // script thread processed the touch move event, mark this false.
                        if let Some(info) = self.touch_handler.get_touch_sequence_mut(sequence_id) {
                            info.prevent_move = TouchMoveAllowed::Prevented;
                            if let TouchSequenceState::PendingFling { .. } = info.state {
                                info.state = TouchSequenceState::Finished;
                            }
                            self.touch_handler.set_handling_touch_move(
                                self.touch_handler.current_sequence_id,
                                false,
                            );
                            self.touch_handler
                                .remove_pending_touch_move_action(sequence_id);
                        }
                    },
                    TouchEventType::Up => {
                        // Note: We don't have to consider PendingFling here, since we handle that
                        // in the DefaultAllowed case of the touch_move event.
                        // Note: Removing can and should fail, if we still have an active Fling,
                        let Some(info) =
                            &mut self.touch_handler.get_touch_sequence_mut(sequence_id)
                        else {
                            // The sequence ID could already be removed, e.g. if Fling finished,
                            // before the touch_up event was handled (since fling can start
                            // immediately if move was previously allowed, and clicks are anyway not
                            // happening from fling).
                            return;
                        };
                        match info.state {
                            TouchSequenceState::PendingClick(_) => {
                                info.state = TouchSequenceState::Finished;
                                self.touch_handler.remove_touch_sequence(sequence_id);
                            },
                            TouchSequenceState::Flinging { .. } => {
                                // We can't remove the touch sequence yet
                            },
                            TouchSequenceState::Finished => {
                                self.touch_handler.remove_touch_sequence(sequence_id);
                            },
                            TouchSequenceState::Touching |
                            TouchSequenceState::Panning { .. } |
                            TouchSequenceState::Pinching |
                            TouchSequenceState::MultiTouch |
                            TouchSequenceState::PendingFling { .. } => {
                                // It's possible to transition from Pinch to pan, Which means that
                                // a touch_up event for a pinch might have arrived here, but we
                                // already transitioned to pan or even PendingFling.
                                // We don't need to do anything in these cases though.
                            },
                        }
                    },
                    TouchEventType::Cancel => {
                        // We could still have pending event handlers, so we remove the pending
                        // actions, and try to remove the touch sequence.
                        self.touch_handler
                            .remove_pending_touch_move_action(sequence_id);
                        self.touch_handler.try_remove_touch_sequence(sequence_id);
                    },
                }
            },
            TouchEventResult::DefaultAllowed(sequence_id, event_type) => {
                debug!(
                    "Touch event {:?} in sequence {:?} allowed",
                    event_type, sequence_id
                );
                match event_type {
                    TouchEventType::Down => {},
                    TouchEventType::Move => {
                        if let Some(action) =
                            self.touch_handler.pending_touch_move_action(sequence_id)
                        {
                            match action {
                                TouchMoveAction::Scroll(delta, point) => self
                                    .on_scroll_window_event(
                                        ScrollLocation::Delta(LayoutVector2D::from_untyped(
                                            delta.to_untyped(),
                                        )),
                                        point.cast(),
                                    ),
                                TouchMoveAction::Zoom(magnification, scroll_delta) => {
                                    let cursor = Point2D::new(-1, -1);
                                    // Make sure this hits the base layer.
                                    // The order of these events doesn't matter, because zoom is handled by
                                    // a root display list and the scroll event here is handled by the scroll
                                    // applied to the content display list.
                                    self.pending_scroll_zoom_events
                                        .push(ScrollZoomEvent::PinchZoom(magnification));
                                    self.pending_scroll_zoom_events
                                        .push(ScrollZoomEvent::Scroll(ScrollEvent {
                                            scroll_location: ScrollLocation::Delta(
                                                LayoutVector2D::from_untyped(
                                                    scroll_delta.to_untyped(),
                                                ),
                                            ),
                                            cursor,
                                            event_count: 1,
                                        }));
                                },
                                TouchMoveAction::NoAction => {
                                    // This shouldn't happen, but we can also just ignore it.
                                },
                            }
                            self.touch_handler
                                .remove_pending_touch_move_action(sequence_id);
                        }
                        self.touch_handler
                            .set_handling_touch_move(self.touch_handler.current_sequence_id, false);
                        if let Some(info) = self.touch_handler.get_touch_sequence_mut(sequence_id) {
                            info.prevent_move = TouchMoveAllowed::Allowed;
                            if let TouchSequenceState::PendingFling { velocity, cursor } =
                                info.state
                            {
                                info.state = TouchSequenceState::Flinging { velocity, cursor }
                            }
                        }
                    },
                    TouchEventType::Up => {
                        let Some(info) = self.touch_handler.get_touch_sequence_mut(sequence_id)
                        else {
                            // The sequence was already removed because there is no default action.
                            return;
                        };
                        match info.state {
                            TouchSequenceState::PendingClick(point) => {
                                info.state = TouchSequenceState::Finished;
                                // PreventDefault from touch_down may have been processed after
                                // touch_up already occurred.
                                if !info.prevent_click {
                                    self.simulate_mouse_click(point);
                                }
                                self.touch_handler.remove_touch_sequence(sequence_id);
                            },
                            TouchSequenceState::Flinging { .. } => {
                                // We can't remove the touch sequence yet
                            },
                            TouchSequenceState::Finished => {
                                self.touch_handler.remove_touch_sequence(sequence_id);
                            },
                            TouchSequenceState::Panning { .. } |
                            TouchSequenceState::Pinching |
                            TouchSequenceState::PendingFling { .. } => {
                                // It's possible to transition from Pinch to pan, Which means that
                                // a touch_up event for a pinch might have arrived here, but we
                                // already transitioned to pan or even PendingFling.
                                // We don't need to do anything in these cases though.
                            },
                            TouchSequenceState::MultiTouch | TouchSequenceState::Touching => {
                                // We transitioned to touching from multi-touch or pinching.
                            },
                        }
                    },
                    TouchEventType::Cancel => {
                        self.touch_handler
                            .remove_pending_touch_move_action(sequence_id);
                        self.touch_handler.try_remove_touch_sequence(sequence_id);
                    },
                }
            },
        }
    }

    /// <http://w3c.github.io/touch-events/#mouse-events>
    fn simulate_mouse_click(&mut self, point: DevicePoint) {
        let button = MouseButton::Left;
        self.dispatch_input_event(InputEvent::MouseMove(MouseMoveEvent { point }));
        self.dispatch_input_event(InputEvent::MouseButton(MouseButtonEvent {
            button,
            action: MouseButtonAction::Down,
            point,
        }));
        self.dispatch_input_event(InputEvent::MouseButton(MouseButtonEvent {
            button,
            action: MouseButtonAction::Up,
            point,
        }));
        self.dispatch_input_event(InputEvent::MouseButton(MouseButtonEvent {
            button,
            action: MouseButtonAction::Click,
            point,
        }));
    }

    pub fn notify_scroll_event(
        &mut self,
        scroll_location: ScrollLocation,
        cursor: DeviceIntPoint,
        event_type: TouchEventType,
    ) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }

        match event_type {
            TouchEventType::Move => self.on_scroll_window_event(scroll_location, cursor),
            TouchEventType::Up | TouchEventType::Cancel => {
                self.on_scroll_window_event(scroll_location, cursor);
            },
            TouchEventType::Down => {
                self.on_scroll_window_event(scroll_location, cursor);
            },
        }
    }

    fn on_scroll_window_event(&mut self, scroll_location: ScrollLocation, cursor: DeviceIntPoint) {
        self.pending_scroll_zoom_events
            .push(ScrollZoomEvent::Scroll(ScrollEvent {
                scroll_location,
                cursor,
                event_count: 1,
            }));
    }

    pub(crate) fn process_pending_scroll_events(&mut self, compositor: &mut IOCompositor) {
        if self.pending_scroll_zoom_events.is_empty() {
            return;
        }

        // Batch up all scroll events into one, or else we'll do way too much painting.
        let mut combined_scroll_event: Option<ScrollEvent> = None;
        let mut combined_magnification = 1.0;
        for scroll_event in self.pending_scroll_zoom_events.drain(..) {
            match scroll_event {
                ScrollZoomEvent::PinchZoom(magnification) => {
                    combined_magnification *= magnification
                },
                ScrollZoomEvent::Scroll(scroll_event_info) => {
                    let combined_event = match combined_scroll_event.as_mut() {
                        None => {
                            combined_scroll_event = Some(scroll_event_info);
                            continue;
                        },
                        Some(combined_event) => combined_event,
                    };

                    match (
                        combined_event.scroll_location,
                        scroll_event_info.scroll_location,
                    ) {
                        (ScrollLocation::Delta(old_delta), ScrollLocation::Delta(new_delta)) => {
                            // Mac OS X sometimes delivers scroll events out of vsync during a
                            // fling. This causes events to get bunched up occasionally, causing
                            // nasty-looking "pops". To mitigate this, during a fling we average
                            // deltas instead of summing them.
                            let old_event_count = Scale::new(combined_event.event_count as f32);
                            combined_event.event_count += 1;
                            let new_event_count = Scale::new(combined_event.event_count as f32);
                            combined_event.scroll_location = ScrollLocation::Delta(
                                (old_delta * old_event_count + new_delta) / new_event_count,
                            );
                        },
                        (ScrollLocation::Start, _) | (ScrollLocation::End, _) => {
                            // Once we see Start or End, we shouldn't process any more events.
                            break;
                        },
                        (_, ScrollLocation::Start) | (_, ScrollLocation::End) => {
                            // If this is an event which is scrolling to the start or end of the page,
                            // disregard other pending events and exit the loop.
                            *combined_event = scroll_event_info;
                            break;
                        },
                    }
                },
            }
        }

        let zoom_changed = compositor
            .set_pinch_zoom_level(compositor.pinch_zoom_level().get() * combined_magnification);
        let scroll_result = combined_scroll_event.and_then(|combined_event| {
            self.scroll_node_at_device_point(
                combined_event.cursor.to_f32(),
                combined_event.scroll_location,
                compositor,
            )
        });
        if !zoom_changed && scroll_result.is_none() {
            return;
        }

        let mut transaction = Transaction::new();
        if zoom_changed {
            compositor.send_root_pipeline_display_list_in_transaction(&mut transaction);
        }

        if let Some((pipeline_id, external_id, offset)) = scroll_result {
            let offset = LayoutVector2D::new(-offset.x, -offset.y);
            transaction.set_scroll_offsets(
                external_id,
                vec![SampledScrollOffset {
                    offset,
                    generation: 0,
                }],
            );
            self.send_scroll_positions_to_layout_for_pipeline(pipeline_id);
        }

        compositor.generate_frame(&mut transaction, RenderReasons::APZ);
        self.global.borrow_mut().send_transaction(transaction);
    }

    /// Perform a hit test at the given [`DevicePoint`] and apply the [`ScrollLocation`]
    /// scrolling to the applicable scroll node under that point. If a scroll was
    /// performed, returns the [`PipelineId`] of the node scrolled, the id, and the final
    /// scroll delta.
    fn scroll_node_at_device_point(
        &mut self,
        cursor: DevicePoint,
        scroll_location: ScrollLocation,
        compositor: &mut IOCompositor,
    ) -> Option<(PipelineId, ExternalScrollId, LayoutVector2D)> {
        let scroll_location = match scroll_location {
            ScrollLocation::Delta(delta) => {
                let device_pixels_per_page = compositor.device_pixels_per_page_pixel();
                let scaled_delta = (Vector2D::from_untyped(delta.to_untyped()) /
                    device_pixels_per_page)
                    .to_untyped();
                let calculated_delta = LayoutVector2D::from_untyped(scaled_delta);
                ScrollLocation::Delta(calculated_delta)
            },
            // Leave ScrollLocation unchanged if it is Start or End location.
            ScrollLocation::Start | ScrollLocation::End => scroll_location,
        };

        let get_pipeline_details = |pipeline_id| self.pipelines.get(&pipeline_id);
        let hit_test_results = self
            .global
            .borrow()
            .hit_test_at_point_with_flags_and_pipeline(
                cursor,
                HitTestFlags::FIND_ALL,
                None,
                get_pipeline_details,
            );

        // Iterate through all hit test results, processing only the first node of each pipeline.
        // This is needed to propagate the scroll events from a pipeline representing an iframe to
        // its ancestor pipelines.
        let mut previous_pipeline_id = None;
        for CompositorHitTestResult {
            pipeline_id,
            scroll_tree_node,
            ..
        } in hit_test_results.iter()
        {
            let pipeline_details = self.pipelines.get_mut(pipeline_id)?;
            if previous_pipeline_id.replace(pipeline_id) != Some(pipeline_id) {
                let scroll_result = pipeline_details
                    .scroll_tree
                    .scroll_node_or_ancestor(scroll_tree_node, scroll_location);
                if let Some((external_id, offset)) = scroll_result {
                    return Some((*pipeline_id, external_id, offset));
                }
            }
        }
        None
    }

    /// Simulate a pinch zoom
    pub fn set_pinch_zoom(&mut self, magnification: f32) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }

        // TODO: Scroll to keep the center in view?
        self.pending_scroll_zoom_events
            .push(ScrollZoomEvent::PinchZoom(magnification));
    }
}
#[derive(Debug)]
pub struct WebViewManager<WebView> {
    /// Our top-level browsing contexts. In the WebRender scene, their pipelines are the children of
    /// a single root pipeline that also applies any pinch zoom transformation.
    webviews: HashMap<WebViewId, WebView>,

    /// The order to paint them in, topmost last.
    pub(crate) painting_order: Vec<WebViewId>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnknownWebView(pub WebViewId);

impl<WebView> Default for WebViewManager<WebView> {
    fn default() -> Self {
        Self {
            webviews: Default::default(),
            painting_order: Default::default(),
        }
    }
}

impl<WebView> WebViewManager<WebView> {
    pub fn remove(&mut self, webview_id: WebViewId) -> Result<WebView, UnknownWebView> {
        self.painting_order.retain(|b| *b != webview_id);
        self.webviews
            .remove(&webview_id)
            .ok_or(UnknownWebView(webview_id))
    }

    pub fn get(&self, webview_id: WebViewId) -> Option<&WebView> {
        self.webviews.get(&webview_id)
    }

    pub fn get_mut(&mut self, webview_id: WebViewId) -> Option<&mut WebView> {
        self.webviews.get_mut(&webview_id)
    }

    /// Returns true iff the painting order actually changed.
    pub fn show(&mut self, webview_id: WebViewId) -> Result<bool, UnknownWebView> {
        if !self.webviews.contains_key(&webview_id) {
            return Err(UnknownWebView(webview_id));
        }
        if !self.painting_order.contains(&webview_id) {
            self.painting_order.push(webview_id);
            return Ok(true);
        }
        Ok(false)
    }

    /// Returns true iff the painting order actually changed.
    pub fn hide(&mut self, webview_id: WebViewId) -> Result<bool, UnknownWebView> {
        if !self.webviews.contains_key(&webview_id) {
            return Err(UnknownWebView(webview_id));
        }
        if self.painting_order.contains(&webview_id) {
            self.painting_order.retain(|b| *b != webview_id);
            return Ok(true);
        }
        Ok(false)
    }

    /// Returns true iff the painting order actually changed.
    pub fn hide_all(&mut self) -> bool {
        if !self.painting_order.is_empty() {
            self.painting_order.clear();
            return true;
        }
        false
    }

    /// Returns true iff the painting order actually changed.
    pub fn raise_to_top(&mut self, webview_id: WebViewId) -> Result<bool, UnknownWebView> {
        if !self.webviews.contains_key(&webview_id) {
            return Err(UnknownWebView(webview_id));
        }
        if self.painting_order.last() != Some(&webview_id) {
            self.hide(webview_id)?;
            self.show(webview_id)?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn painting_order(&self) -> impl Iterator<Item = (&WebViewId, &WebView)> {
        self.painting_order
            .iter()
            .flat_map(move |webview_id| self.get(*webview_id).map(|b| (webview_id, b)))
    }

    pub fn entry(&mut self, webview_id: WebViewId) -> Entry<'_, WebViewId, WebView> {
        self.webviews.entry(webview_id)
    }

    pub fn iter(&self) -> Values<'_, WebViewId, WebView> {
        self.webviews.values()
    }

    pub fn iter_mut(&mut self) -> ValuesMut<'_, WebViewId, WebView> {
        self.webviews.values_mut()
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZeroU32;

    use base::id::{
        BrowsingContextId, BrowsingContextIndex, PipelineNamespace, PipelineNamespaceId, WebViewId,
    };

    use crate::webview::{UnknownWebView, WebViewAlreadyExists, WebViewManager};

    fn top_level_id(namespace_id: u32, index: u32) -> WebViewId {
        WebViewId(BrowsingContextId {
            namespace_id: PipelineNamespaceId(namespace_id),
            index: BrowsingContextIndex(NonZeroU32::new(index).unwrap()),
        })
    }

    fn webviews_sorted<WebView: Clone>(
        webviews: &WebViewManager<WebView>,
    ) -> Vec<(WebViewId, WebView)> {
        let mut keys = webviews.webviews.keys().collect::<Vec<_>>();
        keys.sort();
        keys.iter()
            .map(|&id| (*id, webviews.webviews.get(id).cloned().unwrap()))
            .collect()
    }

    #[test]
    fn test() {
        PipelineNamespace::install(PipelineNamespaceId(0));
        let mut webviews = WebViewManager::default();

        // add() adds the webview to the map, but not the painting order.
        assert!(webviews.add(WebViewId::new(), 'a').is_ok());
        assert!(webviews.add(WebViewId::new(), 'b').is_ok());
        assert!(webviews.add(WebViewId::new(), 'c').is_ok());
        assert_eq!(
            webviews_sorted(&webviews),
            vec![
                (top_level_id(0, 1), 'a'),
                (top_level_id(0, 2), 'b'),
                (top_level_id(0, 3), 'c'),
            ]
        );
        assert!(webviews.painting_order.is_empty());

        // add() returns WebViewAlreadyExists if the webview id already exists.
        assert_eq!(
            webviews.add(top_level_id(0, 3), 'd'),
            Err(WebViewAlreadyExists(top_level_id(0, 3)))
        );

        // Other methods return UnknownWebView or None if the webview id doesn’t exist.
        assert_eq!(
            webviews.remove(top_level_id(1, 1)),
            Err(UnknownWebView(top_level_id(1, 1)))
        );
        assert_eq!(webviews.get(top_level_id(1, 1)), None);
        assert_eq!(webviews.get_mut(top_level_id(1, 1)), None);
        assert_eq!(
            webviews.show(top_level_id(1, 1)),
            Err(UnknownWebView(top_level_id(1, 1)))
        );
        assert_eq!(
            webviews.hide(top_level_id(1, 1)),
            Err(UnknownWebView(top_level_id(1, 1)))
        );
        assert_eq!(
            webviews.raise_to_top(top_level_id(1, 1)),
            Err(UnknownWebView(top_level_id(1, 1)))
        );

        // For webviews not yet visible, both show() and raise_to_top() add the given webview on top.
        assert_eq!(webviews.show(top_level_id(0, 2)), Ok(true));
        assert_eq!(webviews.show(top_level_id(0, 2)), Ok(false));
        assert_eq!(webviews.painting_order, vec![top_level_id(0, 2)]);
        assert_eq!(webviews.raise_to_top(top_level_id(0, 1)), Ok(true));
        assert_eq!(webviews.raise_to_top(top_level_id(0, 1)), Ok(false));
        assert_eq!(
            webviews.painting_order,
            vec![top_level_id(0, 2), top_level_id(0, 1)]
        );
        assert_eq!(webviews.show(top_level_id(0, 3)), Ok(true));
        assert_eq!(webviews.show(top_level_id(0, 3)), Ok(false));
        assert_eq!(
            webviews.painting_order,
            vec![top_level_id(0, 2), top_level_id(0, 1), top_level_id(0, 3)]
        );

        // For webviews already visible, show() does nothing, while raise_to_top() makes it on top.
        assert_eq!(webviews.show(top_level_id(0, 1)), Ok(false));
        assert_eq!(
            webviews.painting_order,
            vec![top_level_id(0, 2), top_level_id(0, 1), top_level_id(0, 3)]
        );
        assert_eq!(webviews.raise_to_top(top_level_id(0, 1)), Ok(true));
        assert_eq!(webviews.raise_to_top(top_level_id(0, 1)), Ok(false));
        assert_eq!(
            webviews.painting_order,
            vec![top_level_id(0, 2), top_level_id(0, 3), top_level_id(0, 1)]
        );

        // hide() removes the webview from the painting order, but not the map.
        assert_eq!(webviews.hide(top_level_id(0, 3)), Ok(true));
        assert_eq!(webviews.hide(top_level_id(0, 3)), Ok(false));
        assert_eq!(
            webviews.painting_order,
            vec![top_level_id(0, 2), top_level_id(0, 1)]
        );
        assert_eq!(
            webviews_sorted(&webviews),
            vec![
                (top_level_id(0, 1), 'a'),
                (top_level_id(0, 2), 'b'),
                (top_level_id(0, 3), 'c'),
            ]
        );

        // painting_order() returns only the visible webviews, in painting order.
        let mut painting_order = webviews.painting_order();
        assert_eq!(painting_order.next(), Some((&top_level_id(0, 2), &'b')));
        assert_eq!(painting_order.next(), Some((&top_level_id(0, 1), &'a')));
        assert_eq!(painting_order.next(), None);
        drop(painting_order);

        // remove() removes the given webview from both the map and the painting order.
        assert!(webviews.remove(top_level_id(0, 1)).is_ok());
        assert!(webviews.remove(top_level_id(0, 2)).is_ok());
        assert!(webviews.remove(top_level_id(0, 3)).is_ok());
        assert!(webviews_sorted(&webviews).is_empty());
        assert!(webviews.painting_order.is_empty());
    }
}
