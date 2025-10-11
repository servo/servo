/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref, RefCell};
use std::collections::hash_map::Entry;
use std::env;
use std::fs::create_dir_all;
use std::iter::once;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use base::Epoch;
use base::cross_process_instant::CrossProcessInstant;
use base::generic_channel::{GenericSender, RoutedReceiver};
use base::id::{PipelineId, RenderingGroupId, WebViewId};
use bitflags::bitflags;
use compositing_traits::display_list::{CompositorDisplayListInfo, ScrollTree, ScrollType};
use compositing_traits::rendering_context::RenderingContext;
use compositing_traits::{
    CompositionPipeline, CompositorMsg, ImageUpdate, PipelineExitSource, SendableFrameTree,
    WebViewTrait,
};
use constellation_traits::{EmbedderToConstellationMessage, PaintMetricEvent};
use crossbeam_channel::Sender;
use dpi::PhysicalSize;
use embedder_traits::{
    CompositorHitTestResult, InputEventAndId, ScreenshotCaptureError, ShutdownState,
    ViewportDetails,
};
use euclid::{Point2D, Scale, Size2D, Transform3D};
use image::RgbaImage;
use ipc_channel::ipc::{self, IpcSharedMemory};
use log::{debug, info, trace, warn};
use profile_traits::mem::{
    ProcessReports, ProfilerRegistration, Report, ReportKind, perform_memory_report,
};
use profile_traits::time::{self as profile_time, ProfilerCategory};
use profile_traits::{path, time_profile};
use rustc_hash::{FxHashMap, FxHashSet};
use servo_config::pref;
use servo_geometry::DeviceIndependentPixel;
use style_traits::CSSPixel;
use webrender::{CaptureBits, RenderApi, Transaction};
use webrender_api::units::{
    DeviceIntPoint, DeviceIntRect, DevicePixel, DevicePoint, DeviceRect, LayoutPoint, LayoutRect,
    LayoutSize, WorldPoint,
};
use webrender_api::{
    self, BuiltDisplayList, DirtyRect, DisplayListPayload, DocumentId, Epoch as WebRenderEpoch,
    ExternalScrollId, FontInstanceFlags, FontInstanceKey, FontInstanceOptions, FontKey,
    FontVariation, ImageKey, PipelineId as WebRenderPipelineId, PropertyBinding,
    ReferenceFrameKind, RenderReasons, SampledScrollOffset, ScrollLocation, SpaceAndClipInfo,
    SpatialId, SpatialTreeItemKey, TransformStyle,
};

use crate::InitialCompositorState;
use crate::refresh_driver::RefreshDriver;
use crate::screenshot::ScreenshotTaker;
use crate::webview_manager::WebViewManager;
use crate::webview_renderer::{PinchZoomResult, UnknownWebView, WebViewRenderer};

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

    /// Tracks whether we are in the process of shutting down, or have shut down and should close
    /// the compositor. This is shared with the `Servo` instance.
    shutdown_state: Rc<Cell<ShutdownState>>,

    /// The port on which we receive messages.
    compositor_receiver: RoutedReceiver<CompositorMsg>,

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

    /// The last position in the rendered view that the mouse moved over. This becomes `None`
    /// when the mouse leaves the rendered view.
    pub(crate) last_mouse_move_position: Option<DevicePoint>,

    /// A [`FrameRequestDelayer`] which is used to wait for canvas image updates to
    /// arrive before requesting a new frame, as these happen asynchronously with
    /// `ScriptThread` display list construction.
    frame_delayer: FrameDelayer,
}

/// NB: Never block on the constellation, because sometimes the constellation blocks on us.
pub struct IOCompositor {
    /// Data that is shared by all WebView renderers.
    global: Rc<RefCell<ServoRenderer>>,

    /// Our [`WebViewRenderer`]s, one for every `WebView`.
    webview_renderers: WebViewManager<WebViewRenderer>,

    /// Tracks whether or not the view needs to be repainted.
    needs_repaint: Cell<RepaintReason>,

    /// The webrender renderer.
    webrender: Option<webrender::Renderer>,

    /// The [`RenderingContext`] instance that webrender targets, which is the viewport.
    rendering_context: Rc<dyn RenderingContext>,

    /// The number of frames pending to receive from WebRender.
    pending_frames: Cell<usize>,

    /// A [`ScreenshotTaker`] responsible for handling all screenshot requests.
    screenshot_taker: ScreenshotTaker,

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

    /// Whether animations are running
    pub animations_running: bool,

    /// Whether there are animation callbacks
    pub animation_callbacks_running: bool,

    /// Whether to use less resources by stopping animations.
    pub throttled: bool,

    /// The compositor-side [ScrollTree]. This is used to allow finding and scrolling
    /// nodes in the compositor before forwarding new offsets to WebRender.
    pub scroll_tree: ScrollTree,

    /// The paint metric status of the first paint.
    pub first_paint_metric: PaintMetricState,

    /// The paint metric status of the first contentful paint.
    pub first_contentful_paint_metric: PaintMetricState,

    /// The CSS pixel to device pixel scale of the viewport of this pipeline, including
    /// page zoom, but not including any pinch zoom amount. This is used to detect
    /// situations where the current display list is for an old scale.
    pub viewport_scale: Option<Scale<f32, CSSPixel, DevicePixel>>,

    /// Which parts of Servo have reported that this `Pipeline` has exited. Only when all
    /// have done so will it be discarded.
    pub exited: PipelineExitSource,

    /// The [`Epoch`] of the latest display list received for this `Pipeline` or `None` if no
    /// display list has been received.
    pub display_list_epoch: Option<Epoch>,
}

impl PipelineDetails {
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
            viewport_scale: None,
            animations_running: false,
            animation_callbacks_running: false,
            throttled: false,
            scroll_tree: ScrollTree::default(),
            first_paint_metric: PaintMetricState::Waiting,
            first_contentful_paint_metric: PaintMetricState::Waiting,
            exited: PipelineExitSource::empty(),
            display_list_epoch: None,
        }
    }

    fn install_new_scroll_tree(&mut self, new_scroll_tree: ScrollTree) {
        let old_scroll_offsets = self.scroll_tree.scroll_offsets();
        self.scroll_tree = new_scroll_tree;
        self.scroll_tree.set_all_scroll_offsets(&old_scroll_offsets);
    }
}

impl ServoRenderer {
    pub fn shutdown_state(&self) -> ShutdownState {
        self.shutdown_state.get()
    }

    pub(crate) fn hit_test_at_point(&self, point: DevicePoint) -> Vec<CompositorHitTestResult> {
        // DevicePoint and WorldPoint are the same for us.
        let world_point = WorldPoint::from_untyped(point.to_untyped());
        let results = self
            .webrender_api
            .hit_test(self.webrender_document, world_point);

        results
            .items
            .iter()
            .map(|item| {
                let pipeline_id = item.pipeline.into();
                let external_scroll_id = ExternalScrollId(item.tag.0, item.pipeline);
                CompositorHitTestResult {
                    pipeline_id,
                    point_in_viewport: Point2D::from_untyped(item.point_in_viewport.to_untyped()),
                    external_scroll_id,
                }
            })
            .collect()
    }

    pub(crate) fn send_transaction(&mut self, transaction: Transaction) {
        self.webrender_api
            .send_transaction(self.webrender_document, transaction);
    }
}

impl IOCompositor {
    pub fn new(state: InitialCompositorState) -> Self {
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
                compositor_receiver: state.receiver,
                constellation_sender: state.constellation_chan,
                time_profiler_chan: state.time_profiler_chan,
                webrender_api: state.webrender_api,
                webrender_document: state.webrender_document,
                webrender_gl: state.webrender_gl,
                #[cfg(feature = "webxr")]
                webxr_main_thread: state.webxr_main_thread,
                last_mouse_move_position: None,
                frame_delayer: Default::default(),
            })),
            webview_renderers: WebViewManager::default(),
            needs_repaint: Cell::default(),
            webrender: Some(state.webrender),
            rendering_context: state.rendering_context,
            pending_frames: Default::default(),
            screenshot_taker: Default::default(),
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

    pub(crate) fn rendering_context(&self) -> &dyn RenderingContext {
        &*self.rendering_context
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

    pub(crate) fn webview_renderer(&self, webview_id: WebViewId) -> Option<&WebViewRenderer> {
        self.webview_renderers.get(webview_id)
    }

    pub(crate) fn set_needs_repaint(&self, reason: RepaintReason) {
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
                let mut reports = vec![
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

                perform_memory_report(|ops| {
                    reports.push(Report {
                        path: path!["compositor", "scroll-tree"],
                        kind: ReportKind::ExplicitJemallocHeapSize,
                        size: self.webview_renderers.scroll_trees_memory_usage(ops),
                    });
                });

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

            CompositorMsg::NewWebRenderFrameReady(..) => {
                unreachable!("New WebRender frames should be handled in the caller.");
            },

            CompositorMsg::SendInitialTransaction(webview_id, pipeline_id) => {
                let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) else {
                    return warn!("Could not find WebView for incoming display list");
                };

                let starting_epoch = Epoch(0);
                let details = webview_renderer.ensure_pipeline_details(pipeline_id.into());
                details.display_list_epoch = Some(starting_epoch);

                let mut txn = Transaction::new();
                txn.set_display_list(starting_epoch.into(), (pipeline_id, Default::default()));
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
                        offset,
                        generation: 0,
                    }],
                );
                self.generate_frame(&mut txn, RenderReasons::APZ);
                self.global.borrow_mut().send_transaction(txn);
            },

            CompositorMsg::UpdateEpoch {
                webview_id,
                pipeline_id,
                epoch,
            } => {
                let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) else {
                    return warn!("Could not find WebView for Epoch update.");
                };
                webview_renderer
                    .ensure_pipeline_details(pipeline_id)
                    .display_list_epoch = Some(Epoch(epoch.0));
            },

            CompositorMsg::SendDisplayList {
                webview_id,
                display_list_descriptor,
                display_list_receiver,
            } => {
                if !self.webview_renderers.is_shown(webview_id) {
                    return debug!("Ignoring display list for hidden webview {:?}", webview_id);
                }
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

                let old_scale = webview_renderer.device_pixels_per_page_pixel();

                let pipeline_id = display_list_info.pipeline_id;
                let details = webview_renderer.ensure_pipeline_details(pipeline_id.into());
                details.install_new_scroll_tree(display_list_info.scroll_tree);
                details.viewport_scale =
                    Some(display_list_info.viewport_details.hidpi_scale_factor);

                let epoch = display_list_info.epoch.into();
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
                let is_root_pipeline =
                    Some(pipeline_id.into()) == webview_renderer.root_pipeline_id;
                if is_root_pipeline && old_scale != webview_renderer.device_pixels_per_page_pixel()
                {
                    self.send_root_pipeline_display_list_in_transaction(&mut transaction);
                }

                transaction.set_display_list(epoch, (pipeline_id, built_display_list));
                self.update_transaction_with_all_scroll_offsets(&mut transaction);
                self.global.borrow_mut().send_transaction(transaction);
            },

            CompositorMsg::GenerateFrame => {
                let mut global = self.global.borrow_mut();
                global.frame_delayer.set_pending_frame(true);

                if !global.frame_delayer.needs_new_frame() {
                    return;
                }

                let mut transaction = Transaction::new();
                self.generate_frame(&mut transaction, RenderReasons::SCENE);
                global.send_transaction(transaction);

                let waiting_pipelines = global.frame_delayer.take_waiting_pipelines();
                let _ = global.constellation_sender.send(
                    EmbedderToConstellationMessage::NoLongerWaitingOnAsynchronousImageUpdates(
                        waiting_pipelines,
                    ),
                );
                global.frame_delayer.set_pending_frame(false);
                self.screenshot_taker
                    .prepare_screenshot_requests_for_render(self)
            },

            CompositorMsg::GenerateImageKey(sender) => {
                let _ = sender.send(self.global.borrow().webrender_api.generate_image_key());
            },

            CompositorMsg::GenerateImageKeysForPipeline(pipeline_id) => {
                let image_keys = (0..pref!(image_key_batch_size))
                    .map(|_| self.global.borrow().webrender_api.generate_image_key())
                    .collect();
                if let Err(error) = self.global.borrow().constellation_sender.send(
                    EmbedderToConstellationMessage::SendImageKeysForPipeline(
                        pipeline_id,
                        image_keys,
                    ),
                ) {
                    warn!("Sending Image Keys to Constellation failed with({error:?}).");
                }
            },
            CompositorMsg::UpdateImages(updates) => {
                let mut global = self.global.borrow_mut();
                let mut txn = Transaction::new();
                for update in updates {
                    match update {
                        ImageUpdate::AddImage(key, desc, data) => {
                            txn.add_image(key, desc, data.into(), None)
                        },
                        ImageUpdate::DeleteImage(key) => {
                            txn.delete_image(key);
                            global.frame_delayer.delete_image(key);
                        },
                        ImageUpdate::UpdateImage(key, desc, data, epoch) => {
                            if let Some(epoch) = epoch {
                                global.frame_delayer.update_image(key, epoch);
                            }
                            txn.update_image(key, desc, data.into(), &DirtyRect::All)
                        },
                    }
                }

                if global.frame_delayer.needs_new_frame() {
                    global.frame_delayer.set_pending_frame(false);
                    self.generate_frame(&mut txn, RenderReasons::SCENE);
                    let waiting_pipelines = global.frame_delayer.take_waiting_pipelines();
                    let _ = global.constellation_sender.send(
                        EmbedderToConstellationMessage::NoLongerWaitingOnAsynchronousImageUpdates(
                            waiting_pipelines,
                        ),
                    );
                    self.screenshot_taker
                        .prepare_screenshot_requests_for_render(self);
                }

                global.send_transaction(txn);
            },

            CompositorMsg::DelayNewFrameForCanvas(pipeline_id, canvas_epoch, image_keys) => self
                .global
                .borrow_mut()
                .frame_delayer
                .add_delay(pipeline_id, canvas_epoch, image_keys),

            CompositorMsg::AddFont(font_key, data, index) => {
                self.add_font(font_key, index, data);
            },

            CompositorMsg::AddSystemFont(font_key, native_handle) => {
                let mut transaction = Transaction::new();
                transaction.add_native_font(font_key, native_handle);
                self.global.borrow_mut().send_transaction(transaction);
            },

            CompositorMsg::AddFontInstance(
                font_instance_key,
                font_key,
                size,
                flags,
                variations,
            ) => {
                self.add_font_instance(font_instance_key, font_key, size, flags, variations);
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

            CompositorMsg::GenerateFontKeys(
                number_of_font_keys,
                number_of_font_instance_keys,
                result_sender,
                rendering_group_id,
            ) => {
                self.handle_generate_font_keys(
                    number_of_font_keys,
                    number_of_font_instance_keys,
                    result_sender,
                    rendering_group_id,
                );
            },
            CompositorMsg::Viewport(webview_id, viewport_description) => {
                if let Some(webview) = self.webview_renderers.get_mut(webview_id) {
                    webview.set_viewport_description(viewport_description);
                }
            },
            CompositorMsg::ScreenshotReadinessReponse(webview_id, pipelines_and_epochs) => {
                self.screenshot_taker.handle_screenshot_readiness_reply(
                    webview_id,
                    pipelines_and_epochs,
                    self,
                );
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
                rendering_group_id,
            ) => {
                self.handle_generate_font_keys(
                    number_of_font_keys,
                    number_of_font_instance_keys,
                    result_sender,
                    rendering_group_id,
                );
            },
            _ => {
                debug!("Ignoring message ({:?} while shutting down", msg);
            },
        }
    }

    /// Generate the font keys and send them to the `result_sender`.
    /// Currently `RenderingGroupId` is not used.
    fn handle_generate_font_keys(
        &self,
        number_of_font_keys: usize,
        number_of_font_instance_keys: usize,
        result_sender: GenericSender<(Vec<FontKey>, Vec<FontInstanceKey>)>,
        _rendering_group_id: RenderingGroupId,
    ) {
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
    }

    /// Queue a new frame in the transaction and increase the pending frames count.
    pub(crate) fn generate_frame(&self, transaction: &mut Transaction, reason: RenderReasons) {
        transaction.generate_frame(0, true /* present */, false /* tracked */, reason);
        self.pending_frames.set(self.pending_frames.get() + 1);
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

    pub fn on_zoom_window_event(&mut self, webview_id: WebViewId, new_zoom: f32) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }

        if let Some(webview_renderer) = self.webview_renderers.get_mut(webview_id) {
            webview_renderer.set_page_zoom(Scale::new(new_zoom));
        }
    }

    /// Returns true if any animation callbacks (ie `requestAnimationFrame`) are waiting for a response.
    fn animation_callbacks_running(&self) -> bool {
        self.webview_renderers
            .iter()
            .any(WebViewRenderer::animation_callbacks_running)
    }

    /// Render the WebRender scene to the active `RenderingContext`.
    pub fn render(&mut self) {
        self.global
            .borrow()
            .refresh_driver
            .notify_will_paint(self.webview_renderers.iter());

        self.render_inner();

        // We've painted the default target, which means that from the embedder's perspective,
        // the scene no longer needs to be repainted.
        self.needs_repaint.set(RepaintReason::empty());
    }

    #[servo_tracing::instrument(skip_all)]
    fn render_inner(&mut self) {
        if let Err(err) = self.rendering_context.make_current() {
            warn!("Failed to make the rendering context current: {:?}", err);
        }
        self.assert_no_gl_error();

        if let Some(webrender) = self.webrender.as_mut() {
            webrender.update();
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
        self.screenshot_taker.maybe_take_screenshots(self);
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
                        #[cfg(feature = "tracing")]
                        tracing::info!(
                            name: "FirstPaint",
                            servo_profiling = true,
                            epoch = ?epoch,
                            paint_time = ?paint_time,
                            pipeline_id = ?pipeline_id,
                        );
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
                        #[cfg(feature = "tracing")]
                        tracing::info!(
                            name: "FirstContentfulPaint",
                            servo_profiling = true,
                            epoch = ?epoch,
                            paint_time = ?paint_time,
                            pipeline_id = ?pipeline_id,
                        );
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
    pub fn receiver(&self) -> Ref<'_, RoutedReceiver<CompositorMsg>> {
        Ref::map(self.global.borrow(), |global| &global.compositor_receiver)
    }

    #[servo_tracing::instrument(skip_all)]
    pub fn handle_messages(&mut self, mut messages: Vec<CompositorMsg>) {
        // Pull out the `NewWebRenderFrameReady` messages from the list of messages and handle them
        // at the end of this function. This prevents overdraw when more than a single message of
        // this type of received. In addition, if any of these frames need a repaint, that reflected
        // when calling `handle_new_webrender_frame_ready`.
        let mut repaint_needed = false;
        let mut saw_webrender_frame_ready = false;

        messages.retain(|message| match message {
            CompositorMsg::NewWebRenderFrameReady(_, need_repaint) => {
                self.pending_frames.set(self.pending_frames.get() - 1);
                repaint_needed |= need_repaint;
                saw_webrender_frame_ready = true;

                false
            },
            _ => true,
        });

        for message in messages {
            self.handle_browser_message(message);
            if self.global.borrow().shutdown_state() == ShutdownState::FinishedShuttingDown {
                return;
            }
        }

        if saw_webrender_frame_ready {
            self.handle_new_webrender_frame_ready(repaint_needed);
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
                transaction.set_scroll_offsets(
                    update.external_scroll_id,
                    vec![SampledScrollOffset {
                        offset: update.offset,
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
        variations: Vec<FontVariation>,
    ) {
        let variations = if pref!(layout_variable_fonts_enabled) {
            variations
        } else {
            vec![]
        };

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
            variations,
        );

        self.global.borrow_mut().send_transaction(transaction);
    }

    fn add_font(&mut self, font_key: FontKey, index: u32, data: Arc<IpcSharedMemory>) {
        let mut transaction = Transaction::new();
        transaction.add_raw_font(font_key, (**data).into(), index);
        self.global.borrow_mut().send_transaction(transaction);
    }

    pub fn notify_input_event(&mut self, webview_id: WebViewId, event: InputEventAndId) {
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

    pub fn device_pixels_per_page_pixel(
        &self,
        webview_id: WebViewId,
    ) -> Scale<f32, CSSPixel, DevicePixel> {
        self.webview_renderers
            .get(webview_id)
            .map(WebViewRenderer::device_pixels_per_page_pixel)
            .unwrap_or_default()
    }

    fn webrender_document(&self) -> DocumentId {
        self.global.borrow().webrender_document
    }

    fn shutdown_state(&self) -> ShutdownState {
        self.global.borrow().shutdown_state()
    }

    fn refresh_cursor(&self) {
        let global = self.global.borrow();
        let Some(last_mouse_move_position) = global.last_mouse_move_position else {
            return;
        };

        let Some(hit_test_result) = global
            .hit_test_at_point(last_mouse_move_position)
            .first()
            .cloned()
        else {
            return;
        };

        if let Err(error) =
            global
                .constellation_sender
                .send(EmbedderToConstellationMessage::RefreshCursor(
                    hit_test_result.pipeline_id,
                ))
        {
            warn!("Sending event to constellation failed ({:?}).", error);
        }
    }

    fn handle_new_webrender_frame_ready(&mut self, repaint_needed: bool) {
        if repaint_needed {
            self.refresh_cursor();
        }
        if repaint_needed || self.animation_callbacks_running() {
            self.set_needs_repaint(RepaintReason::NewWebRenderFrame);
        }

        // If we received a new frame and a repaint isn't necessary, it may be that this
        // is the last frame that was pending. In that case, trigger a manual repaint so
        // that the screenshot can be taken at the end of the repaint procedure.
        if !repaint_needed {
            self.screenshot_taker
                .maybe_trigger_paint_for_screenshot(self);
        }
    }

    /// Whether or not the renderer is waiting on a frame, either because it has been sent
    /// to WebRender and is not ready yet or because the [`FrameDelayer`] is delaying a frame
    /// waiting for asynchronous (canvas) image updates to complete.
    pub(crate) fn has_pending_frames(&self) -> bool {
        self.pending_frames.get() != 0 || self.global.borrow().frame_delayer.pending_frame
    }

    pub fn request_screenshot(
        &self,
        webview_id: WebViewId,
        rect: Option<DeviceRect>,
        callback: Box<dyn FnOnce(Result<RgbaImage, ScreenshotCaptureError>) + 'static>,
    ) {
        self.screenshot_taker
            .request_screenshot(webview_id, rect, callback);
        let _ = self.global.borrow().constellation_sender.send(
            EmbedderToConstellationMessage::RequestScreenshotReadiness(webview_id),
        );
    }
}

/// A struct that is reponsible for delaying frame requests until all new canvas images
/// for a particular "update the rendering" call in the `ScriptThread` have been
/// sent to WebRender.
///
/// These images may be updated in WebRender asynchronously in the canvas task. A frame
/// is then requested if:
///
///  - The renderer has received a GenerateFrame message from a `ScriptThread`.
///  - All pending image updates have finished and have been noted in the [`FrameDelayer`].
#[derive(Default)]
struct FrameDelayer {
    /// The latest [`Epoch`] of canvas images that have been sent to WebRender. Note
    /// that this only records the `Epoch`s for canvases and only ones that are involved
    /// in "update the rendering".
    image_epochs: FxHashMap<ImageKey, Epoch>,
    /// A map of all pending canvas images
    pending_canvas_images: FxHashMap<ImageKey, Epoch>,
    /// Whether or not we have a pending frame.
    pending_frame: bool,
    /// A list of pipelines that should be notified when we are no longer waiting for
    /// canvas images.
    waiting_pipelines: FxHashSet<PipelineId>,
}

impl FrameDelayer {
    fn delete_image(&mut self, image_key: ImageKey) {
        self.image_epochs.remove(&image_key);
        self.pending_canvas_images.remove(&image_key);
    }

    fn update_image(&mut self, image_key: ImageKey, epoch: Epoch) {
        self.image_epochs.insert(image_key, epoch);
        let Entry::Occupied(entry) = self.pending_canvas_images.entry(image_key) else {
            return;
        };
        if *entry.get() <= epoch {
            entry.remove();
        }
    }

    fn add_delay(
        &mut self,
        pipeline_id: PipelineId,
        canvas_epoch: Epoch,
        image_keys: Vec<ImageKey>,
    ) {
        for image_key in image_keys.into_iter() {
            // If we've already seen the necessary epoch for this image, do not
            // start waiting for it.
            if self
                .image_epochs
                .get(&image_key)
                .is_some_and(|epoch_seen| *epoch_seen >= canvas_epoch)
            {
                continue;
            }
            self.pending_canvas_images.insert(image_key, canvas_epoch);
        }
        self.waiting_pipelines.insert(pipeline_id);
    }

    fn needs_new_frame(&self) -> bool {
        self.pending_frame && self.pending_canvas_images.is_empty()
    }

    fn set_pending_frame(&mut self, value: bool) {
        self.pending_frame = value;
    }

    fn take_waiting_pipelines(&mut self) -> Vec<PipelineId> {
        self.waiting_pipelines.drain().collect()
    }
}
