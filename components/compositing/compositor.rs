/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::env;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::iter::once;
use std::mem::take;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base::cross_process_instant::CrossProcessInstant;
use base::id::{PipelineId, TopLevelBrowsingContextId, WebViewId};
use base::{Epoch, WebRenderEpochToU16};
use bitflags::bitflags;
use compositing_traits::{
    CompositionPipeline, CompositorMsg, CompositorReceiver, ConstellationMsg, SendableFrameTree,
};
use crossbeam_channel::Sender;
use dpi::PhysicalSize;
use embedder_traits::{
    Cursor, InputEvent, MouseButtonEvent, MouseMoveEvent, ShutdownState, TouchEventType,
};
use euclid::{Box2D, Point2D, Rect, Scale, Size2D, Transform3D};
use fnv::FnvHashMap;
use ipc_channel::ipc::{self, IpcSharedMemory};
use libc::c_void;
use log::{debug, info, trace, warn};
use pixels::{CorsStatus, Image, PixelFormat};
use profile_traits::time::{self as profile_time, ProfilerCategory};
use profile_traits::time_profile;
use script_traits::{
    AnimationState, AnimationTickType, ScriptThreadMessage, WindowSizeData, WindowSizeType,
};
use servo_config::opts;
use servo_geometry::DeviceIndependentPixel;
use style_traits::{CSSPixel, PinchZoomFactor};
use webrender::{CaptureBits, RenderApi, Transaction};
use webrender_api::units::{
    DeviceIntPoint, DeviceIntRect, DevicePixel, DevicePoint, DeviceRect, LayoutPoint, LayoutRect,
    LayoutSize, LayoutVector2D, WorldPoint,
};
use webrender_api::{
    self, BuiltDisplayList, DirtyRect, DisplayListPayload, DocumentId, Epoch as WebRenderEpoch,
    ExternalScrollId, FontInstanceFlags, FontInstanceKey, FontInstanceOptions, FontKey,
    HitTestFlags, PipelineId as WebRenderPipelineId, PropertyBinding, ReferenceFrameKind,
    RenderReasons, SampledScrollOffset, ScrollLocation, SpaceAndClipInfo, SpatialId,
    SpatialTreeItemKey, TransformStyle,
};
use webrender_traits::display_list::{HitTestInfo, ScrollTree};
use webrender_traits::rendering_context::RenderingContext;
use webrender_traits::{
    CompositorHitTestResult, CrossProcessCompositorMessage, ImageUpdate, UntrustedNodeAddress,
};

use crate::InitialCompositorState;
use crate::webview::{UnknownWebView, WebView, WebViewManager};
use crate::windowing::{self, EmbedderCoordinates, WebRenderDebugOption, WindowMethods};

#[derive(Debug, PartialEq)]
enum UnableToComposite {
    NotReadyToPaintImage(NotReadyToPaint),
}

#[derive(Debug, PartialEq)]
enum NotReadyToPaint {
    AnimationsActive,
    JustNotifiedConstellation,
    WaitingOnConstellation,
}

// Default viewport constraints
const MAX_ZOOM: f32 = 8.0;
const MIN_ZOOM: f32 = 0.1;

/// Holds the state when running reftests that determines when it is
/// safe to save the output image.
#[derive(Clone, Copy, Debug, PartialEq)]
enum ReadyState {
    Unknown,
    WaitingForConstellationReply,
    ReadyToSaveImage,
}
/// Data that is shared by all WebView renderers.
pub struct ServoRenderer {
    /// This is a temporary map between [`PipelineId`]s and their associated [`WebViewId`]. Once
    /// all renderer operations become per-`WebView` this map can be removed, but we still sometimes
    /// need to work backwards to figure out what `WebView` is associated with a `Pipeline`.
    pub(crate) pipeline_to_webview_map: HashMap<PipelineId, WebViewId>,

    /// Tracks whether we are in the process of shutting down, or have shut down and should close
    /// the compositor. This is shared with the `Servo` instance.
    shutdown_state: Rc<Cell<ShutdownState>>,

    /// The port on which we receive messages.
    compositor_receiver: CompositorReceiver,

    /// The channel on which messages can be sent to the constellation.
    pub(crate) constellation_sender: Sender<ConstellationMsg>,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: profile_time::ProfilerChan,

    /// The WebRender [`RenderApi`] interface used to communicate with WebRender.
    pub(crate) webrender_api: RenderApi,

    /// The active webrender document.
    pub(crate) webrender_document: DocumentId,

    /// The GL bindings for webrender
    webrender_gl: Rc<dyn gleam::gl::Gl>,

    /// The string representing the version of Servo that is running. This is used to tag
    /// WebRender capture output.
    version_string: String,

    #[cfg(feature = "webxr")]
    /// Some XR devices want to run on the main thread.
    webxr_main_thread: webxr::MainThreadRegistry,

    /// True to translate mouse input into touch events.
    pub(crate) convert_mouse_to_touch: bool,

    /// Current mouse cursor.
    cursor: Cursor,
}

/// NB: Never block on the constellation, because sometimes the constellation blocks on us.
pub struct IOCompositor {
    /// Data that is shared by all WebView renderers.
    global: Rc<RefCell<ServoRenderer>>,

    /// Our top-level browsing contexts.
    webviews: WebViewManager<WebView>,

    /// The application window.
    pub window: Rc<dyn WindowMethods>,

    /// "Mobile-style" zoom that does not reflow the page.
    viewport_zoom: PinchZoomFactor,

    /// Viewport zoom constraints provided by @viewport.
    min_viewport_zoom: Option<PinchZoomFactor>,
    max_viewport_zoom: Option<PinchZoomFactor>,

    /// "Desktop-style" zoom that resizes the viewport to fit the window.
    page_zoom: Scale<f32, CSSPixel, DeviceIndependentPixel>,

    /// Tracks whether or not the view needs to be repainted.
    needs_repaint: Cell<RepaintReason>,

    /// Tracks whether the zoom action has happened recently.
    zoom_action: bool,

    /// The time of the last zoom action has started.
    zoom_time: f64,

    /// Used by the logic that determines when it is safe to output an
    /// image for the reftest framework.
    ready_to_save_state: ReadyState,

    /// The webrender renderer.
    webrender: Option<webrender::Renderer>,

    /// The surfman instance that webrender targets
    rendering_context: Rc<dyn RenderingContext>,

    /// The coordinates of the native window, its view and the screen.
    embedder_coordinates: EmbedderCoordinates,

    /// Current cursor position.
    cursor_pos: DevicePoint,

    /// The number of frames pending to receive from WebRender.
    pending_frames: usize,

    /// The [`Instant`] of the last animation tick, used to avoid flooding the Constellation and
    /// ScriptThread with a deluge of animation ticks.
    last_animation_tick: Instant,
}

/// Why we need to be repainted. This is used for debugging.
#[derive(Clone, Copy, Default)]
struct RepaintReason(u8);

bitflags! {
    impl RepaintReason: u8 {
        /// We're performing the single repaint in headless mode.
        const ReadyForScreenshot = 1 << 0;
        /// We're performing a repaint to run an animation.
        const ChangedAnimationState = 1 << 1;
        /// A new WebRender frame has arrived.
        const NewWebRenderFrame = 1 << 2;
        /// The window has been resized and will need to be synchronously repainted.
        const Resize = 1 << 3;
    }
}

pub(crate) struct PipelineDetails {
    /// The pipeline associated with this PipelineDetails object.
    pub pipeline: Option<CompositionPipeline>,

    /// The [`PipelineId`] of this pipeline.
    pub id: PipelineId,

    /// The id of the parent pipeline, if any.
    pub parent_pipeline_id: Option<PipelineId>,

    /// The epoch of the most recent display list for this pipeline. Note that this display
    /// list might not be displayed, as WebRender processes display lists asynchronously.
    pub most_recent_display_list_epoch: Option<WebRenderEpoch>,

    /// Whether animations are running
    pub animations_running: bool,

    /// Whether there are animation callbacks
    pub animation_callbacks_running: bool,

    /// Whether to use less resources by stopping animations.
    pub throttled: bool,

    /// Hit test items for this pipeline. This is used to map WebRender hit test
    /// information to the full information necessary for Servo.
    pub hit_test_items: Vec<HitTestInfo>,

    /// The compositor-side [ScrollTree]. This is used to allow finding and scrolling
    /// nodes in the compositor before forwarding new offsets to WebRender.
    pub scroll_tree: ScrollTree,

    /// A per-pipeline queue of display lists that have not yet been rendered by WebRender. Layout
    /// expects WebRender to paint each given epoch. Once the compositor paints a frame with that
    /// epoch's display list, it will be removed from the queue and the paint time will be recorded
    /// as a metric. In case new display lists come faster than painting a metric might never be
    /// recorded.
    pub pending_paint_metrics: Vec<Epoch>,
}

impl PipelineDetails {
    pub(crate) fn animations_or_animation_callbacks_running(&self) -> bool {
        self.animations_running || self.animation_callbacks_running
    }

    pub(crate) fn animation_callbacks_running(&self) -> bool {
        self.animation_callbacks_running
    }

    pub(crate) fn tick_animations(&self, compositor: &IOCompositor) -> bool {
        let animation_callbacks_running = self.animation_callbacks_running;
        let animations_running = self.animations_running;
        if !animation_callbacks_running && !animations_running {
            return false;
        }

        if self.throttled {
            return false;
        }

        let mut tick_type = AnimationTickType::empty();
        if animations_running {
            tick_type.insert(AnimationTickType::CSS_ANIMATIONS_AND_TRANSITIONS);
        }
        if animation_callbacks_running {
            tick_type.insert(AnimationTickType::REQUEST_ANIMATION_FRAME);
        }

        let msg = ConstellationMsg::TickAnimation(self.id, tick_type);
        if let Err(e) = compositor.global.borrow().constellation_sender.send(msg) {
            warn!("Sending tick to constellation failed ({:?}).", e);
        }
        true
    }
}

impl PipelineDetails {
    pub(crate) fn new(id: PipelineId) -> PipelineDetails {
        PipelineDetails {
            pipeline: None,
            id,
            parent_pipeline_id: None,
            most_recent_display_list_epoch: None,
            animations_running: false,
            animation_callbacks_running: false,
            throttled: false,
            hit_test_items: Vec::new(),
            scroll_tree: ScrollTree::default(),
            pending_paint_metrics: Vec::new(),
        }
    }

    fn install_new_scroll_tree(&mut self, new_scroll_tree: ScrollTree) {
        let old_scroll_offsets: FnvHashMap<ExternalScrollId, LayoutVector2D> = self
            .scroll_tree
            .nodes
            .drain(..)
            .filter_map(|node| match (node.external_id(), node.offset()) {
                (Some(external_id), Some(offset)) => Some((external_id, offset)),
                _ => None,
            })
            .collect();

        self.scroll_tree = new_scroll_tree;
        for node in self.scroll_tree.nodes.iter_mut() {
            match node.external_id() {
                Some(external_id) => match old_scroll_offsets.get(&external_id) {
                    Some(new_offset) => node.set_offset(*new_offset),
                    None => continue,
                },
                _ => continue,
            };
        }
    }
}

impl ServoRenderer {
    pub fn shutdown_state(&self) -> ShutdownState {
        self.shutdown_state.get()
    }

    pub(crate) fn hit_test_at_point<'a>(
        &self,
        point: DevicePoint,
        details_for_pipeline: impl Fn(PipelineId) -> Option<&'a PipelineDetails>,
    ) -> Option<CompositorHitTestResult> {
        self.hit_test_at_point_with_flags_and_pipeline(
            point,
            HitTestFlags::empty(),
            None,
            details_for_pipeline,
        )
        .first()
        .cloned()
    }

    // TODO: split this into first half (global) and second half (one for whole compositor, one for webview)
    pub(crate) fn hit_test_at_point_with_flags_and_pipeline<'a>(
        &self,
        point: DevicePoint,
        flags: HitTestFlags,
        pipeline_id: Option<WebRenderPipelineId>,
        details_for_pipeline: impl Fn(PipelineId) -> Option<&'a PipelineDetails>,
    ) -> Vec<CompositorHitTestResult> {
        // DevicePoint and WorldPoint are the same for us.
        let world_point = WorldPoint::from_untyped(point.to_untyped());
        let results =
            self.webrender_api
                .hit_test(self.webrender_document, pipeline_id, world_point, flags);

        results
            .items
            .iter()
            .filter_map(|item| {
                let pipeline_id = item.pipeline.into();
                let details = details_for_pipeline(pipeline_id)?;

                // If the epoch in the tag does not match the current epoch of the pipeline,
                // then the hit test is against an old version of the display list and we
                // should ignore this hit test for now.
                match details.most_recent_display_list_epoch {
                    Some(epoch) if epoch.as_u16() == item.tag.1 => {},
                    _ => return None,
                }

                let info = &details.hit_test_items[item.tag.0 as usize];
                Some(CompositorHitTestResult {
                    pipeline_id,
                    point_in_viewport: item.point_in_viewport.to_untyped(),
                    point_relative_to_item: item.point_relative_to_item.to_untyped(),
                    node: UntrustedNodeAddress(info.node as *const c_void),
                    cursor: info.cursor,
                    scroll_tree_node: info.scroll_tree_node,
                })
            })
            .collect()
    }

    pub(crate) fn send_transaction(&mut self, transaction: Transaction) {
        self.webrender_api
            .send_transaction(self.webrender_document, transaction);
    }

    pub(crate) fn update_cursor(&mut self, result: &CompositorHitTestResult) {
        let cursor = match result.cursor {
            Some(cursor) if cursor != self.cursor => cursor,
            _ => return,
        };

        let Some(webview_id) = self
            .pipeline_to_webview_map
            .get(&result.pipeline_id)
            .cloned()
        else {
            warn!("Couldn't update cursor for non-WebView-associated pipeline");
            return;
        };

        self.cursor = cursor;
        if let Err(e) = self
            .constellation_sender
            .send(ConstellationMsg::SetCursor(webview_id, cursor))
        {
            warn!("Sending event to constellation failed ({:?}).", e);
        }
    }
}

impl IOCompositor {
    pub fn new(
        window: Rc<dyn WindowMethods>,
        state: InitialCompositorState,
        convert_mouse_to_touch: bool,
        version_string: String,
    ) -> Self {
        let compositor = IOCompositor {
            global: Rc::new(RefCell::new(ServoRenderer {
                shutdown_state: state.shutdown_state,
                pipeline_to_webview_map: Default::default(),
                compositor_receiver: state.receiver,
                constellation_sender: state.constellation_chan,
                time_profiler_chan: state.time_profiler_chan,
                webrender_api: state.webrender_api,
                webrender_document: state.webrender_document,
                webrender_gl: state.webrender_gl,
                version_string,
                #[cfg(feature = "webxr")]
                webxr_main_thread: state.webxr_main_thread,
                convert_mouse_to_touch,
                cursor: Cursor::None,
            })),
            webviews: WebViewManager::default(),
            embedder_coordinates: window.get_coordinates(),
            window,
            needs_repaint: Cell::default(),
            page_zoom: Scale::new(1.0),
            viewport_zoom: PinchZoomFactor::new(1.0),
            min_viewport_zoom: Some(PinchZoomFactor::new(1.0)),
            max_viewport_zoom: None,
            zoom_action: false,
            zoom_time: 0f64,
            ready_to_save_state: ReadyState::Unknown,
            webrender: Some(state.webrender),
            rendering_context: state.rendering_context,
            cursor_pos: DevicePoint::new(0.0, 0.0),
            pending_frames: 0,
            last_animation_tick: Instant::now(),
        };

        {
            let gl = &compositor.global.borrow().webrender_gl;
            info!("Running on {}", gl.get_string(gleam::gl::RENDERER));
            info!("OpenGL Version {}", gl.get_string(gleam::gl::VERSION));
        }
        compositor.assert_gl_framebuffer_complete();
        compositor
    }

    pub fn deinit(&mut self) {
        if let Err(err) = self.rendering_context.make_current() {
            warn!("Failed to make the rendering context current: {:?}", err);
        }
        if let Some(webrender) = self.webrender.take() {
            webrender.deinit();
        }
    }

    fn set_needs_repaint(&self, reason: RepaintReason) {
        let mut needs_repaint = self.needs_repaint.get();
        needs_repaint.insert(reason);
        self.needs_repaint.set(needs_repaint);
    }

    pub fn needs_repaint(&self) -> bool {
        !self.needs_repaint.get().is_empty()
    }

    pub fn finish_shutting_down(&mut self) {
        // Drain compositor port, sometimes messages contain channels that are blocking
        // another thread from finishing (i.e. SetFrameTree).
        while self
            .global
            .borrow_mut()
            .compositor_receiver
            .try_recv_compositor_msg()
            .is_some()
        {}

        // Tell the profiler, memory profiler, and scrolling timer to shut down.
        if let Ok((sender, receiver)) = ipc::channel() {
            self.global
                .borrow()
                .time_profiler_chan
                .send(profile_time::ProfilerMsg::Exit(sender));
            let _ = receiver.recv();
        }
    }

    fn handle_browser_message(&mut self, msg: CompositorMsg) {
        trace_msg_from_constellation!(msg, "{msg:?}");

        match self.shutdown_state() {
            ShutdownState::NotShuttingDown => {},
            ShutdownState::ShuttingDown => {
                self.handle_browser_message_while_shutting_down(msg);
                return;
            },
            ShutdownState::FinishedShuttingDown => {
                // Messages to the compositor are ignored after shutdown is complete.
                return;
            },
        }

        match msg {
            CompositorMsg::ChangeRunningAnimationsState(
                webview_id,
                pipeline_id,
                animation_state,
            ) => {
                let mut throttled = true;
                if let Some(webview) = self.webviews.get_mut(webview_id) {
                    throttled =
                        webview.change_running_animations_state(pipeline_id, animation_state);
                }

                // These operations should eventually happen per-WebView, but they are global now as rendering
                // is still global to all WebViews.
                if !throttled && animation_state == AnimationState::AnimationsPresent {
                    self.set_needs_repaint(RepaintReason::ChangedAnimationState);
                }

                if !throttled && animation_state == AnimationState::AnimationCallbacksPresent {
                    // We need to fetch the WebView again in order to avoid a double borrow.
                    if let Some(webview) = self.webviews.get(webview_id) {
                        webview.tick_animations_for_pipeline(pipeline_id, self);
                    }
                }
            },

            CompositorMsg::CreateOrUpdateWebView(frame_tree) => {
                self.set_frame_tree_for_webview(&frame_tree);
            },

            CompositorMsg::RemoveWebView(top_level_browsing_context_id) => {
                self.remove_webview(top_level_browsing_context_id);
            },

            CompositorMsg::TouchEventProcessed(webview_id, result) => {
                let Some(webview) = self.webviews.get_mut(webview_id) else {
                    warn!("Handling input event for unknown webview: {webview_id}");
                    return;
                };
                webview.on_touch_event_processed(result);
            },

            CompositorMsg::CreatePng(page_rect, reply) => {
                let res = self.render_to_shared_memory(page_rect);
                if let Err(ref e) = res {
                    info!("Error retrieving PNG: {:?}", e);
                }
                let img = res.unwrap_or(None);
                if let Err(e) = reply.send(img) {
                    warn!("Sending reply to create png failed ({:?}).", e);
                }
            },

            CompositorMsg::IsReadyToSaveImageReply(is_ready) => {
                assert_eq!(
                    self.ready_to_save_state,
                    ReadyState::WaitingForConstellationReply
                );
                if is_ready && self.pending_frames == 0 {
                    self.ready_to_save_state = ReadyState::ReadyToSaveImage;
                } else {
                    self.ready_to_save_state = ReadyState::Unknown;
                }
                self.set_needs_repaint(RepaintReason::ReadyForScreenshot);
            },

            CompositorMsg::SetThrottled(webview_id, pipeline_id, throttled) => {
                if let Some(webview) = self.webviews.get_mut(webview_id) {
                    webview.set_throttled(pipeline_id, throttled);
                    self.process_animations(true);
                }
            },

            CompositorMsg::PipelineExited(webview_id, pipeline_id, sender) => {
                debug!(
                    "Compositor got pipeline exited: {:?} {:?}",
                    webview_id, pipeline_id
                );
                if let Some(webview) = self.webviews.get_mut(webview_id) {
                    webview.remove_pipeline(pipeline_id);
                }
                let _ = sender.send(());
            },

            CompositorMsg::NewWebRenderFrameReady(_document_id, recomposite_needed) => {
                self.pending_frames -= 1;

                if recomposite_needed {
                    let details_for_pipeline = |pipeline_id| self.details_for_pipeline(pipeline_id);
                    let result = self
                        .global
                        .borrow()
                        .hit_test_at_point(self.cursor_pos, details_for_pipeline);
                    if let Some(result) = result {
                        self.global.borrow_mut().update_cursor(&result);
                    }
                }

                if recomposite_needed || self.animation_callbacks_running() {
                    self.set_needs_repaint(RepaintReason::NewWebRenderFrame);
                }
            },

            CompositorMsg::LoadComplete(_) => {
                if opts::get().wait_for_stable_image {
                    self.set_needs_repaint(RepaintReason::ReadyForScreenshot);
                }
            },

            CompositorMsg::WebDriverMouseButtonEvent(webview_id, action, button, x, y) => {
                let dppx = self.device_pixels_per_page_pixel();
                let point = dppx.transform_point(Point2D::new(x, y));
                let Some(webview) = self.webviews.get_mut(webview_id) else {
                    warn!("Handling input event for unknown webview: {webview_id}");
                    return;
                };
                webview.dispatch_input_event(InputEvent::MouseButton(MouseButtonEvent {
                    point,
                    action,
                    button,
                }));
            },

            CompositorMsg::WebDriverMouseMoveEvent(webview_id, x, y) => {
                let dppx = self.device_pixels_per_page_pixel();
                let point = dppx.transform_point(Point2D::new(x, y));
                let Some(webview) = self.webviews.get_mut(webview_id) else {
                    warn!("Handling input event for unknown webview: {webview_id}");
                    return;
                };
                webview.dispatch_input_event(InputEvent::MouseMove(MouseMoveEvent { point }));
            },

            CompositorMsg::PendingPaintMetric(webview_id, pipeline_id, epoch) => {
                if let Some(webview) = self.webviews.get_mut(webview_id) {
                    webview.add_pending_paint_metric(pipeline_id, epoch);
                }
            },

            CompositorMsg::CrossProcess(cross_proces_message) => {
                self.handle_cross_process_message(cross_proces_message);
            },
        }
    }

    /// Accept messages from content processes that need to be relayed to the WebRender
    /// instance in the parent process.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_cross_process_message(&mut self, msg: CrossProcessCompositorMessage) {
        match msg {
            CrossProcessCompositorMessage::SendInitialTransaction(pipeline) => {
                let mut txn = Transaction::new();
                txn.set_display_list(WebRenderEpoch(0), (pipeline, Default::default()));
                self.generate_frame(&mut txn, RenderReasons::SCENE);
                self.global.borrow_mut().send_transaction(txn);
            },

            CrossProcessCompositorMessage::SendScrollNode(
                webview_id,
                pipeline_id,
                point,
                external_scroll_id,
            ) => {
                let Some(webview) = self.webviews.get_mut(webview_id) else {
                    return;
                };

                let pipeline_id = pipeline_id.into();
                let Some(pipeline_details) = webview.pipelines.get_mut(&pipeline_id) else {
                    return;
                };

                let offset = LayoutVector2D::new(point.x, point.y);
                if !pipeline_details
                    .scroll_tree
                    .set_scroll_offsets_for_node_with_external_scroll_id(
                        external_scroll_id,
                        -offset,
                    )
                {
                    warn!("Could not scroll not with id: {external_scroll_id:?}");
                    return;
                }

                let mut txn = Transaction::new();
                txn.set_scroll_offsets(
                    external_scroll_id,
                    vec![SampledScrollOffset {
                        offset,
                        generation: 0,
                    }],
                );
                self.generate_frame(&mut txn, RenderReasons::APZ);
                self.global.borrow_mut().send_transaction(txn);
            },

            CrossProcessCompositorMessage::SendDisplayList {
                webview_id,
                display_list_info,
                display_list_descriptor,
                display_list_receiver,
            } => {
                // This must match the order from the sender, currently in `shared/script/lib.rs`.
                let items_data = match display_list_receiver.recv() {
                    Ok(display_list_data) => display_list_data,
                    Err(error) => {
                        return warn!(
                            "Could not receive WebRender display list items data: {error}"
                        );
                    },
                };
                let cache_data = match display_list_receiver.recv() {
                    Ok(display_list_data) => display_list_data,
                    Err(error) => {
                        return warn!(
                            "Could not receive WebRender display list cache data: {error}"
                        );
                    },
                };
                let spatial_tree = match display_list_receiver.recv() {
                    Ok(display_list_data) => display_list_data,
                    Err(error) => {
                        return warn!(
                            "Could not receive WebRender display list spatial tree: {error}."
                        );
                    },
                };
                let built_display_list = BuiltDisplayList::from_data(
                    DisplayListPayload {
                        items_data,
                        cache_data,
                        spatial_tree,
                    },
                    display_list_descriptor,
                );

                #[cfg(feature = "tracing")]
                let _span = tracing::trace_span!(
                    "ScriptToCompositorMsg::BuiltDisplayList",
                    servo_profiling = true,
                )
                .entered();

                let Some(webview) = self.webviews.get_mut(webview_id) else {
                    return warn!("Could not find WebView for incoming display list");
                };

                let pipeline_id = display_list_info.pipeline_id;
                let details = webview.ensure_pipeline_details(pipeline_id.into());
                details.most_recent_display_list_epoch = Some(display_list_info.epoch);
                details.hit_test_items = display_list_info.hit_test_info;
                details.install_new_scroll_tree(display_list_info.scroll_tree);

                let mut transaction = Transaction::new();
                transaction
                    .set_display_list(display_list_info.epoch, (pipeline_id, built_display_list));
                self.update_transaction_with_all_scroll_offsets(&mut transaction);
                self.generate_frame(&mut transaction, RenderReasons::SCENE);
                self.global.borrow_mut().send_transaction(transaction);
            },

            CrossProcessCompositorMessage::HitTest(pipeline, point, flags, sender) => {
                // When a display list is sent to WebRender, it starts scene building in a
                // separate thread and then that display list is available for hit testing.
                // Without flushing scene building, any hit test we do might be done against
                // a previous scene, if the last one we sent hasn't finished building.
                //
                // TODO(mrobinson): Flushing all scene building is a big hammer here, because
                // we might only be interested in a single pipeline. The only other option
                // would be to listen to the TransactionNotifier for previous per-pipeline
                // transactions, but that isn't easily compatible with the event loop wakeup
                // mechanism from libserver.
                self.global.borrow().webrender_api.flush_scene_builder();

                let details_for_pipeline = |pipeline_id| self.details_for_pipeline(pipeline_id);
                let result = self
                    .global
                    .borrow()
                    .hit_test_at_point_with_flags_and_pipeline(
                        point,
                        flags,
                        pipeline,
                        details_for_pipeline,
                    );
                let _ = sender.send(result);
            },

            CrossProcessCompositorMessage::GenerateImageKey(sender) => {
                let _ = sender.send(self.global.borrow().webrender_api.generate_image_key());
            },

            CrossProcessCompositorMessage::UpdateImages(updates) => {
                let mut txn = Transaction::new();
                for update in updates {
                    match update {
                        ImageUpdate::AddImage(key, desc, data) => {
                            txn.add_image(key, desc, data.into(), None)
                        },
                        ImageUpdate::DeleteImage(key) => txn.delete_image(key),
                        ImageUpdate::UpdateImage(key, desc, data) => {
                            txn.update_image(key, desc, data.into(), &DirtyRect::All)
                        },
                    }
                }
                self.global.borrow_mut().send_transaction(txn);
            },

            CrossProcessCompositorMessage::AddFont(font_key, data, index) => {
                self.add_font(font_key, index, data);
            },

            CrossProcessCompositorMessage::AddSystemFont(font_key, native_handle) => {
                let mut transaction = Transaction::new();
                transaction.add_native_font(font_key, native_handle);
                self.global.borrow_mut().send_transaction(transaction);
            },

            CrossProcessCompositorMessage::AddFontInstance(
                font_instance_key,
                font_key,
                size,
                flags,
            ) => {
                self.add_font_instance(font_instance_key, font_key, size, flags);
            },

            CrossProcessCompositorMessage::RemoveFonts(keys, instance_keys) => {
                let mut transaction = Transaction::new();

                for instance in instance_keys.into_iter() {
                    transaction.delete_font_instance(instance);
                }
                for key in keys.into_iter() {
                    transaction.delete_font(key);
                }

                self.global.borrow_mut().send_transaction(transaction);
            },

            CrossProcessCompositorMessage::AddImage(key, desc, data) => {
                let mut txn = Transaction::new();
                txn.add_image(key, desc, data.into(), None);
                self.global.borrow_mut().send_transaction(txn);
            },

            CrossProcessCompositorMessage::GenerateFontKeys(
                number_of_font_keys,
                number_of_font_instance_keys,
                result_sender,
            ) => {
                let font_keys = (0..number_of_font_keys)
                    .map(|_| self.global.borrow().webrender_api.generate_font_key())
                    .collect();
                let font_instance_keys = (0..number_of_font_instance_keys)
                    .map(|_| {
                        self.global
                            .borrow()
                            .webrender_api
                            .generate_font_instance_key()
                    })
                    .collect();
                let _ = result_sender.send((font_keys, font_instance_keys));
            },
            CrossProcessCompositorMessage::GetClientWindowRect(req) => {
                if let Err(e) = req.send(self.embedder_coordinates.window_rect) {
                    warn!("Sending response to get client window failed ({:?}).", e);
                }
            },
            CrossProcessCompositorMessage::GetScreenSize(req) => {
                if let Err(e) = req.send(self.embedder_coordinates.screen_size) {
                    warn!("Sending response to get screen size failed ({:?}).", e);
                }
            },
            CrossProcessCompositorMessage::GetAvailableScreenSize(req) => {
                if let Err(e) = req.send(self.embedder_coordinates.available_screen_size) {
                    warn!(
                        "Sending response to get screen avail size failed ({:?}).",
                        e
                    );
                }
            },
        }
    }

    /// Handle messages sent to the compositor during the shutdown process. In general,
    /// the things the compositor can do in this state are limited. It's very important to
    /// answer any synchronous messages though as other threads might be waiting on the
    /// results to finish their own shut down process. We try to do as little as possible
    /// during this time.
    ///
    /// When that involves generating WebRender ids, our approach here is to simply
    /// generate them, but assume they will never be used, since once shutting down the
    /// compositor no longer does any WebRender frame generation.
    fn handle_browser_message_while_shutting_down(&mut self, msg: CompositorMsg) {
        match msg {
            CompositorMsg::PipelineExited(webview_id, pipeline_id, sender) => {
                debug!(
                    "Compositor got pipeline exited: {:?} {:?}",
                    webview_id, pipeline_id
                );
                if let Some(webview) = self.webviews.get_mut(webview_id) {
                    webview.remove_pipeline(pipeline_id);
                }
                let _ = sender.send(());
            },
            CompositorMsg::CrossProcess(CrossProcessCompositorMessage::GenerateImageKey(
                sender,
            )) => {
                let _ = sender.send(self.global.borrow().webrender_api.generate_image_key());
            },
            CompositorMsg::CrossProcess(CrossProcessCompositorMessage::GenerateFontKeys(
                number_of_font_keys,
                number_of_font_instance_keys,
                result_sender,
            )) => {
                let font_keys = (0..number_of_font_keys)
                    .map(|_| self.global.borrow().webrender_api.generate_font_key())
                    .collect();
                let font_instance_keys = (0..number_of_font_instance_keys)
                    .map(|_| {
                        self.global
                            .borrow()
                            .webrender_api
                            .generate_font_instance_key()
                    })
                    .collect();
                let _ = result_sender.send((font_keys, font_instance_keys));
            },
            CompositorMsg::CrossProcess(CrossProcessCompositorMessage::GetClientWindowRect(
                req,
            )) => {
                if let Err(e) = req.send(self.embedder_coordinates.window_rect) {
                    warn!("Sending response to get client window failed ({:?}).", e);
                }
            },
            CompositorMsg::CrossProcess(CrossProcessCompositorMessage::GetScreenSize(req)) => {
                if let Err(e) = req.send(self.embedder_coordinates.screen_size) {
                    warn!("Sending response to get screen size failed ({:?}).", e);
                }
            },
            CompositorMsg::CrossProcess(CrossProcessCompositorMessage::GetAvailableScreenSize(
                req,
            )) => {
                if let Err(e) = req.send(self.embedder_coordinates.available_screen_size) {
                    warn!(
                        "Sending response to get screen avail size failed ({:?}).",
                        e
                    );
                }
            },
            CompositorMsg::NewWebRenderFrameReady(..) => {
                // Subtract from the number of pending frames, but do not do any compositing.
                self.pending_frames -= 1;
            },
            _ => {
                debug!("Ignoring message ({:?} while shutting down", msg);
            },
        }
    }

    /// Queue a new frame in the transaction and increase the pending frames count.
    pub(crate) fn generate_frame(&mut self, transaction: &mut Transaction, reason: RenderReasons) {
        self.pending_frames += 1;
        transaction.generate_frame(0, true /* present */, reason);
    }

    /// Set the root pipeline for our WebRender scene to a display list that consists of an iframe
    /// for each visible top-level browsing context, applying a transformation on the root for
    /// pinch zoom, page zoom, and HiDPI scaling.
    fn send_root_pipeline_display_list(&mut self) {
        let mut transaction = Transaction::new();
        self.send_root_pipeline_display_list_in_transaction(&mut transaction);
        self.generate_frame(&mut transaction, RenderReasons::SCENE);
        self.global.borrow_mut().send_transaction(transaction);
    }

    /// Set the root pipeline for our WebRender scene to a display list that consists of an iframe
    /// for each visible top-level browsing context, applying a transformation on the root for
    /// pinch zoom, page zoom, and HiDPI scaling.
    pub(crate) fn send_root_pipeline_display_list_in_transaction(
        &self,
        transaction: &mut Transaction,
    ) {
        // Every display list needs a pipeline, but we'd like to choose one that is unlikely
        // to conflict with our content pipelines, which start at (1, 1). (0, 0) is WebRender's
        // dummy pipeline, so we choose (0, 1).
        let root_pipeline = WebRenderPipelineId(0, 1);
        transaction.set_root_pipeline(root_pipeline);

        let mut builder = webrender_api::DisplayListBuilder::new(root_pipeline);
        builder.begin();

        let zoom_factor = self.device_pixels_per_page_pixel().0;
        let zoom_reference_frame = builder.push_reference_frame(
            LayoutPoint::zero(),
            SpatialId::root_reference_frame(root_pipeline),
            TransformStyle::Flat,
            PropertyBinding::Value(Transform3D::scale(zoom_factor, zoom_factor, 1.)),
            ReferenceFrameKind::Transform {
                is_2d_scale_translation: true,
                should_snap: true,
                paired_with_perspective: false,
            },
            SpatialTreeItemKey::new(0, 0),
        );

        let scaled_viewport_size =
            self.rendering_context.size2d().to_f32().to_untyped() / zoom_factor;
        let scaled_viewport_rect = LayoutRect::from_origin_and_size(
            LayoutPoint::zero(),
            LayoutSize::from_untyped(scaled_viewport_size),
        );

        let root_clip_id = builder.define_clip_rect(zoom_reference_frame, scaled_viewport_rect);
        let clip_chain_id = builder.define_clip_chain(None, [root_clip_id]);
        for (_, webview) in self.webviews.painting_order() {
            if let Some(pipeline_id) = webview.root_pipeline_id {
                let scaled_webview_rect = webview.rect / zoom_factor;
                builder.push_iframe(
                    LayoutRect::from_untyped(&scaled_webview_rect.to_untyped()),
                    LayoutRect::from_untyped(&scaled_webview_rect.to_untyped()),
                    &SpaceAndClipInfo {
                        spatial_id: zoom_reference_frame,
                        clip_chain_id,
                    },
                    pipeline_id.into(),
                    true,
                );
            }
        }

        let built_display_list = builder.end();

        // NB: We are always passing 0 as the epoch here, but this doesn't seem to
        // be an issue. WebRender will still update the scene and generate a new
        // frame even though the epoch hasn't changed.
        transaction.set_display_list(WebRenderEpoch(0), built_display_list);
        self.update_transaction_with_all_scroll_offsets(transaction);
    }

    /// Update the given transaction with the scroll offsets of all active scroll nodes in
    /// the WebRender scene. This is necessary because WebRender does not preserve scroll
    /// offsets between scroll tree modifications. If a display list could potentially
    /// modify a scroll tree branch, WebRender needs to have scroll offsets for that
    /// branch.
    ///
    /// TODO(mrobinson): Could we only send offsets for the branch being modified
    /// and not the entire scene?
    fn update_transaction_with_all_scroll_offsets(&self, transaction: &mut Transaction) {
        for webview in self.webviews.iter() {
            for details in webview.pipelines.values() {
                for node in details.scroll_tree.nodes.iter() {
                    let (Some(offset), Some(external_id)) = (node.offset(), node.external_id())
                    else {
                        continue;
                    };

                    let offset = LayoutVector2D::new(-offset.x, -offset.y);
                    transaction.set_scroll_offsets(
                        external_id,
                        vec![SampledScrollOffset {
                            offset,
                            generation: 0,
                        }],
                    );
                }
            }
        }
    }

    pub fn add_webview(&mut self, webview_id: WebViewId) {
        let size = self.rendering_context.size2d().to_f32();
        self.webviews.entry(webview_id).or_insert(WebView::new(
            webview_id,
            Box2D::from_origin_and_size(Point2D::origin(), size),
            self.global.clone(),
        ));
    }

    fn set_frame_tree_for_webview(&mut self, frame_tree: &SendableFrameTree) {
        debug!("{}: Setting frame tree for webview", frame_tree.pipeline.id);

        let webview_id = frame_tree.pipeline.top_level_browsing_context_id;
        let Some(webview) = self.webviews.get_mut(webview_id) else {
            warn!(
                "Attempted to set frame tree on unknown WebView (perhaps closed?): {webview_id:?}"
            );
            return;
        };

        webview.set_frame_tree(frame_tree);
        self.send_root_pipeline_display_list();
    }

    fn remove_webview(&mut self, webview_id: WebViewId) {
        debug!("{}: Removing", webview_id);
        if self.webviews.remove(webview_id).is_err() {
            warn!("{webview_id}: Removing unknown webview");
            return;
        };

        self.send_root_pipeline_display_list();
    }

    pub fn move_resize_webview(&mut self, webview_id: TopLevelBrowsingContextId, rect: DeviceRect) {
        debug!("{webview_id}: Moving and/or resizing webview; rect={rect:?}");
        let rect_changed;
        let size_changed;
        match self.webviews.get_mut(webview_id) {
            Some(webview) => {
                rect_changed = rect != webview.rect;
                size_changed = rect.size() != webview.rect.size();
                webview.rect = rect;
            },
            None => {
                warn!("{webview_id}: MoveResizeWebView on unknown webview id");
                return;
            },
        };

        if rect_changed {
            if size_changed {
                self.send_window_size_message_for_top_level_browser_context(rect, webview_id);
            }

            self.send_root_pipeline_display_list();
        }
    }

    pub fn show_webview(
        &mut self,
        webview_id: WebViewId,
        hide_others: bool,
    ) -> Result<(), UnknownWebView> {
        debug!("{webview_id}: Showing webview; hide_others={hide_others}");
        let painting_order_changed = if hide_others {
            let result = self
                .webviews
                .painting_order()
                .map(|(&id, _)| id)
                .ne(once(webview_id));
            self.webviews.hide_all();
            self.webviews.show(webview_id)?;
            result
        } else {
            self.webviews.show(webview_id)?
        };
        if painting_order_changed {
            self.send_root_pipeline_display_list();
        }
        Ok(())
    }

    pub fn hide_webview(&mut self, webview_id: WebViewId) -> Result<(), UnknownWebView> {
        debug!("{webview_id}: Hiding webview");
        if self.webviews.hide(webview_id)? {
            self.send_root_pipeline_display_list();
        }
        Ok(())
    }

    pub fn raise_webview_to_top(
        &mut self,
        webview_id: WebViewId,
        hide_others: bool,
    ) -> Result<(), UnknownWebView> {
        debug!("{webview_id}: Raising webview to top; hide_others={hide_others}");
        let painting_order_changed = if hide_others {
            let result = self
                .webviews
                .painting_order()
                .map(|(&id, _)| id)
                .ne(once(webview_id));
            self.webviews.hide_all();
            self.webviews.raise_to_top(webview_id)?;
            result
        } else {
            self.webviews.raise_to_top(webview_id)?
        };
        if painting_order_changed {
            self.send_root_pipeline_display_list();
        }
        Ok(())
    }

    fn send_window_size_message_for_top_level_browser_context(
        &self,
        rect: DeviceRect,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
    ) {
        // The device pixel ratio used by the style system should include the scale from page pixels
        // to device pixels, but not including any pinch zoom.
        let device_pixel_ratio = self.device_pixels_per_page_pixel_not_including_page_zoom();
        let initial_viewport = rect.size().to_f32() / device_pixel_ratio;
        let msg = ConstellationMsg::WindowSize(
            top_level_browsing_context_id,
            WindowSizeData {
                device_pixel_ratio,
                initial_viewport,
            },
            WindowSizeType::Resize,
        );
        if let Err(e) = self.global.borrow().constellation_sender.send(msg) {
            warn!("Sending window resize to constellation failed ({:?}).", e);
        }
    }

    pub fn on_embedder_window_moved(&mut self) {
        self.embedder_coordinates = self.window.get_coordinates();
    }

    pub fn resize_rendering_context(&mut self, new_size: PhysicalSize<u32>) -> bool {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return false;
        }

        let old_hidpi_factor = self.embedder_coordinates.hidpi_factor;
        self.embedder_coordinates = self.window.get_coordinates();
        if self.embedder_coordinates.hidpi_factor == old_hidpi_factor &&
            self.rendering_context.size() == new_size
        {
            return false;
        }

        self.rendering_context.resize(new_size);

        let mut transaction = Transaction::new();
        let output_region = DeviceIntRect::new(
            Point2D::zero(),
            Point2D::new(new_size.width as i32, new_size.height as i32),
        );
        transaction.set_document_view(output_region);
        self.global.borrow_mut().send_transaction(transaction);

        self.update_after_zoom_or_hidpi_change();
        self.set_needs_repaint(RepaintReason::Resize);
        true
    }

    /// If there are any animations running, dispatches appropriate messages to the constellation.
    fn process_animations(&mut self, force: bool) {
        // When running animations in order to dump a screenshot (not after a full composite), don't send
        // animation ticks faster than about 60Hz.
        //
        // TODO: This should be based on the refresh rate of the screen and also apply to all
        // animation ticks, not just ones sent while waiting to dump screenshots. This requires
        // something like a refresh driver concept though.
        if !force && (Instant::now() - self.last_animation_tick) < Duration::from_millis(16) {
            return;
        }
        self.last_animation_tick = Instant::now();

        #[cfg(feature = "webxr")]
        let webxr_running = self.global.borrow().webxr_main_thread.running();
        #[cfg(not(feature = "webxr"))]
        let webxr_running = false;

        let any_webviews_animating = !self
            .webviews
            .iter()
            .all(|webview| !webview.tick_all_animations(self));

        let animation_state = if !any_webviews_animating && !webxr_running {
            windowing::AnimationState::Idle
        } else {
            windowing::AnimationState::Animating
        };

        self.window.set_animation_state(animation_state);
    }

    fn hidpi_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        self.embedder_coordinates.hidpi_factor
    }

    pub(crate) fn device_pixels_per_page_pixel(&self) -> Scale<f32, CSSPixel, DevicePixel> {
        self.device_pixels_per_page_pixel_not_including_page_zoom() * self.pinch_zoom_level()
    }

    fn device_pixels_per_page_pixel_not_including_page_zoom(
        &self,
    ) -> Scale<f32, CSSPixel, DevicePixel> {
        self.page_zoom * self.hidpi_factor()
    }

    pub fn on_zoom_reset_window_event(&mut self) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }

        self.page_zoom = Scale::new(1.0);
        self.update_after_zoom_or_hidpi_change();
    }

    pub fn on_zoom_window_event(&mut self, magnification: f32) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }

        self.page_zoom =
            Scale::new((self.page_zoom.get() * magnification).clamp(MIN_ZOOM, MAX_ZOOM));
        self.update_after_zoom_or_hidpi_change();
    }

    fn update_after_zoom_or_hidpi_change(&mut self) {
        for (top_level_browsing_context_id, webview) in self.webviews.painting_order() {
            self.send_window_size_message_for_top_level_browser_context(
                webview.rect,
                *top_level_browsing_context_id,
            );
        }

        // Update the root transform in WebRender to reflect the new zoom.
        self.send_root_pipeline_display_list();
    }

    fn details_for_pipeline(&self, pipeline_id: PipelineId) -> Option<&PipelineDetails> {
        let webview_id = self
            .global
            .borrow()
            .pipeline_to_webview_map
            .get(&pipeline_id)
            .cloned()?;
        self.webviews.get(webview_id)?.pipelines.get(&pipeline_id)
    }

    // Check if any pipelines currently have active animations or animation callbacks.
    fn animations_or_animation_callbacks_running(&self) -> bool {
        self.webviews
            .iter()
            .any(WebView::animations_or_animation_callbacks_running)
    }

    /// Returns true if any animation callbacks (ie `requestAnimationFrame`) are waiting for a response.
    fn animation_callbacks_running(&self) -> bool {
        self.webviews
            .iter()
            .any(WebView::animation_callbacks_running)
    }

    /// Query the constellation to see if the current compositor
    /// output matches the current frame tree output, and if the
    /// associated script threads are idle.
    fn is_ready_to_paint_image_output(&mut self) -> Result<(), NotReadyToPaint> {
        match self.ready_to_save_state {
            ReadyState::Unknown => {
                // Unsure if the output image is stable.

                // Collect the currently painted epoch of each pipeline that is
                // complete (i.e. has *all* layers painted to the requested epoch).
                // This gets sent to the constellation for comparison with the current
                // frame tree.
                let mut pipeline_epochs = HashMap::new();
                for id in self.webviews.iter().flat_map(WebView::pipeline_ids) {
                    if let Some(WebRenderEpoch(epoch)) = self
                        .webrender
                        .as_ref()
                        .and_then(|wr| wr.current_epoch(self.webrender_document(), id.into()))
                    {
                        let epoch = Epoch(epoch);
                        pipeline_epochs.insert(*id, epoch);
                    }
                }

                // Pass the pipeline/epoch states to the constellation and check
                // if it's safe to output the image.
                let msg = ConstellationMsg::IsReadyToSaveImage(pipeline_epochs);
                if let Err(e) = self.global.borrow().constellation_sender.send(msg) {
                    warn!("Sending ready to save to constellation failed ({:?}).", e);
                }
                self.ready_to_save_state = ReadyState::WaitingForConstellationReply;
                Err(NotReadyToPaint::JustNotifiedConstellation)
            },
            ReadyState::WaitingForConstellationReply => {
                // If waiting on a reply from the constellation to the last
                // query if the image is stable, then assume not ready yet.
                Err(NotReadyToPaint::WaitingOnConstellation)
            },
            ReadyState::ReadyToSaveImage => {
                // Constellation has replied at some point in the past
                // that the current output image is stable and ready
                // for saving.
                // Reset the flag so that we check again in the future
                // TODO: only reset this if we load a new document?
                self.ready_to_save_state = ReadyState::Unknown;
                Ok(())
            },
        }
    }

    /// Render the WebRender scene to the active `RenderingContext`. If successful, trigger
    /// the next round of animations.
    pub fn render(&mut self) -> bool {
        if let Err(error) = self.render_inner() {
            warn!("Unable to render: {error:?}");
            return false;
        }

        // We've painted the default target, which means that from the embedder's perspective,
        // the scene no longer needs to be repainted.
        self.needs_repaint.set(RepaintReason::empty());

        // Queue up any subsequent paints for animations.
        self.process_animations(true);

        true
    }

    /// Render the WebRender scene to the shared memory, without updating other state of this
    /// [`IOCompositor`]. If succesful return the output image in shared memory.
    fn render_to_shared_memory(
        &mut self,
        page_rect: Option<Rect<f32, CSSPixel>>,
    ) -> Result<Option<Image>, UnableToComposite> {
        self.render_inner()?;

        let size = self.rendering_context.size2d().to_i32();
        let rect = if let Some(rect) = page_rect {
            let rect = self.device_pixels_per_page_pixel().transform_rect(&rect);

            let x = rect.origin.x as i32;
            // We need to convert to the bottom-left origin coordinate
            // system used by OpenGL
            let y = (size.height as f32 - rect.origin.y - rect.size.height) as i32;
            let w = rect.size.width as i32;
            let h = rect.size.height as i32;

            DeviceIntRect::from_origin_and_size(Point2D::new(x, y), Size2D::new(w, h))
        } else {
            DeviceIntRect::from_origin_and_size(Point2D::origin(), size)
        };

        Ok(self
            .rendering_context
            .read_to_image(rect)
            .map(|image| Image {
                width: image.width(),
                height: image.height(),
                format: PixelFormat::RGBA8,
                bytes: ipc::IpcSharedMemory::from_bytes(&image),
                id: None,
                cors_status: CorsStatus::Safe,
            }))
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn render_inner(&mut self) -> Result<(), UnableToComposite> {
        if let Err(err) = self.rendering_context.make_current() {
            warn!("Failed to make the rendering context current: {:?}", err);
        }
        self.assert_no_gl_error();

        if let Some(webrender) = self.webrender.as_mut() {
            webrender.update();
        }

        if opts::get().wait_for_stable_image {
            // The current image may be ready to output. However, if there are animations active,
            // tick those instead and continue waiting for the image output to be stable AND
            // all active animations to complete.
            if self.animations_or_animation_callbacks_running() {
                self.process_animations(false);
                return Err(UnableToComposite::NotReadyToPaintImage(
                    NotReadyToPaint::AnimationsActive,
                ));
            }
            if let Err(result) = self.is_ready_to_paint_image_output() {
                return Err(UnableToComposite::NotReadyToPaintImage(result));
            }
        }

        self.rendering_context.prepare_for_rendering();

        let time_profiler_chan = self.global.borrow().time_profiler_chan.clone();
        time_profile!(
            ProfilerCategory::Compositing,
            None,
            time_profiler_chan,
            || {
                trace!("Compositing");

                // Paint the scene.
                // TODO(gw): Take notice of any errors the renderer returns!
                self.clear_background();
                if let Some(webrender) = self.webrender.as_mut() {
                    let size = self.rendering_context.size2d().to_i32();
                    webrender.render(size, 0 /* buffer_age */).ok();
                }
            },
        );

        self.send_pending_paint_metrics_messages_after_composite();
        Ok(())
    }

    /// Send all pending paint metrics messages after a composite operation, which may advance
    /// the epoch for pipelines in the WebRender scene.
    ///
    /// If there are pending paint metrics, we check if any of the painted epochs is one
    /// of the ones that the paint metrics recorder is expecting. In that case, we get the
    /// current time, inform the constellation about it and remove the pending metric from
    /// the list.
    fn send_pending_paint_metrics_messages_after_composite(&mut self) {
        let paint_time = CrossProcessInstant::now();
        let document_id = self.webrender_document();
        for webview_details in self.webviews.iter_mut() {
            // For each pipeline, determine the current epoch and update paint timing if necessary.
            for (pipeline_id, pipeline) in webview_details.pipelines.iter_mut() {
                if pipeline.pending_paint_metrics.is_empty() {
                    continue;
                }
                let Some(composition_pipeline) = pipeline.pipeline.as_ref() else {
                    continue;
                };

                let Some(WebRenderEpoch(current_epoch)) = self
                    .webrender
                    .as_ref()
                    .and_then(|wr| wr.current_epoch(document_id, pipeline_id.into()))
                else {
                    continue;
                };

                let current_epoch = Epoch(current_epoch);
                let Some(index) = pipeline
                    .pending_paint_metrics
                    .iter()
                    .position(|epoch| *epoch == current_epoch)
                else {
                    continue;
                };

                // Remove all epochs that were pending before the current epochs. They were not and will not,
                // be painted.
                pipeline.pending_paint_metrics.drain(0..index);

                if let Err(error) =
                    composition_pipeline
                        .script_chan
                        .send(ScriptThreadMessage::SetEpochPaintTime(
                            *pipeline_id,
                            current_epoch,
                            paint_time,
                        ))
                {
                    warn!("Sending RequestLayoutPaintMetric message to layout failed ({error:?}).");
                }
            }
        }
    }

    fn clear_background(&self) {
        let gl = &self.global.borrow().webrender_gl;
        self.assert_gl_framebuffer_complete();

        // Always clear the entire RenderingContext, regardless of how many WebViews there are
        // or where they are positioned. This is so WebView actually clears even before the
        // first WebView is ready.
        let color = servo_config::pref!(shell_background_color_rgba);
        gl.clear_color(
            color[0] as f32,
            color[1] as f32,
            color[2] as f32,
            color[3] as f32,
        );
        gl.clear(gleam::gl::COLOR_BUFFER_BIT);
    }

    #[track_caller]
    fn assert_no_gl_error(&self) {
        debug_assert_eq!(
            self.global.borrow().webrender_gl.get_error(),
            gleam::gl::NO_ERROR
        );
    }

    #[track_caller]
    fn assert_gl_framebuffer_complete(&self) {
        debug_assert_eq!(
            (
                self.global.borrow().webrender_gl.get_error(),
                self.global
                    .borrow()
                    .webrender_gl
                    .check_frame_buffer_status(gleam::gl::FRAMEBUFFER)
            ),
            (gleam::gl::NO_ERROR, gleam::gl::FRAMEBUFFER_COMPLETE)
        );
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    pub fn receive_messages(&mut self) {
        // Check for new messages coming from the other threads in the system.
        let mut compositor_messages = vec![];
        let mut found_recomposite_msg = false;
        while let Some(msg) = self
            .global
            .borrow_mut()
            .compositor_receiver
            .try_recv_compositor_msg()
        {
            match msg {
                CompositorMsg::NewWebRenderFrameReady(..) if found_recomposite_msg => {
                    // Only take one of duplicate NewWebRendeFrameReady messages, but do subtract
                    // one frame from the pending frames.
                    self.pending_frames -= 1;
                },
                CompositorMsg::NewWebRenderFrameReady(..) => {
                    found_recomposite_msg = true;
                    compositor_messages.push(msg)
                },
                _ => compositor_messages.push(msg),
            }
        }
        for msg in compositor_messages {
            self.handle_browser_message(msg);

            if self.global.borrow().shutdown_state() == ShutdownState::FinishedShuttingDown {
                return;
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    pub fn perform_updates(&mut self) -> bool {
        if self.global.borrow().shutdown_state() == ShutdownState::FinishedShuttingDown {
            return false;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as f64;
        // If a pinch-zoom happened recently, ask for tiles at the new resolution
        if self.zoom_action && now - self.zoom_time > 0.3 {
            self.zoom_action = false;
        }

        #[cfg(feature = "webxr")]
        // Run the WebXR main thread
        self.global.borrow_mut().webxr_main_thread.run_one_frame();

        // The WebXR thread may make a different context current
        if let Err(err) = self.rendering_context.make_current() {
            warn!("Failed to make the rendering context current: {:?}", err);
        }
        let mut webviews = take(&mut self.webviews);
        for webview in webviews.iter_mut() {
            webview.process_pending_scroll_events(self);
        }
        self.webviews = webviews;
        self.global.borrow().shutdown_state() != ShutdownState::FinishedShuttingDown
    }

    pub fn pinch_zoom_level(&self) -> Scale<f32, DevicePixel, DevicePixel> {
        Scale::new(self.viewport_zoom.get())
    }

    pub(crate) fn set_pinch_zoom_level(&mut self, mut zoom: f32) -> bool {
        if let Some(min) = self.min_viewport_zoom {
            zoom = f32::max(min.get(), zoom);
        }
        if let Some(max) = self.max_viewport_zoom {
            zoom = f32::min(max.get(), zoom);
        }

        let old_zoom = std::mem::replace(&mut self.viewport_zoom, PinchZoomFactor::new(zoom));
        old_zoom != self.viewport_zoom
    }

    pub fn toggle_webrender_debug(&mut self, option: WebRenderDebugOption) {
        let Some(webrender) = self.webrender.as_mut() else {
            return;
        };
        let mut flags = webrender.get_debug_flags();
        let flag = match option {
            WebRenderDebugOption::Profiler => {
                webrender::DebugFlags::PROFILER_DBG |
                    webrender::DebugFlags::GPU_TIME_QUERIES |
                    webrender::DebugFlags::GPU_SAMPLE_QUERIES
            },
            WebRenderDebugOption::TextureCacheDebug => webrender::DebugFlags::TEXTURE_CACHE_DBG,
            WebRenderDebugOption::RenderTargetDebug => webrender::DebugFlags::RENDER_TARGET_DBG,
        };
        flags.toggle(flag);
        webrender.set_debug_flags(flags);

        let mut txn = Transaction::new();
        self.generate_frame(&mut txn, RenderReasons::TESTING);
        self.global.borrow_mut().send_transaction(txn);
    }

    pub fn capture_webrender(&mut self) {
        let capture_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();
        let available_path = [env::current_dir(), Ok(env::temp_dir())]
            .iter()
            .filter_map(|val| {
                val.as_ref()
                    .map(|dir| dir.join("webrender-captures").join(&capture_id))
                    .ok()
            })
            .find(|val| create_dir_all(val).is_ok());

        let Some(capture_path) = available_path else {
            eprintln!("Couldn't create a path for WebRender captures.");
            return;
        };

        println!("Saving WebRender capture to {capture_path:?}");
        self.global
            .borrow()
            .webrender_api
            .save_capture(capture_path.clone(), CaptureBits::all());

        let version_file_path = capture_path.join("servo-version.txt");
        if let Err(error) = File::create(version_file_path)
            .and_then(|mut file| write!(file, "{}", self.global.borrow().version_string))
        {
            eprintln!("Unable to write servo version for WebRender Capture: {error:?}");
        }
    }

    fn add_font_instance(
        &mut self,
        instance_key: FontInstanceKey,
        font_key: FontKey,
        size: f32,
        flags: FontInstanceFlags,
    ) {
        let mut transaction = Transaction::new();

        let font_instance_options = FontInstanceOptions {
            flags,
            ..Default::default()
        };
        transaction.add_font_instance(
            instance_key,
            font_key,
            size,
            Some(font_instance_options),
            None,
            Vec::new(),
        );

        self.global.borrow_mut().send_transaction(transaction);
    }

    fn add_font(&mut self, font_key: FontKey, index: u32, data: Arc<IpcSharedMemory>) {
        let mut transaction = Transaction::new();
        transaction.add_raw_font(font_key, (**data).into(), index);
        self.global.borrow_mut().send_transaction(transaction);
    }

    pub fn notify_input_event(&mut self, webview_id: WebViewId, event: InputEvent) {
        if let Some(webview) = self.webviews.get_mut(webview_id) {
            webview.notify_input_event(event);
        }
    }

    pub fn notify_scroll_event(
        &mut self,
        webview_id: WebViewId,
        scroll_location: ScrollLocation,
        cursor: DeviceIntPoint,
        event_type: TouchEventType,
    ) {
        if let Some(webview) = self.webviews.get_mut(webview_id) {
            webview.notify_scroll_event(scroll_location, cursor, event_type);
        }
    }

    pub fn on_vsync(&mut self, webview_id: WebViewId) {
        if let Some(webview) = self.webviews.get_mut(webview_id) {
            webview.on_vsync();
        }
    }

    pub fn set_pinch_zoom(&mut self, webview_id: WebViewId, magnification: f32) {
        if let Some(webview) = self.webviews.get_mut(webview_id) {
            webview.set_pinch_zoom(magnification);
        }
    }

    fn webrender_document(&self) -> DocumentId {
        self.global.borrow().webrender_document
    }

    fn shutdown_state(&self) -> ShutdownState {
        self.global.borrow().shutdown_state()
    }
}
