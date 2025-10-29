/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref, RefCell, RefMut};
use std::env;
use std::fs::create_dir_all;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use base::generic_channel::RoutedReceiver;
use base::id::WebViewId;
use bitflags::bitflags;
use canvas_traits::webgl::WebGLThreads;
use compositing_traits::{CompositorMsg, WebRenderExternalImageRegistry, WebViewTrait};
use constellation_traits::EmbedderToConstellationMessage;
use crossbeam_channel::Sender;
use dpi::PhysicalSize;
use embedder_traits::{
    InputEventAndId, InputEventId, InputEventResult, ScreenshotCaptureError, Scroll, ShutdownState,
    ViewportDetails, WebViewPoint, WebViewRect,
};
use euclid::{Scale, Size2D};
use image::RgbaImage;
use ipc_channel::ipc::{self};
use log::debug;
use profile_traits::mem::{
    ProcessReports, ProfilerRegistration, Report, ReportKind, perform_memory_report,
};
use profile_traits::path;
use profile_traits::time::{self as profile_time};
use servo_geometry::DeviceIndependentPixel;
use style_traits::CSSPixel;
use webrender::{CaptureBits, MemoryReport};
use webrender_api::units::{DevicePixel, DevicePoint, DeviceRect};

use crate::InitialCompositorState;
use crate::painter::Painter;
use crate::webview_renderer::UnknownWebView;

/// An option to control what kind of WebRender debugging is enabled while Servo is running.
#[derive(Copy, Clone)]
pub enum WebRenderDebugOption {
    Profiler,
    TextureCacheDebug,
    RenderTargetDebug,
}

/// NB: Never block on the constellation, because sometimes the constellation blocks on us.
pub struct IOCompositor {
    /// All of the [`Painters`] for this [`IOCompositor`]. Each [`Painter`] handles painting to
    /// a single [`RenderingContext`].
    painters: Vec<Rc<RefCell<Painter>>>,

    /// Tracks whether we are in the process of shutting down, or have shut down and should close
    /// the compositor. This is shared with the `Servo` instance.
    shutdown_state: Rc<Cell<ShutdownState>>,

    /// The port on which we receive messages.
    compositor_receiver: RoutedReceiver<CompositorMsg>,

    /// The channel on which messages can be sent to the constellation.
    pub(crate) embedder_to_constellation_sender: Sender<EmbedderToConstellationMessage>,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: profile_time::ProfilerChan,

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

impl IOCompositor {
    pub fn new(state: InitialCompositorState) -> Rc<RefCell<Self>> {
        let registration = state.mem_profiler_chan.prepare_memory_reporting(
            "compositor".into(),
            state.compositor_proxy.clone(),
            CompositorMsg::CollectMemoryReport,
        );

        let compositor = Rc::new(RefCell::new(IOCompositor {
            painters: Default::default(),
            shutdown_state: state.shutdown_state,
            compositor_receiver: state.receiver,
            embedder_to_constellation_sender: state.embedder_to_constellation_sender.clone(),
            time_profiler_chan: state.time_profiler_chan,
            _mem_profiler_registration: registration,
        }));

        let painter = Painter::new(
            state.rendering_context,
            state.compositor_proxy,
            state.event_loop_waker,
            state.refresh_driver,
            state.shaders_path,
            compositor.borrow().embedder_to_constellation_sender.clone(),
            #[cfg(feature = "webxr")]
            state.webxr_registry,
        );

        compositor
            .borrow_mut()
            .painters
            .push(Rc::new(RefCell::new(painter)));

        compositor
    }

    pub(crate) fn painter<'a>(&'a self) -> Ref<'a, Painter> {
        self.painters[0].borrow()
    }

    pub(crate) fn painter_mut<'a>(&'a self) -> RefMut<'a, Painter> {
        self.painters[0].borrow_mut()
    }

    pub fn deinit(&mut self) {
        for painter in &self.painters {
            painter.borrow_mut().deinit();
        }
    }

    pub fn rendering_context_size(&self) -> Size2D<u32, DevicePixel> {
        self.painter().rendering_context.size2d()
    }

    pub fn webgl_threads(&self) -> WebGLThreads {
        self.painter().webgl_threads.clone()
    }

    pub fn webrender_external_images(&self) -> Arc<Mutex<WebRenderExternalImageRegistry>> {
        self.painter().webrender_external_images.clone()
    }

    pub fn webxr_running(&self) -> bool {
        #[cfg(feature = "webxr")]
        {
            self.painter().webxr_main_thread.running()
        }
        #[cfg(not(feature = "webxr"))]
        {
            false
        }
    }

    #[cfg(feature = "webxr")]
    pub fn webxr_main_thread_registry(&self) -> webxr_api::Registry {
        self.painter().webxr_main_thread.registry()
    }

    #[cfg(feature = "webgpu")]
    pub fn webgpu_image_map(&self) -> webgpu::canvas_context::WGPUImageMap {
        self.painter().webgpu_image_map.clone()
    }

    pub(crate) fn set_needs_repaint(&self, reason: RepaintReason) {
        self.painter().set_needs_repaint(reason);
    }

    pub fn needs_repaint(&self) -> bool {
        self.painter().needs_repaint()
    }

    pub fn finish_shutting_down(&self) {
        // Drain compositor port, sometimes messages contain channels that are blocking
        // another thread from finishing (i.e. SetFrameTree).
        while self.compositor_receiver.try_recv().is_ok() {}

        // Tell the profiler, memory profiler, and scrolling timer to shut down.
        if let Ok((sender, receiver)) = ipc::channel() {
            self.time_profiler_chan
                .send(profile_time::ProfilerMsg::Exit(sender));
            let _ = receiver.recv();
        }
    }

    fn handle_browser_message(&self, msg: CompositorMsg) {
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
                self.collect_memory_report(sender);
            },
            CompositorMsg::ChangeRunningAnimationsState(
                webview_id,
                pipeline_id,
                animation_state,
            ) => {
                self.painter_mut().change_running_animations_state(
                    webview_id,
                    pipeline_id,
                    animation_state,
                );
            },
            CompositorMsg::CreateOrUpdateWebView(frame_tree) => {
                self.painter_mut().set_frame_tree_for_webview(&frame_tree);
            },
            CompositorMsg::RemoveWebView(webview_id) => {
                self.painter_mut().remove_webview(webview_id);
            },
            CompositorMsg::SetThrottled(webview_id, pipeline_id, throttled) => {
                self.painter_mut()
                    .set_throttled(webview_id, pipeline_id, throttled);
            },
            CompositorMsg::PipelineExited(webview_id, pipeline_id, pipeline_exit_source) => {
                self.painter_mut().notify_pipeline_exited(
                    webview_id,
                    pipeline_id,
                    pipeline_exit_source,
                );
            },
            CompositorMsg::NewWebRenderFrameReady(..) => {
                unreachable!("New WebRender frames should be handled in the caller.");
            },
            CompositorMsg::SendInitialTransaction(webview_id, pipeline_id) => {
                self.painter_mut()
                    .send_initial_pipeline_transaction(webview_id, pipeline_id);
            },
            CompositorMsg::ScrollNodeByDelta(
                webview_id,
                pipeline_id,
                offset,
                external_scroll_id,
            ) => {
                self.painter_mut().scroll_node_by_delta(
                    webview_id,
                    pipeline_id,
                    offset,
                    external_scroll_id,
                );
            },
            CompositorMsg::ScrollViewportByDelta(webview_id, delta) => {
                self.painter_mut()
                    .scroll_viewport_by_delta(webview_id, delta);
            },
            CompositorMsg::UpdateEpoch {
                webview_id,
                pipeline_id,
                epoch,
            } => {
                self.painter_mut()
                    .update_epoch(webview_id, pipeline_id, epoch);
            },
            CompositorMsg::SendDisplayList {
                webview_id,
                display_list_descriptor,
                display_list_receiver,
            } => {
                self.painter_mut().handle_new_display_list(
                    webview_id,
                    display_list_descriptor,
                    display_list_receiver,
                );
            },
            CompositorMsg::GenerateFrame => {
                self.painter_mut().generate_frame_for_script();
            },
            CompositorMsg::GenerateImageKey(sender) => {
                let _ = sender.send(self.painter().generate_image_key());
            },
            CompositorMsg::GenerateImageKeysForPipeline(pipeline_id) => {
                let _ = self.embedder_to_constellation_sender.send(
                    EmbedderToConstellationMessage::SendImageKeysForPipeline(
                        pipeline_id,
                        self.painter().generate_image_keys(),
                    ),
                );
            },
            CompositorMsg::UpdateImages(updates) => {
                self.painter_mut().update_images(updates);
            },
            CompositorMsg::DelayNewFrameForCanvas(pipeline_id, canvas_epoch, image_keys) => self
                .painter_mut()
                .delay_new_frames_for_canvas(pipeline_id, canvas_epoch, image_keys),
            CompositorMsg::AddFont(font_key, data, index) => {
                self.painter_mut().add_font(font_key, data, index);
            },
            CompositorMsg::AddSystemFont(font_key, native_handle) => {
                self.painter_mut().add_system_font(font_key, native_handle);
            },
            CompositorMsg::AddFontInstance(
                font_instance_key,
                font_key,
                size,
                flags,
                variations,
            ) => {
                self.painter_mut().add_font_instance(
                    font_instance_key,
                    font_key,
                    size,
                    flags,
                    variations,
                );
            },
            CompositorMsg::RemoveFonts(keys, instance_keys) => {
                self.painter_mut().remove_fonts(keys, instance_keys);
            },
            CompositorMsg::GenerateFontKeys(
                number_of_font_keys,
                number_of_font_instance_keys,
                result_sender,
                _rendering_group_id,
            ) => {
                let _ = result_sender.send(
                    self.painter_mut()
                        .generate_font_keys(number_of_font_keys, number_of_font_instance_keys),
                );
            },
            CompositorMsg::Viewport(webview_id, viewport_description) => {
                self.painter_mut()
                    .set_viewport_description(webview_id, viewport_description);
            },
            CompositorMsg::ScreenshotReadinessReponse(webview_id, pipelines_and_epochs) => {
                self.painter()
                    .handle_screenshot_readiness_reply(webview_id, pipelines_and_epochs);
            },
        }
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
            reports.push(Report {
                path: path!["compositor", "scroll-tree"],
                kind: ReportKind::ExplicitJemallocHeapSize,
                size: self
                    .painter()
                    .webview_renderers
                    .scroll_trees_memory_usage(ops),
            });
        });

        sender.send(ProcessReports::new(reports));
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
    fn handle_browser_message_while_shutting_down(&self, msg: CompositorMsg) {
        match msg {
            CompositorMsg::PipelineExited(webview_id, pipeline_id, pipeline_exit_source) => {
                self.painter_mut().notify_pipeline_exited(
                    webview_id,
                    pipeline_id,
                    pipeline_exit_source,
                );
            },
            CompositorMsg::GenerateImageKey(sender) => {
                let _ = sender.send(self.painter().webrender_api.generate_image_key());
            },
            CompositorMsg::GenerateFontKeys(
                number_of_font_keys,
                number_of_font_instance_keys,
                result_sender,
                _rendering_group_id,
            ) => {
                let _ = result_sender.send(
                    self.painter_mut()
                        .generate_font_keys(number_of_font_keys, number_of_font_instance_keys),
                );
            },
            _ => {
                debug!("Ignoring message ({:?} while shutting down", msg);
            },
        }
    }

    pub fn add_webview(&self, webview: Box<dyn WebViewTrait>, viewport_details: ViewportDetails) {
        self.painter_mut().add_webview(webview, viewport_details);
    }

    pub fn show_webview(
        &self,
        webview_id: WebViewId,
        hide_others: bool,
    ) -> Result<(), UnknownWebView> {
        self.painter_mut().show_webview(webview_id, hide_others)
    }

    pub fn hide_webview(&self, webview_id: WebViewId) -> Result<(), UnknownWebView> {
        self.painter_mut().hide_webview(webview_id)
    }

    pub fn raise_webview_to_top(
        &self,
        webview_id: WebViewId,
        hide_others: bool,
    ) -> Result<(), UnknownWebView> {
        self.painter_mut()
            .raise_webview_to_top(webview_id, hide_others)
    }

    pub fn move_resize_webview(&self, webview_id: WebViewId, rect: DeviceRect) {
        if self.shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        self.painter_mut().move_resize_webview(webview_id, rect);
    }

    pub fn set_hidpi_scale_factor(
        &self,
        webview_id: WebViewId,
        new_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    ) {
        if self.shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        self.painter_mut()
            .set_hidpi_scale_factor(webview_id, new_scale_factor);
    }

    pub fn resize_rendering_context(&self, new_size: PhysicalSize<u32>) {
        if self.shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        self.painter_mut().resize_rendering_context(new_size);
    }

    pub fn set_page_zoom(&self, webview_id: WebViewId, new_zoom: f32) {
        if self.shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        self.painter_mut().set_page_zoom(webview_id, new_zoom);
    }

    pub fn page_zoom(&self, webview_id: WebViewId) -> f32 {
        self.painter().page_zoom(webview_id)
    }

    /// Render the WebRender scene to the active `RenderingContext`.
    pub fn render(&self) {
        self.painter_mut().render(&self.time_profiler_chan);
    }

    /// Get the message receiver for this [`IOCompositor`].
    pub fn receiver(&self) -> &RoutedReceiver<CompositorMsg> {
        &self.compositor_receiver
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
                self.painter().decrement_pending_frames();
                repaint_needed |= need_repaint;
                saw_webrender_frame_ready = true;

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

        if saw_webrender_frame_ready {
            self.handle_new_webrender_frame_ready(repaint_needed);
        }
    }

    #[servo_tracing::instrument(skip_all)]
    pub fn perform_updates(&self) -> bool {
        if self.shutdown_state() == ShutdownState::FinishedShuttingDown {
            return false;
        }

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

    pub fn capture_webrender(&self) {
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
        self.painter()
            .webrender_api
            .save_capture(capture_path.clone(), CaptureBits::all());
    }

    pub fn notify_input_event(&self, webview_id: WebViewId, event: InputEventAndId) {
        if self.shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        self.painter_mut().notify_input_event(webview_id, event);
    }

    pub fn notify_scroll_event(&self, webview_id: WebViewId, scroll: Scroll, point: WebViewPoint) {
        if self.shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        self.painter_mut()
            .notify_scroll_event(webview_id, scroll, point);
    }

    pub fn pinch_zoom(&self, webview_id: WebViewId, pinch_zoom_delta: f32, center: DevicePoint) {
        if self.shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        self.painter_mut()
            .pinch_zoom(webview_id, pinch_zoom_delta, center);
    }

    pub fn device_pixels_per_page_pixel(
        &self,
        webview_id: WebViewId,
    ) -> Scale<f32, CSSPixel, DevicePixel> {
        self.painter_mut().device_pixels_per_page_pixel(webview_id)
    }

    pub(crate) fn shutdown_state(&self) -> ShutdownState {
        self.shutdown_state.get()
    }

    fn handle_new_webrender_frame_ready(&mut self, repaint_needed: bool) {
        let painter = self.painter();
        if repaint_needed {
            painter.refresh_cursor()
        }

        if repaint_needed || painter.animation_callbacks_running() {
            self.set_needs_repaint(RepaintReason::NewWebRenderFrame);
        }

        // If we received a new frame and a repaint isn't necessary, it may be that this
        // is the last frame that was pending. In that case, trigger a manual repaint so
        // that the screenshot can be taken at the end of the repaint procedure.
        if !repaint_needed {
            painter
                .screenshot_taker
                .maybe_trigger_paint_for_screenshot(&painter);
        }
    }

    pub fn request_screenshot(
        &self,
        webview_id: WebViewId,
        rect: Option<WebViewRect>,
        callback: Box<dyn FnOnce(Result<RgbaImage, ScreenshotCaptureError>) + 'static>,
    ) {
        self.painter()
            .request_screenshot(webview_id, rect, callback);
    }

    pub fn notify_input_event_handled(
        &self,
        webview_id: WebViewId,
        input_event_id: InputEventId,
        result: InputEventResult,
    ) {
        self.painter_mut()
            .notify_input_event_handled(webview_id, input_event_id, result);
    }
}
