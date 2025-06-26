/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref, RefCell};
use std::collections::HashMap;
use std::env;
use std::fs::create_dir_all;
use std::iter::once;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use base::cross_process_instant::CrossProcessInstant;
use base::id::{PipelineId, WebViewId};
use base::{Epoch, WebRenderEpochToU16};
use bitflags::bitflags;
use compositing_traits::display_list::{
    CompositorDisplayListInfo, HitTestInfo, ScrollTree, ScrollType,
};
use compositing_traits::rendering_context::RenderingContext;
use compositing_traits::{
    CompositionPipeline, CompositorMsg, ImageUpdate, PipelineExitSource, SendableFrameTree,
    WebViewTrait,
};
use constellation_traits::{EmbedderToConstellationMessage, PaintMetricEvent};
use crossbeam_channel::{Receiver, Sender};
use dpi::PhysicalSize;
use embedder_traits::{
    CompositorHitTestResult, Cursor, InputEvent, MouseButtonEvent, MouseMoveEvent, ShutdownState,
    UntrustedNodeAddress, ViewportDetails, WheelDelta, WheelEvent, WheelMode,
};
use euclid::{Point2D, Rect, Scale, Size2D, Transform3D, Vector2D};
use ipc_channel::ipc::{self, IpcSharedMemory};
use libc::c_void;
use log::{debug, info, trace, warn};
use pixels::{CorsStatus, ImageFrame, ImageMetadata, PixelFormat, RasterImage};
use profile_traits::mem::{ProcessReports, ProfilerRegistration, Report, ReportKind};
use profile_traits::time::{self as profile_time, ProfilerCategory};
use profile_traits::{path, time_profile};
use servo_config::opts;
use servo_geometry::DeviceIndependentPixel;
use style_traits::CSSPixel;
use webrender::{CaptureBits, RenderApi, Transaction};
use webrender_api::units::{
    DeviceIntPoint, DeviceIntRect, DevicePixel, DevicePoint, DeviceRect, LayoutPoint, LayoutRect,
    LayoutSize, LayoutVector2D, WorldPoint,
};
use webrender_api::{
    self, BuiltDisplayList, DirtyRect, DisplayListPayload, DocumentId, Epoch as WebRenderEpoch,
    FontInstanceFlags, FontInstanceKey, FontInstanceOptions, FontKey, HitTestFlags,
    PipelineId as WebRenderPipelineId, PropertyBinding, ReferenceFrameKind, RenderReasons,
    SampledScrollOffset, ScrollLocation, SpaceAndClipInfo, SpatialId, SpatialTreeItemKey,
    TransformStyle,
};

use crate::InitialCompositorState;
use crate::refresh_driver::RefreshDriver;
use crate::webview_manager::WebViewManager;
use crate::webview_renderer::{PinchZoomResult, UnknownWebView, WebViewRenderer};

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

/// Holds the state when running reftests that determines when it is
/// safe to save the output image.
#[derive(Clone, Copy, Debug, PartialEq)]
enum ReadyState {
    Unknown,
    WaitingForConstellationReply,
    ReadyToSaveImage,
}

/// An option to control what kind of WebRender debugging is enabled while Servo is running.
#[derive(Clone)]
pub enum WebRenderDebugOption {
    Profiler,
    TextureCacheDebug,
    RenderTargetDebug,
}
/// Data that is shared by all WebView renderers.
pub struct ServoRenderer {
    /// The [`RefreshDriver`] which manages the rythym of painting.
    refresh_driver: RefreshDriver,

    /// This is a temporary map between [`PipelineId`]s and their associated [`WebViewId`]. Once
    /// all renderer operations become per-`WebView` this map can be removed, but we still sometimes
    /// need to work backwards to figure out what `WebView` is associated with a `Pipeline`.
    pub(crate) pipeline_to_webview_map: HashMap<PipelineId, WebViewId>,

    /// Tracks whether we are in the process of shutting down, or have shut down and should close
    /// the compositor. This is shared with the `Servo` instance.
    shutdown_state: Rc<Cell<ShutdownState>>,

    /// The port on which we receive messages.
    compositor_receiver: Receiver<CompositorMsg>,

    /// The channel on which messages can be sent to the constellation.
    pub(crate) constellation_sender: Sender<EmbedderToConstellationMessage>,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: profile_time::ProfilerChan,

    /// The WebRender [`RenderApi`] interface used to communicate with WebRender.
    pub(crate) webrender_api: RenderApi,

    /// The active webrender document.
    pub(crate) webrender_document: DocumentId,

    /// The GL bindings for webrender
    webrender_gl: Rc<dyn gleam::gl::Gl>,

    #[cfg(feature = "webxr")]
    /// Some XR devices want to run on the main thread.
    webxr_main_thread: webxr::MainThreadRegistry,

    /// True to translate mouse input into touch events.
    pub(crate) convert_mouse_to_touch: bool,

    /// Current mouse cursor.
    cursor: Cursor,

    /// Current cursor position.
    cursor_pos: DevicePoint,
}

/// NB: Never block on the constellation, because sometimes the constellation blocks on us.
pub struct IOCompositor {
    /// Data that is shared by all WebView renderers.
    global: Rc<RefCell<ServoRenderer>>,

    /// Our [`WebViewRenderer`]s, one for every `WebView`.
    webview_renderers: WebViewManager<WebViewRenderer>,

    /// Tracks whether or not the view needs to be repainted.
    needs_repaint: Cell<RepaintReason>,

    /// Used by the logic that determines when it is safe to output an
    /// image for the reftest framework.
    ready_to_save_state: ReadyState,

    /// The webrender renderer.
    webrender: Option<webrender::Renderer>,

    /// The surfman instance that webrender targets
    rendering_context: Rc<dyn RenderingContext>,

    /// The number of frames pending to receive from WebRender.
    pending_frames: usize,

    /// A handle to the memory profiler which will automatically unregister
    /// when it's dropped.
    _mem_profiler_registration: ProfilerRegistration,
}

/// Why we need to be repainted. This is used for debugging.
#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct RepaintReason(u8);

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

/// The paint status of a particular pipeline in the Servo renderer. This is used to trigger metrics
/// in script (via the constellation) when display lists are received.
///
/// See <https://w3c.github.io/paint-timing/#first-contentful-paint>.
#[derive(PartialEq)]
pub(crate) enum PaintMetricState {
    /// The renderer is still waiting to process a display list which triggers this metric.
    Waiting,
    /// The renderer has processed the display list which will trigger this event, marked the Servo
    /// instance ready to paint, and is waiting for the given epoch to actually be rendered.
    Seen(WebRenderEpoch, bool /* first_reflow */),
    /// The metric has been sent to the constellation and no more work needs to be done.
    Sent,
}

pub(crate) struct PipelineDetails {
    /// The pipeline associated with this PipelineDetails object.
    pub pipeline: Option<CompositionPipeline>,

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

    /// The paint metric status of the first paint.
    pub first_paint_metric: PaintMetricState,

    /// The paint metric status of the first contentful paint.
    pub first_contentful_paint_metric: PaintMetricState,

    /// Which parts of Servo have reported that this `Pipeline` has exited. Only when all
    /// have done so will it be discarded.
    pub exited: PipelineExitSource,
}

impl PipelineDetails {
    pub(crate) fn animations_or_animation_callbacks_running(&self) -> bool {
        self.animations_running || self.animation_callbacks_running
    }

    pub(crate) fn animation_callbacks_running(&self) -> bool {
        self.animation_callbacks_running
    }

    pub(crate) fn animating(&self) -> bool {
        !self.throttled && (self.animation_callbacks_running || self.animations_running)
    }
}

impl PipelineDetails {
    pub(crate) fn new() -> PipelineDetails {
        PipelineDetails {
            pipeline: None,
            parent_pipeline_id: None,
            most_recent_display_list_epoch: None,
            animations_running: false,
            animation_callbacks_running: false,
            throttled: false,
            hit_test_items: Vec::new(),
            scroll_tree: ScrollTree::default(),
            first_paint_metric: PaintMetricState::Waiting,
            first_contentful_paint_metric: PaintMetricState::Waiting,
            exited: PipelineExitSource::empty(),
        }
    }

    fn install_new_scroll_tree(&mut self, new_scroll_tree: ScrollTree) {
        let old_scroll_offsets = self.scroll_tree.scroll_offsets();
        self.scroll_tree = new_scroll_tree;
        self.scroll_tree.set_all_scroll_offsets(&old_scroll_offsets);
    }
}

pub enum HitTestError {
    EpochMismatch,
    Others,
}

impl ServoRenderer {
    pub fn shutdown_state(&self) -> ShutdownState {
        self.shutdown_state.get()
    }

    pub(crate) fn hit_test_at_point<'a>(
        &self,
        point: DevicePoint,
        details_for_pipeline: impl Fn(PipelineId) -> Option<&'a PipelineDetails>,
    ) -> Result<CompositorHitTestResult, HitTestError> {
        match self.hit_test_at_point_with_flags_and_pipeline(
            point,
            HitTestFlags::empty(),
            None,
            details_for_pipeline,
        ) {
            Ok(hit_test_results) => hit_test_results
                .first()
                .cloned()
                .ok_or(HitTestError::Others),
            Err(error) => Err(error),
        }
    }

    // TODO: split this into first half (global) and second half (one for whole compositor, one for webview)
    pub(crate) fn hit_test_at_point_with_flags_and_pipeline<'a>(
        &self,
        point: DevicePoint,
        flags: HitTestFlags,
        pipeline_id: Option<WebRenderPipelineId>,
        details_for_pipeline: impl Fn(PipelineId) -> Option<&'a PipelineDetails>,
    ) -> Result<Vec<CompositorHitTestResult>, HitTestError> {
        // DevicePoint and WorldPoint are the same for us.
        let world_point = WorldPoint::from_untyped(point.to_untyped());
        let results =
            self.webrender_api
                .hit_test(self.webrender_document, pipeline_id, world_point, flags);

        let mut epoch_mismatch = false;
        let results = results
            .items
            .iter()
            .filter_map(|item| {
                let pipeline_id = item.pipeline.into();
                let details = details_for_pipeline(pipeline_id)?;

                // If the epoch in the tag does not match the current epoch of the pipeline,
                // then the hit test is against an old version of the display list.
                match details.most_recent_display_list_epoch {
                    Some(epoch) => {
                        if epoch.as_u16() != item.tag.1 {
                            // It's too early to hit test for now.
                            // New scene building is in progress.
                            epoch_mismatch = true;
                            return None;
                        }
                    },
                    _ => return None,
                }

                let offset = details
                    .scroll_tree
                    .scroll_offset(pipeline_id.root_scroll_id())
                    .unwrap_or_default();
                let point_in_initial_containing_block =
                    (item.point_in_viewport + offset).to_untyped();

                let info = &details.hit_test_items[item.tag.0 as usize];
                Some(CompositorHitTestResult {
                    pipeline_id,
                    point_in_viewport: Point2D::from_untyped(item.point_in_viewport.to_untyped()),
                    point_relative_to_initial_containing_block: Point2D::from_untyped(
                        point_in_initial_containing_block,
                    ),
                    point_relative_to_item: Point2D::from_untyped(
                        item.point_relative_to_item.to_untyped(),
                    ),
                    node: UntrustedNodeAddress(info.node as *const c_void),
                    cursor: info.cursor,
                    scroll_tree_node: info.scroll_tree_node,
                })
            })
            .collect();

        if epoch_mismatch {
            return Err(HitTestError::EpochMismatch);
        }

        Ok(results)
    }

    pub(crate) fn send_transaction(&mut self, transaction: Transaction) {
        self.webrender_api
            .send_transaction(self.webrender_document, transaction);
    }

    pub(crate) fn update_cursor_from_hittest(
        &mut self,
        pos: DevicePoint,
        result: &CompositorHitTestResult,
    ) {
        if let Some(webview_id) = self
            .pipeline_to_webview_map
            .get(&result.pipeline_id)
            .copied()
        {
            self.update_cursor(pos, webview_id, result.cursor);
        } else {
            warn!("Couldn't update cursor for non-WebView-associated pipeline");
        };
    }

    pub(crate) fn update_cursor(
        &mut self,
        pos: DevicePoint,
        webview_id: WebViewId,
        cursor: Option<Cursor>,
    ) {
        self.cursor_pos = pos;

        let cursor = match cursor {
            Some(cursor) if cursor != self.cursor => cursor,
            _ => return,
        };

        self.cursor = cursor;
        if let Err(e) = self
            .constellation_sender
            .send(EmbedderToConstellationMessage::SetCursor(
                webview_id, cursor,
            ))
        {
            warn!("Sending event to constellation failed ({:?}).", e);
        }
    }
}

impl IOCompositor {
    pub fn new(state: InitialCompositorState, convert_mouse_to_touch: bool) -> Self {
        let registration = state.mem_profiler_chan.prepare_memory_reporting(
            "compositor".into(),
            state.sender.clone(),
            CompositorMsg::CollectMemoryReport,
        );
        let compositor = IOCompositor {
            global: Rc::new(RefCell::new(ServoRenderer {
                refresh_driver: RefreshDriver::new(
                    state.constellation_chan.clone(),
                    state.event_loop_waker,
                ),
                shutdown_state: state.shutdown_state,
                pipeline_to_webview_map: Default::default(),
                compositor_receiver: state.receiver,
                constellation_sender: state.constellation_chan,
                time_profiler_chan: state.time_profiler_chan,
                webrender_api: state.webrender_api,
                webrender_document: state.webrender_document,
                webrender_gl: state.webrender_gl,
                #[cfg(feature = "webxr")]
                webxr_main_thread: state.webxr_main_thread,
                convert_mouse_to_touch,
                cursor: Cursor::None,
                cursor_pos: DevicePoint::new(0.0, 0.0),
            })),
            webview_renderers: WebViewManager::default(),
            needs_repaint: Cell::default(),
            ready_to_save_state: ReadyState::Unknown,
            webrender: Some(state.webrender),
            rendering_context: state.rendering_context,
            pending_frames: 0,
            _mem_profiler_registration: registration,
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

    pub fn rendering_context_size(&self) -> Size2D<u32, DevicePixel> {
        self.rendering_context.size2d()
    }

    pub fn webxr_running(&self) -> bool {
        #[cfg(feature = "webxr")]
        {
            self.global.borrow().webxr_main_thread.running()
        }
        #[cfg(not(feature = "webxr"))]
        {
            false
        }
    }

    fn set_needs_repaint(&self, reason: RepaintReason) {
        let mut needs_repaint = self.needs_repaint.get();
        needs_repaint.insert(reason);
        self.needs_repaint.set(needs_repaint);
    }

    pub fn needs_repaint(&self) -> bool {
        let repaint_reason = self.needs_repaint.get();
        if repaint_reason.is_empty() {
            return false;
        }

        !self
            .global
            .borrow()
            .refresh_driver
            .wait_to_paint(repaint_reason)
    }

    pub fn finish_shutting_down(&mut self) {
        // Drain compositor port, sometimes messages contain channels that are blocking
        // another thread from finishing (i.e. SetFrameTree).
        while self
            .global
            .borrow_mut()
            .compositor_receiver
            .try_recv()
            .is_ok()
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
            CompositorMsg::CollectMemoryReport(sender) => {
                let ops =
                    wr_malloc_size_of::MallocSizeOfOps::new(servo_allocator::usable_size, None);
                let report = self.global.borrow().webrender_api.report_memory(ops);
                let reports = vec![
                    Report {
                        path: path!["webrender", "fonts"],
                        kind: ReportKind::ExplicitJemallocHeapSize,
                        size: report.fonts,
                    },
                    Report {
                        path: path!["webrender", "images"],
                        kind: ReportKind::ExplicitJemallocHeapSize,
                        size: report.images,
                    },
                    Report {
                        path: path!["webrender", "display-list"],
                        kind: ReportKind::ExplicitJemallocHeapSize,
                        size: report.display_list,
                    },
                ];
                sender.send(ProcessReports::new(reports));
            },

            CompositorMsg::ChangeRunningAnimationsState(
                webview_id,
                pipeline_id,
                animation_state,
            ) => {
                let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) else {
                    return;
                };

                if webview_renderer
                    .change_pipeline_running_animations_state(pipeline_id, animation_state)
                {
                    self.global
                        .borrow()
                        .refresh_driver
                        .notify_animation_state_changed(webview_renderer);
                }
            },

            CompositorMsg::CreateOrUpdateWebView(frame_tree) => {
                self.set_frame_tree_for_webview(&frame_tree);
            },

            CompositorMsg::RemoveWebView(webview_id) => {
                self.remove_webview(webview_id);
            },

            CompositorMsg::TouchEventProcessed(webview_id, result) => {
                let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) else {
                    warn!("Handling input event for unknown webview: {webview_id}");
                    return;
                };
                webview_renderer.on_touch_event_processed(result);
            },

            CompositorMsg::CreatePng(webview_id, page_rect, reply) => {
                let res = self.render_to_shared_memory(webview_id, page_rect);
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
                let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) else {
                    return;
                };

                if webview_renderer.set_throttled(pipeline_id, throttled) {
                    self.global
                        .borrow()
                        .refresh_driver
                        .notify_animation_state_changed(webview_renderer);
                }
            },

            CompositorMsg::PipelineExited(webview_id, pipeline_id, pipeline_exit_source) => {
                debug!(
                    "Compositor got pipeline exited: {:?} {:?}",
                    webview_id, pipeline_id
                );
                if let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) {
                    webview_renderer.pipeline_exited(pipeline_id, pipeline_exit_source);
                }
            },

            CompositorMsg::NewWebRenderFrameReady(_document_id, recomposite_needed) => {
                self.pending_frames -= 1;
                let point: DevicePoint = self.global.borrow().cursor_pos;

                if recomposite_needed {
                    let details_for_pipeline = |pipeline_id| self.details_for_pipeline(pipeline_id);
                    let result = self
                        .global
                        .borrow()
                        .hit_test_at_point(point, details_for_pipeline);
                    if let Ok(result) = result {
                        self.global
                            .borrow_mut()
                            .update_cursor_from_hittest(point, &result);
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

            CompositorMsg::WebDriverMouseButtonEvent(
                webview_id,
                action,
                button,
                x,
                y,
                message_id,
            ) => {
                let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) else {
                    warn!("Handling input event for unknown webview: {webview_id}");
                    return;
                };
                let dppx = webview_renderer.device_pixels_per_page_pixel();
                let point = dppx.transform_point(Point2D::new(x, y));
                webview_renderer.dispatch_point_input_event(
                    InputEvent::MouseButton(MouseButtonEvent::new(action, button, point))
                        .with_webdriver_message_id(message_id),
                );
            },

            CompositorMsg::WebDriverMouseMoveEvent(webview_id, x, y, message_id) => {
                let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) else {
                    warn!("Handling input event for unknown webview: {webview_id}");
                    return;
                };
                let dppx = webview_renderer.device_pixels_per_page_pixel();
                let point = dppx.transform_point(Point2D::new(x, y));
                webview_renderer.dispatch_point_input_event(
                    InputEvent::MouseMove(MouseMoveEvent::new(point))
                        .with_webdriver_message_id(message_id),
                );
            },

            CompositorMsg::WebDriverWheelScrollEvent(webview_id, x, y, dx, dy, message_id) => {
                let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) else {
                    warn!("Handling input event for unknown webview: {webview_id}");
                    return;
                };
                // The sign of wheel delta value definition in uievent
                // is inverted compared to `winit`s wheel delta. Hence,
                // here we invert the sign to mimic wheel scroll
                // implementation in `headed_window.rs`.
                let dx = -dx;
                let dy = -dy;
                let delta = WheelDelta {
                    x: dx,
                    y: dy,
                    z: 0.0,
                    mode: WheelMode::DeltaPixel,
                };
                let dppx = webview_renderer.device_pixels_per_page_pixel();
                let point = dppx.transform_point(Point2D::new(x, y));
                let scroll_delta = dppx.transform_vector(Vector2D::new(dx as f32, dy as f32));
                webview_renderer.dispatch_point_input_event(
                    InputEvent::Wheel(WheelEvent::new(delta, point))
                        .with_webdriver_message_id(message_id),
                );
                webview_renderer.on_webdriver_wheel_action(scroll_delta, point);
            },

            CompositorMsg::SendInitialTransaction(pipeline) => {
                let mut txn = Transaction::new();
                txn.set_display_list(WebRenderEpoch(0), (pipeline, Default::default()));
                self.generate_frame(&mut txn, RenderReasons::SCENE);
                self.global.borrow_mut().send_transaction(txn);
            },

            CompositorMsg::SendScrollNode(webview_id, pipeline_id, offset, external_scroll_id) => {
                let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) else {
                    return;
                };

                let pipeline_id = pipeline_id.into();
                let Some(pipeline_details) = webview_renderer.pipelines.get_mut(&pipeline_id)
                else {
                    return;
                };

                let Some(offset) = pipeline_details
                    .scroll_tree
                    .set_scroll_offset_for_node_with_external_scroll_id(
                        external_scroll_id,
                        offset,
                        ScrollType::Script,
                    )
                else {
                    // The renderer should be fully up-to-date with script at this point and script
                    // should never try to scroll to an invalid location.
                    warn!("Could not scroll node with id: {external_scroll_id:?}");
                    return;
                };

                let mut txn = Transaction::new();
                txn.set_scroll_offsets(
                    external_scroll_id,
                    vec![SampledScrollOffset {
                        offset: -offset,
                        generation: 0,
                    }],
                );
                self.generate_frame(&mut txn, RenderReasons::APZ);
                self.global.borrow_mut().send_transaction(txn);
            },

            CompositorMsg::SendDisplayList {
                webview_id,
                display_list_descriptor,
                display_list_receiver,
            } => {
                // This must match the order from the sender, currently in `shared/script/lib.rs`.
                let display_list_info = match display_list_receiver.recv() {
                    Ok(display_list_info) => display_list_info,
                    Err(error) => {
                        return warn!("Could not receive display list info: {error}");
                    },
                };
                let display_list_info: CompositorDisplayListInfo =
                    match bincode::deserialize(&display_list_info) {
                        Ok(display_list_info) => display_list_info,
                        Err(error) => {
                            return warn!("Could not deserialize display list info: {error}");
                        },
                    };
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

                let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) else {
                    return warn!("Could not find WebView for incoming display list");
                };
                // WebRender is not ready until we receive "NewWebRenderFrameReady"
                webview_renderer.webrender_frame_ready.set(false);

                let pipeline_id = display_list_info.pipeline_id;
                let details = webview_renderer.ensure_pipeline_details(pipeline_id.into());
                details.most_recent_display_list_epoch = Some(display_list_info.epoch);
                details.hit_test_items = display_list_info.hit_test_info;
                details.install_new_scroll_tree(display_list_info.scroll_tree);

                let epoch = display_list_info.epoch;
                let first_reflow = display_list_info.first_reflow;
                if details.first_paint_metric == PaintMetricState::Waiting {
                    details.first_paint_metric = PaintMetricState::Seen(epoch, first_reflow);
                }
                if details.first_contentful_paint_metric == PaintMetricState::Waiting &&
                    display_list_info.is_contentful
                {
                    details.first_contentful_paint_metric =
                        PaintMetricState::Seen(epoch, first_reflow);
                }

                let mut transaction = Transaction::new();
                transaction
                    .set_display_list(display_list_info.epoch, (pipeline_id, built_display_list));
                self.update_transaction_with_all_scroll_offsets(&mut transaction);
                self.generate_frame(&mut transaction, RenderReasons::SCENE);
                self.global.borrow_mut().send_transaction(transaction);
            },

            CompositorMsg::HitTest(pipeline, point, flags, sender) => {
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
                    )
                    .unwrap_or_default();
                let _ = sender.send(result);
            },

            CompositorMsg::GenerateImageKey(sender) => {
                let _ = sender.send(self.global.borrow().webrender_api.generate_image_key());
            },

            CompositorMsg::UpdateImages(updates) => {
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

            CompositorMsg::AddFont(font_key, data, index) => {
                self.add_font(font_key, index, data);
            },

            CompositorMsg::AddSystemFont(font_key, native_handle) => {
                let mut transaction = Transaction::new();
                transaction.add_native_font(font_key, native_handle);
                self.global.borrow_mut().send_transaction(transaction);
            },

            CompositorMsg::AddFontInstance(font_instance_key, font_key, size, flags) => {
                self.add_font_instance(font_instance_key, font_key, size, flags);
            },

            CompositorMsg::RemoveFonts(keys, instance_keys) => {
                let mut transaction = Transaction::new();

                for instance in instance_keys.into_iter() {
                    transaction.delete_font_instance(instance);
                }
                for key in keys.into_iter() {
                    transaction.delete_font(key);
                }

                self.global.borrow_mut().send_transaction(transaction);
            },

            CompositorMsg::AddImage(key, desc, data) => {
                let mut txn = Transaction::new();
                txn.add_image(key, desc, data.into(), None);
                self.global.borrow_mut().send_transaction(txn);
            },

            CompositorMsg::GenerateFontKeys(
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
            CompositorMsg::GetClientWindowRect(webview_id, response_sender) => {
                let client_window_rect = self
                    .webview_renderers
                    .get(webview_id)
                    .map(|webview_renderer| {
                        webview_renderer.client_window_rect(self.rendering_context.size2d())
                    })
                    .unwrap_or_default();
                if let Err(error) = response_sender.send(client_window_rect) {
                    warn!("Sending response to get client window failed ({error:?}).");
                }
            },
            CompositorMsg::GetScreenSize(webview_id, response_sender) => {
                let screen_size = self
                    .webview_renderers
                    .get(webview_id)
                    .map(WebViewRenderer::screen_size)
                    .unwrap_or_default();
                if let Err(error) = response_sender.send(screen_size) {
                    warn!("Sending response to get screen size failed ({error:?}).");
                }
            },
            CompositorMsg::GetAvailableScreenSize(webview_id, response_sender) => {
                let available_screen_size = self
                    .webview_renderers
                    .get(webview_id)
                    .map(WebViewRenderer::available_screen_size)
                    .unwrap_or_default();
                if let Err(error) = response_sender.send(available_screen_size) {
                    warn!("Sending response to get screen size failed ({error:?}).");
                }
            },
            CompositorMsg::Viewport(webview_id, viewport_description) => {
                if let Some(webview) = self.webview_renderers.get_mut(webview_id) {
                    webview.set_viewport_description(viewport_description);
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
            CompositorMsg::PipelineExited(webview_id, pipeline_id, pipeline_exit_source) => {
                debug!(
                    "Compositor got pipeline exited: {:?} {:?}",
                    webview_id, pipeline_id
                );
                if let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) {
                    webview_renderer.pipeline_exited(pipeline_id, pipeline_exit_source);
                }
            },
            CompositorMsg::GenerateImageKey(sender) => {
                let _ = sender.send(self.global.borrow().webrender_api.generate_image_key());
            },
            CompositorMsg::GenerateFontKeys(
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
            CompositorMsg::GetClientWindowRect(_, response_sender) => {
                if let Err(error) = response_sender.send(Default::default()) {
                    warn!("Sending response to get client window failed ({error:?}).");
                }
            },
            CompositorMsg::GetScreenSize(_, response_sender) => {
                if let Err(error) = response_sender.send(Default::default()) {
                    warn!("Sending response to get client window failed ({error:?}).");
                }
            },
            CompositorMsg::GetAvailableScreenSize(_, response_sender) => {
                if let Err(error) = response_sender.send(Default::default()) {
                    warn!("Sending response to get client window failed ({error:?}).");
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

        let root_reference_frame = SpatialId::root_reference_frame(root_pipeline);

        let viewport_size = self.rendering_context.size2d().to_f32().to_untyped();
        let viewport_rect = LayoutRect::from_origin_and_size(
            LayoutPoint::zero(),
            LayoutSize::from_untyped(viewport_size),
        );

        let root_clip_id = builder.define_clip_rect(root_reference_frame, viewport_rect);
        let clip_chain_id = builder.define_clip_chain(None, [root_clip_id]);
        for (_, webview_renderer) in self.webview_renderers.painting_order() {
            let Some(pipeline_id) = webview_renderer.root_pipeline_id else {
                continue;
            };

            let device_pixels_per_page_pixel = webview_renderer.device_pixels_per_page_pixel().0;
            let webview_reference_frame = builder.push_reference_frame(
                LayoutPoint::zero(),
                root_reference_frame,
                TransformStyle::Flat,
                PropertyBinding::Value(Transform3D::scale(
                    device_pixels_per_page_pixel,
                    device_pixels_per_page_pixel,
                    1.,
                )),
                ReferenceFrameKind::Transform {
                    is_2d_scale_translation: true,
                    should_snap: true,
                    paired_with_perspective: false,
                },
                SpatialTreeItemKey::new(0, 0),
            );

            let scaled_webview_rect = webview_renderer.rect / device_pixels_per_page_pixel;
            builder.push_iframe(
                LayoutRect::from_untyped(&scaled_webview_rect.to_untyped()),
                LayoutRect::from_untyped(&scaled_webview_rect.to_untyped()),
                &SpaceAndClipInfo {
                    spatial_id: webview_reference_frame,
                    clip_chain_id,
                },
                pipeline_id.into(),
                true,
            );
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
        for webview_renderer in self.webview_renderers.iter() {
            for details in webview_renderer.pipelines.values() {
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

    pub fn add_webview(
        &mut self,
        webview: Box<dyn WebViewTrait>,
        viewport_details: ViewportDetails,
    ) {
        self.webview_renderers
            .entry(webview.id())
            .or_insert(WebViewRenderer::new(
                self.global.clone(),
                webview,
                viewport_details,
            ));
    }

    fn set_frame_tree_for_webview(&mut self, frame_tree: &SendableFrameTree) {
        debug!("{}: Setting frame tree for webview", frame_tree.pipeline.id);

        let webview_id = frame_tree.pipeline.webview_id;
        let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) else {
            warn!(
                "Attempted to set frame tree on unknown WebView (perhaps closed?): {webview_id:?}"
            );
            return;
        };

        webview_renderer.set_frame_tree(frame_tree);
        self.send_root_pipeline_display_list();
    }

    fn remove_webview(&mut self, webview_id: WebViewId) {
        debug!("{}: Removing", webview_id);
        if self.webview_renderers.remove(webview_id).is_err() {
            warn!("{webview_id}: Removing unknown webview");
            return;
        };

        self.send_root_pipeline_display_list();
    }

    pub fn show_webview(
        &mut self,
        webview_id: WebViewId,
        hide_others: bool,
    ) -> Result<(), UnknownWebView> {
        debug!("{webview_id}: Showing webview; hide_others={hide_others}");
        let painting_order_changed = if hide_others {
            let result = self
                .webview_renderers
                .painting_order()
                .map(|(&id, _)| id)
                .ne(once(webview_id));
            self.webview_renderers.hide_all();
            self.webview_renderers.show(webview_id)?;
            result
        } else {
            self.webview_renderers.show(webview_id)?
        };
        if painting_order_changed {
            self.send_root_pipeline_display_list();
        }
        Ok(())
    }

    pub fn hide_webview(&mut self, webview_id: WebViewId) -> Result<(), UnknownWebView> {
        debug!("{webview_id}: Hiding webview");
        if self.webview_renderers.hide(webview_id)? {
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
                .webview_renderers
                .painting_order()
                .map(|(&id, _)| id)
                .ne(once(webview_id));
            self.webview_renderers.hide_all();
            self.webview_renderers.raise_to_top(webview_id)?;
            result
        } else {
            self.webview_renderers.raise_to_top(webview_id)?
        };
        if painting_order_changed {
            self.send_root_pipeline_display_list();
        }
        Ok(())
    }

    pub fn move_resize_webview(&mut self, webview_id: WebViewId, rect: DeviceRect) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) else {
            return;
        };
        if !webview_renderer.set_rect(rect) {
            return;
        }

        self.send_root_pipeline_display_list();
        self.set_needs_repaint(RepaintReason::Resize);
    }

    pub fn set_hidpi_scale_factor(
        &mut self,
        webview_id: WebViewId,
        new_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    ) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) else {
            return;
        };
        if !webview_renderer.set_hidpi_scale_factor(new_scale_factor) {
            return;
        }

        self.send_root_pipeline_display_list();
        self.set_needs_repaint(RepaintReason::Resize);
    }

    pub fn resize_rendering_context(&mut self, new_size: PhysicalSize<u32>) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        if self.rendering_context.size() == new_size {
            return;
        }

        self.rendering_context.resize(new_size);

        let mut transaction = Transaction::new();
        let output_region = DeviceIntRect::new(
            Point2D::zero(),
            Point2D::new(new_size.width as i32, new_size.height as i32),
        );
        transaction.set_document_view(output_region);
        self.global.borrow_mut().send_transaction(transaction);

        self.send_root_pipeline_display_list();
        self.set_needs_repaint(RepaintReason::Resize);
    }

    pub fn on_zoom_reset_window_event(&mut self, webview_id: WebViewId) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }

        if let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) {
            webview_renderer.set_page_zoom(1.0);
        }
        self.send_root_pipeline_display_list();
    }

    pub fn on_zoom_window_event(&mut self, webview_id: WebViewId, magnification: f32) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }

        if let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) {
            webview_renderer.set_page_zoom(magnification);
        }
        self.send_root_pipeline_display_list();
    }

    fn details_for_pipeline(&self, pipeline_id: PipelineId) -> Option<&PipelineDetails> {
        let webview_id = self
            .global
            .borrow()
            .pipeline_to_webview_map
            .get(&pipeline_id)
            .cloned()?;
        self.webview_renderers
            .get(webview_id)?
            .pipelines
            .get(&pipeline_id)
    }

    // Check if any pipelines currently have active animations or animation callbacks.
    fn animations_or_animation_callbacks_running(&self) -> bool {
        self.webview_renderers
            .iter()
            .any(WebViewRenderer::animations_or_animation_callbacks_running)
    }

    /// Returns true if any animation callbacks (ie `requestAnimationFrame`) are waiting for a response.
    fn animation_callbacks_running(&self) -> bool {
        self.webview_renderers
            .iter()
            .any(WebViewRenderer::animation_callbacks_running)
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
                for id in self
                    .webview_renderers
                    .iter()
                    .flat_map(WebViewRenderer::pipeline_ids)
                {
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
                let msg = EmbedderToConstellationMessage::IsReadyToSaveImage(pipeline_epochs);
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
        self.global
            .borrow()
            .refresh_driver
            .notify_will_paint(self.webview_renderers.iter());

        if let Err(error) = self.render_inner() {
            warn!("Unable to render: {error:?}");
            return false;
        }

        // We've painted the default target, which means that from the embedder's perspective,
        // the scene no longer needs to be repainted.
        self.needs_repaint.set(RepaintReason::empty());

        true
    }

    /// Render the WebRender scene to the shared memory, without updating other state of this
    /// [`IOCompositor`]. If succesful return the output image in shared memory.
    fn render_to_shared_memory(
        &mut self,
        webview_id: WebViewId,
        page_rect: Option<Rect<f32, CSSPixel>>,
    ) -> Result<Option<RasterImage>, UnableToComposite> {
        self.render_inner()?;

        let size = self.rendering_context.size2d().to_i32();
        let rect = if let Some(rect) = page_rect {
            let scale = self
                .webview_renderers
                .get(webview_id)
                .map(WebViewRenderer::device_pixels_per_page_pixel)
                .unwrap_or_else(|| Scale::new(1.0));
            let rect = scale.transform_rect(&rect);

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
            .map(|image| RasterImage {
                metadata: ImageMetadata {
                    width: image.width(),
                    height: image.height(),
                },
                format: PixelFormat::RGBA8,
                frames: vec![ImageFrame {
                    delay: None,
                    byte_range: 0..image.len(),
                    width: image.width(),
                    height: image.height(),
                }],
                bytes: ipc::IpcSharedMemory::from_bytes(&image),
                id: None,
                cors_status: CorsStatus::Safe,
            }))
    }

    #[servo_tracing::instrument(skip_all)]
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
            // continue waiting for the image output to be stable AND all active animations to complete.
            if self.animations_or_animation_callbacks_running() {
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
        for webview_renderer in self.webview_renderers.iter_mut() {
            for (pipeline_id, pipeline) in webview_renderer.pipelines.iter_mut() {
                let Some(current_epoch) = self
                    .webrender
                    .as_ref()
                    .and_then(|wr| wr.current_epoch(document_id, pipeline_id.into()))
                else {
                    continue;
                };

                match pipeline.first_paint_metric {
                    // We need to check whether the current epoch is later, because
                    // CrossProcessCompositorMessage::SendInitialTransaction sends an
                    // empty display list to WebRender which can happen before we receive
                    // the first "real" display list.
                    PaintMetricState::Seen(epoch, first_reflow) if epoch <= current_epoch => {
                        assert!(epoch <= current_epoch);
                        if let Err(error) = self.global.borrow().constellation_sender.send(
                            EmbedderToConstellationMessage::PaintMetric(
                                *pipeline_id,
                                PaintMetricEvent::FirstPaint(paint_time, first_reflow),
                            ),
                        ) {
                            warn!(
                                "Sending paint metric event to constellation failed ({error:?})."
                            );
                        }
                        pipeline.first_paint_metric = PaintMetricState::Sent;
                    },
                    _ => {},
                }

                match pipeline.first_contentful_paint_metric {
                    PaintMetricState::Seen(epoch, first_reflow) if epoch <= current_epoch => {
                        if let Err(error) = self.global.borrow().constellation_sender.send(
                            EmbedderToConstellationMessage::PaintMetric(
                                *pipeline_id,
                                PaintMetricEvent::FirstContentfulPaint(paint_time, first_reflow),
                            ),
                        ) {
                            warn!(
                                "Sending paint metric event to constellation failed ({error:?})."
                            );
                        }
                        pipeline.first_contentful_paint_metric = PaintMetricState::Sent;
                    },
                    _ => {},
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

    /// Get the message receiver for this [`IOCompositor`].
    pub fn receiver(&self) -> Ref<Receiver<CompositorMsg>> {
        Ref::map(self.global.borrow(), |global| &global.compositor_receiver)
    }

    #[servo_tracing::instrument(skip_all)]
    pub fn handle_messages(&mut self, mut messages: Vec<CompositorMsg>) {
        // Check for new messages coming from the other threads in the system.
        let mut found_recomposite_msg = false;
        messages.retain(|message| {
            match message {
                CompositorMsg::NewWebRenderFrameReady(..) if found_recomposite_msg => {
                    // Only take one of duplicate NewWebRendeFrameReady messages, but do subtract
                    // one frame from the pending frames.
                    self.pending_frames -= 1;
                    false
                },
                CompositorMsg::NewWebRenderFrameReady(..) => {
                    found_recomposite_msg = true;

                    // Process all pending events
                    // FIXME: Shouldn't `webview_frame_ready` be stored globally and why can't `pending_frames`
                    // be used here?
                    self.webview_renderers.iter().for_each(|webview| {
                        webview.dispatch_pending_point_input_events();
                        webview.webrender_frame_ready.set(true);
                    });

                    true
                },
                _ => true,
            }
        });

        for message in messages {
            self.handle_browser_message(message);
            if self.global.borrow().shutdown_state() == ShutdownState::FinishedShuttingDown {
                return;
            }
        }
    }

    #[servo_tracing::instrument(skip_all)]
    pub fn perform_updates(&mut self) -> bool {
        if self.global.borrow().shutdown_state() == ShutdownState::FinishedShuttingDown {
            return false;
        }

        #[cfg(feature = "webxr")]
        // Run the WebXR main thread
        self.global.borrow_mut().webxr_main_thread.run_one_frame();

        // The WebXR thread may make a different context current
        if let Err(err) = self.rendering_context.make_current() {
            warn!("Failed to make the rendering context current: {:?}", err);
        }

        let mut need_zoom = false;
        let scroll_offset_updates: Vec<_> = self
            .webview_renderers
            .iter_mut()
            .filter_map(|webview_renderer| {
                let (zoom, scroll_result) =
                    webview_renderer.process_pending_scroll_and_pinch_zoom_events();
                need_zoom = need_zoom || (zoom == PinchZoomResult::DidPinchZoom);
                scroll_result
            })
            .collect();

        if need_zoom || !scroll_offset_updates.is_empty() {
            let mut transaction = Transaction::new();
            if need_zoom {
                self.send_root_pipeline_display_list_in_transaction(&mut transaction);
            }
            for update in scroll_offset_updates {
                let offset = LayoutVector2D::new(-update.offset.x, -update.offset.y);
                transaction.set_scroll_offsets(
                    update.external_scroll_id,
                    vec![SampledScrollOffset {
                        offset,
                        generation: 0,
                    }],
                );
            }

            self.generate_frame(&mut transaction, RenderReasons::APZ);
            self.global.borrow_mut().send_transaction(transaction);
        }

        self.global.borrow().shutdown_state() != ShutdownState::FinishedShuttingDown
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
        if let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) {
            webview_renderer.notify_input_event(event);
        }
    }

    pub fn notify_scroll_event(
        &mut self,
        webview_id: WebViewId,
        scroll_location: ScrollLocation,
        cursor: DeviceIntPoint,
    ) {
        if let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) {
            webview_renderer.notify_scroll_event(scroll_location, cursor);
        }
    }

    pub fn on_vsync(&mut self, webview_id: WebViewId) {
        if let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) {
            webview_renderer.on_vsync();
        }
    }

    pub fn set_pinch_zoom(&mut self, webview_id: WebViewId, magnification: f32) {
        if let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) {
            webview_renderer.set_pinch_zoom(magnification);
        }
    }

    fn webrender_document(&self) -> DocumentId {
        self.global.borrow().webrender_document
    }

    fn shutdown_state(&self) -> ShutdownState {
        self.global.borrow().shutdown_state()
    }
}
