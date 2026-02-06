/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use base::id::{PipelineId, WebViewId};
use constellation_traits::{EmbedderToConstellationMessage, WindowSizeType};
use crossbeam_channel::Sender;
use embedder_traits::{
    AnimationState, InputEvent, InputEventAndId, InputEventId, InputEventResult, MouseButton,
    MouseButtonAction, MouseButtonEvent, MouseMoveEvent, PaintHitTestResult, Scroll,
    ScrollEvent as EmbedderScrollEvent, TouchEvent, TouchEventType, ViewportDetails, WebViewPoint,
    WheelEvent,
};
use euclid::{Scale, Vector2D};
use log::{debug, warn};
use malloc_size_of::MallocSizeOf;
use paint_api::display_list::ScrollType;
use paint_api::viewport_description::{
    DEFAULT_PAGE_ZOOM, MAX_PAGE_ZOOM, MIN_PAGE_ZOOM, ViewportDescription,
};
use paint_api::{PipelineExitSource, SendableFrameTree, WebViewTrait};
use rustc_hash::FxHashMap;
use servo_geometry::DeviceIndependentPixel;
use style_traits::CSSPixel;
use webrender::RenderApi;
use webrender_api::units::{DevicePixel, DevicePoint, DeviceRect, DeviceVector2D, LayoutVector2D};
use webrender_api::{DocumentId, ExternalScrollId, ScrollLocation};

use crate::paint::RepaintReason;
use crate::painter::Painter;
use crate::pinch_zoom::PinchZoom;
use crate::pipeline_details::PipelineDetails;
use crate::refresh_driver::BaseRefreshDriver;
use crate::touch::{PendingTouchInputEvent, TouchHandler, TouchMoveAllowed, TouchSequenceState};

#[derive(Clone, Copy)]
pub(crate) struct ScrollEvent {
    /// Scroll by this offset, or to Start or End
    pub scroll: Scroll,
    /// Scroll the scroll node that is found at this point.
    pub point: DevicePoint,
    /// The number of OS events that have been coalesced together into this one event.
    pub event_count: u32,
}

#[derive(Clone, Copy)]
pub(crate) enum ScrollZoomEvent {
    /// A pinch zoom event that magnifies the view by the given factor from the given
    /// center point.
    PinchZoom(f32, DevicePoint),
    /// A scroll event that scrolls the scroll node at the given location by the
    /// given amount.
    Scroll(ScrollEvent),
}

#[derive(Clone, Debug)]
pub(crate) struct ScrollResult {
    pub hit_test_result: PaintHitTestResult,
    /// The [`ExternalScrollId`] of the node that was actually scrolled.
    ///
    /// Note that this is an inclusive ancestor of `external_scroll_id` in
    /// [`Self::hit_test_result`].
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
    /// Tracks details about each active pipeline that `Paint` knows about.
    pub pipelines: FxHashMap<PipelineId, PipelineDetails>,
    /// Pending scroll/zoom events.
    pending_scroll_zoom_events: Vec<ScrollZoomEvent>,
    /// A map of pending wheel events. These are events that have been sent to script,
    /// but are waiting for processing. When they are handled by script, they may trigger
    /// scroll events depending on whether `preventDefault()` was called on the event.
    pending_wheel_events: FxHashMap<InputEventId, WheelEvent>,
    /// Touch input state machine
    touch_handler: TouchHandler,
    /// "Desktop-style" zoom that resizes the viewport to fit the window.
    pub page_zoom: Scale<f32, CSSPixel, DeviceIndependentPixel>,
    /// "Mobile-style" zoom that does not reflow the page. When there is no [`PinchZoom`] a
    /// zoom factor of 1.0 is implied and the [`PinchZoom::transform`] will be the identity.
    pinch_zoom: PinchZoom,
    /// The HiDPI scale factor for the `WebView` associated with this renderer. This is controlled
    /// by the embedding layer.
    hidpi_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    /// Whether or not this [`WebViewRenderer`] is hidden.
    hidden: bool,
    /// Whether or not this [`WebViewRenderer`] isn't throttled and has a pipeline with
    /// active animations or animation frame callbacks.
    animating: bool,
    /// A [`ViewportDescription`] for this [`WebViewRenderer`], which contains the limitations
    /// and initial values for zoom derived from the `viewport` meta tag in web content.
    viewport_description: Option<ViewportDescription>,

    //
    // Data that is shared with the parent renderer.
    //
    /// The channel on which messages can be sent to the constellation.
    embedder_to_constellation_sender: Sender<EmbedderToConstellationMessage>,
    /// The [`BaseRefreshDriver`] which manages the painting of `WebView`s during animations.
    refresh_driver: Rc<BaseRefreshDriver>,
    /// The active webrender document.
    webrender_document: DocumentId,
}

impl WebViewRenderer {
    pub(crate) fn new(
        renderer_webview: Box<dyn WebViewTrait>,
        viewport_details: ViewportDetails,
        embedder_to_constellation_sender: Sender<EmbedderToConstellationMessage>,
        refresh_driver: Rc<BaseRefreshDriver>,
        webrender_document: DocumentId,
    ) -> Self {
        let hidpi_scale_factor = viewport_details.hidpi_scale_factor;
        let size = viewport_details.size * viewport_details.hidpi_scale_factor;
        let rect = DeviceRect::from_origin_and_size(DevicePoint::origin(), size);
        let webview_id = renderer_webview.id();
        Self {
            id: webview_id,
            webview: renderer_webview,
            root_pipeline_id: None,
            rect,
            pipelines: Default::default(),
            touch_handler: TouchHandler::new(webview_id),
            pending_scroll_zoom_events: Default::default(),
            pending_wheel_events: Default::default(),
            page_zoom: DEFAULT_PAGE_ZOOM,
            pinch_zoom: PinchZoom::new(rect),
            hidpi_scale_factor: Scale::new(hidpi_scale_factor.0),
            hidden: false,
            animating: false,
            viewport_description: None,
            embedder_to_constellation_sender,
            refresh_driver,
            webrender_document,
        }
    }

    fn hit_test(&self, webrender_api: &RenderApi, point: DevicePoint) -> Vec<PaintHitTestResult> {
        Painter::hit_test_at_point_with_api_and_document(
            webrender_api,
            self.webrender_document,
            point,
        )
    }

    pub(crate) fn animation_callbacks_running(&self) -> bool {
        self.pipelines
            .values()
            .any(PipelineDetails::animation_callbacks_running)
    }

    pub(crate) fn animating(&self) -> bool {
        self.animating
    }

    pub(crate) fn hidden(&self) -> bool {
        self.hidden
    }

    /// Set whether this [`WebViewRenderer`] is in the hidden state or not. Return `true` if the
    /// value changed or `false` otherwise.
    pub(crate) fn set_hidden(&mut self, new_value: bool) -> bool {
        let old_value = std::mem::replace(&mut self.hidden, new_value);
        new_value != old_value
    }

    /// Returns the [`PipelineDetails`] for the given [`PipelineId`], creating it if needed.
    pub(crate) fn ensure_pipeline_details(
        &mut self,
        pipeline_id: PipelineId,
    ) -> &mut PipelineDetails {
        self.pipelines
            .entry(pipeline_id)
            .or_insert_with(PipelineDetails::new)
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
    }

    pub(crate) fn set_frame_tree(&mut self, frame_tree: &SendableFrameTree) {
        let pipeline_id = frame_tree.pipeline.id;
        let old_pipeline_id = self.root_pipeline_id.replace(pipeline_id);

        if old_pipeline_id != self.root_pipeline_id {
            debug!(
                "Updating webview ({:?}) from pipeline {:?} to {:?}",
                3, old_pipeline_id, self.root_pipeline_id
            );
        }

        self.set_frame_tree_on_pipeline_details(frame_tree, None);
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

        let _ = self.embedder_to_constellation_sender.send(
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

    /// Update touch-based animations (currently just fling) during a `RefreshDriver`-based
    /// frame tick. Returns `true` if we should continue observing frames (the fling is ongoing)
    /// or `false` if we should stop observing frames (the fling has finished).
    pub(crate) fn update_touch_handling_at_new_frame_start(&mut self) -> bool {
        let Some(fling_action) = self.touch_handler.notify_new_frame_start() else {
            return false;
        };

        self.on_scroll_window_event(
            Scroll::Delta((-fling_action.delta).into()),
            fling_action.cursor,
        );
        true
    }

    fn dispatch_input_event_with_hit_testing(
        &mut self,
        render_api: &RenderApi,
        event: InputEventAndId,
    ) -> bool {
        let event_point = event
            .event
            .point()
            .map(|point| point.as_device_point(self.device_pixels_per_page_pixel()));
        let hit_test_result = match event_point {
            Some(point) => {
                let hit_test_result = match event.event {
                    InputEvent::Touch(_) => self.touch_handler.get_hit_test_result_cache_value(),
                    _ => None,
                }
                .or_else(|| self.hit_test(render_api, point).into_iter().nth(0));
                if hit_test_result.is_none() {
                    warn!("Empty hit test result for input event, ignoring.");
                    return false;
                }
                hit_test_result
            },
            None => None,
        };

        if let Err(error) = self.embedder_to_constellation_sender.send(
            EmbedderToConstellationMessage::ForwardInputEvent(self.id, event, hit_test_result),
        ) {
            warn!("Sending event to constellation failed ({error:?}).");
            false
        } else {
            true
        }
    }

    pub(crate) fn notify_input_event(
        &mut self,
        render_api: &RenderApi,
        repaint_reason: &Cell<RepaintReason>,
        event_and_id: InputEventAndId,
    ) {
        if let InputEvent::Touch(touch_event) = event_and_id.event {
            self.on_touch_event(render_api, repaint_reason, touch_event, event_and_id.id);
            return;
        }

        if let InputEvent::Wheel(wheel_event) = event_and_id.event {
            self.pending_wheel_events
                .insert(event_and_id.id, wheel_event);
        }

        self.dispatch_input_event_with_hit_testing(render_api, event_and_id);
    }

    fn send_touch_event(
        &mut self,
        render_api: &RenderApi,
        event: TouchEvent,
        id: InputEventId,
    ) -> bool {
        let cancelable = event.is_cancelable();
        let event_type = event.event_type;

        let input_event_and_id = InputEventAndId {
            event: InputEvent::Touch(event),
            id,
        };

        let result = self.dispatch_input_event_with_hit_testing(render_api, input_event_and_id);

        // We only post-process events that are actually cancelable. Uncancelable ones
        // are processed immediately and can be ignored once they have been sent to the
        // Constellation.
        if cancelable && result {
            self.touch_handler
                .add_pending_touch_input_event(id, event_type);
        }

        result
    }

    pub(crate) fn on_touch_event(
        &mut self,
        render_api: &RenderApi,
        repaint_reason: &Cell<RepaintReason>,
        event: TouchEvent,
        id: InputEventId,
    ) {
        match event.event_type {
            TouchEventType::Down => self.on_touch_down(render_api, event, id),
            TouchEventType::Move => self.on_touch_move(render_api, event, id),
            TouchEventType::Up => self.on_touch_up(render_api, event, id),
            TouchEventType::Cancel => self.on_touch_cancel(render_api, event, id),
        }

        self.touch_handler
            .add_touch_move_refresh_observer_if_necessary(
                self.refresh_driver.clone(),
                repaint_reason,
            );
    }

    fn on_touch_down(&mut self, render_api: &RenderApi, event: TouchEvent, id: InputEventId) {
        let point = event
            .point
            .as_device_point(self.device_pixels_per_page_pixel());
        self.touch_handler.on_touch_down(event.id, point);
        self.send_touch_event(render_api, event, id);
    }

    fn on_touch_move(&mut self, render_api: &RenderApi, mut event: TouchEvent, id: InputEventId) {
        let point = event
            .point
            .as_device_point(self.device_pixels_per_page_pixel());
        let action = self.touch_handler.on_touch_move(
            event.id,
            point,
            self.device_pixels_per_page_pixel_not_including_pinch_zoom()
                .get(),
        );
        if let Some(action) = action {
            // if first move processed and allowed, we directly process the move event,
            // without waiting for the script handler.
            if self
                .touch_handler
                .move_allowed(self.touch_handler.current_sequence_id)
            {
                // https://w3c.github.io/touch-events/#cancelability
                event.disable_cancelable();
                self.pending_scroll_zoom_events.push(action);
            }
            // When the event is touchmove, if the script thread is processing the touch
            // move event, we skip sending the event to the script thread.
            // This prevents the script thread from stacking up for a large amount of time.
            if !self
                .touch_handler
                .is_handling_touch_move(self.touch_handler.current_sequence_id) &&
                self.send_touch_event(render_api, event, id) &&
                event.is_cancelable()
            {
                self.touch_handler
                    .set_handling_touch_move(self.touch_handler.current_sequence_id, true);
            }
        }
    }

    fn on_touch_up(&mut self, render_api: &RenderApi, event: TouchEvent, id: InputEventId) {
        let point = event
            .point
            .as_device_point(self.device_pixels_per_page_pixel());
        self.touch_handler.on_touch_up(event.id, point);
        self.send_touch_event(render_api, event, id);
    }

    fn on_touch_cancel(&mut self, render_api: &RenderApi, event: TouchEvent, id: InputEventId) {
        let point = event
            .point
            .as_device_point(self.device_pixels_per_page_pixel());
        self.touch_handler.on_touch_cancel(event.id, point);
        self.send_touch_event(render_api, event, id);
    }

    fn on_touch_event_processed(
        &mut self,
        render_api: &RenderApi,
        pending_touch_input_event: PendingTouchInputEvent,
        result: InputEventResult,
    ) {
        let PendingTouchInputEvent {
            sequence_id,
            event_type,
        } = pending_touch_input_event;

        if result.contains(InputEventResult::DefaultPrevented) {
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
                        .remove_pending_touch_move_actions(sequence_id);
                },
                TouchEventType::Move => {
                    // script thread processed the touch move event, mark this false.
                    if let Some(info) = self.touch_handler.get_touch_sequence_mut(sequence_id) {
                        info.prevent_move = TouchMoveAllowed::Prevented;
                        if let TouchSequenceState::PendingFling { .. } = info.state {
                            info.state = TouchSequenceState::Finished;
                        }
                        self.touch_handler
                            .set_handling_touch_move(self.touch_handler.current_sequence_id, false);
                        self.touch_handler
                            .remove_pending_touch_move_actions(sequence_id);
                    }
                },
                TouchEventType::Up => {
                    // Note: We don't have to consider PendingFling here, since we handle that
                    // in the DefaultAllowed case of the touch_move event.
                    // Note: Removing can and should fail, if we still have an active Fling,
                    let Some(info) = &mut self.touch_handler.get_touch_sequence_mut(sequence_id)
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
                        .remove_pending_touch_move_actions(sequence_id);
                    self.touch_handler.try_remove_touch_sequence(sequence_id);
                },
            }
        } else {
            debug!(
                "Touch event {:?} in sequence {:?} allowed",
                event_type, sequence_id
            );
            match event_type {
                TouchEventType::Down => {},
                TouchEventType::Move => {
                    self.pending_scroll_zoom_events.extend(
                        self.touch_handler
                            .take_pending_touch_move_actions(sequence_id),
                    );
                    self.touch_handler
                        .set_handling_touch_move(self.touch_handler.current_sequence_id, false);
                    if let Some(info) = self.touch_handler.get_touch_sequence_mut(sequence_id) {
                        if info.prevent_move == TouchMoveAllowed::Pending {
                            info.prevent_move = TouchMoveAllowed::Allowed;
                            if let TouchSequenceState::PendingFling { velocity, point } = info.state
                            {
                                info.state = TouchSequenceState::Flinging { velocity, point }
                            }
                        }
                    }
                },
                TouchEventType::Up => {
                    let Some(info) = self.touch_handler.get_touch_sequence_mut(sequence_id) else {
                        // The sequence was already removed because there is no default action.
                        return;
                    };
                    match info.state {
                        TouchSequenceState::PendingClick(point) => {
                            info.state = TouchSequenceState::Finished;
                            // PreventDefault from touch_down may have been processed after
                            // touch_up already occurred.
                            if !info.prevent_click {
                                self.simulate_mouse_click(render_api, point);
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
                        .remove_pending_touch_move_actions(sequence_id);
                    self.touch_handler.try_remove_touch_sequence(sequence_id);
                },
            }
        }
    }

    /// <http://w3c.github.io/touch-events/#mouse-events>
    fn simulate_mouse_click(&mut self, render_api: &RenderApi, point: DevicePoint) {
        let button = MouseButton::Left;
        self.dispatch_input_event_with_hit_testing(
            render_api,
            InputEvent::MouseMove(MouseMoveEvent::new_compatibility_for_touch(point.into())).into(),
        );
        self.dispatch_input_event_with_hit_testing(
            render_api,
            InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Down,
                button,
                point.into(),
            ))
            .into(),
        );
        self.dispatch_input_event_with_hit_testing(
            render_api,
            InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Up,
                button,
                point.into(),
            ))
            .into(),
        );
    }

    pub(crate) fn notify_scroll_event(&mut self, scroll: Scroll, point: WebViewPoint) {
        let point = point.as_device_point(self.device_pixels_per_page_pixel());
        self.on_scroll_window_event(scroll, point);
    }

    fn on_scroll_window_event(&mut self, scroll: Scroll, cursor: DevicePoint) {
        self.pending_scroll_zoom_events
            .push(ScrollZoomEvent::Scroll(ScrollEvent {
                scroll,
                point: cursor,
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
        render_api: &RenderApi,
    ) -> (PinchZoomResult, Option<ScrollResult>) {
        if self.pending_scroll_zoom_events.is_empty() {
            return (PinchZoomResult::DidNotPinchZoom, None);
        }

        // Batch up all scroll events and changes to pinch zoom into a single change, or
        // else we'll do way too much painting.
        let mut combined_scroll_event: Option<ScrollEvent> = None;
        let mut new_pinch_zoom = self.pinch_zoom;
        let device_pixels_per_page_pixel = self.device_pixels_per_page_pixel();

        for scroll_event in self.pending_scroll_zoom_events.drain(..) {
            match scroll_event {
                ScrollZoomEvent::PinchZoom(factor, center) => {
                    new_pinch_zoom.zoom(factor, center);
                },
                ScrollZoomEvent::Scroll(scroll_event_info) => {
                    let combined_event = match combined_scroll_event.as_mut() {
                        None => {
                            combined_scroll_event = Some(scroll_event_info);
                            continue;
                        },
                        Some(combined_event) => combined_event,
                    };

                    match (combined_event.scroll, scroll_event_info.scroll) {
                        (Scroll::Delta(old_delta), Scroll::Delta(new_delta)) => {
                            // Mac OS X sometimes delivers scroll events out of vsync during a
                            // fling. This causes events to get bunched up occasionally, causing
                            // nasty-looking "pops". To mitigate this, during a fling we average
                            // deltas instead of summing them.
                            let old_event_count = combined_event.event_count as f32;
                            combined_event.event_count += 1;
                            let new_event_count = combined_event.event_count as f32;
                            let old_delta =
                                old_delta.as_device_vector(device_pixels_per_page_pixel);
                            let new_delta =
                                new_delta.as_device_vector(device_pixels_per_page_pixel);
                            let delta = (old_delta * old_event_count + new_delta) / new_event_count;
                            combined_event.scroll = Scroll::Delta(delta.into());
                        },
                        (Scroll::Start, _) | (Scroll::End, _) => {
                            // Once we see Start or End, we shouldn't process any more events.
                            break;
                        },
                        (_, Scroll::Start) | (_, Scroll::End) => {
                            // If this is an event which is scrolling to the start or end of the page,
                            // disregard other pending events and exit the loop.
                            *combined_event = scroll_event_info;
                            break;
                        },
                    }
                },
            }
        }

        // When zoomed in via pinch zoom, first try to move the center of the zoom and use the rest
        // of the delta for scrolling. This allows moving the zoomed into viewport around in the
        // unzoomed viewport before actually scrolling the underlying layers.
        if let Some(combined_scroll_event) = combined_scroll_event.as_mut() {
            new_pinch_zoom.pan(
                &mut combined_scroll_event.scroll,
                self.device_pixels_per_page_pixel(),
            )
        }

        let scroll_result = combined_scroll_event.and_then(|combined_event| {
            self.scroll_node_at_device_point(
                render_api,
                combined_event.point.to_f32(),
                combined_event.scroll,
            )
        });
        if let Some(ref scroll_result) = scroll_result {
            self.send_scroll_positions_to_layout_for_pipeline(
                scroll_result.hit_test_result.pipeline_id,
            );
            self.dispatch_scroll_event(
                scroll_result.external_scroll_id,
                scroll_result.hit_test_result.clone(),
            );
        } else {
            self.touch_handler.stop_fling_if_needed();
        }

        // Additionally notify pinch zoom update to the script.
        let pinch_zoom_result = self.set_pinch_zoom(new_pinch_zoom);
        if pinch_zoom_result == PinchZoomResult::DidPinchZoom {
            self.send_pinch_zoom_infos_to_script();
        }

        (pinch_zoom_result, scroll_result)
    }

    /// Perform a hit test at the given [`DevicePoint`] and apply the [`Scroll`]
    /// scrolling to the applicable scroll node under that point. If a scroll was
    /// performed, returns the hit test result contains [`PipelineId`] of the node
    /// scrolled, the id, and the final scroll delta.
    fn scroll_node_at_device_point(
        &mut self,
        render_api: &RenderApi,
        cursor: DevicePoint,
        scroll: Scroll,
    ) -> Option<ScrollResult> {
        let scroll_location = match scroll {
            Scroll::Delta(delta) => {
                let device_pixels_per_page = self.device_pixels_per_page_pixel();
                let calculate_delta =
                    delta.as_device_vector(device_pixels_per_page) / device_pixels_per_page;
                ScrollLocation::Delta(calculate_delta.cast_unit())
            },
            Scroll::Start => ScrollLocation::Start,
            Scroll::End => ScrollLocation::End,
        };

        let hit_test_results: Vec<_> = self
            .touch_handler
            .get_hit_test_result_cache_value()
            .map(|result| vec![result])
            .unwrap_or_else(|| self.hit_test(render_api, cursor));

        // Iterate through all hit test results, processing only the first node of each pipeline.
        // This is needed to propagate the scroll events from a pipeline representing an iframe to
        // its ancestor pipelines.
        let mut previous_pipeline_id = None;
        for hit_test_result in hit_test_results {
            let pipeline_details = self.pipelines.get_mut(&hit_test_result.pipeline_id)?;
            if previous_pipeline_id.replace(hit_test_result.pipeline_id) !=
                Some(hit_test_result.pipeline_id)
            {
                let scroll_result = pipeline_details.scroll_tree.scroll_node_or_ancestor(
                    hit_test_result.external_scroll_id,
                    scroll_location,
                    ScrollType::InputEvents,
                );
                if let Some((external_scroll_id, offset)) = scroll_result {
                    // We would like to cache the hit test for the node that that actually scrolls
                    // while panning, which we don't know until right now (as some nodes
                    // might be at the end of their scroll area). In particular, directionality of
                    // scroll matters. That's why this is done here and not as soon as the touch
                    // starts.
                    self.touch_handler.set_hit_test_result_cache_value(
                        hit_test_result.clone(),
                        self.device_pixels_per_page_pixel(),
                    );
                    return Some(ScrollResult {
                        hit_test_result,
                        external_scroll_id,
                        offset,
                    });
                }
            }
        }
        None
    }

    /// Scroll the viewport (root pipeline, root scroll node) of this WebView, but first
    /// attempting to pan the pinch zoom viewport. This is called when processing
    /// key-based scrolling from script.
    pub(crate) fn scroll_viewport_by_delta(
        &mut self,
        delta: LayoutVector2D,
    ) -> (PinchZoomResult, Vec<ScrollResult>) {
        let device_pixels_per_page_pixel = self.device_pixels_per_page_pixel();
        let delta_in_device_pixels = delta.cast_unit() * device_pixels_per_page_pixel;
        let remaining = self.pinch_zoom.pan_with_device_scroll(
            Scroll::Delta(delta_in_device_pixels.into()),
            device_pixels_per_page_pixel,
        );

        let pinch_zoom_result = match remaining == delta_in_device_pixels {
            true => PinchZoomResult::DidNotPinchZoom,
            false => PinchZoomResult::DidPinchZoom,
        };
        if remaining == Vector2D::zero() {
            return (pinch_zoom_result, vec![]);
        }

        let Some(root_pipeline_id) = self.root_pipeline_id else {
            return (pinch_zoom_result, vec![]);
        };
        let Some(root_pipeline) = self.pipelines.get_mut(&root_pipeline_id) else {
            return (pinch_zoom_result, vec![]);
        };

        let remaining = remaining / device_pixels_per_page_pixel;
        let Some((external_scroll_id, offset)) = root_pipeline.scroll_tree.scroll_node_or_ancestor(
            ExternalScrollId(0, root_pipeline_id.into()),
            ScrollLocation::Delta(remaining.cast_unit()),
            // These are initiated only by keyboard events currently.
            ScrollType::InputEvents,
        ) else {
            return (pinch_zoom_result, vec![]);
        };

        let hit_test_result = PaintHitTestResult {
            pipeline_id: root_pipeline_id,
            // It's difficult to get a good value for this as it needs to be piped
            // all the way through script and back here.
            point_in_viewport: Default::default(),
            external_scroll_id,
        };

        self.send_scroll_positions_to_layout_for_pipeline(root_pipeline_id);
        self.dispatch_scroll_event(external_scroll_id, hit_test_result.clone());

        if pinch_zoom_result == PinchZoomResult::DidNotPinchZoom {
            self.send_pinch_zoom_infos_to_script();
        }

        let scroll_result = ScrollResult {
            hit_test_result,
            external_scroll_id,
            offset,
        };
        (pinch_zoom_result, vec![scroll_result])
    }

    /// Send [`PinchZoom`] update to the script's root pipeline.
    fn send_pinch_zoom_infos_to_script(&self) {
        // Pinch-zoom is applicable only to the root pipeline.
        let Some(pipeline_id) = self.root_pipeline_id else {
            return;
        };

        let pinch_zoom_infos = self.pinch_zoom.get_pinch_zoom_infos_for_script(
            self.device_pixels_per_page_pixel_not_including_pinch_zoom(),
        );

        let _ = self.embedder_to_constellation_sender.send(
            EmbedderToConstellationMessage::UpdatePinchZoomInfos(pipeline_id, pinch_zoom_infos),
        );
    }

    fn dispatch_scroll_event(
        &self,
        external_id: ExternalScrollId,
        hit_test_result: PaintHitTestResult,
    ) {
        let event = InputEvent::Scroll(EmbedderScrollEvent { external_id }).into();
        let msg = EmbedderToConstellationMessage::ForwardInputEvent(
            self.id,
            event,
            Some(hit_test_result),
        );
        if let Err(e) = self.embedder_to_constellation_sender.send(msg) {
            warn!("Sending scroll event to constellation failed ({:?}).", e);
        }
    }

    pub(crate) fn pinch_zoom(&self) -> PinchZoom {
        self.pinch_zoom
    }

    fn set_pinch_zoom(&mut self, requested_pinch_zoom: PinchZoom) -> PinchZoomResult {
        if requested_pinch_zoom == self.pinch_zoom {
            return PinchZoomResult::DidNotPinchZoom;
        }

        self.pinch_zoom = requested_pinch_zoom;
        PinchZoomResult::DidPinchZoom
    }

    pub(crate) fn set_page_zoom(
        &mut self,
        new_page_zoom: Scale<f32, CSSPixel, DeviceIndependentPixel>,
    ) {
        let new_page_zoom = new_page_zoom.clamp(MIN_PAGE_ZOOM, MAX_PAGE_ZOOM);
        let old_zoom = std::mem::replace(&mut self.page_zoom, new_page_zoom);
        if old_zoom != self.page_zoom {
            self.send_window_size_message();
        }
    }

    /// The scale to use when displaying this [`WebViewRenderer`] in WebRender
    /// including both viewport scale (page zoom and hidpi scale) as well as any
    /// pinch zoom applied. This is based on the latest display list received,
    /// as page zoom changes are applied asynchronously and the rendered view
    /// should reflect the latest display list.
    pub(crate) fn device_pixels_per_page_pixel(&self) -> Scale<f32, CSSPixel, DevicePixel> {
        let viewport_scale = self
            .root_pipeline_id
            .and_then(|pipeline_id| self.pipelines.get(&pipeline_id))
            .and_then(|pipeline| pipeline.viewport_scale)
            .unwrap_or_else(|| self.page_zoom * self.hidpi_scale_factor);
        viewport_scale * self.pinch_zoom.zoom_factor()
    }

    /// The current viewport scale (hidpi scale and page zoom and not pinch
    /// zoom) based on the current setting of the WebView. Note that this may
    /// not be the rendered viewport zoom as that is based on the latest display
    /// list and zoom changes are applied asynchronously.
    pub(crate) fn device_pixels_per_page_pixel_not_including_pinch_zoom(
        &self,
    ) -> Scale<f32, CSSPixel, DevicePixel> {
        self.page_zoom * self.hidpi_scale_factor
    }

    /// Adjust the pinch zoom of the [`WebView`] by the given zoom delta.
    pub(crate) fn adjust_pinch_zoom(&mut self, magnification: f32, center: DevicePoint) {
        if magnification == 1.0 {
            return;
        }

        self.pending_scroll_zoom_events
            .push(ScrollZoomEvent::PinchZoom(magnification, center));
    }

    fn send_window_size_message(&self) {
        // The device pixel ratio used by the style system should include the scale from page pixels
        // to device pixels, but not including any pinch zoom.
        let device_pixel_ratio = self.device_pixels_per_page_pixel_not_including_pinch_zoom();
        let initial_viewport = self.rect.size().to_f32() / device_pixel_ratio;
        let _ = self.embedder_to_constellation_sender.send(
            EmbedderToConstellationMessage::ChangeViewportDetails(
                self.id,
                ViewportDetails {
                    hidpi_scale_factor: device_pixel_ratio,
                    size: initial_viewport,
                },
                WindowSizeType::Resize,
            ),
        );
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
            self.pinch_zoom.resize_unscaled_viewport(new_rect);
            self.send_pinch_zoom_infos_to_script();
        }
        old_rect != self.rect
    }

    pub fn set_viewport_description(&mut self, viewport_description: ViewportDescription) {
        self.set_page_zoom(Scale::new(
            viewport_description.clamp_page_zoom(viewport_description.initial_scale.get()),
        ));
        self.viewport_description = Some(viewport_description);
    }

    pub(crate) fn scroll_trees_memory_usage(
        &self,
        ops: &mut malloc_size_of::MallocSizeOfOps,
    ) -> usize {
        self.pipelines
            .values()
            .map(|pipeline| pipeline.scroll_tree.size_of(ops))
            .sum::<usize>()
    }

    pub(crate) fn notify_input_event_handled(
        &mut self,
        render_api: &RenderApi,
        repaint_reason: &Cell<RepaintReason>,
        id: InputEventId,
        result: InputEventResult,
    ) {
        if let Some(pending_touch_input_event) =
            self.touch_handler.take_pending_touch_input_event(id)
        {
            self.on_touch_event_processed(render_api, pending_touch_input_event, result);
            self.touch_handler
                .add_touch_move_refresh_observer_if_necessary(
                    self.refresh_driver.clone(),
                    repaint_reason,
                );
        }

        if let Some(wheel_event) = self.pending_wheel_events.remove(&id) {
            if !result.contains(InputEventResult::DefaultPrevented) {
                // A scroll delta for a wheel event is the inverse of the wheel delta.
                let scroll_delta =
                    DeviceVector2D::new(-wheel_event.delta.x as f32, -wheel_event.delta.y as f32);
                self.notify_scroll_event(Scroll::Delta(scroll_delta.into()), wheel_event.point);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnknownWebView(pub WebViewId);
