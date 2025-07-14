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
use base::id::{PipelineId, RenderingGroupId, WebViewId};
use base::{Epoch, generic_channel};
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
use embedder_traits::{CompositorHitTestResult, InputEvent, ShutdownState, ViewportDetails};
use euclid::{Point2D, Rect, Scale, Size2D, Transform3D};
use ipc_channel::ipc::{self, IpcSender, IpcSharedMemory};
use log::{debug, error, info, trace, warn};
use pixels::{CorsStatus, ImageFrame, ImageMetadata, PixelFormat, RasterImage};
use profile_traits::mem::{ProcessReports, ProfilerRegistration, Report, ReportKind};
use profile_traits::time::{self as profile_time, ProfilerCategory};
use profile_traits::{path, time_profile};
use servo_config::{opts, pref};
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
    FontVariation, HitTestFlags, PropertyBinding, ReferenceFrameKind, RenderReasons,
    SampledScrollOffset, ScrollLocation, SpaceAndClipInfo, SpatialId, SpatialTreeItemKey,
    TransformStyle,
};

use crate::InitialCompositorState;
use crate::refresh_driver::RefreshDriver;
use crate::webview_manager::{WebRenderInstance, WebViewManager};
use crate::webview_renderer::{PinchZoomResult, UnknownWebView, WebViewRenderer};

#[derive(Debug, PartialEq)]
pub enum UnableToComposite {
    NotReadyToPaintImage(NotReadyToPaint),
}

#[derive(Debug, PartialEq)]
pub enum NotReadyToPaint {
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

    /// Tracks whether we are in the process of shutting down, or have shut down and should close
    /// the compositor. This is shared with the `Servo` instance.
    shutdown_state: Rc<Cell<ShutdownState>>,

    /// The port on which we receive messages.
    compositor_receiver: generic_channel::RoutedReceiver<CompositorMsg>,

    /// The channel on which messages can be sent to the constellation.
    pub(crate) constellation_sender: Sender<EmbedderToConstellationMessage>,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: profile_time::ProfilerChan,

    #[cfg(feature = "webxr")]
    /// Some XR devices want to run on the main thread.
    webxr_main_thread: webxr::MainThreadRegistry,

    /// True to translate mouse input into touch events.
    pub(crate) convert_mouse_to_touch: bool,

    /// The last position in the rendered view that the mouse moved over. This becomes `None`
    /// when the mouse leaves the rendered view.
    pub(crate) last_mouse_move_position: Option<DevicePoint>,
}

/// NB: Never block on the constellation, because sometimes the constellation blocks on us.
pub struct IOCompositor {
    /// Data that is shared by all WebView renderers.
    global: Rc<RefCell<ServoRenderer>>,

    /// Our [`WebViewRenderer`]s, one for every `WebView`.
    webview_renderers: WebViewManager<WebViewRenderer>,

    /// Used by the logic that determines when it is safe to output an
    /// image for the reftest framework.
    ready_to_save_state: ReadyState,

    /// The number of frames pending to receive from WebRender.
    pending_frames: usize,

    /// A handle to the memory profiler which will automatically unregister
    /// when it's dropped.
    _mem_profiler_registration: ProfilerRegistration,
}

/// Why we need to be repainted. This is used for debugging.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

    pub(crate) fn hit_test_at_point(
        &self,
        point: DevicePoint,
        webrender_instance: &WebRenderInstance,
    ) -> Vec<CompositorHitTestResult> {
        self.hit_test_at_point_with_flags(point, HitTestFlags::empty(), webrender_instance)
    }

    // TODO: split this into first half (global) and second half (one for whole compositor, one for webview)
    pub(crate) fn hit_test_at_point_with_flags(
        &self,
        point: DevicePoint,
        flags: HitTestFlags,
        webrender_instance: &WebRenderInstance,
    ) -> Vec<CompositorHitTestResult> {
        // DevicePoint and WorldPoint are the same for us.
        let world_point = WorldPoint::from_untyped(point.to_untyped());
        let results = webrender_instance.webrender_api.hit_test(
            webrender_instance.webrender_document,
            None, /* pipeline_id */
            world_point,
            flags,
        );

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
}

impl IOCompositor {
    pub fn new(state: InitialCompositorState, convert_mouse_to_touch: bool) -> Self {
        let registration = state.mem_profiler_chan.prepare_memory_reporting(
            "compositor".into(),
            state.sender.clone(),
            CompositorMsg::CollectMemoryReport,
        );
        let mut webview_renderers = WebViewManager::new(state.sender);
        webview_renderers.add_webview_group(None, state.rendering_context);
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
                #[cfg(feature = "webxr")]
                webxr_main_thread: state.webxr_main_thread,
                convert_mouse_to_touch,
                last_mouse_move_position: None,
            })),
            webview_renderers,
            ready_to_save_state: ReadyState::Unknown,
            pending_frames: 0,
            _mem_profiler_registration: registration,
        };
        state.webrender.deinit();

        //compositor.assert_gl_framebuffer_complete();

        compositor
    }

    pub fn deinit(&mut self) {
        warn!("Deinit calling");
        self.webview_renderers.deinit();
    }

    pub fn rendering_context_size(&self) -> Size2D<u32, DevicePixel> {
        self.webview_renderers.rendering_context_size()
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

    pub fn webrender_instance_for_pipeline(
        &self,
        pipeline_id: PipelineId,
    ) -> (&RenderApi, DocumentId) {
        let webview_id = self
            .global
            .borrow()
            .pipeline_to_webview_map
            .get(&pipeline_id)
            .unwrap()
            .clone();
        let group_id = self
            .webview_renderers
            .group_id(webview_id)
            .expect("No group id");
        let wri = self.webview_renderers.webrender_instance(group_id);
        (&wri.webrender_api, wri.webrender_document)
    }

    fn set_needs_repaint(&self, webview_group_id: RenderingGroupId, reason: RepaintReason) {
        let rc = self.webview_renderers.webrender_instance(webview_group_id);
        rc.needs_repaint.set(reason)
    }

    pub fn needs_repaint(&self) -> bool {
        let repaint_reason = self.webview_renderers.needs_repaint();
        if repaint_reason.is_empty() {
            return false;
        }

        !self
            .global
            .borrow()
            .refresh_driver
            .wait_to_paint(repaint_reason)
    }

    fn handle_generate_font_keys(
        &self,
        number_of_font_keys: usize,
        number_of_font_instance_keys: usize,
        result_sender: IpcSender<(Vec<FontKey>, Vec<FontInstanceKey>)>,
        webview_id: WebViewId,
    ) {
        let group_id = self
            .webview_renderers
            .group_id(webview_id)
            .expect("No group");
        let wri = self.webview_renderers.webrender_instance(group_id);
        let font_keys = (0..number_of_font_keys)
            .map(|_| wri.webrender_api.generate_font_key())
            .collect::<Vec<FontKey>>();

        let font_instance_keys = (0..number_of_font_instance_keys)
            .map(|_| wri.webrender_api.generate_font_instance_key())
            .collect::<Vec<FontInstanceKey>>();

        let _ = result_sender.send((font_keys, font_instance_keys));
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
                let reports = self
                    .webview_renderers
                    .rendering_contexts()
                    .enumerate()
                    .flat_map(|(index, wri)| {
                        let ops = wr_malloc_size_of::MallocSizeOfOps::new(
                            servo_allocator::usable_size,
                            None,
                        );
                        let report = wri.webrender_api.report_memory(ops);
                        vec![
                            Report {
                                path: path!["webrender", index.to_string(), "fonts"],
                                kind: ReportKind::ExplicitJemallocHeapSize,
                                size: report.fonts,
                            },
                            Report {
                                path: path!["webrender", index.to_string(), "images"],
                                kind: ReportKind::ExplicitJemallocHeapSize,
                                size: report.images,
                            },
                            Report {
                                path: path!["webrender", index.to_string(), "display-list"],
                                kind: ReportKind::ExplicitJemallocHeapSize,
                                size: report.display_list,
                            },
                        ]
                    })
                    .collect();

                sender.send(ProcessReports::new(reports));
            },

            CompositorMsg::ChangeRunningAnimationsState(
                webview_id,
                pipeline_id,
                animation_state,
            ) => {
                let Some(webview_renderer) = self.webview_renderers.get_webview_mut(webview_id)
                else {
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
                let Some((webview_renderer, webrender)) =
                    self.webview_renderers.get_webview_webrender_mut(webview_id)
                else {
                    warn!("Handling input event for unknown webview: {webview_id}");
                    return;
                };
                webview_renderer.on_touch_event_processed(result, webrender);
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
                for (_webview, webrender_instance) in
                    self.webview_renderers.webrender_instance_mut()
                {
                    webrender_instance
                        .needs_repaint
                        .set(RepaintReason::ReadyForScreenshot);
                }
            },

            CompositorMsg::SetThrottled(webview_id, pipeline_id, throttled) => {
                let Some(webview_renderer) = self.webview_renderers.get_webview_mut(webview_id)
                else {
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
                if let Some(webview_renderer) = self.webview_renderers.get_webview_mut(webview_id) {
                    webview_renderer.pipeline_exited(pipeline_id, pipeline_exit_source);
                    // TODO This does not seem to remove the pipeline/webview from webview_renderer.
                }
            },

            CompositorMsg::NewWebRenderFrameReady(
                _document_id,
                rendering_groupd_id,
                recomposite_needed,
            ) => {
                self.handle_new_webrender_frame_ready(recomposite_needed, rendering_groupd_id);
            },

            CompositorMsg::LoadComplete(webview_id) => {
                if opts::get().wait_for_stable_image {
                    let group_id = self
                        .webview_renderers
                        .group_id(webview_id)
                        .expect("No group id for this webview");
                    self.set_needs_repaint(group_id, RepaintReason::ReadyForScreenshot);
                }
            },

            CompositorMsg::SendInitialTransaction(pipeline) => {
                let mut txn = Transaction::new();
                txn.set_display_list(WebRenderEpoch(0), (pipeline, Default::default()));
                self.generate_frame(&mut txn, RenderReasons::SCENE);
                if let Some(webview_id) = self
                    .global
                    .borrow()
                    .pipeline_to_webview_map
                    .get(&pipeline.into())
                {
                    self.webview_renderers
                        .send_transaction(webview_id.clone(), txn);
                } else {
                    error!("You are trying to send to pipeline that does not exist yet.");
                }
            },

            CompositorMsg::SendScrollNode(webview_id, pipeline_id, offset, external_scroll_id) => {
                let Some(webview_renderer) = self.webview_renderers.get_webview_mut(webview_id)
                else {
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
                self.webview_renderers.send_transaction(webview_id, txn);
            },

            CompositorMsg::SendDisplayList {
                webview_id,
                display_list_descriptor,
                display_list_receiver,
            } => {
                info!("Sending display list to {webview_id}");

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

                let Some(webview_renderer) = self.webview_renderers.get_webview_mut(webview_id)
                else {
                    return warn!("Could not find WebView for incoming display list");
                };

                let old_scale = webview_renderer.device_pixels_per_page_pixel();

                let pipeline_id = display_list_info.pipeline_id;
                let details = webview_renderer.ensure_pipeline_details(pipeline_id.into());
                details.install_new_scroll_tree(display_list_info.scroll_tree);
                details.viewport_scale =
                    Some(display_list_info.viewport_details.hidpi_scale_factor);

                let epoch = display_list_info.epoch;
                let first_reflow = display_list_info.first_reflow;
                if details.first_paint_metric == PaintMetricState::Waiting {
                    details.first_paint_metric = PaintMetricState::Seen(epoch, first_reflow);
                }
                if details.first_contentful_paint_metric == PaintMetricState::Waiting
                    && display_list_info.is_contentful
                {
                    details.first_contentful_paint_metric =
                        PaintMetricState::Seen(epoch, first_reflow);
                }

                let mut transaction = Transaction::new();

                let is_root_pipeline =
                    Some(pipeline_id.into()) == webview_renderer.root_pipeline_id;
                if is_root_pipeline && old_scale != webview_renderer.device_pixels_per_page_pixel()
                {
                    let group_id = {
                        let global = self.global.borrow();
                        let webview_id = global
                            .pipeline_to_webview_map
                            .get(&pipeline_id.into())
                            .unwrap();
                        self.webview_renderers.group_id(*webview_id).unwrap()
                    };
                    self.send_root_pipeline_display_list_in_transaction(group_id, &mut transaction);
                }

                transaction
                    .set_display_list(display_list_info.epoch, (pipeline_id, built_display_list));
                self.update_transaction_with_all_scroll_offsets(&mut transaction);
                self.generate_frame(&mut transaction, RenderReasons::SCENE);
                self.webview_renderers
                    .send_transaction(webview_id, transaction);
            },

            CompositorMsg::GenerateImageKey(sender) => {
                self.handle_generate_image_keys(sender);
            },

            CompositorMsg::GenerateImageKeysForPipeline(pipeline_id) => {
                let webview_id = self
                    .global
                    .borrow()
                    .pipeline_to_webview_map
                    .get(&pipeline_id)
                    .unwrap()
                    .clone();
                let group_id = self.webview_renderers.group_id(webview_id).expect("F");
                let rtc = self.webview_renderers.webrender_instance(group_id);

                let image_keys = (0..pref!(image_key_batch_size))
                    .map(|_| rtc.webrender_api.generate_image_key())
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
                for update in updates {
                    let mut txn = Transaction::new();
                    let key = match update {
                        ImageUpdate::AddImage(key, desc, data) => {
                            txn.add_image(key, desc, data.into(), None);
                            key.0
                        },
                        ImageUpdate::DeleteImage(key) => {
                            txn.delete_image(key);
                            key.0
                        },
                        ImageUpdate::UpdateImage(key, desc, data) => {
                            txn.update_image(key, desc, data.into(), &DirtyRect::All);
                            key.0
                        },
                    };
                    self.webview_renderers
                        .send_transaction_to_namespace_id(txn, key);
                }
            },

            CompositorMsg::AddFont(font_key, data, index) => {
                self.add_font(font_key, index, data);
            },

            CompositorMsg::AddSystemFont(font_key, native_handle) => {
                let mut transaction = Transaction::new();
                transaction.add_native_font(font_key, native_handle.clone());
                self.webview_renderers
                    .send_transaction_to_namespace_id(transaction, font_key.0);
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
                for (key, instance) in keys.iter().zip(instance_keys) {
                    let mut transaction = Transaction::new();
                    transaction.delete_font_instance(instance.clone());
                    transaction.delete_font(key.clone());
                    self.webview_renderers
                        .send_transaction_to_namespace_id(transaction, key.0);
                }
            },

            CompositorMsg::GenerateFontKeys(
                number_of_font_keys,
                number_of_font_instance_keys,
                result_sender,
                webview_id,
            ) => {
                self.handle_generate_font_keys(
                    number_of_font_keys,
                    number_of_font_instance_keys,
                    result_sender,
                    webview_id,
                );
            },
            CompositorMsg::Viewport(webview_id, viewport_description) => {
                if let Some(webview) = self.webview_renderers.get_webview_mut(webview_id) {
                    webview.set_viewport_description(viewport_description);
                }
            },
        }
    }

    fn handle_generate_image_keys(&mut self, sender: IpcSender<webrender_api::ImageKey>) {
        let keys = self
            .webview_renderers
            .rendering_contexts()
            .map(|v| v.webrender_api.generate_image_key())
            .next()
            .unwrap();
        let _ = sender.send(keys);
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
                if let Some(webview_renderer) = self.webview_renderers.get_webview_mut(webview_id) {
                    webview_renderer.pipeline_exited(pipeline_id, pipeline_exit_source);
                }
            },
            CompositorMsg::GenerateImageKey(sender) => {
                self.handle_generate_image_keys(sender);
            },
            CompositorMsg::GenerateFontKeys(
                number_of_font_keys,
                number_of_font_instance_keys,
                result_sender,
                webview_id,
            ) => {
                self.handle_generate_font_keys(
                    number_of_font_keys,
                    number_of_font_instance_keys,
                    result_sender,
                    webview_id,
                );
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
    fn send_root_pipeline_display_list(&mut self, webview_group_id: RenderingGroupId) {
        let mut transaction = Transaction::new();
        self.send_root_pipeline_display_list_in_transaction(
            webview_group_id.clone(),
            &mut transaction,
        );
        self.generate_frame(&mut transaction, RenderReasons::SCENE);

        self.webview_renderers
            .send_transaction_to_group(webview_group_id, transaction);
    }

    /// Set the root pipeline for our WebRender scene to a display list that consists of an iframe
    /// for each visible top-level browsing context, applying a transformation on the root for
    /// pinch zoom, page zoom, and HiDPI scaling.
    pub(crate) fn send_root_pipeline_display_list_in_transaction(
        &self,
        webview_group_id: RenderingGroupId,
        transaction: &mut Transaction,
    ) {
        let render_instance = self
            .webview_renderers
            .webrender_instance(webview_group_id.clone());
        // Every display list needs a pipeline, but we'd like to choose one that is unlikely
        // to conflict with our content pipelines, which start at (1, 1). (0, 0) is WebRender's
        // dummy pipeline, so we choose (0, 1).
        let root_pipeline = webview_group_id.webrender_pipeline_id();
        transaction.set_root_pipeline(root_pipeline);

        let mut builder = webrender_api::DisplayListBuilder::new(root_pipeline);
        builder.begin();

        let root_reference_frame = SpatialId::root_reference_frame(root_pipeline);

        let viewport_size = render_instance
            .rendering_context
            .size2d()
            .to_f32()
            .to_untyped();
        let viewport_rect = LayoutRect::from_origin_and_size(
            LayoutPoint::zero(),
            LayoutSize::from_untyped(viewport_size),
        );

        let root_clip_id = builder.define_clip_rect(root_reference_frame, viewport_rect);
        let clip_chain_id = builder.define_clip_chain(None, [root_clip_id]);
        for (_, webview_renderer) in self.webview_renderers.painting_order(webview_group_id) {
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
        let groups = self.webview_renderers.groups();
        let first_group = groups.first().expect("NO first group");
        self.webview_renderers.add_webview(
            first_group.clone(),
            webview.id(),
            WebViewRenderer::new(self.global.clone(), webview, viewport_details),
        );
    }

    pub fn add_webview_new_group(
        &mut self,
        webview: Box<dyn WebViewTrait>,
        rendering_context: Rc<dyn RenderingContext>,
        viewport_details: ViewportDetails,
    ) {
        info!(
            "ADD_WEBVIEW_NEW_GROUP WITH ID: {:?}",
            webview.rendering_group_id()
        );
        let group_id = self
            .webview_renderers
            .add_webview_group(webview.rendering_group_id(), rendering_context);
        let wvid = webview.id();
        let wvr = WebViewRenderer::new(self.global.clone(), webview, viewport_details);
        self.webview_renderers.add_webview(group_id, wvid, wvr);
    }

    fn set_frame_tree_for_webview(&mut self, frame_tree: &SendableFrameTree) {
        warn!("{}: Setting frame tree for webview", frame_tree.pipeline.id);

        let webview_id = frame_tree.pipeline.webview_id;
        let Some(webview_renderer) = self.webview_renderers.get_webview_mut(webview_id) else {
            warn!(
                "Attempted to set frame tree on unknown WebView (perhaps closed?): {webview_id:?}"
            );
            return;
        };

        webview_renderer.set_frame_tree(frame_tree);
        let group_id = self
            .webview_renderers
            .group_id(webview_id)
            .expect("Could not find group id");
        self.send_root_pipeline_display_list(group_id);
    }

    fn remove_webview(&mut self, webview_id: WebViewId) {
        debug!("{}: Removing", webview_id);
        if self.webview_renderers.remove(webview_id).is_err() {
            warn!("{webview_id}: Removing unknown webview");
            return;
        };

        let group_id = self
            .webview_renderers
            .group_id(webview_id)
            .expect("Could not find group id");
        self.send_root_pipeline_display_list(group_id);
    }

    pub fn show_webview(
        &mut self,
        webview_id: WebViewId,
        hide_others: bool,
    ) -> Result<(), UnknownWebView> {
        debug!("{webview_id}: Showing webview; hide_others={hide_others}");
        let group_id = self
            .webview_renderers
            .group_id(webview_id)
            .expect("NOT IN GROUP");
        let painting_order_changed = if hide_others {
            let painting_order = self.webview_renderers.painting_order(group_id.clone());
            let result = painting_order.map(|(&id, _)| id).ne(once(webview_id));
            self.webview_renderers.hide_all(group_id.clone());
            self.webview_renderers.show(webview_id)?;
            result
        } else {
            self.webview_renderers.show(webview_id)?
        };
        if painting_order_changed {
            self.send_root_pipeline_display_list(group_id);
        }
        Ok(())
    }

    pub fn hide_webview(&mut self, webview_id: WebViewId) -> Result<(), UnknownWebView> {
        debug!("{webview_id}: Hiding webview");
        let group_id = self
            .webview_renderers
            .group_id(webview_id)
            .expect("No group id");
        if self.webview_renderers.hide(webview_id)? {
            self.send_root_pipeline_display_list(group_id);
        }
        Ok(())
    }

    pub fn raise_webview_to_top(
        &mut self,
        webview_id: WebViewId,
        hide_others: bool,
    ) -> Result<(), UnknownWebView> {
        debug!("{webview_id}: Raising webview to top; hide_others={hide_others}");
        let group_id = self
            .webview_renderers
            .group_id(webview_id)
            .expect("Could not find group id");
        let painting_order_changed = if hide_others {
            let painting_order = self.webview_renderers.painting_order(group_id.clone());
            let result = painting_order.map(|(&id, _)| id).ne(once(webview_id));
            self.webview_renderers.hide_all(group_id.clone());
            self.webview_renderers.raise_to_top(webview_id)?;
            result
        } else {
            self.webview_renderers.raise_to_top(webview_id)?
        };
        if painting_order_changed {
            self.send_root_pipeline_display_list(group_id);
        }
        Ok(())
    }

    pub fn move_resize_webview(&mut self, webview_id: WebViewId, rect: DeviceRect) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        let Some(webview_renderer) = self.webview_renderers.get_webview_mut(webview_id) else {
            return;
        };
        if !webview_renderer.set_rect(rect) {
            return;
        }

        let group_id = self
            .webview_renderers
            .group_id(webview_id)
            .expect("Could not find group id");
        self.send_root_pipeline_display_list(group_id.clone());
        self.set_needs_repaint(group_id, RepaintReason::Resize);
    }

    pub fn set_hidpi_scale_factor(
        &mut self,
        webview_id: WebViewId,
        new_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    ) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        let Some(webview_renderer) = self.webview_renderers.get_webview_mut(webview_id) else {
            return;
        };
        if !webview_renderer.set_hidpi_scale_factor(new_scale_factor) {
            return;
        }

        let group_id = self
            .webview_renderers
            .group_id(webview_id)
            .expect("Could not find group id");
        self.send_root_pipeline_display_list(group_id.clone());
        self.set_needs_repaint(group_id, RepaintReason::Resize);
    }

    pub fn resize_rendering_context(&mut self, webview_id: WebViewId, new_size: PhysicalSize<u32>) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }
        let webview_group_id = self
            .webview_renderers
            .group_id(webview_id)
            .expect("Could not find groupid");
        let render_instance = self
            .webview_renderers
            .webrender_instance(webview_group_id.clone());
        if render_instance.rendering_context.size() == new_size {
            return;
        }

        render_instance.rendering_context.resize(new_size);

        let mut transaction = Transaction::new();
        let output_region = DeviceIntRect::new(
            Point2D::zero(),
            Point2D::new(new_size.width as i32, new_size.height as i32),
        );
        transaction.set_document_view(output_region);
        self.webview_renderers
            .send_transaction(webview_id, transaction);

        self.send_root_pipeline_display_list(webview_group_id.clone());
        self.set_needs_repaint(webview_group_id, RepaintReason::Resize);
    }

    pub fn on_zoom_reset_window_event(&mut self, webview_id: WebViewId) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }

        if let Some(webview_renderer) = self.webview_renderers.get_webview_mut(webview_id) {
            webview_renderer.set_page_zoom(Scale::new(1.0));
        }
        let group_id = self
            .webview_renderers
            .group_id(webview_id)
            .expect("Could not find group id");
        self.send_root_pipeline_display_list(group_id);
    }

    pub fn on_zoom_window_event(&mut self, webview_id: WebViewId, magnification: f32) {
        if self.global.borrow().shutdown_state() != ShutdownState::NotShuttingDown {
            return;
        }

        if let Some(webview_renderer) = self.webview_renderers.get_webview_mut(webview_id) {
            let current_page_zoom = webview_renderer.page_zoom();
            webview_renderer.set_page_zoom(current_page_zoom * Scale::new(magnification));
        }
        let group_id = self
            .webview_renderers
            .group_id(webview_id)
            .expect("Could not find group id");
        self.send_root_pipeline_display_list(group_id);
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
                    let webview_id = self
                        .global
                        .borrow()
                        .pipeline_to_webview_map
                        .get(id)
                        .expect("Could not find webview_id")
                        .clone();
                    let group_id = self
                        .webview_renderers
                        .group_id(webview_id)
                        .expect("Could not find");
                    let rendering_context = self.webview_renderers.webrender_instance(group_id);
                    let document_id = self.webview_renderers.document_id(&webview_id);

                    if let Some(WebRenderEpoch(epoch)) = rendering_context
                        .webrender
                        .current_epoch(document_id, id.into())
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
    pub fn render(&mut self, webview_group_id: RenderingGroupId) -> bool {
        self.global
            .borrow()
            .refresh_driver
            .notify_will_paint(self.webview_renderers.iter());
        self.send_root_pipeline_display_list(webview_group_id.clone());
        if let Err(error) = self.render_inner(webview_group_id.clone()) {
            warn!("Unable to render: {error:?}");
            return false;
        }

        // We've painted the default target, which means that from the embedder's perspective,
        // the scene no longer needs to be repainted.
        self.webview_renderers
            .webrender_instance(webview_group_id)
            .needs_repaint
            .set(RepaintReason::empty());

        true
    }

    /// Render the WebRender scene to the shared memory, without updating other state of this
    /// [`IOCompositor`]. If succesful return the output image in shared memory.
    pub fn render_to_shared_memory(
        &mut self,
        webview_id: WebViewId,
        page_rect: Option<Rect<f32, CSSPixel>>,
    ) -> Result<Option<RasterImage>, UnableToComposite> {
        let group_id = self.webview_renderers.group_id(webview_id).unwrap();
        self.render_inner(group_id.clone())?;
        let render_instance = self.webview_renderers.webrender_instance(group_id);

        let size = render_instance.rendering_context.size2d().to_i32();
        let rect = if let Some(rect) = page_rect {
            let scale = self
                .webview_renderers
                .get_webview(webview_id)
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

        Ok(render_instance
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
    fn render_inner(
        &mut self,
        webview_group_id: RenderingGroupId,
    ) -> Result<(), UnableToComposite> {
        self.webview_renderers
            .assert_no_gl_error(webview_group_id.clone());

        {
            self.clear_background(webview_group_id.clone());
            let render_instance = &mut self
                .webview_renderers
                .render_instance_mut(webview_group_id.clone());
            if let Err(err) = render_instance.rendering_context.make_current() {
                warn!("Failed to make the rendering context current: {:?}", err);
            }

            debug_assert_eq!(
                render_instance.webrender_gl.get_error(),
                gleam::gl::NO_ERROR
            );
            render_instance.webrender.update();
            debug_assert_eq!(
                render_instance.webrender_gl.get_error(),
                gleam::gl::NO_ERROR
            );
        }

        if opts::get().wait_for_stable_image {
            if let Err(result) = self.is_ready_to_paint_image_output() {
                return Err(UnableToComposite::NotReadyToPaintImage(result));
            }
        }

        {
            let render_instance = &mut self.webview_renderers.render_instance_mut(webview_group_id);

            render_instance.rendering_context.prepare_for_rendering();
            debug_assert_eq!(
                render_instance.webrender_gl.get_error(),
                gleam::gl::NO_ERROR
            );

            let time_profiler_chan = self.global.borrow().time_profiler_chan.clone();
            time_profile!(
                ProfilerCategory::Compositing,
                None,
                time_profiler_chan,
                || {
                    trace!("Compositing");

                    // Paint the scene.
                    // TODO(gw): Take notice of any errors the renderer returns!

                    let size = render_instance.rendering_context.size2d().to_i32();

                    let _ = render_instance.webrender.render(size, 0 /* buffer_age */);
                }
            );
        }

        self.send_pending_paint_metrics_messages_after_composite();
        Ok(())
    }

    pub fn present_all(&self) {
        self.webview_renderers.present_all();
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
        //let document_id = self.webrender_document();
        for (webview_renderer, webrender_instance) in
            self.webview_renderers.webrender_instance_mut()
        {
            for (pipeline_id, pipeline) in webview_renderer.pipelines.iter_mut() {
                let Some(current_epoch) = webrender_instance
                    .webrender
                    .current_epoch(webrender_instance.webrender_document, pipeline_id.into())
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

    fn clear_background(&self, webview_group_id: RenderingGroupId) {
        self.webview_renderers.clear_background(webview_group_id);
    }

    /// Get the message receiver for this [`IOCompositor`].
    pub fn receiver(&self) -> Ref<'_, generic_channel::RoutedReceiver<CompositorMsg>> {
        Ref::map(self.global.borrow(), |global| &global.compositor_receiver)
    }

    #[servo_tracing::instrument(skip_all)]
    pub fn handle_messages(&mut self, mut messages: Vec<CompositorMsg>) {
        //error!("frame_ready_msgs {:?}", messages);

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

        let groups = self.webview_renderers.groups();
        for webview_group_id in groups {
            warn!("perform_update for {webview_group_id:?}");
            let render_instance = self
                .webview_renderers
                .webrender_instance(webview_group_id.clone());

            // The WebXR thread may make a different context current
            if let Err(err) = render_instance.rendering_context.make_current() {
                warn!("Failed to make the rendering context current: {:?}", err);
            }
            let mut need_zoom = false;
            let scroll_offset_updates: Vec<_> = self
                .webview_renderers
                .webrender_instance_mut()
                .filter_map(|(webview_renderer, webrender_instance)| {
                    let (zoom, scroll_result) = webview_renderer
                        .process_pending_scroll_and_pinch_zoom_events(webrender_instance);
                    need_zoom = need_zoom || (zoom == PinchZoomResult::DidPinchZoom);
                    scroll_result
                })
                .collect();

            if need_zoom || !scroll_offset_updates.is_empty() {
                let mut transaction = Transaction::new();
                if need_zoom {
                    self.send_root_pipeline_display_list_in_transaction(
                        webview_group_id.clone(),
                        &mut transaction,
                    );
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
                self.webview_renderers
                    .send_transaction_to_group(webview_group_id, transaction);
            }
        }
        self.global.borrow().shutdown_state() != ShutdownState::FinishedShuttingDown
    }

    pub fn toggle_webrender_debug(&mut self, option: WebRenderDebugOption) {
        let mut flags = webrender_api::DebugFlags::default();
        let flag = match option {
            WebRenderDebugOption::Profiler => {
                webrender::DebugFlags::PROFILER_DBG
                    | webrender::DebugFlags::GPU_TIME_QUERIES
                    | webrender::DebugFlags::GPU_SAMPLE_QUERIES
            },
            WebRenderDebugOption::TextureCacheDebug => webrender::DebugFlags::TEXTURE_CACHE_DBG,
            WebRenderDebugOption::RenderTargetDebug => webrender::DebugFlags::RENDER_TARGET_DBG,
        };
        flags.toggle(flag);
        self.webview_renderers.set_webrender_debug_flags(flags);

        /*
        let mut txn = Transaction::new();
        self.generate_frame(&mut txn, RenderReasons::TESTING);
        self.global.borrow_mut().send_transaction(txn);
        */
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

        for webrender_api in self
            .webview_renderers
            .rendering_contexts()
            .map(|r| &r.webrender_api)
        {
            webrender_api.save_capture(capture_path.clone(), CaptureBits::all());
        }
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
        self.webview_renderers
            .send_transaction_to_namespace_id(transaction, font_key.0);
    }

    fn add_font(&mut self, font_key: FontKey, index: u32, data: Arc<IpcSharedMemory>) {
        let mut transaction = Transaction::new();
        transaction.add_raw_font(font_key, (**data).into(), index);
        self.webview_renderers
            .send_transaction_to_namespace_id(transaction, font_key.0);
    }

    pub fn notify_input_event(&mut self, webview_id: WebViewId, event: InputEvent) {
        if let Some((webview_renderer, webrender)) =
            self.webview_renderers.get_webview_webrender_mut(webview_id)
        {
            webview_renderer.notify_input_event(event, webrender);
        }
    }

    pub fn notify_scroll_event(
        &mut self,
        webview_id: WebViewId,
        scroll_location: ScrollLocation,
        cursor: DeviceIntPoint,
    ) {
        if let Some(webview_renderer) = self.webview_renderers.get_webview_mut(webview_id) {
            webview_renderer.notify_scroll_event(scroll_location, cursor);
        }
    }

    pub fn on_vsync(&mut self, webview_id: WebViewId) {
        if let Some(webview_renderer) = self.webview_renderers.get_webview_mut(webview_id) {
            webview_renderer.on_vsync();
        }
    }

    pub fn set_pinch_zoom(&mut self, webview_id: WebViewId, magnification: f32) {
        if let Some(webview_renderer) = self.webview_renderers.get_webview_mut(webview_id) {
            webview_renderer.set_pinch_zoom(magnification);
        }
    }

    fn shutdown_state(&self) -> ShutdownState {
        self.global.borrow().shutdown_state()
    }

    fn refresh_cursor(&self, webview_group_id: RenderingGroupId) {
        let global = self.global.borrow();
        let Some(last_mouse_move_position) = global.last_mouse_move_position else {
            return;
        };

        let webrender_instance = self.webview_renderers.render_instance(webview_group_id);
        let Some(hit_test_result) = global
            .hit_test_at_point(last_mouse_move_position, webrender_instance)
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

    fn handle_new_webrender_frame_ready(
        &mut self,
        recomposite_needed: bool,
        webview_group_id: RenderingGroupId,
    ) {
        self.pending_frames -= 1;
        if recomposite_needed {
            self.refresh_cursor(webview_group_id.clone());
        }
        if recomposite_needed || self.animation_callbacks_running() {
            self.set_needs_repaint(webview_group_id, RepaintReason::NewWebRenderFrame);
        }
    }
}
