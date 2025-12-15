/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::env;
use std::fs::create_dir_all;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use base::generic_channel::{GenericSender, RoutedReceiver};
use base::id::{PainterId, PipelineId, WebViewId};
use bitflags::bitflags;
use canvas_traits::webgl::{WebGLContextId, WebGLThreads};
use compositing_traits::rendering_context::RenderingContext;
use compositing_traits::{
    PaintMessage, PaintProxy, PainterSurfmanDetails, PainterSurfmanDetailsMap,
    WebRenderExternalImageIdManager, WebViewTrait,
};
use constellation_traits::EmbedderToConstellationMessage;
use crossbeam_channel::Sender;
use dpi::PhysicalSize;
use embedder_traits::{
    EventLoopWaker, InputEventAndId, InputEventId, InputEventResult, ScreenshotCaptureError,
    Scroll, ShutdownState, ViewportDetails, WebViewPoint, WebViewRect,
};
use euclid::{Scale, Size2D};
use image::RgbaImage;
use ipc_channel::ipc::{self};
use log::{debug, warn};
use profile_traits::mem::{
    ProcessReports, ProfilerRegistration, Report, ReportKind, perform_memory_report,
};
use profile_traits::path;
use profile_traits::time::{self as profile_time};
use servo_config::pref;
use servo_geometry::DeviceIndependentPixel;
use style_traits::CSSPixel;
use surfman::Device;
use surfman::chains::SwapChains;
use webgl::WebGLComm;
use webgl::webgl_thread::WebGLContextBusyMap;
#[cfg(feature = "webgpu")]
use webgpu::canvas_context::WebGpuExternalImageMap;
use webrender::{CaptureBits, MemoryReport};
use webrender_api::units::{DevicePixel, DevicePoint};
use webrender_api::{FontInstanceKey, FontKey, ImageKey};

use crate::InitialPaintState;
use crate::painter::Painter;
use crate::webview_renderer::UnknownWebView;

/// An option to control what kind of WebRender debugging is enabled while Servo is running.
#[derive(Copy, Clone)]
pub enum WebRenderDebugOption {
    Profiler,
    TextureCacheDebug,
    RenderTargetDebug,
}

/// [`Paint`] is Servo's rendering subsystem. It has a few responsibilities:
///
/// 1. Maintain a WebRender instance for each [`RenderingContext`] that Servo knows about.
///    [`RenderingContext`]s are per-`WebView`, but more than one `WebView` can use the same
///    [`RenderingContext`]. This allows multiple `WebView`s to share the same WebRender
///    instance which is more efficient. This is useful for tabbed web browsers.
/// 2. Receive display lists from the layout of all of the currently active `Pipeline`s
///    (frames). These display lists are sent to WebRender, and new frames are generated.
///    Once the frame is ready the [`Painter`] for the WebRender instance will ask libservo
///    to inform the embedder that a new frame is ready so that it can trigger a paint.
/// 3. Drive animation and animation callback updates. Animation updates should ideally be
///    coordinated with the system vsync signal, so the `RefreshDriver` is exposed in the
///    API to allow the embedder to do this. The [`Painter`] then asks its `WebView`s to
///    update their rendering, which triggers layouts.
/// 4. Eagerly handle scrolling and touch events. In order to avoid latency when handling
///    these kind of actions, each [`Painter`] will eagerly process touch events and
///    perform panning and zooming operations on their WebRender contents -- informing the
///    WebView contents asynchronously.
///
/// `Paint` and all of its contained structs should **never** block on the Constellation,
/// because sometimes the Constellation blocks on us.
pub struct Paint {
    /// All of the [`Painters`] for this [`Paint`]. Each [`Painter`] handles painting to
    /// a single [`RenderingContext`].
    painters: Vec<Rc<RefCell<Painter>>>,

    /// A [`PaintProxy`] which can be used to allow other parts of Servo to communicate
    /// with this [`Paint`].
    pub(crate) paint_proxy: PaintProxy,

    /// An [`EventLoopWaker`] used to wake up the main embedder event loop when the renderer needs
    /// to run.
    pub(crate) event_loop_waker: Box<dyn EventLoopWaker>,

    /// Tracks whether we are in the process of shutting down, or have shut down and
    /// should shut down `Paint`. This is shared with the `Servo` instance.
    shutdown_state: Rc<Cell<ShutdownState>>,

    /// The port on which we receive messages.
    paint_receiver: RoutedReceiver<PaintMessage>,

    /// The channel on which messages can be sent to the constellation.
    pub(crate) embedder_to_constellation_sender: Sender<EmbedderToConstellationMessage>,

    /// The [`WebRenderExternalImageIdManager`] used to generate new `ExternalImageId`s.
    webrender_external_image_id_manager: WebRenderExternalImageIdManager,

    /// A [`HashMap`] of [`PainterId`] to the Surfaman types (`Device`, `Adapter`) that
    /// are specific to a particular [`Painter`].
    pub(crate) painter_surfman_details_map: PainterSurfmanDetailsMap,

    /// A [`HashMap`] of `WebGLContextId` to a usage count. This count indicates when
    /// WebRender is still rendering the context. This is used to ensure properly clean
    /// up of all Surfman `Surface`s.
    pub(crate) busy_webgl_contexts_map: WebGLContextBusyMap,

    /// The [`WebGLThreads`] for this renderer.
    webgl_threads: WebGLThreads,

    /// The shared [`SwapChains`] used by [`WebGLThreads`] for this renderer.
    pub(crate) swap_chains: SwapChains<WebGLContextId, Device>,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: profile_time::ProfilerChan,

    /// A handle to the memory profiler which will automatically unregister
    /// when it's dropped.
    _mem_profiler_registration: ProfilerRegistration,

    /// Some XR devices want to run on the main thread.
    #[cfg(feature = "webxr")]
    webxr_main_thread: RefCell<webxr::MainThreadRegistry>,

    /// An map of external images shared between all `WebGpuExternalImages`.
    #[cfg(feature = "webgpu")]
    webgpu_image_map: std::cell::OnceCell<WebGpuExternalImageMap>,
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
        /// A fling has started and a repaint needs to happen to process the animation.
        const StartedFlinging = 1 << 4;
    }
}

impl Paint {
    pub fn new(state: InitialPaintState) -> Rc<RefCell<Self>> {
        let registration = state.mem_profiler_chan.prepare_memory_reporting(
            "paint".into(),
            state.paint_proxy.clone(),
            PaintMessage::CollectMemoryReport,
        );

        let webrender_external_image_id_manager = WebRenderExternalImageIdManager::default();
        let painter_surfman_details_map = PainterSurfmanDetailsMap::default();
        let WebGLComm {
            webgl_threads,
            swap_chains,
            busy_webgl_context_map,
            #[cfg(feature = "webxr")]
            webxr_layer_grand_manager,
        } = WebGLComm::new(
            state.paint_proxy.cross_process_paint_api.clone(),
            webrender_external_image_id_manager.clone(),
            painter_surfman_details_map.clone(),
        );

        // Create the WebXR main thread
        #[cfg(feature = "webxr")]
        let webxr_main_thread = {
            use servo_config::pref;

            let mut webxr_main_thread = webxr::MainThreadRegistry::new(
                state.event_loop_waker.clone(),
                webxr_layer_grand_manager,
            )
            .expect("Failed to create WebXR device registry");
            if pref!(dom_webxr_enabled) {
                state.webxr_registry.register(&mut webxr_main_thread);
            }
            webxr_main_thread
        };

        Rc::new(RefCell::new(Paint {
            painters: Default::default(),
            paint_proxy: state.paint_proxy,
            event_loop_waker: state.event_loop_waker,
            shutdown_state: state.shutdown_state,
            paint_receiver: state.receiver,
            embedder_to_constellation_sender: state.embedder_to_constellation_sender.clone(),
            webrender_external_image_id_manager,
            webgl_threads,
            swap_chains,
            time_profiler_chan: state.time_profiler_chan,
            _mem_profiler_registration: registration,
            painter_surfman_details_map,
            busy_webgl_contexts_map: busy_webgl_context_map,
            #[cfg(feature = "webxr")]
            webxr_main_thread: RefCell::new(webxr_main_thread),
            #[cfg(feature = "webgpu")]
            webgpu_image_map: Default::default(),
        }))
    }

    pub fn register_rendering_context(
        &mut self,
        rendering_context: Rc<dyn RenderingContext>,
    ) -> PainterId {
        if let Some(painter_id) = self.painters.iter().find_map(|painter| {
            let painter = painter.borrow();
            if Rc::ptr_eq(&painter.rendering_context, &rendering_context) {
                Some(painter.painter_id)
            } else {
                None
            }
        }) {
            return painter_id;
        }

        let painter = Painter::new(rendering_context.clone(), self);
        let connection = rendering_context
            .connection()
            .expect("Failed to get connection");
        let adapter = connection
            .create_adapter()
            .expect("Failed to create adapter");

        let painter_surfman_details = PainterSurfmanDetails {
            connection,
            adapter,
        };
        self.painter_surfman_details_map
            .insert(painter.painter_id, painter_surfman_details);

        let painter_id = painter.painter_id;
        self.painters.push(Rc::new(RefCell::new(painter)));
        painter_id
    }

    fn remove_painter(&mut self, painter_id: PainterId) {
        self.painters
            .retain(|painter| painter.borrow().painter_id != painter_id);
        self.painter_surfman_details_map.remove(painter_id);
    }

    pub(crate) fn maybe_painter<'a>(&'a self, painter_id: PainterId) -> Option<Ref<'a, Painter>> {
        self.painters
            .iter()
            .map(|painter| painter.borrow())
            .find(|painter| painter.painter_id == painter_id)
    }

    pub(crate) fn painter<'a>(&'a self, painter_id: PainterId) -> Ref<'a, Painter> {
        self.maybe_painter(painter_id)
            .expect("painter_id not found")
    }

    pub(crate) fn maybe_painter_mut<'a>(
        &'a self,
        painter_id: PainterId,
    ) -> Option<RefMut<'a, Painter>> {
        self.painters
            .iter()
            .map(|painter| painter.borrow_mut())
            .find(|painter| painter.painter_id == painter_id)
    }

    pub(crate) fn painter_mut<'a>(&'a self, painter_id: PainterId) -> RefMut<'a, Painter> {
        self.maybe_painter_mut(painter_id)
            .expect("painter_id not found")
    }

    pub fn painter_id(&self) -> PainterId {
        self.painters[0].borrow().painter_id
    }

    pub fn rendering_context_size(&self, painter_id: PainterId) -> Size2D<u32, DevicePixel> {
        self.painter(painter_id).rendering_context.size2d()
    }

    pub fn webgl_threads(&self) -> WebGLThreads {
        self.webgl_threads.clone()
    }

    pub fn webrender_external_image_id_manager(&self) -> WebRenderExternalImageIdManager {
        self.webrender_external_image_id_manager.clone()
    }

    pub fn webxr_running(&self) -> bool {
        #[cfg(feature = "webxr")]
        {
            self.webxr_main_thread.borrow().running()
        }
        #[cfg(not(feature = "webxr"))]
        {
            false
        }
    }

    #[cfg(feature = "webxr")]
    pub fn webxr_main_thread_registry(&self) -> webxr_api::Registry {
        self.webxr_main_thread.borrow().registry()
    }

    #[cfg(feature = "webgpu")]
    pub fn webgpu_image_map(&self) -> WebGpuExternalImageMap {
        self.webgpu_image_map.get_or_init(Default::default).clone()
    }

    pub fn webviews_needing_repaint(&self) -> Vec<WebViewId> {
        self.painters
            .iter()
            .flat_map(|painter| painter.borrow().webviews_needing_repaint())
            .collect()
    }

    pub fn finish_shutting_down(&self) {
        // Drain paint port, sometimes messages contain channels that are blocking
        // another thread from finishing (i.e. SetFrameTree).
        while self.paint_receiver.try_recv().is_ok() {}

        let (webgl_exit_sender, webgl_exit_receiver) =
            ipc::channel().expect("Failed to create IPC channel!");
        if !self
            .webgl_threads
            .exit(webgl_exit_sender)
            .is_ok_and(|_| webgl_exit_receiver.recv().is_ok())
        {
            warn!("Could not exit WebGLThread.");
        }

        // Tell the profiler, memory profiler, and scrolling timer to shut down.
        if let Ok((sender, receiver)) = ipc::channel() {
            self.time_profiler_chan
                .send(profile_time::ProfilerMsg::Exit(sender));
            let _ = receiver.recv();
        }
    }

    fn handle_browser_message(&self, msg: PaintMessage) {
        trace_msg_from_constellation!(msg, "{msg:?}");

        match self.shutdown_state() {
            ShutdownState::NotShuttingDown => {},
            ShutdownState::ShuttingDown => {
                self.handle_browser_message_while_shutting_down(msg);
                return;
            },
            ShutdownState::FinishedShuttingDown => {
                // Messages to Paint are ignored after shutdown is complete.
                return;
            },
        }

        match msg {
            PaintMessage::CollectMemoryReport(sender) => {
                self.collect_memory_report(sender);
            },
            PaintMessage::ChangeRunningAnimationsState(
                webview_id,
                pipeline_id,
                animation_state,
            ) => {
                if let Some(mut painter) = self.maybe_painter_mut(webview_id.into()) {
                    painter.change_running_animations_state(
                        webview_id,
                        pipeline_id,
                        animation_state,
                    );
                }
            },
            PaintMessage::SetFrameTreeForWebView(webview_id, frame_tree) => {
                if let Some(mut painter) = self.maybe_painter_mut(webview_id.into()) {
                    painter.set_frame_tree_for_webview(&frame_tree);
                }
            },
            PaintMessage::SetThrottled(webview_id, pipeline_id, throttled) => {
                if let Some(mut painter) = self.maybe_painter_mut(webview_id.into()) {
                    painter.set_throttled(webview_id, pipeline_id, throttled);
                }
            },
            PaintMessage::PipelineExited(webview_id, pipeline_id, pipeline_exit_source) => {
                if let Some(mut painter) = self.maybe_painter_mut(webview_id.into()) {
                    painter.notify_pipeline_exited(webview_id, pipeline_id, pipeline_exit_source);
                }
            },
            PaintMessage::NewWebRenderFrameReady(..) => {
                unreachable!("New WebRender frames should be handled in the caller.");
            },
            PaintMessage::SendInitialTransaction(webview_id, pipeline_id) => {
                if let Some(mut painter) = self.maybe_painter_mut(webview_id.into()) {
                    painter.send_initial_pipeline_transaction(webview_id, pipeline_id);
                }
            },
            PaintMessage::ScrollNodeByDelta(
                webview_id,
                pipeline_id,
                offset,
                external_scroll_id,
            ) => {
                if let Some(mut painter) = self.maybe_painter_mut(webview_id.into()) {
                    painter.scroll_node_by_delta(
                        webview_id,
                        pipeline_id,
                        offset,
                        external_scroll_id,
                    );
                }
            },
            PaintMessage::ScrollViewportByDelta(webview_id, delta) => {
                if let Some(mut painter) = self.maybe_painter_mut(webview_id.into()) {
                    painter.scroll_viewport_by_delta(webview_id, delta);
                }
            },
            PaintMessage::UpdateEpoch {
                webview_id,
                pipeline_id,
                epoch,
            } => {
                if let Some(mut painter) = self.maybe_painter_mut(webview_id.into()) {
                    painter.update_epoch(webview_id, pipeline_id, epoch);
                }
            },
            PaintMessage::SendDisplayList {
                webview_id,
                display_list_descriptor,
                display_list_info_receiver,
                display_list_data_receiver,
            } => {
                if let Some(mut painter) = self.maybe_painter_mut(webview_id.into()) {
                    painter.handle_new_display_list(
                        webview_id,
                        display_list_descriptor,
                        display_list_info_receiver,
                        display_list_data_receiver,
                    );
                }
            },
            PaintMessage::GenerateFrame(painter_ids) => {
                for painter_id in painter_ids {
                    if let Some(mut painter) = self.maybe_painter_mut(painter_id) {
                        painter.generate_frame_for_script();
                    }
                }
            },
            PaintMessage::GenerateImageKey(webview_id, result_sender) => {
                self.handle_generate_image_key(webview_id, result_sender);
            },
            PaintMessage::GenerateImageKeysForPipeline(webview_id, pipeline_id) => {
                self.handle_generate_image_keys_for_pipeline(webview_id, pipeline_id);
            },
            PaintMessage::UpdateImages(painter_id, updates) => {
                if let Some(mut painter) = self.maybe_painter_mut(painter_id) {
                    painter.update_images(updates);
                }
            },
            PaintMessage::DelayNewFrameForCanvas(
                webview_id,
                pipeline_id,
                canvas_epoch,
                image_keys,
            ) => {
                if let Some(mut painter) = self.maybe_painter_mut(webview_id.into()) {
                    painter.delay_new_frames_for_canvas(pipeline_id, canvas_epoch, image_keys);
                }
            },
            PaintMessage::AddFont(painter_id, font_key, data, index) => {
                debug_assert!(painter_id == font_key.into());

                if let Some(mut painter) = self.maybe_painter_mut(painter_id) {
                    painter.add_font(font_key, data, index);
                }
            },
            PaintMessage::AddSystemFont(painter_id, font_key, native_handle) => {
                debug_assert!(painter_id == font_key.into());

                if let Some(mut painter) = self.maybe_painter_mut(painter_id) {
                    painter.add_system_font(font_key, native_handle);
                }
            },
            PaintMessage::AddFontInstance(
                painter_id,
                font_instance_key,
                font_key,
                size,
                flags,
                variations,
            ) => {
                debug_assert!(painter_id == font_key.into());
                debug_assert!(painter_id == font_instance_key.into());

                if let Some(mut painter) = self.maybe_painter_mut(painter_id) {
                    painter.add_font_instance(font_instance_key, font_key, size, flags, variations);
                }
            },
            PaintMessage::RemoveFonts(painter_id, keys, instance_keys) => {
                if let Some(mut painter) = self.maybe_painter_mut(painter_id) {
                    painter.remove_fonts(keys, instance_keys);
                }
            },
            PaintMessage::GenerateFontKeys(
                number_of_font_keys,
                number_of_font_instance_keys,
                result_sender,
                painter_id,
            ) => {
                self.handle_generate_font_keys(
                    number_of_font_keys,
                    number_of_font_instance_keys,
                    result_sender,
                    painter_id,
                );
            },
            PaintMessage::Viewport(webview_id, viewport_description) => {
                if let Some(mut painter) = self.maybe_painter_mut(webview_id.into()) {
                    painter.set_viewport_description(webview_id, viewport_description);
                }
            },
            PaintMessage::ScreenshotReadinessReponse(webview_id, pipelines_and_epochs) => {
                if let Some(painter) = self.maybe_painter(webview_id.into()) {
                    painter.handle_screenshot_readiness_reply(webview_id, pipelines_and_epochs);
                }
            },
            PaintMessage::SendLCPCandidate(lcp_candidate, webview_id, pipeline_id, epoch) => {
                if let Some(mut painter) = self.maybe_painter_mut(webview_id.into()) {
                    painter.append_lcp_candidate(lcp_candidate, webview_id, pipeline_id, epoch);
                }
            },
        }
    }

    pub fn remove_webview(&mut self, webview_id: WebViewId) {
        let painter_id = webview_id.into();

        {
            let mut painter = self.painter_mut(painter_id);
            painter.remove_webview(webview_id);
            if !painter.is_empty() {
                return;
            }
        }

        self.remove_painter(painter_id);
    }

    fn collect_memory_report(&self, sender: profile_traits::mem::ReportsChan) {
        let mut memory_report = MemoryReport::default();
        for painter in &self.painters {
            memory_report += painter.borrow().report_memory();
        }

        let mut reports = vec![
            Report {
                path: path!["webrender", "fonts"],
                kind: ReportKind::ExplicitJemallocHeapSize,
                size: memory_report.fonts,
            },
            Report {
                path: path!["webrender", "images"],
                kind: ReportKind::ExplicitJemallocHeapSize,
                size: memory_report.images,
            },
            Report {
                path: path!["webrender", "display-list"],
                kind: ReportKind::ExplicitJemallocHeapSize,
                size: memory_report.display_list,
            },
        ];

        perform_memory_report(|ops| {
            let scroll_trees_memory_usage = self
                .painters
                .iter()
                .map(|painter| painter.borrow().scroll_trees_memory_usage(ops))
                .sum();
            reports.push(Report {
                path: path!["paint", "scroll-tree"],
                kind: ReportKind::ExplicitJemallocHeapSize,
                size: scroll_trees_memory_usage,
            });
        });

        sender.send(ProcessReports::new(reports));
    }

    /// Handle messages sent to `Paint` during the shutdown process. In general,
    /// the things `Paint` can do in this state are limited. It's very important to
    /// answer any synchronous messages though as other threads might be waiting on the
    /// results to finish their own shut down process. We try to do as little as possible
    /// during this time.
    ///
    /// When that involves generating WebRender ids, our approach here is to simply
    /// generate them, but assume they will never be used, since once shutting down
    /// `Paint` no longer does any WebRender frame generation.
    fn handle_browser_message_while_shutting_down(&self, msg: PaintMessage) {
        match msg {
            PaintMessage::PipelineExited(webview_id, pipeline_id, pipeline_exit_source) => {
                if let Some(mut painter) = self.maybe_painter_mut(webview_id.into()) {
                    painter.notify_pipeline_exited(webview_id, pipeline_id, pipeline_exit_source);
                }
            },
            PaintMessage::GenerateImageKey(webview_id, result_sender) => {
                self.handle_generate_image_key(webview_id, result_sender);
            },
            PaintMessage::GenerateImageKeysForPipeline(webview_id, pipeline_id) => {
                self.handle_generate_image_keys_for_pipeline(webview_id, pipeline_id);
            },
            PaintMessage::GenerateFontKeys(
                number_of_font_keys,
                number_of_font_instance_keys,
                result_sender,
                painter_id,
            ) => {
                self.handle_generate_font_keys(
                    number_of_font_keys,
                    number_of_font_instance_keys,
                    result_sender,
                    painter_id,
                );
            },
            _ => {
                debug!("Ignoring message ({:?} while shutting down", msg);
            },
        }
    }

    pub fn add_webview(&self, webview: Box<dyn WebViewTrait>, viewport_details: ViewportDetails) {
        self.painter_mut(webview.id().into())
            .add_webview(webview, viewport_details);
    }

    pub fn show_webview(&self, webview_id: WebViewId) -> Result<(), UnknownWebView> {
        self.painter_mut(webview_id.into())
            .set_webview_hidden(webview_id, false)
    }

    pub fn hide_webview(&self, webview_id: WebViewId) -> Result<(), UnknownWebView> {
        self.painter_mut(webview_id.into())
            .set_webview_hidden(webview_id, true)
    }

    pub fn set_hidpi_scale_factor(
        &self,
        webview_id: WebViewId,
        new_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    ) {
        if self.shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        self.painter_mut(webview_id.into())
            .set_hidpi_scale_factor(webview_id, new_scale_factor);
    }

    pub fn resize_rendering_context(&self, webview_id: WebViewId, new_size: PhysicalSize<u32>) {
        if self.shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        self.painter_mut(webview_id.into())
            .resize_rendering_context(new_size);
    }

    pub fn set_page_zoom(&self, webview_id: WebViewId, new_zoom: f32) {
        if self.shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        self.painter_mut(webview_id.into())
            .set_page_zoom(webview_id, new_zoom);
    }

    pub fn page_zoom(&self, webview_id: WebViewId) -> f32 {
        self.painter(webview_id.into()).page_zoom(webview_id)
    }

    /// Render the WebRender scene to the active `RenderingContext`.
    pub fn render(&self, webview_id: WebViewId) {
        self.painter_mut(webview_id.into())
            .render(&self.time_profiler_chan);
    }

    /// Get the message receiver for this [`Paint`].
    pub fn receiver(&self) -> &RoutedReceiver<PaintMessage> {
        &self.paint_receiver
    }

    #[servo_tracing::instrument(skip_all)]
    pub fn handle_messages(&self, mut messages: Vec<PaintMessage>) {
        // Pull out the `NewWebRenderFrameReady` messages from the list of messages and handle them
        // at the end of this function. This prevents overdraw when more than a single message of
        // this type of received. In addition, if any of these frames need a repaint, that reflected
        // when calling `handle_new_webrender_frame_ready`.
        let mut saw_webrender_frame_ready_for_painter = HashMap::new();
        messages.retain(|message| match message {
            PaintMessage::NewWebRenderFrameReady(painter_id, _document_id, need_repaint) => {
                if let Some(painter) = self.maybe_painter(*painter_id) {
                    painter.decrement_pending_frames();
                    *saw_webrender_frame_ready_for_painter
                        .entry(*painter_id)
                        .or_insert(*need_repaint) |= *need_repaint;
                }

                false
            },
            _ => true,
        });

        for message in messages {
            self.handle_browser_message(message);
            if self.shutdown_state() == ShutdownState::FinishedShuttingDown {
                return;
            }
        }

        for (painter_id, repaint_needed) in saw_webrender_frame_ready_for_painter.iter() {
            if let Some(painter) = self.maybe_painter(*painter_id) {
                painter.handle_new_webrender_frame_ready(*repaint_needed);
            }
        }
    }

    #[servo_tracing::instrument(skip_all)]
    pub fn perform_updates(&self) -> bool {
        if self.shutdown_state() == ShutdownState::FinishedShuttingDown {
            return false;
        }

        // Run the WebXR main thread
        #[cfg(feature = "webxr")]
        self.webxr_main_thread.borrow_mut().run_one_frame();

        for painter in &self.painters {
            painter.borrow_mut().perform_updates();
        }

        self.shutdown_state() != ShutdownState::FinishedShuttingDown
    }

    pub fn toggle_webrender_debug(&self, option: WebRenderDebugOption) {
        for painter in &self.painters {
            painter.borrow_mut().toggle_webrender_debug(option);
        }
    }

    pub fn capture_webrender(&self, webview_id: WebViewId) {
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
            log::error!("Couldn't create a path for WebRender captures.");
            return;
        };

        log::info!("Saving WebRender capture to {capture_path:?}");
        self.painter(webview_id.into())
            .webrender_api
            .save_capture(capture_path.clone(), CaptureBits::all());
    }

    pub fn notify_input_event(&self, webview_id: WebViewId, event: InputEventAndId) {
        if self.shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        self.painter_mut(webview_id.into())
            .notify_input_event(webview_id, event);
    }

    pub fn notify_scroll_event(&self, webview_id: WebViewId, scroll: Scroll, point: WebViewPoint) {
        if self.shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        self.painter_mut(webview_id.into())
            .notify_scroll_event(webview_id, scroll, point);
    }

    pub fn pinch_zoom(&self, webview_id: WebViewId, pinch_zoom_delta: f32, center: DevicePoint) {
        if self.shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        self.painter_mut(webview_id.into())
            .pinch_zoom(webview_id, pinch_zoom_delta, center);
    }

    pub fn device_pixels_per_page_pixel(
        &self,
        webview_id: WebViewId,
    ) -> Scale<f32, CSSPixel, DevicePixel> {
        self.painter_mut(webview_id.into())
            .device_pixels_per_page_pixel(webview_id)
    }

    pub(crate) fn shutdown_state(&self) -> ShutdownState {
        self.shutdown_state.get()
    }

    pub fn request_screenshot(
        &self,
        webview_id: WebViewId,
        rect: Option<WebViewRect>,
        callback: Box<dyn FnOnce(Result<RgbaImage, ScreenshotCaptureError>) + 'static>,
    ) {
        self.painter(webview_id.into())
            .request_screenshot(webview_id, rect, callback);
    }

    pub fn notify_input_event_handled(
        &self,
        webview_id: WebViewId,
        input_event_id: InputEventId,
        result: InputEventResult,
    ) {
        if let Some(mut painter) = self.maybe_painter_mut(webview_id.into()) {
            painter.notify_input_event_handled(webview_id, input_event_id, result);
        }
    }

    /// Generate an image key from the appropriate [`Painter`] or, if it is unknown, generate
    /// a dummy image key. The unknown case needs to be handled because requests for keys
    /// could theoretically come after a [`Painter`] has been released. A dummy key is okay
    /// in this case because we will never render again in that case.
    fn handle_generate_image_key(
        &self,
        webview_id: WebViewId,
        result_sender: GenericSender<ImageKey>,
    ) {
        let painter_id = webview_id.into();
        let image_key = self.maybe_painter(painter_id).map_or_else(
            || ImageKey::new(painter_id.into(), 0),
            |painter| painter.webrender_api.generate_image_key(),
        );
        let _ = result_sender.send(image_key);
    }

    /// Generate image keys from the appropriate [`Painter`] or, if it is unknown, generate
    /// dummy image keys. The unknown case needs to be handled because requests for keys
    /// could theoretically come after a [`Painter`] has been released. A dummy key is okay
    /// in this case because we will never render again in that case.
    fn handle_generate_image_keys_for_pipeline(
        &self,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
    ) {
        let painter_id = webview_id.into();
        let painter = self.maybe_painter(painter_id);
        let image_keys = (0..pref!(image_key_batch_size))
            .map(|_| {
                painter.as_ref().map_or_else(
                    || ImageKey::new(painter_id.into(), 0),
                    |painter| painter.webrender_api.generate_image_key(),
                )
            })
            .collect();

        let _ = self.embedder_to_constellation_sender.send(
            EmbedderToConstellationMessage::SendImageKeysForPipeline(pipeline_id, image_keys),
        );
    }

    /// Generate font keys from the appropriate [`Painter`] or, if it is unknown, generate
    /// dummy font keys. The unknown case needs to be handled because requests for keys
    /// could theoretically come after a [`Painter`] has been released. A dummy key is okay
    /// in this case because we will never render again in that case.
    fn handle_generate_font_keys(
        &self,
        number_of_font_keys: usize,
        number_of_font_instance_keys: usize,
        result_sender: GenericSender<(Vec<FontKey>, Vec<FontInstanceKey>)>,
        painter_id: PainterId,
    ) {
        let painter = self.maybe_painter(painter_id);
        let font_keys = (0..number_of_font_keys)
            .map(|_| {
                painter.as_ref().map_or_else(
                    || FontKey::new(painter_id.into(), 0),
                    |painter| painter.webrender_api.generate_font_key(),
                )
            })
            .collect();
        let font_instance_keys = (0..number_of_font_instance_keys)
            .map(|_| {
                painter.as_ref().map_or_else(
                    || FontInstanceKey::new(painter_id.into(), 0),
                    |painter| painter.webrender_api.generate_font_instance_key(),
                )
            })
            .collect();

        let _ = result_sender.send((font_keys, font_instance_keys));
    }
}
