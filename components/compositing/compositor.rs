/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::HashMap;
use std::env;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::iter::once;
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
use embedder_traits::{
    Cursor, InputEvent, MouseButton, MouseButtonAction, MouseButtonEvent, MouseMoveEvent,
    TouchAction, TouchEvent, TouchEventType, TouchId,
};
use euclid::{Point2D, Rect, Scale, Size2D, Transform3D, Vector2D};
use fnv::{FnvHashMap, FnvHashSet};
use image::{DynamicImage, ImageFormat};
use ipc_channel::ipc::{self, IpcSharedMemory};
use libc::c_void;
use log::{debug, error, info, trace, warn};
use pixels::{CorsStatus, Image, PixelFormat};
use profile_traits::time::{self as profile_time, ProfilerCategory};
use profile_traits::time_profile;
use script_traits::{
    AnimationState, AnimationTickType, EventResult, ScriptThreadMessage, ScrollState,
    WindowSizeData, WindowSizeType,
};
use servo_geometry::DeviceIndependentPixel;
use style_traits::{CSSPixel, PinchZoomFactor};
use webrender::{CaptureBits, RenderApi, Transaction};
use webrender_api::units::{
    DeviceIntPoint, DeviceIntSize, DevicePixel, DevicePoint, DeviceRect, LayoutPoint, LayoutRect,
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

use crate::touch::TouchHandler;
use crate::webview::{UnknownWebView, WebView, WebViewAlreadyExists, WebViewManager};
use crate::windowing::{self, EmbedderCoordinates, WebRenderDebugOption, WindowMethods};
use crate::InitialCompositorState;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FrameTreeId(u32);

impl FrameTreeId {
    pub fn next(&mut self) {
        self.0 += 1;
    }
}

/// Data that is shared by all WebView renderers.
pub struct ServoRenderer {
    /// Our top-level browsing contexts.
    webviews: WebViewManager<WebView>,

    /// Tracks whether we are in the process of shutting down, or have shut down and should close
    /// the compositor.
    shutdown_state: ShutdownState,

    /// The port on which we receive messages.
    compositor_receiver: CompositorReceiver,

    /// The channel on which messages can be sent to the constellation.
    constellation_sender: Sender<ConstellationMsg>,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: profile_time::ProfilerChan,

    /// The WebRender [`RenderApi`] interface used to communicate with WebRender.
    webrender_api: RenderApi,

    /// The GL bindings for webrender
    webrender_gl: Rc<dyn gleam::gl::Gl>,

    /// True to exit after page load ('-x').
    exit_after_load: bool,

    /// The string representing the version of Servo that is running. This is used to tag
    /// WebRender capture output.
    version_string: String,

    #[cfg(feature = "webxr")]
    /// Some XR devices want to run on the main thread.
    webxr_main_thread: webxr::MainThreadRegistry,
}

/// NB: Never block on the constellation, because sometimes the constellation blocks on us.
pub struct IOCompositor {
    /// Data that is shared by all WebView renderers.
    global: ServoRenderer,

    /// The application window.
    pub window: Rc<dyn WindowMethods>,

    /// Tracks details about each active pipeline that the compositor knows about.
    pipeline_details: HashMap<PipelineId, PipelineDetails>,

    /// "Mobile-style" zoom that does not reflow the page.
    viewport_zoom: PinchZoomFactor,

    /// Viewport zoom constraints provided by @viewport.
    min_viewport_zoom: Option<PinchZoomFactor>,
    max_viewport_zoom: Option<PinchZoomFactor>,

    /// "Desktop-style" zoom that resizes the viewport to fit the window.
    page_zoom: Scale<f32, CSSPixel, DeviceIndependentPixel>,

    /// The type of composition to perform
    composite_target: CompositeTarget,

    /// Tracks whether or not the view needs to be repainted.
    needs_repaint: Cell<RepaintReason>,

    /// Tracks whether the zoom action has happened recently.
    zoom_action: bool,

    /// The time of the last zoom action has started.
    zoom_time: f64,

    /// The current frame tree ID (used to reject old paint buffers)
    frame_tree_id: FrameTreeId,

    /// Touch input state machine
    touch_handler: TouchHandler,

    /// Pending scroll/zoom events.
    pending_scroll_zoom_events: Vec<ScrollZoomEvent>,

    /// Used by the logic that determines when it is safe to output an
    /// image for the reftest framework.
    ready_to_save_state: ReadyState,

    /// The webrender renderer.
    webrender: Option<webrender::Renderer>,

    /// The active webrender document.
    webrender_document: DocumentId,

    /// The surfman instance that webrender targets
    rendering_context: Rc<dyn RenderingContext>,

    /// A per-pipeline queue of display lists that have not yet been rendered by WebRender. Layout
    /// expects WebRender to paint each given epoch. Once the compositor paints a frame with that
    /// epoch's display list, it will be removed from the queue and the paint time will be recorded
    /// as a metric. In case new display lists come faster than painting a metric might never be
    /// recorded.
    pending_paint_metrics: HashMap<PipelineId, Vec<Epoch>>,

    /// The coordinates of the native window, its view and the screen.
    embedder_coordinates: EmbedderCoordinates,

    /// Current mouse cursor.
    cursor: Cursor,

    /// Current cursor position.
    cursor_pos: DevicePoint,

    /// True to translate mouse input into touch events.
    convert_mouse_to_touch: bool,

    /// The number of frames pending to receive from WebRender.
    pending_frames: usize,

    /// The [`Instant`] of the last animation tick, used to avoid flooding the Constellation and
    /// ScriptThread with a deluge of animation ticks.
    last_animation_tick: Instant,
}

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShutdownState {
    NotShuttingDown,
    ShuttingDown,
    FinishedShuttingDown,
}

struct PipelineDetails {
    /// The pipeline associated with this PipelineDetails object.
    pipeline: Option<CompositionPipeline>,

    /// The id of the parent pipeline, if any.
    parent_pipeline_id: Option<PipelineId>,

    /// The epoch of the most recent display list for this pipeline. Note that this display
    /// list might not be displayed, as WebRender processes display lists asynchronously.
    most_recent_display_list_epoch: Option<WebRenderEpoch>,

    /// Whether animations are running
    animations_running: bool,

    /// Whether there are animation callbacks
    animation_callbacks_running: bool,

    /// Whether to use less resources by stopping animations.
    throttled: bool,

    /// Hit test items for this pipeline. This is used to map WebRender hit test
    /// information to the full information necessary for Servo.
    hit_test_items: Vec<HitTestInfo>,

    /// The compositor-side [ScrollTree]. This is used to allow finding and scrolling
    /// nodes in the compositor before forwarding new offsets to WebRender.
    scroll_tree: ScrollTree,
}

impl PipelineDetails {
    fn new() -> PipelineDetails {
        PipelineDetails {
            pipeline: None,
            parent_pipeline_id: None,
            most_recent_display_list_epoch: None,
            animations_running: false,
            animation_callbacks_running: false,
            throttled: false,
            hit_test_items: Vec::new(),
            scroll_tree: ScrollTree::default(),
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

#[derive(Clone, Debug, PartialEq)]
pub enum CompositeTarget {
    /// Draw to a OpenGL framebuffer object that will then be used by the compositor to composite
    /// to [`RenderingContext::framebuffer_object`]
    ContextFbo,

    /// Draw to an uncompressed image in shared memory.
    SharedMemory,

    /// Draw to a PNG file on disk, then exit the browser (for reftests).
    PngFile(Rc<String>),
}

impl IOCompositor {
    pub fn new(
        window: Rc<dyn WindowMethods>,
        state: InitialCompositorState,
        composite_target: CompositeTarget,
        exit_after_load: bool,
        convert_mouse_to_touch: bool,
        version_string: String,
    ) -> Self {
        let compositor = IOCompositor {
            global: ServoRenderer {
                shutdown_state: ShutdownState::NotShuttingDown,
                webviews: WebViewManager::default(),
                compositor_receiver: state.receiver,
                constellation_sender: state.constellation_chan,
                time_profiler_chan: state.time_profiler_chan,
                webrender_api: state.webrender_api,
                webrender_gl: state.webrender_gl,
                exit_after_load,
                version_string,
                #[cfg(feature = "webxr")]
                webxr_main_thread: state.webxr_main_thread,
            },
            embedder_coordinates: window.get_coordinates(),
            window,
            pipeline_details: HashMap::new(),
            needs_repaint: Cell::default(),
            touch_handler: TouchHandler::new(),
            pending_scroll_zoom_events: Vec::new(),
            composite_target,
            page_zoom: Scale::new(1.0),
            viewport_zoom: PinchZoomFactor::new(1.0),
            min_viewport_zoom: Some(PinchZoomFactor::new(1.0)),
            max_viewport_zoom: None,
            zoom_action: false,
            zoom_time: 0f64,
            frame_tree_id: FrameTreeId(0),
            ready_to_save_state: ReadyState::Unknown,
            webrender: Some(state.webrender),
            webrender_document: state.webrender_document,
            rendering_context: state.rendering_context,
            pending_paint_metrics: HashMap::new(),
            cursor: Cursor::None,
            cursor_pos: DevicePoint::new(0.0, 0.0),
            convert_mouse_to_touch,
            pending_frames: 0,
            last_animation_tick: Instant::now(),
        };

        let gl = &compositor.global.webrender_gl;
        info!("Running on {}", gl.get_string(gleam::gl::RENDERER));
        info!("OpenGL Version {}", gl.get_string(gleam::gl::VERSION));
        compositor.assert_gl_framebuffer_complete();
        compositor
    }

    pub fn shutdown_state(&self) -> ShutdownState {
        self.global.shutdown_state
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

    fn update_cursor(&mut self, result: &CompositorHitTestResult) {
        let cursor = match result.cursor {
            Some(cursor) if cursor != self.cursor => cursor,
            _ => return,
        };
        let Some(webview_id) = self
            .pipeline_details(result.pipeline_id)
            .pipeline
            .as_ref()
            .map(|composition_pipeline| composition_pipeline.top_level_browsing_context_id)
        else {
            warn!(
                "Updating cursor for not-yet-rendered pipeline: {}",
                result.pipeline_id
            );
            return;
        };

        self.cursor = cursor;
        let msg = ConstellationMsg::SetCursor(webview_id, cursor);
        if let Err(e) = self.global.constellation_sender.send(msg) {
            warn!("Sending event to constellation failed ({:?}).", e);
        }
    }

    pub fn start_shutting_down(&mut self) {
        if self.global.shutdown_state != ShutdownState::NotShuttingDown {
            warn!("Requested shutdown while already shutting down");
            return;
        }

        debug!("Compositor sending Exit message to Constellation");
        if let Err(e) = self
            .global
            .constellation_sender
            .send(ConstellationMsg::Exit)
        {
            warn!("Sending exit message to constellation failed ({:?}).", e);
        }

        self.global.shutdown_state = ShutdownState::ShuttingDown;
    }

    fn finish_shutting_down(&mut self) {
        debug!("Compositor received message that constellation shutdown is complete");

        // Drain compositor port, sometimes messages contain channels that are blocking
        // another thread from finishing (i.e. SetFrameTree).
        while self
            .global
            .compositor_receiver
            .try_recv_compositor_msg()
            .is_some()
        {}

        // Tell the profiler, memory profiler, and scrolling timer to shut down.
        if let Ok((sender, receiver)) = ipc::channel() {
            self.global
                .time_profiler_chan
                .send(profile_time::ProfilerMsg::Exit(sender));
            let _ = receiver.recv();
        }

        self.global.shutdown_state = ShutdownState::FinishedShuttingDown;
    }

    fn handle_browser_message(&mut self, msg: CompositorMsg) {
        trace_msg_from_constellation!(msg, "{msg:?}");

        match self.global.shutdown_state {
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
            CompositorMsg::ShutdownComplete => {
                error!("Received `ShutdownComplete` while not shutting down.");
                self.finish_shutting_down();
            },

            CompositorMsg::ChangeRunningAnimationsState(pipeline_id, animation_state) => {
                self.change_running_animations_state(pipeline_id, animation_state);
            },

            CompositorMsg::CreateOrUpdateWebView(frame_tree) => {
                self.set_frame_tree_for_webview(&frame_tree);
                self.send_scroll_positions_to_layout_for_pipeline(&frame_tree.pipeline.id);
            },

            CompositorMsg::RemoveWebView(top_level_browsing_context_id) => {
                self.remove_webview(top_level_browsing_context_id);
            },

            CompositorMsg::TouchEventProcessed(result) => {
                self.on_touch_event_processed(result);
            },

            CompositorMsg::CreatePng(page_rect, reply) => {
                let res = self.composite_specific_target(CompositeTarget::SharedMemory, page_rect);
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

            CompositorMsg::SetThrottled(pipeline_id, throttled) => {
                self.pipeline_details(pipeline_id).throttled = throttled;
                self.process_animations(true);
            },

            CompositorMsg::PipelineExited(pipeline_id, sender) => {
                debug!("Compositor got pipeline exited: {:?}", pipeline_id);
                self.remove_pipeline_root_layer(pipeline_id);
                let _ = sender.send(());
            },

            CompositorMsg::NewWebRenderFrameReady(_document_id, recomposite_needed) => {
                self.pending_frames -= 1;

                if recomposite_needed {
                    if let Some(result) = self.hit_test_at_point(self.cursor_pos) {
                        self.update_cursor(&result);
                    }
                }

                if recomposite_needed || self.animation_callbacks_active() {
                    self.set_needs_repaint(RepaintReason::NewWebRenderFrame);
                }
            },

            CompositorMsg::LoadComplete(_) => {
                // If we're painting in headless mode, schedule a recomposite.
                if matches!(self.composite_target, CompositeTarget::PngFile(_)) ||
                    self.global.exit_after_load
                {
                    self.set_needs_repaint(RepaintReason::ReadyForScreenshot);
                }
            },

            CompositorMsg::WebDriverMouseButtonEvent(action, button, x, y) => {
                let dppx = self.device_pixels_per_page_pixel();
                let point = dppx.transform_point(Point2D::new(x, y));
                self.dispatch_input_event(InputEvent::MouseButton(MouseButtonEvent {
                    point,
                    action,
                    button,
                }));
            },

            CompositorMsg::WebDriverMouseMoveEvent(x, y) => {
                let dppx = self.device_pixels_per_page_pixel();
                let point = dppx.transform_point(Point2D::new(x, y));
                self.dispatch_input_event(InputEvent::MouseMove(MouseMoveEvent { point }));
            },

            CompositorMsg::PendingPaintMetric(pipeline_id, epoch) => {
                self.pending_paint_metrics
                    .entry(pipeline_id)
                    .or_default()
                    .push(epoch);
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
                self.global
                    .webrender_api
                    .send_transaction(self.webrender_document, txn);
            },

            CrossProcessCompositorMessage::SendScrollNode(
                pipeline_id,
                point,
                external_scroll_id,
            ) => {
                let pipeline_id = pipeline_id.into();
                let pipeline_details = match self.pipeline_details.get_mut(&pipeline_id) {
                    Some(details) => details,
                    None => return,
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
                self.global
                    .webrender_api
                    .send_transaction(self.webrender_document, txn);
            },

            CrossProcessCompositorMessage::SendDisplayList {
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
                        )
                    },
                };
                let cache_data = match display_list_receiver.recv() {
                    Ok(display_list_data) => display_list_data,
                    Err(error) => {
                        return warn!(
                            "Could not receive WebRender display list cache data: {error}"
                        )
                    },
                };
                let spatial_tree = match display_list_receiver.recv() {
                    Ok(display_list_data) => display_list_data,
                    Err(error) => {
                        return warn!(
                            "Could not receive WebRender display list spatial tree: {error}."
                        )
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
                let pipeline_id = display_list_info.pipeline_id;
                let details = self.pipeline_details(pipeline_id.into());
                details.most_recent_display_list_epoch = Some(display_list_info.epoch);
                details.hit_test_items = display_list_info.hit_test_info;
                details.install_new_scroll_tree(display_list_info.scroll_tree);

                let mut transaction = Transaction::new();
                transaction
                    .set_display_list(display_list_info.epoch, (pipeline_id, built_display_list));
                self.update_transaction_with_all_scroll_offsets(&mut transaction);
                self.generate_frame(&mut transaction, RenderReasons::SCENE);
                self.global
                    .webrender_api
                    .send_transaction(self.webrender_document, transaction);
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
                self.global.webrender_api.flush_scene_builder();

                let result = self.hit_test_at_point_with_flags_and_pipeline(point, flags, pipeline);
                let _ = sender.send(result);
            },

            CrossProcessCompositorMessage::GenerateImageKey(sender) => {
                let _ = sender.send(self.global.webrender_api.generate_image_key());
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
                self.global
                    .webrender_api
                    .send_transaction(self.webrender_document, txn);
            },

            CrossProcessCompositorMessage::AddFont(font_key, data, index) => {
                self.add_font(font_key, index, data);
            },

            CrossProcessCompositorMessage::AddSystemFont(font_key, native_handle) => {
                let mut transaction = Transaction::new();
                transaction.add_native_font(font_key, native_handle);
                self.global
                    .webrender_api
                    .send_transaction(self.webrender_document, transaction);
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

                self.global
                    .webrender_api
                    .send_transaction(self.webrender_document, transaction);
            },

            CrossProcessCompositorMessage::AddImage(key, desc, data) => {
                let mut txn = Transaction::new();
                txn.add_image(key, desc, data.into(), None);
                self.global
                    .webrender_api
                    .send_transaction(self.webrender_document, txn);
            },

            CrossProcessCompositorMessage::GenerateFontKeys(
                number_of_font_keys,
                number_of_font_instance_keys,
                result_sender,
            ) => {
                let font_keys = (0..number_of_font_keys)
                    .map(|_| self.global.webrender_api.generate_font_key())
                    .collect();
                let font_instance_keys = (0..number_of_font_instance_keys)
                    .map(|_| self.global.webrender_api.generate_font_instance_key())
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
            CompositorMsg::ShutdownComplete => {
                self.finish_shutting_down();
            },
            CompositorMsg::PipelineExited(pipeline_id, sender) => {
                debug!("Compositor got pipeline exited: {:?}", pipeline_id);
                self.remove_pipeline_root_layer(pipeline_id);
                let _ = sender.send(());
            },
            CompositorMsg::CrossProcess(CrossProcessCompositorMessage::GenerateImageKey(
                sender,
            )) => {
                let _ = sender.send(self.global.webrender_api.generate_image_key());
            },
            CompositorMsg::CrossProcess(CrossProcessCompositorMessage::GenerateFontKeys(
                number_of_font_keys,
                number_of_font_instance_keys,
                result_sender,
            )) => {
                let font_keys = (0..number_of_font_keys)
                    .map(|_| self.global.webrender_api.generate_font_key())
                    .collect();
                let font_instance_keys = (0..number_of_font_instance_keys)
                    .map(|_| self.global.webrender_api.generate_font_instance_key())
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
    fn generate_frame(&mut self, transaction: &mut Transaction, reason: RenderReasons) {
        self.pending_frames += 1;
        transaction.generate_frame(0, true /* present */, reason);
    }

    /// Sets or unsets the animations-running flag for the given pipeline, and schedules a
    /// recomposite if necessary.
    fn change_running_animations_state(
        &mut self,
        pipeline_id: PipelineId,
        animation_state: AnimationState,
    ) {
        match animation_state {
            AnimationState::AnimationsPresent => {
                let throttled = self.pipeline_details(pipeline_id).throttled;
                self.pipeline_details(pipeline_id).animations_running = true;
                if !throttled {
                    self.set_needs_repaint(RepaintReason::ChangedAnimationState);
                }
            },
            AnimationState::AnimationCallbacksPresent => {
                let throttled = self.pipeline_details(pipeline_id).throttled;
                self.pipeline_details(pipeline_id)
                    .animation_callbacks_running = true;
                if !throttled {
                    self.tick_animations_for_pipeline(pipeline_id);
                }
            },
            AnimationState::NoAnimationsPresent => {
                self.pipeline_details(pipeline_id).animations_running = false;
            },
            AnimationState::NoAnimationCallbacksPresent => {
                self.pipeline_details(pipeline_id)
                    .animation_callbacks_running = false;
            },
        }
    }

    fn pipeline_details(&mut self, pipeline_id: PipelineId) -> &mut PipelineDetails {
        self.pipeline_details
            .entry(pipeline_id)
            .or_insert_with(PipelineDetails::new);
        self.pipeline_details
            .get_mut(&pipeline_id)
            .expect("Insert then get failed!")
    }

    pub fn pipeline(&self, pipeline_id: PipelineId) -> Option<&CompositionPipeline> {
        match self.pipeline_details.get(&pipeline_id) {
            Some(details) => details.pipeline.as_ref(),
            None => {
                warn!(
                    "Compositor layer has an unknown pipeline ({:?}).",
                    pipeline_id
                );
                None
            },
        }
    }

    /// Set the root pipeline for our WebRender scene to a display list that consists of an iframe
    /// for each visible top-level browsing context, applying a transformation on the root for
    /// pinch zoom, page zoom, and HiDPI scaling.
    fn send_root_pipeline_display_list(&mut self) {
        let mut transaction = Transaction::new();
        self.send_root_pipeline_display_list_in_transaction(&mut transaction);
        self.generate_frame(&mut transaction, RenderReasons::SCENE);
        self.global
            .webrender_api
            .send_transaction(self.webrender_document, transaction);
    }

    /// Set the root pipeline for our WebRender scene to a display list that consists of an iframe
    /// for each visible top-level browsing context, applying a transformation on the root for
    /// pinch zoom, page zoom, and HiDPI scaling.
    fn send_root_pipeline_display_list_in_transaction(&self, transaction: &mut Transaction) {
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
            self.embedder_coordinates.get_viewport().size().to_f32() / zoom_factor;
        let scaled_viewport_size = LayoutSize::from_untyped(scaled_viewport_size.to_untyped());
        let scaled_viewport_rect =
            LayoutRect::from_origin_and_size(LayoutPoint::zero(), scaled_viewport_size);

        let root_clip_id = builder.define_clip_rect(zoom_reference_frame, scaled_viewport_rect);
        let clip_chain_id = builder.define_clip_chain(None, [root_clip_id]);
        for (_, webview) in self.global.webviews.painting_order() {
            if let Some(pipeline_id) = webview.pipeline_id {
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
        for details in self.pipeline_details.values() {
            for node in details.scroll_tree.nodes.iter() {
                let (Some(offset), Some(external_id)) = (node.offset(), node.external_id()) else {
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

    fn set_frame_tree_for_webview(&mut self, frame_tree: &SendableFrameTree) {
        debug!("{}: Setting frame tree for webview", frame_tree.pipeline.id);

        let top_level_browsing_context_id = frame_tree.pipeline.top_level_browsing_context_id;
        if let Some(webview) = self.global.webviews.get_mut(top_level_browsing_context_id) {
            let new_pipeline_id = Some(frame_tree.pipeline.id);
            if new_pipeline_id != webview.pipeline_id {
                debug!(
                    "{:?}: Updating webview from pipeline {:?} to {:?}",
                    top_level_browsing_context_id, webview.pipeline_id, new_pipeline_id
                );
            }
            webview.pipeline_id = new_pipeline_id;
        } else {
            let top_level_browsing_context_id = frame_tree.pipeline.top_level_browsing_context_id;
            let pipeline_id = Some(frame_tree.pipeline.id);
            debug!(
                "{:?}: Creating new webview with pipeline {:?}",
                top_level_browsing_context_id, pipeline_id
            );
            if let Err(WebViewAlreadyExists(webview_id)) = self.global.webviews.add(
                top_level_browsing_context_id,
                WebView {
                    pipeline_id,
                    rect: self.embedder_coordinates.get_viewport().to_f32(),
                },
            ) {
                error!("{webview_id}: Creating webview that already exists");
                return;
            }
            let msg = ConstellationMsg::WebViewOpened(top_level_browsing_context_id);
            if let Err(e) = self.global.constellation_sender.send(msg) {
                warn!("Sending event to constellation failed ({:?}).", e);
            }
        }

        self.send_root_pipeline_display_list();
        self.create_or_update_pipeline_details_with_frame_tree(frame_tree, None);
        self.reset_scroll_tree_for_unattached_pipelines(frame_tree);

        self.frame_tree_id.next();
    }

    fn remove_webview(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        debug!("{}: Removing", top_level_browsing_context_id);
        let Ok(webview) = self.global.webviews.remove(top_level_browsing_context_id) else {
            warn!("{top_level_browsing_context_id}: Removing unknown webview");
            return;
        };

        self.send_root_pipeline_display_list();
        if let Some(pipeline_id) = webview.pipeline_id {
            self.remove_pipeline_details_recursively(pipeline_id);
        }

        self.frame_tree_id.next();
    }

    pub fn move_resize_webview(&mut self, webview_id: TopLevelBrowsingContextId, rect: DeviceRect) {
        debug!("{webview_id}: Moving and/or resizing webview; rect={rect:?}");
        let rect_changed;
        let size_changed;
        match self.global.webviews.get_mut(webview_id) {
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
                .global
                .webviews
                .painting_order()
                .map(|(&id, _)| id)
                .ne(once(webview_id));
            self.global.webviews.hide_all();
            self.global.webviews.show(webview_id)?;
            result
        } else {
            self.global.webviews.show(webview_id)?
        };
        if painting_order_changed {
            self.send_root_pipeline_display_list();
        }
        Ok(())
    }

    pub fn hide_webview(&mut self, webview_id: WebViewId) -> Result<(), UnknownWebView> {
        debug!("{webview_id}: Hiding webview");
        if self.global.webviews.hide(webview_id)? {
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
                .global
                .webviews
                .painting_order()
                .map(|(&id, _)| id)
                .ne(once(webview_id));
            self.global.webviews.hide_all();
            self.global.webviews.raise_to_top(webview_id)?;
            result
        } else {
            self.global.webviews.raise_to_top(webview_id)?
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
        if let Err(e) = self.global.constellation_sender.send(msg) {
            warn!("Sending window resize to constellation failed ({:?}).", e);
        }
    }

    fn reset_scroll_tree_for_unattached_pipelines(&mut self, frame_tree: &SendableFrameTree) {
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

        self.pipeline_details
            .iter_mut()
            .filter(|(id, _)| !attached_pipelines.contains(id))
            .for_each(|(_, details)| {
                details.scroll_tree.nodes.iter_mut().for_each(|node| {
                    node.set_offset(LayoutVector2D::zero());
                })
            })
    }

    fn create_or_update_pipeline_details_with_frame_tree(
        &mut self,
        frame_tree: &SendableFrameTree,
        parent_pipeline_id: Option<PipelineId>,
    ) {
        let pipeline_id = frame_tree.pipeline.id;
        let pipeline_details = self.pipeline_details(pipeline_id);
        pipeline_details.pipeline = Some(frame_tree.pipeline.clone());
        pipeline_details.parent_pipeline_id = parent_pipeline_id;

        for kid in &frame_tree.children {
            self.create_or_update_pipeline_details_with_frame_tree(kid, Some(pipeline_id));
        }
    }

    fn remove_pipeline_details_recursively(&mut self, pipeline_id: PipelineId) {
        self.pipeline_details.remove(&pipeline_id);

        let children = self
            .pipeline_details
            .iter()
            .filter(|(_, pipeline_details)| {
                pipeline_details.parent_pipeline_id == Some(pipeline_id)
            })
            .map(|(&pipeline_id, _)| pipeline_id)
            .collect::<Vec<_>>();

        for kid in children {
            self.remove_pipeline_details_recursively(kid);
        }
    }

    fn remove_pipeline_root_layer(&mut self, pipeline_id: PipelineId) {
        self.pipeline_details.remove(&pipeline_id);
    }

    pub fn on_embedder_window_moved(&mut self) {
        self.embedder_coordinates = self.window.get_coordinates();
    }

    pub fn on_rendering_context_resized(&mut self) -> bool {
        if self.global.shutdown_state != ShutdownState::NotShuttingDown {
            return false;
        }

        let old_coords = self.embedder_coordinates;
        self.embedder_coordinates = self.window.get_coordinates();

        if self.embedder_coordinates.viewport != old_coords.viewport {
            let mut transaction = Transaction::new();
            let size = self.embedder_coordinates.get_viewport();
            transaction.set_document_view(size);
            self.rendering_context.resize(size.size().to_untyped());
            self.global
                .webrender_api
                .send_transaction(self.webrender_document, transaction);
        }

        // A size change could also mean a resolution change.
        if self.embedder_coordinates.hidpi_factor == old_coords.hidpi_factor &&
            self.embedder_coordinates.viewport == old_coords.viewport
        {
            return false;
        }

        self.update_after_zoom_or_hidpi_change();
        self.set_needs_repaint(RepaintReason::Resize);
        true
    }

    fn dispatch_input_event(&mut self, event: InputEvent) {
        // Events that do not need to do hit testing are sent directly to the
        // constellation to filter down.
        let Some(point) = event.point() else {
            return;
        };

        // If we can't find a pipeline to send this event to, we cannot continue.
        let Some(result) = self.hit_test_at_point(point) else {
            return;
        };

        self.update_cursor(&result);

        if let Err(error) = self
            .global
            .constellation_sender
            .send(ConstellationMsg::ForwardInputEvent(event, Some(result)))
        {
            warn!("Sending event to constellation failed ({error:?}).");
        }
    }

    pub fn on_input_event(&mut self, event: InputEvent) {
        if self.global.shutdown_state != ShutdownState::NotShuttingDown {
            return;
        }

        if let InputEvent::Touch(event) = event {
            self.on_touch_event(event);
            return;
        }

        if self.convert_mouse_to_touch {
            match event {
                InputEvent::MouseButton(event) => {
                    match event.action {
                        MouseButtonAction::Click => {},
                        MouseButtonAction::Down => self.on_touch_down(TouchEvent {
                            event_type: TouchEventType::Down,
                            id: TouchId(0),
                            point: event.point,
                            action: TouchAction::NoAction,
                        }),
                        MouseButtonAction::Up => self.on_touch_up(TouchEvent {
                            event_type: TouchEventType::Up,
                            id: TouchId(0),
                            point: event.point,
                            action: TouchAction::NoAction,
                        }),
                    }
                    return;
                },
                InputEvent::MouseMove(event) => {
                    self.on_touch_move(TouchEvent {
                        event_type: TouchEventType::Move,
                        id: TouchId(0),
                        point: event.point,
                        action: TouchAction::NoAction,
                    });
                    return;
                },
                _ => {},
            }
        }

        self.dispatch_input_event(event);
    }

    fn hit_test_at_point(&self, point: DevicePoint) -> Option<CompositorHitTestResult> {
        self.hit_test_at_point_with_flags_and_pipeline(point, HitTestFlags::empty(), None)
            .first()
            .cloned()
    }

    fn hit_test_at_point_with_flags_and_pipeline(
        &self,
        point: DevicePoint,
        flags: HitTestFlags,
        pipeline_id: Option<WebRenderPipelineId>,
    ) -> Vec<CompositorHitTestResult> {
        // DevicePoint and WorldPoint are the same for us.
        let world_point = WorldPoint::from_untyped(point.to_untyped());
        let results = self.global.webrender_api.hit_test(
            self.webrender_document,
            pipeline_id,
            world_point,
            flags,
        );

        results
            .items
            .iter()
            .filter_map(|item| {
                let pipeline_id = item.pipeline.into();
                let details = match self.pipeline_details.get(&pipeline_id) {
                    Some(details) => details,
                    None => return None,
                };

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

    fn send_touch_event(&self, event: TouchEvent) {
        let Some(result) = self.hit_test_at_point(event.point) else {
            return;
        };

        let event = InputEvent::Touch(event);
        if let Err(e) = self
            .global
            .constellation_sender
            .send(ConstellationMsg::ForwardInputEvent(event, Some(result)))
        {
            warn!("Sending event to constellation failed ({:?}).", e);
        }
    }

    pub fn on_touch_event(&mut self, event: TouchEvent) {
        if self.global.shutdown_state != ShutdownState::NotShuttingDown {
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
        let action: TouchAction = self.touch_handler.on_touch_move(event.id, event.point);
        if TouchAction::NoAction != action {
            if !self.touch_handler.prevent_move {
                match action {
                    TouchAction::Scroll(delta, point) => self.on_scroll_window_event(
                        ScrollLocation::Delta(LayoutVector2D::from_untyped(delta.to_untyped())),
                        point.cast(),
                    ),
                    TouchAction::Zoom(magnification, scroll_delta) => {
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
            } else {
                event.action = action;
            }
            self.send_touch_event(event);
        }
    }

    fn on_touch_up(&mut self, mut event: TouchEvent) {
        let action = self.touch_handler.on_touch_up(event.id, event.point);
        event.action = action;
        self.send_touch_event(event);
    }

    fn on_touch_cancel(&mut self, event: TouchEvent) {
        // Send the event to script.
        self.touch_handler.on_touch_cancel(event.id, event.point);
        self.send_touch_event(event)
    }

    fn on_touch_event_processed(&mut self, result: EventResult) {
        match result {
            EventResult::DefaultPrevented(event_type) => {
                match event_type {
                    TouchEventType::Down | TouchEventType::Move => {
                        self.touch_handler.prevent_move = true;
                    },
                    _ => {},
                }
                self.touch_handler.prevent_click = true;
            },
            EventResult::DefaultAllowed(action) => {
                self.touch_handler.prevent_move = false;
                match action {
                    TouchAction::Click(point) => {
                        if !self.touch_handler.prevent_click {
                            self.simulate_mouse_click(point);
                        }
                    },
                    TouchAction::Flinging(velocity, point) => {
                        self.touch_handler.on_fling(velocity, point);
                    },
                    TouchAction::Scroll(delta, point) => self.on_scroll_window_event(
                        ScrollLocation::Delta(LayoutVector2D::from_untyped(delta.to_untyped())),
                        point.cast(),
                    ),
                    TouchAction::Zoom(magnification, scroll_delta) => {
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

    pub fn on_scroll_event(
        &mut self,
        scroll_location: ScrollLocation,
        cursor: DeviceIntPoint,
        event_type: TouchEventType,
    ) {
        if self.global.shutdown_state != ShutdownState::NotShuttingDown {
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

    fn process_pending_scroll_events(&mut self) {
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
            self.send_root_pipeline_display_list_in_transaction(&mut transaction);
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
            self.send_scroll_positions_to_layout_for_pipeline(&pipeline_id);
        }

        self.generate_frame(&mut transaction, RenderReasons::APZ);
        self.global
            .webrender_api
            .send_transaction(self.webrender_document, transaction);
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

        let hit_test_results =
            self.hit_test_at_point_with_flags_and_pipeline(cursor, HitTestFlags::FIND_ALL, None);

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
            if previous_pipeline_id.replace(pipeline_id) != Some(pipeline_id) {
                let scroll_result = self
                    .pipeline_details
                    .get_mut(pipeline_id)?
                    .scroll_tree
                    .scroll_node_or_ancestor(scroll_tree_node, scroll_location);
                if let Some((external_id, offset)) = scroll_result {
                    return Some((*pipeline_id, external_id, offset));
                }
            }
        }
        None
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

        let mut pipeline_ids = vec![];
        for (pipeline_id, pipeline_details) in &self.pipeline_details {
            if (pipeline_details.animations_running || pipeline_details.animation_callbacks_running) &&
                !pipeline_details.throttled
            {
                pipeline_ids.push(*pipeline_id);
            }
        }
        #[cfg(feature = "webxr")]
        let webxr_running = self.global.webxr_main_thread.running();
        #[cfg(not(feature = "webxr"))]
        let webxr_running = false;
        let animation_state = if pipeline_ids.is_empty() && !webxr_running {
            windowing::AnimationState::Idle
        } else {
            windowing::AnimationState::Animating
        };
        self.window.set_animation_state(animation_state);
        for pipeline_id in &pipeline_ids {
            self.tick_animations_for_pipeline(*pipeline_id)
        }
    }

    fn tick_animations_for_pipeline(&mut self, pipeline_id: PipelineId) {
        let animation_callbacks_running = self
            .pipeline_details(pipeline_id)
            .animation_callbacks_running;
        let animations_running = self.pipeline_details(pipeline_id).animations_running;
        if !animation_callbacks_running && !animations_running {
            return;
        }

        let mut tick_type = AnimationTickType::empty();
        if animations_running {
            tick_type.insert(AnimationTickType::CSS_ANIMATIONS_AND_TRANSITIONS);
        }
        if animation_callbacks_running {
            tick_type.insert(AnimationTickType::REQUEST_ANIMATION_FRAME);
        }

        let msg = ConstellationMsg::TickAnimation(pipeline_id, tick_type);
        if let Err(e) = self.global.constellation_sender.send(msg) {
            warn!("Sending tick to constellation failed ({:?}).", e);
        }
    }

    fn hidpi_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        if matches!(self.composite_target, CompositeTarget::PngFile(_)) {
            return Scale::new(1.0);
        }
        self.embedder_coordinates.hidpi_factor
    }

    fn device_pixels_per_page_pixel(&self) -> Scale<f32, CSSPixel, DevicePixel> {
        self.device_pixels_per_page_pixel_not_including_page_zoom() * self.pinch_zoom_level()
    }

    fn device_pixels_per_page_pixel_not_including_page_zoom(
        &self,
    ) -> Scale<f32, CSSPixel, DevicePixel> {
        self.page_zoom * self.hidpi_factor()
    }

    pub fn on_zoom_reset_window_event(&mut self) {
        if self.global.shutdown_state != ShutdownState::NotShuttingDown {
            return;
        }

        self.page_zoom = Scale::new(1.0);
        self.update_after_zoom_or_hidpi_change();
    }

    pub fn on_zoom_window_event(&mut self, magnification: f32) {
        if self.global.shutdown_state != ShutdownState::NotShuttingDown {
            return;
        }

        self.page_zoom =
            Scale::new((self.page_zoom.get() * magnification).clamp(MIN_ZOOM, MAX_ZOOM));
        self.update_after_zoom_or_hidpi_change();
    }

    fn update_after_zoom_or_hidpi_change(&mut self) {
        for (top_level_browsing_context_id, webview) in self.global.webviews.painting_order() {
            self.send_window_size_message_for_top_level_browser_context(
                webview.rect,
                *top_level_browsing_context_id,
            );
        }

        // Update the root transform in WebRender to reflect the new zoom.
        self.send_root_pipeline_display_list();
    }

    /// Simulate a pinch zoom
    pub fn on_pinch_zoom_window_event(&mut self, magnification: f32) {
        if self.global.shutdown_state != ShutdownState::NotShuttingDown {
            return;
        }

        // TODO: Scroll to keep the center in view?
        self.pending_scroll_zoom_events
            .push(ScrollZoomEvent::PinchZoom(magnification));
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

    fn send_scroll_positions_to_layout_for_pipeline(&self, pipeline_id: &PipelineId) {
        let details = match self.pipeline_details.get(pipeline_id) {
            Some(details) => details,
            None => return,
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
            let message = ScriptThreadMessage::SetScrollStates(*pipeline_id, scroll_states);
            let _ = pipeline.script_chan.send(message);
        }
    }

    // Check if any pipelines currently have active animations or animation callbacks.
    fn animations_active(&self) -> bool {
        for details in self.pipeline_details.values() {
            // If animations are currently running, then don't bother checking
            // with the constellation if the output image is stable.
            if details.animations_running {
                return true;
            }
            if details.animation_callbacks_running {
                return true;
            }
        }

        false
    }

    /// Returns true if any animation callbacks (ie `requestAnimationFrame`) are waiting for a response.
    fn animation_callbacks_active(&self) -> bool {
        self.pipeline_details
            .values()
            .any(|details| details.animation_callbacks_running)
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
                for id in self.pipeline_details.keys() {
                    if let Some(WebRenderEpoch(epoch)) = self
                        .webrender
                        .as_ref()
                        .and_then(|wr| wr.current_epoch(self.webrender_document, id.into()))
                    {
                        let epoch = Epoch(epoch);
                        pipeline_epochs.insert(*id, epoch);
                    }
                }

                // Pass the pipeline/epoch states to the constellation and check
                // if it's safe to output the image.
                let msg = ConstellationMsg::IsReadyToSaveImage(pipeline_epochs);
                if let Err(e) = self.global.constellation_sender.send(msg) {
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

    pub fn composite(&mut self) {
        if let Err(error) = self.composite_specific_target(self.composite_target.clone(), None) {
            warn!("Unable to composite: {error:?}");
            return;
        }

        // We've painted the default target, which means that from the embedder's perspective,
        // the scene no longer needs to be repainted.
        self.needs_repaint.set(RepaintReason::empty());

        // Queue up any subsequent paints for animations.
        self.process_animations(true);

        if matches!(self.composite_target, CompositeTarget::PngFile(_)) ||
            self.global.exit_after_load
        {
            println!("Shutting down the Constellation after generating an output file or exit flag specified");
            self.start_shutting_down();
        }
    }

    /// Composite to the given target if any, or the current target otherwise.
    /// Returns Ok if composition was performed or Err if it was not possible to composite for some
    /// reason. When the target is [CompositeTarget::SharedMemory], the image is read back from the
    /// GPU and returned as Ok(Some(png::Image)), otherwise we return Ok(None).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn composite_specific_target(
        &mut self,
        target: CompositeTarget,
        page_rect: Option<Rect<f32, CSSPixel>>,
    ) -> Result<Option<Image>, UnableToComposite> {
        let size = self.embedder_coordinates.framebuffer.to_u32();
        if let Err(err) = self.rendering_context.make_current() {
            warn!("Failed to make the rendering context current: {:?}", err);
        }
        self.assert_no_gl_error();

        if let Some(webrender) = self.webrender.as_mut() {
            webrender.update();
        }

        let wait_for_stable_image = matches!(
            target,
            CompositeTarget::SharedMemory | CompositeTarget::PngFile(_)
        ) || self.global.exit_after_load;

        if wait_for_stable_image {
            // The current image may be ready to output. However, if there are animations active,
            // tick those instead and continue waiting for the image output to be stable AND
            // all active animations to complete.
            if self.animations_active() {
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

        time_profile!(
            ProfilerCategory::Compositing,
            None,
            self.global.time_profiler_chan.clone(),
            || {
                trace!("Compositing");

                let size =
                    DeviceIntSize::from_untyped(self.embedder_coordinates.framebuffer.to_untyped());

                // Paint the scene.
                // TODO(gw): Take notice of any errors the renderer returns!
                self.clear_background();
                if let Some(webrender) = self.webrender.as_mut() {
                    webrender.render(size, 0 /* buffer_age */).ok();
                }
            },
        );

        self.send_pending_paint_metrics_messages_after_composite();

        let (x, y, width, height) = if let Some(rect) = page_rect {
            let rect = self.device_pixels_per_page_pixel().transform_rect(&rect);

            let x = rect.origin.x as i32;
            // We need to convert to the bottom-left origin coordinate
            // system used by OpenGL
            let y = (size.height as f32 - rect.origin.y - rect.size.height) as i32;
            let w = rect.size.width as u32;
            let h = rect.size.height as u32;

            (x, y, w, h)
        } else {
            (0, 0, size.width, size.height)
        };

        let rv = match target {
            CompositeTarget::ContextFbo => None,
            CompositeTarget::SharedMemory => self
                .rendering_context
                .read_to_image(Rect::new(
                    Point2D::new(x as u32, y as u32),
                    Size2D::new(width, height),
                ))
                .map(|image| Image {
                    width: image.width(),
                    height: image.height(),
                    format: PixelFormat::RGBA8,
                    bytes: ipc::IpcSharedMemory::from_bytes(&image),
                    id: None,
                    cors_status: CorsStatus::Safe,
                }),
            CompositeTarget::PngFile(path) => {
                time_profile!(
                    ProfilerCategory::ImageSaving,
                    None,
                    self.global.time_profiler_chan.clone(),
                    || match File::create(&*path) {
                        Ok(mut file) => {
                            if let Some(image) = self.rendering_context.read_to_image(Rect::new(
                                Point2D::new(x as u32, y as u32),
                                Size2D::new(width, height),
                            )) {
                                let dynamic_image = DynamicImage::ImageRgba8(image);
                                if let Err(e) = dynamic_image.write_to(&mut file, ImageFormat::Png)
                                {
                                    error!("Failed to save {} ({}).", path, e);
                                }
                            }
                        },
                        Err(e) => error!("Failed to create {} ({}).", path, e),
                    },
                );
                None
            },
        };

        Ok(rv)
    }

    /// Send all pending paint metrics messages after a composite operation, which may advance
    /// the epoch for pipelines in the WebRender scene.
    ///
    /// If there are pending paint metrics, we check if any of the painted epochs is one
    /// of the ones that the paint metrics recorder is expecting. In that case, we get the
    /// current time, inform the constellation about it and remove the pending metric from
    /// the list.
    fn send_pending_paint_metrics_messages_after_composite(&mut self) {
        if self.pending_paint_metrics.is_empty() {
            return;
        }

        let paint_time = CrossProcessInstant::now();
        let mut pipelines_to_remove = Vec::new();
        let pending_paint_metrics = &mut self.pending_paint_metrics;

        // For each pending paint metrics pipeline id, determine the current
        // epoch and update paint timing if necessary.
        for (pipeline_id, pending_epochs) in pending_paint_metrics.iter_mut() {
            let Some(WebRenderEpoch(current_epoch)) = self
                .webrender
                .as_ref()
                .and_then(|wr| wr.current_epoch(self.webrender_document, pipeline_id.into()))
            else {
                continue;
            };

            // If the pipeline is unknown, stop trying to send paint metrics for it.
            let Some(pipeline) = self
                .pipeline_details
                .get(pipeline_id)
                .and_then(|pipeline_details| pipeline_details.pipeline.as_ref())
            else {
                pipelines_to_remove.push(*pipeline_id);
                continue;
            };

            let current_epoch = Epoch(current_epoch);
            let Some(index) = pending_epochs
                .iter()
                .position(|epoch| *epoch == current_epoch)
            else {
                continue;
            };

            // Remove all epochs that were pending before the current epochs. They were not and will not,
            // be painted.
            pending_epochs.drain(0..index);

            if let Err(error) = pipeline
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

        for pipeline_id in pipelines_to_remove.iter() {
            self.pending_paint_metrics.remove(pipeline_id);
        }
    }

    fn clear_background(&self) {
        let gl = &self.global.webrender_gl;
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
        debug_assert_eq!(self.global.webrender_gl.get_error(), gleam::gl::NO_ERROR);
    }

    #[track_caller]
    fn assert_gl_framebuffer_complete(&self) {
        debug_assert_eq!(
            (
                self.global.webrender_gl.get_error(),
                self.global
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
        while let Some(msg) = self.global.compositor_receiver.try_recv_compositor_msg() {
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

            if self.global.shutdown_state == ShutdownState::FinishedShuttingDown {
                return;
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    pub fn perform_updates(&mut self) -> bool {
        if self.global.shutdown_state == ShutdownState::FinishedShuttingDown {
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
        self.global.webxr_main_thread.run_one_frame();

        // The WebXR thread may make a different context current
        if let Err(err) = self.rendering_context.make_current() {
            warn!("Failed to make the rendering context current: {:?}", err);
        }
        if !self.pending_scroll_zoom_events.is_empty() {
            self.process_pending_scroll_events()
        }
        self.global.shutdown_state != ShutdownState::FinishedShuttingDown
    }

    pub fn pinch_zoom_level(&self) -> Scale<f32, DevicePixel, DevicePixel> {
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
        self.global
            .webrender_api
            .send_transaction(self.webrender_document, txn);
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
            .webrender_api
            .save_capture(capture_path.clone(), CaptureBits::all());

        let version_file_path = capture_path.join("servo-version.txt");
        if let Err(error) = File::create(version_file_path)
            .and_then(|mut file| write!(file, "{}", self.global.version_string))
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

        self.global
            .webrender_api
            .send_transaction(self.webrender_document, transaction);
    }

    fn add_font(&mut self, font_key: FontKey, index: u32, data: Arc<IpcSharedMemory>) {
        let mut transaction = Transaction::new();
        transaction.add_raw_font(font_key, (**data).into(), index);
        self.global
            .webrender_api
            .send_transaction(self.webrender_document, transaction);
    }
}
