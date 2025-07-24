/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::collections::hash_map::{Entry, Keys};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use base::id::{PipelineId, WebViewId};
use compositing_traits::display_list::ScrollType;
use compositing_traits::viewport_description::{
    DEFAULT_ZOOM, MAX_ZOOM, MIN_ZOOM, ViewportDescription,
};
use compositing_traits::{PipelineExitSource, SendableFrameTree, WebViewTrait};
use constellation_traits::{EmbedderToConstellationMessage, WindowSizeType};
use embedder_traits::{
    AnimationState, CompositorHitTestResult, InputEvent, MouseButton, MouseButtonAction,
    MouseButtonEvent, MouseMoveEvent, ScrollEvent as EmbedderScrollEvent, ShutdownState,
    TouchEvent, TouchEventResult, TouchEventType, TouchId, ViewportDetails,
};
use euclid::{Point2D, Scale, Vector2D};
use fnv::FnvHashSet;
use log::{debug, warn};
use servo_geometry::DeviceIndependentPixel;
use style_traits::{CSSPixel, PinchZoomFactor};
use webrender_api::units::{DeviceIntPoint, DevicePixel, DevicePoint, DeviceRect, LayoutVector2D};
use webrender_api::{ExternalScrollId, HitTestFlags, ScrollLocation};

use crate::compositor::{HitTestError, PipelineDetails, ServoRenderer};
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
    /// A pinch zoom event that magnifies the view by the given factor.
    PinchZoom(f32),
    /// A zoom event that establishes the initial zoom from the viewport meta tag.
    InitialViewportZoom(f32),
    /// A scroll event that scrolls the scroll node at the given location by the
    /// given amount.
    Scroll(ScrollEvent),
}

#[derive(Clone, Debug)]
pub(crate) struct ScrollResult {
    pub hit_test_result: CompositorHitTestResult,
    pub external_scroll_id: ExternalScrollId,
    pub offset: LayoutVector2D,
}

#[derive(Debug, PartialEq)]
pub(crate) enum PinchZoomResult {
    DidPinchZoom,
    DidNotPinchZoom,
}

/// A renderer for a libservo `WebView`. This is essentially the [`ServoRenderer`]'s interface to a
/// libservo `WebView`, but the code here cannot depend on libservo in order to prevent circular
/// dependencies, which is why we store a `dyn WebViewTrait` here instead of the `WebView` itself.
pub(crate) struct WebViewRenderer {
    /// The [`WebViewId`] of the `WebView` associated with this [`WebViewDetails`].
    pub id: WebViewId,
    /// The renderer's view of the embedding layer `WebView` as a trait implementation,
    /// so that the renderer doesn't need to depend on the embedding layer. This avoids
    /// a dependency cycle.
    pub webview: Box<dyn WebViewTrait>,
    /// The root [`PipelineId`] of the currently displayed page in this WebView.
    pub root_pipeline_id: Option<PipelineId>,
    /// The rectangle of the [`WebView`] in device pixels, which is the viewport.
    pub rect: DeviceRect,
    /// Tracks details about each active pipeline that the compositor knows about.
    pub pipelines: HashMap<PipelineId, PipelineDetails>,
    /// Data that is shared by all WebView renderers.
    pub(crate) global: Rc<RefCell<ServoRenderer>>,
    /// Pending scroll/zoom events.
    pending_scroll_zoom_events: Vec<ScrollZoomEvent>,
    /// Touch input state machine
    touch_handler: TouchHandler,
    /// "Desktop-style" zoom that resizes the viewport to fit the window.
    pub page_zoom: Scale<f32, CSSPixel, DeviceIndependentPixel>,
    /// "Mobile-style" zoom that does not reflow the page.
    pinch_zoom: PinchZoomFactor,
    /// The HiDPI scale factor for the `WebView` associated with this renderer. This is controlled
    /// by the embedding layer.
    hidpi_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    /// Whether or not this [`WebViewRenderer`] isn't throttled and has a pipeline with
    /// active animations or animation frame callbacks.
    animating: bool,
    /// Pending input events queue. Priavte and only this thread pushes events to it.
    pending_point_input_events: RefCell<VecDeque<InputEvent>>,
    /// WebRender is not ready between `SendDisplayList` and `WebRenderFrameReady` messages.
    pub webrender_frame_ready: Cell<bool>,
    /// Viewport Description
    viewport_description: Option<ViewportDescription>,
}

impl Drop for WebViewRenderer {
    fn drop(&mut self) {
        self.global
            .borrow_mut()
            .pipeline_to_webview_map
            .retain(|_, webview_id| self.id != *webview_id);
    }
}

impl WebViewRenderer {
    pub(crate) fn new(
        global: Rc<RefCell<ServoRenderer>>,
        renderer_webview: Box<dyn WebViewTrait>,
        viewport_details: ViewportDetails,
    ) -> Self {
        let hidpi_scale_factor = viewport_details.hidpi_scale_factor;
        let size = viewport_details.size * viewport_details.hidpi_scale_factor;
        Self {
            id: renderer_webview.id(),
            webview: renderer_webview,
            root_pipeline_id: None,
            rect: DeviceRect::from_origin_and_size(DevicePoint::origin(), size),
            pipelines: Default::default(),
            touch_handler: TouchHandler::new(),
            global,
            pending_scroll_zoom_events: Default::default(),
            page_zoom: Scale::new(DEFAULT_ZOOM),
            pinch_zoom: PinchZoomFactor::new(DEFAULT_ZOOM),
            hidpi_scale_factor: Scale::new(hidpi_scale_factor.0),
            animating: false,
            pending_point_input_events: Default::default(),
            webrender_frame_ready: Cell::default(),
            viewport_description: None,
        }
    }

    pub(crate) fn animation_callbacks_running(&self) -> bool {
        self.pipelines
            .values()
            .any(PipelineDetails::animation_callbacks_running)
    }

    pub(crate) fn pipeline_ids(&self) -> Keys<'_, PipelineId, PipelineDetails> {
        self.pipelines.keys()
    }

    pub(crate) fn animating(&self) -> bool {
        self.animating
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
            PipelineDetails::new()
        })
    }

    pub(crate) fn pipeline_exited(&mut self, pipeline_id: PipelineId, source: PipelineExitSource) {
        let pipeline = self.pipelines.entry(pipeline_id);
        let Entry::Occupied(mut pipeline) = pipeline else {
            return;
        };

        pipeline.get_mut().exited.insert(source);

        // Do not remove pipeline details until both the Constellation and Script have
        // finished processing the pipeline shutdown. This prevents any followup messges
        // from re-adding the pipeline details and creating a zombie.
        if !pipeline.get().exited.is_all() {
            return;
        }

        pipeline.remove_entry();
        self.global
            .borrow_mut()
            .pipeline_to_webview_map
            .remove(&pipeline_id);
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

        let scroll_offsets = details.scroll_tree.scroll_offsets();

        // This might be true if we have not received a display list from the layout
        // associated with this pipeline yet. In that case, the layout is not ready to
        // receive scroll offsets anyway, so just save time and prevent other issues by
        // not sending them.
        if scroll_offsets.is_empty() {
            return;
        }

        let _ = self.global.borrow().constellation_sender.send(
            EmbedderToConstellationMessage::SetScrollStates(pipeline_id, scroll_offsets),
        );
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

    /// Sets or unsets the animations-running flag for the given pipeline. Returns
    /// true if the pipeline has started animating.
    pub(crate) fn change_pipeline_running_animations_state(
        &mut self,
        pipeline_id: PipelineId,
        animation_state: AnimationState,
    ) -> bool {
        let pipeline_details = self.ensure_pipeline_details(pipeline_id);
        let was_animating = pipeline_details.animating();
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
        let started_animating = !was_animating && pipeline_details.animating();

        self.update_animation_state();

        // It's important that an animation tick is triggered even if the
        // WebViewRenderer's overall animation state hasn't changed. It's possible that
        // the WebView was animating, but not producing new display lists. In that case,
        // no repaint will happen and thus no repaint will trigger the next animation tick.
        started_animating
    }

    /// Sets or unsets the throttled flag for the given pipeline. Returns
    /// true if the pipeline has started animating.
    pub(crate) fn set_throttled(&mut self, pipeline_id: PipelineId, throttled: bool) -> bool {
        let pipeline_details = self.ensure_pipeline_details(pipeline_id);
        let was_animating = pipeline_details.animating();
        pipeline_details.throttled = throttled;
        let started_animating = !was_animating && pipeline_details.animating();

        // Throttling a pipeline can cause it to be taken into the "not-animating" state.
        self.update_animation_state();

        // It's important that an animation tick is triggered even if the
        // WebViewRenderer's overall animation state hasn't changed. It's possible that
        // the WebView was animating, but not producing new display lists. In that case,
        // no repaint will happen and thus no repaint will trigger the next animation tick.
        started_animating
    }

    fn update_animation_state(&mut self) {
        self.animating = self.pipelines.values().any(PipelineDetails::animating);
        self.webview.set_animating(self.animating());
    }

    /// On a Window refresh tick (e.g. vsync)
    pub(crate) fn on_vsync(&mut self) {
        if let Some(fling_action) = self.touch_handler.on_vsync() {
            self.on_scroll_window_event(
                ScrollLocation::Delta(-fling_action.delta),
                fling_action.cursor,
            );
        }
    }

    pub(crate) fn dispatch_point_input_event(&self, event: InputEvent) -> bool {
        self.dispatch_point_input_event_internal(event, true)
    }

    pub(crate) fn dispatch_pending_point_input_events(&self) {
        while let Some(event) = self.pending_point_input_events.borrow_mut().pop_front() {
            // TODO: Add multiple retry later if needed.
            self.dispatch_point_input_event_internal(event, false);
        }
    }

    pub(crate) fn dispatch_point_input_event_internal(
        &self,
        mut event: InputEvent,
        retry_on_error: bool,
    ) -> bool {
        // Events that do not need to do hit testing are sent directly to the
        // constellation to filter down.
        let Some(point) = event.point() else {
            return false;
        };

        // Delay the event if the epoch is not synchronized yet (new frame is not ready),
        // or hit test result would fail and the event is rejected anyway.
        if retry_on_error &&
            (!self.webrender_frame_ready.get() ||
                !self.pending_point_input_events.borrow().is_empty())
        {
            self.pending_point_input_events
                .borrow_mut()
                .push_back(event);
            return false;
        }

        // If we can't find a pipeline to send this event to, we cannot continue.
        let get_pipeline_details = |pipeline_id| self.pipelines.get(&pipeline_id);
        let result = match self
            .global
            .borrow()
            .hit_test_at_point(point, get_pipeline_details)
        {
            Ok(hit_test_results) => Some(hit_test_results),
            Err(HitTestError::EpochMismatch) if retry_on_error => {
                self.pending_point_input_events
                    .borrow_mut()
                    .push_back(event.clone());
                return false;
            },
            _ => None,
        };

        match event {
            InputEvent::Touch(ref mut touch_event) => {
                touch_event.init_sequence_id(self.touch_handler.current_sequence_id);
            },
            InputEvent::MouseButton(_) |
            InputEvent::MouseLeave(_) |
            InputEvent::MouseMove(_) |
            InputEvent::Wheel(_) => {
                if let Some(ref result) = result {
                    self.global
                        .borrow_mut()
                        .update_cursor_from_hittest(point, result);
                } else {
                    warn!("Not hit test result.");
                }
            },
            _ => unreachable!("Unexpected input event type: {event:?}"),
        }

        if let Err(error) = self.global.borrow().constellation_sender.send(
            EmbedderToConstellationMessage::ForwardInputEvent(self.id, event, result),
        ) {
            warn!("Sending event to constellation failed ({error:?}).");
            false
        } else {
            true
        }
    }

    pub(crate) fn notify_input_event(&mut self, event: InputEvent) {
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
                    match (event.button, event.action) {
                        (MouseButton::Left, MouseButtonAction::Down) => self.on_touch_down(
                            TouchEvent::new(TouchEventType::Down, TouchId(0), event.point),
                        ),
                        (MouseButton::Left, MouseButtonAction::Up) => self.on_touch_up(
                            TouchEvent::new(TouchEventType::Up, TouchId(0), event.point),
                        ),
                        _ => {},
                    }
                    return;
                },
                InputEvent::MouseMove(event) => {
                    if let Some(state) = self.touch_handler.try_get_current_touch_sequence() {
                        // We assume that the debug option `-Z convert-mouse-to-touch` will only
                        // be used on devices without native touch input, so we can directly
                        // reuse the touch handler for tracking the state of pressed buttons.
                        match state.state {
                            TouchSequenceState::Touching | TouchSequenceState::Panning { .. } => {
                                self.on_touch_move(TouchEvent::new(
                                    TouchEventType::Move,
                                    TouchId(0),
                                    event.point,
                                ));
                            },
                            TouchSequenceState::MultiTouch => {
                                // Multitouch simulation currently is not implemented.
                                // Since we only get one mouse move event, we would need to
                                // dispatch one mouse move event per currently pressed mouse button.
                            },
                            TouchSequenceState::Pinching => {
                                // We only have one mouse button, so Pinching should be impossible.
                                #[cfg(debug_assertions)]
                                log::error!(
                                    "Touch handler is in Pinching state, which should be unreachable with \
                                -Z convert-mouse-to-touch debug option."
                                );
                            },
                            TouchSequenceState::PendingFling { .. } |
                            TouchSequenceState::Flinging { .. } |
                            TouchSequenceState::PendingClick(_) |
                            TouchSequenceState::Finished => {
                                // Mouse movement without a button being pressed is not
                                // translated to touch events.
                            },
                        }
                    }
                    // We don't want to (directly) dispatch mouse events when simulating touch input.
                    return;
                },
                _ => {},
            }
        }

        self.dispatch_point_input_event(event);
    }

    fn send_touch_event(&mut self, event: TouchEvent) -> bool {
        self.dispatch_point_input_event(InputEvent::Touch(event))
    }

    pub(crate) fn on_touch_event(&mut self, event: TouchEvent) {
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
                            if info.prevent_move == TouchMoveAllowed::Pending {
                                info.prevent_move = TouchMoveAllowed::Allowed;
                                if let TouchSequenceState::PendingFling { velocity, cursor } =
                                    info.state
                                {
                                    info.state = TouchSequenceState::Flinging { velocity, cursor }
                                }
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
        self.dispatch_point_input_event(InputEvent::MouseMove(MouseMoveEvent::new(point)));
        self.dispatch_point_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
            MouseButtonAction::Down,
            button,
            point,
        )));
        self.dispatch_point_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
            MouseButtonAction::Up,
            button,
            point,
        )));
    }

    pub(crate) fn notify_scroll_event(
        &mut self,
        scroll_location: ScrollLocation,
        cursor: DeviceIntPoint,
    ) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        self.on_scroll_window_event(scroll_location, cursor);
    }

    fn on_scroll_window_event(&mut self, scroll_location: ScrollLocation, cursor: DeviceIntPoint) {
        self.pending_scroll_zoom_events
            .push(ScrollZoomEvent::Scroll(ScrollEvent {
                scroll_location,
                cursor,
                event_count: 1,
            }));
    }

    /// Process pending scroll events for this [`WebViewRenderer`]. Returns a tuple containing:
    ///
    ///  - A boolean that is true if a zoom occurred.
    ///  - An optional [`ScrollResult`] if a scroll occurred.
    ///
    /// It is up to the caller to ensure that these events update the rendering appropriately.
    pub(crate) fn process_pending_scroll_and_pinch_zoom_events(
        &mut self,
    ) -> (PinchZoomResult, Option<ScrollResult>) {
        if self.pending_scroll_zoom_events.is_empty() {
            return (PinchZoomResult::DidNotPinchZoom, None);
        }

        // Batch up all scroll events into one, or else we'll do way too much painting.
        let mut combined_scroll_event: Option<ScrollEvent> = None;
        let mut base_page_zoom = self.pinch_zoom_level().get();
        let mut combined_magnification = 1.0;
        for scroll_event in self.pending_scroll_zoom_events.drain(..) {
            match scroll_event {
                ScrollZoomEvent::PinchZoom(magnification) => {
                    combined_magnification *= magnification
                },
                ScrollZoomEvent::InitialViewportZoom(magnification) => {
                    base_page_zoom = magnification
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

        let scroll_result = combined_scroll_event.and_then(|combined_event| {
            self.scroll_node_at_device_point(
                combined_event.cursor.to_f32(),
                combined_event.scroll_location,
            )
        });
        if let Some(scroll_result) = scroll_result.clone() {
            self.send_scroll_positions_to_layout_for_pipeline(
                scroll_result.hit_test_result.pipeline_id,
            );
            self.dispatch_scroll_event(
                scroll_result.external_scroll_id,
                scroll_result.hit_test_result,
            );
        }

        let pinch_zoom_result =
            match self.set_pinch_zoom_level(base_page_zoom * combined_magnification) {
                true => PinchZoomResult::DidPinchZoom,
                false => PinchZoomResult::DidNotPinchZoom,
            };

        (pinch_zoom_result, scroll_result)
    }

    /// Perform a hit test at the given [`DevicePoint`] and apply the [`ScrollLocation`]
    /// scrolling to the applicable scroll node under that point. If a scroll was
    /// performed, returns the hit test result contains [`PipelineId`] of the node
    /// scrolled, the id, and the final scroll delta.
    fn scroll_node_at_device_point(
        &mut self,
        cursor: DevicePoint,
        scroll_location: ScrollLocation,
    ) -> Option<ScrollResult> {
        let scroll_location = match scroll_location {
            ScrollLocation::Delta(delta) => {
                let device_pixels_per_page = self.device_pixels_per_page_pixel();
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
            )
            .unwrap_or_default();

        // Iterate through all hit test results, processing only the first node of each pipeline.
        // This is needed to propagate the scroll events from a pipeline representing an iframe to
        // its ancestor pipelines.
        let mut previous_pipeline_id = None;
        for hit_test_result in hit_test_results.iter() {
            let pipeline_details = self.pipelines.get_mut(&hit_test_result.pipeline_id)?;
            if previous_pipeline_id.replace(&hit_test_result.pipeline_id) !=
                Some(&hit_test_result.pipeline_id)
            {
                let scroll_result = pipeline_details.scroll_tree.scroll_node_or_ancestor(
                    &hit_test_result.scroll_tree_node,
                    scroll_location,
                    ScrollType::InputEvents,
                );
                if let Some((external_scroll_id, offset)) = scroll_result {
                    return Some(ScrollResult {
                        hit_test_result: hit_test_result.clone(),
                        external_scroll_id,
                        offset,
                    });
                }
            }
        }
        None
    }

    fn dispatch_scroll_event(
        &self,
        external_id: ExternalScrollId,
        hit_test_result: CompositorHitTestResult,
    ) {
        let event = InputEvent::Scroll(EmbedderScrollEvent { external_id });
        let msg = EmbedderToConstellationMessage::ForwardInputEvent(
            self.id,
            event,
            Some(hit_test_result),
        );
        if let Err(e) = self.global.borrow().constellation_sender.send(msg) {
            warn!("Sending scroll event to constellation failed ({:?}).", e);
        }
    }

    pub(crate) fn pinch_zoom_level(&self) -> Scale<f32, DevicePixel, DevicePixel> {
        Scale::new(self.pinch_zoom.get())
    }

    fn set_pinch_zoom_level(&mut self, mut zoom: f32) -> bool {
        if let Some(viewport) = self.viewport_description.as_ref() {
            zoom = viewport.clamp_zoom(zoom);
        }

        let old_zoom = std::mem::replace(&mut self.pinch_zoom, PinchZoomFactor::new(zoom));
        old_zoom != self.pinch_zoom
    }

    pub(crate) fn set_page_zoom(&mut self, magnification: f32) {
        self.page_zoom =
            Scale::new((self.page_zoom.get() * magnification).clamp(MIN_ZOOM, MAX_ZOOM));
    }

    pub(crate) fn device_pixels_per_page_pixel(&self) -> Scale<f32, CSSPixel, DevicePixel> {
        self.page_zoom * self.hidpi_scale_factor * self.pinch_zoom_level()
    }

    pub(crate) fn device_pixels_per_page_pixel_not_including_pinch_zoom(
        &self,
    ) -> Scale<f32, CSSPixel, DevicePixel> {
        self.page_zoom * self.hidpi_scale_factor
    }

    /// Simulate a pinch zoom
    pub(crate) fn set_pinch_zoom(&mut self, magnification: f32) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }

        // TODO: Scroll to keep the center in view?
        self.pending_scroll_zoom_events
            .push(ScrollZoomEvent::PinchZoom(
                self.viewport_description
                    .clone()
                    .unwrap_or_default()
                    .clamp_zoom(magnification),
            ));
    }

    fn send_window_size_message(&self) {
        // The device pixel ratio used by the style system should include the scale from page pixels
        // to device pixels, but not including any pinch zoom.
        let device_pixel_ratio = self.device_pixels_per_page_pixel_not_including_pinch_zoom();
        let initial_viewport = self.rect.size().to_f32() / device_pixel_ratio;
        let msg = EmbedderToConstellationMessage::ChangeViewportDetails(
            self.id,
            ViewportDetails {
                hidpi_scale_factor: device_pixel_ratio,
                size: initial_viewport,
            },
            WindowSizeType::Resize,
        );
        if let Err(e) = self.global.borrow().constellation_sender.send(msg) {
            warn!("Sending window resize to constellation failed ({:?}).", e);
        }
    }

    /// Set the `hidpi_scale_factor` for this renderer, returning `true` if the value actually changed.
    pub(crate) fn set_hidpi_scale_factor(
        &mut self,
        new_scale: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    ) -> bool {
        let old_scale_factor = std::mem::replace(&mut self.hidpi_scale_factor, new_scale);
        if self.hidpi_scale_factor == old_scale_factor {
            return false;
        }

        self.send_window_size_message();
        true
    }

    /// Set the `rect` for this renderer, returning `true` if the value actually changed.
    pub(crate) fn set_rect(&mut self, new_rect: DeviceRect) -> bool {
        let old_rect = std::mem::replace(&mut self.rect, new_rect);
        if old_rect.size() != self.rect.size() {
            self.send_window_size_message();
        }
        old_rect != self.rect
    }

    pub fn set_viewport_description(&mut self, viewport_description: ViewportDescription) {
        self.pending_scroll_zoom_events
            .push(ScrollZoomEvent::InitialViewportZoom(
                viewport_description
                    .clone()
                    .clamp_zoom(viewport_description.initial_scale.get()),
            ));
        self.viewport_description = Some(viewport_description);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnknownWebView(pub WebViewId);
