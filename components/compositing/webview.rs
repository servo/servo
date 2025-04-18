/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::Keys;
use std::rc::Rc;

use base::id::{PipelineId, WebViewId};
use compositing_traits::{RendererWebView, SendableFrameTree};
use constellation_traits::{EmbedderToConstellationMessage, ScrollState, WindowSizeType};
use embedder_traits::{
    AnimationState, CompositorHitTestResult, InputEvent, MouseButton, MouseButtonAction,
    MouseButtonEvent, MouseMoveEvent, ShutdownState, TouchEvent, TouchEventResult, TouchEventType,
    TouchId, ViewportDetails,
};
use euclid::{Box2D, Point2D, Scale, Size2D, Vector2D};
use fnv::FnvHashSet;
use log::{debug, warn};
use servo_geometry::DeviceIndependentPixel;
use style_traits::{CSSPixel, PinchZoomFactor};
use webrender::Transaction;
use webrender_api::units::{
    DeviceIntPoint, DeviceIntRect, DevicePixel, DevicePoint, DeviceRect, LayoutVector2D,
};
use webrender_api::{
    ExternalScrollId, HitTestFlags, RenderReasons, SampledScrollOffset, ScrollLocation,
};

use crate::IOCompositor;
use crate::compositor::{PipelineDetails, ServoRenderer};
use crate::touch::{TouchHandler, TouchMoveAction, TouchMoveAllowed, TouchSequenceState};

// Default viewport constraints
const MAX_ZOOM: f32 = 8.0;
const MIN_ZOOM: f32 = 0.1;

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
    /// The renderer's view of the embedding layer `WebView` as a trait implementation,
    /// so that the renderer doesn't need to depend on the embedding layer. This avoids
    /// a dependency cycle.
    pub renderer_webview: Box<dyn RendererWebView>,
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
    /// "Desktop-style" zoom that resizes the viewport to fit the window.
    pub page_zoom: Scale<f32, CSSPixel, DeviceIndependentPixel>,
    /// "Mobile-style" zoom that does not reflow the page.
    viewport_zoom: PinchZoomFactor,
    /// Viewport zoom constraints provided by @viewport.
    min_viewport_zoom: Option<PinchZoomFactor>,
    max_viewport_zoom: Option<PinchZoomFactor>,
    /// The HiDPI scale factor for the `WebView` associated with this renderer. This is controlled
    /// by the embedding layer.
    hidpi_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
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
    pub(crate) fn new(
        global: Rc<RefCell<ServoRenderer>>,
        renderer_webview: Box<dyn RendererWebView>,
        viewport_details: ViewportDetails,
    ) -> Self {
        let hidpi_scale_factor = viewport_details.hidpi_scale_factor;
        let size = viewport_details.size * viewport_details.hidpi_scale_factor;
        Self {
            id: renderer_webview.id(),
            renderer_webview,
            root_pipeline_id: None,
            rect: DeviceRect::from_origin_and_size(DevicePoint::origin(), size),
            pipelines: Default::default(),
            touch_handler: TouchHandler::new(),
            global,
            pending_scroll_zoom_events: Default::default(),
            page_zoom: Scale::new(1.0),
            viewport_zoom: PinchZoomFactor::new(1.0),
            min_viewport_zoom: Some(PinchZoomFactor::new(1.0)),
            max_viewport_zoom: None,
            hidpi_scale_factor: Scale::new(hidpi_scale_factor.0),
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

        let _ = self.global.borrow().constellation_sender.send(
            EmbedderToConstellationMessage::SetScrollStates(pipeline_id, scroll_states),
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

    /// Sets or unsets the animations-running flag for the given pipeline. Returns true if
    /// the pipeline is throttled.
    pub(crate) fn change_running_animations_state(
        &mut self,
        pipeline_id: PipelineId,
        animation_state: AnimationState,
    ) -> bool {
        let throttled = {
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
        };

        let animating = self.pipelines.values().any(PipelineDetails::animating);
        self.renderer_webview.set_animating(animating);
        throttled
    }

    pub(crate) fn tick_all_animations(&self, compositor: &IOCompositor) {
        for pipeline_details in self.pipelines.values() {
            pipeline_details.tick_animations(compositor)
        }
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
    pub(crate) fn on_vsync(&mut self) {
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

        if let Err(error) = self.global.borrow().constellation_sender.send(
            EmbedderToConstellationMessage::ForwardInputEvent(self.id, event, Some(result)),
        ) {
            warn!("Sending event to constellation failed ({error:?}).");
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
        if let Err(e) = self.global.borrow().constellation_sender.send(
            EmbedderToConstellationMessage::ForwardInputEvent(self.id, event, Some(result)),
        ) {
            warn!("Sending event to constellation failed ({:?}).", e);
            false
        } else {
            true
        }
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

    pub(crate) fn notify_scroll_event(
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

        let zoom_changed =
            self.set_pinch_zoom_level(self.pinch_zoom_level().get() * combined_magnification);
        let scroll_result = combined_scroll_event.and_then(|combined_event| {
            self.scroll_node_at_device_point(
                combined_event.cursor.to_f32(),
                combined_event.scroll_location,
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
    ) -> Option<(PipelineId, ExternalScrollId, LayoutVector2D)> {
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

    pub(crate) fn pinch_zoom_level(&self) -> Scale<f32, DevicePixel, DevicePixel> {
        Scale::new(self.viewport_zoom.get())
    }

    fn set_pinch_zoom_level(&mut self, mut zoom: f32) -> bool {
        if let Some(min) = self.min_viewport_zoom {
            zoom = f32::max(min.get(), zoom);
        }
        if let Some(max) = self.max_viewport_zoom {
            zoom = f32::min(max.get(), zoom);
        }

        let old_zoom = std::mem::replace(&mut self.viewport_zoom, PinchZoomFactor::new(zoom));
        old_zoom != self.viewport_zoom
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
            .push(ScrollZoomEvent::PinchZoom(magnification));
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

    pub(crate) fn client_window_rect(
        &self,
        rendering_context_size: Size2D<u32, DevicePixel>,
    ) -> Box2D<i32, DeviceIndependentPixel> {
        let screen_geometry = self.renderer_webview.screen_geometry().unwrap_or_default();
        let rect = DeviceIntRect::from_origin_and_size(
            screen_geometry.offset,
            rendering_context_size.to_i32(),
        )
        .to_f32() /
            self.hidpi_scale_factor;
        rect.to_i32()
    }

    pub(crate) fn screen_size(&self) -> Size2D<i32, DeviceIndependentPixel> {
        let screen_geometry = self.renderer_webview.screen_geometry().unwrap_or_default();
        (screen_geometry.size.to_f32() / self.hidpi_scale_factor).to_i32()
    }

    pub(crate) fn available_screen_size(&self) -> Size2D<i32, DeviceIndependentPixel> {
        let screen_geometry = self.renderer_webview.screen_geometry().unwrap_or_default();
        (screen_geometry.available_size.to_f32() / self.hidpi_scale_factor).to_i32()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnknownWebView(pub WebViewId);
