/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::hash_map::Entry;
use std::rc::Rc;
use std::sync::Arc;

use base::Epoch;
use base::cross_process_instant::CrossProcessInstant;
use base::generic_channel::GenericSharedMemory;
use base::id::{PainterId, PipelineId, WebViewId};
use compositing_traits::display_list::{PaintDisplayListInfo, ScrollType};
use compositing_traits::largest_contentful_paint_candidate::LCPCandidate;
use compositing_traits::rendering_context::RenderingContext;
use compositing_traits::viewport_description::ViewportDescription;
use compositing_traits::{
    ImageUpdate, PipelineExitSource, SendableFrameTree, WebRenderExternalImageHandlers,
    WebRenderImageHandlerType, WebViewTrait,
};
use constellation_traits::{EmbedderToConstellationMessage, PaintMetricEvent};
use crossbeam_channel::Sender;
use dpi::PhysicalSize;
use embedder_traits::{
    InputEvent, InputEventAndId, InputEventId, InputEventResult, PaintHitTestResult,
    ScreenshotCaptureError, Scroll, ViewportDetails, WebViewPoint, WebViewRect,
};
use euclid::{Point2D, Rect, Scale, Size2D};
use gleam::gl::RENDERER;
use image::RgbaImage;
use ipc_channel::ipc::IpcBytesReceiver;
use log::{debug, info, warn};
use media::WindowGLContext;
use profile_traits::time::{ProfilerCategory, ProfilerChan};
use profile_traits::time_profile;
use rustc_hash::{FxHashMap, FxHashSet};
use servo_config::{opts, pref};
use servo_geometry::DeviceIndependentPixel;
use smallvec::SmallVec;
use style_traits::CSSPixel;
use webrender::{
    MemoryReport, ONE_TIME_USAGE_HINT, RenderApi, ShaderPrecacheFlags, Transaction, UploadMethod,
};
use webrender_api::units::{
    DevicePixel, DevicePoint, LayoutPoint, LayoutRect, LayoutSize, LayoutTransform, LayoutVector2D,
    WorldPoint,
};
use webrender_api::{
    self, BuiltDisplayList, BuiltDisplayListDescriptor, ColorF, DirtyRect, DisplayListPayload,
    DocumentId, Epoch as WebRenderEpoch, ExternalScrollId, FontInstanceFlags, FontInstanceKey,
    FontInstanceOptions, FontKey, FontVariation, ImageKey, NativeFontHandle,
    PipelineId as WebRenderPipelineId, PropertyBinding, ReferenceFrameKind, RenderReasons,
    SampledScrollOffset, SpaceAndClipInfo, SpatialId, TransformStyle,
};
use wr_malloc_size_of::MallocSizeOfOps;

use crate::Paint;
use crate::largest_contentful_paint_calculator::LargestContentfulPaintCalculator;
use crate::paint::{RepaintReason, WebRenderDebugOption};
use crate::refresh_driver::{AnimationRefreshDriverObserver, BaseRefreshDriver};
use crate::render_notifier::RenderNotifier;
use crate::screenshot::ScreenshotTaker;
use crate::webrender_external_images::WebGLExternalImages;
use crate::webview_renderer::{PinchZoomResult, ScrollResult, UnknownWebView, WebViewRenderer};

/// A [`Painter`] is responsible for all of the painting to a particular [`RenderingContext`].
/// This holds all of the WebRender specific data structures and state necessary for painting
/// and handling events that happen to `WebView`s that use a particular [`RenderingContext`].
/// Notable is that a [`Painter`] might be responsible for painting more than a single
/// [`WebView`] as long as they share the same [`RenderingContext`].
///
/// Each [`Painter`] also has its own [`RefreshDriver`] as well, which may be shared with
/// other [`Painter`]s. It's up to the embedder to decide which [`RefreshDriver`]s are associated
/// with a particular [`RenderingContext`].
pub(crate) struct Painter {
    /// The [`RenderingContext`] instance that webrender targets, which is the viewport.
    pub(crate) rendering_context: Rc<dyn RenderingContext>,

    /// The ID of this painter.
    pub(crate) painter_id: PainterId,

    /// Our [`WebViewRenderer`]s, one for every `WebView`.
    pub(crate) webview_renderers: FxHashMap<WebViewId, WebViewRenderer>,

    /// Tracks whether or not the view needs to be repainted.
    pub(crate) needs_repaint: Cell<RepaintReason>,

    /// The number of frames pending to receive from WebRender.
    pub(crate) pending_frames: Cell<usize>,

    /// The [`BaseRefreshDriver`] which manages the painting of `WebView`s during animations.
    refresh_driver: Rc<BaseRefreshDriver>,

    /// A [`RefreshDriverObserver`] for WebView content animations.
    animation_refresh_driver_observer: Rc<AnimationRefreshDriverObserver>,

    /// The WebRender [`RenderApi`] interface used to communicate with WebRender.
    pub(crate) webrender_api: RenderApi,

    /// The active webrender document.
    pub(crate) webrender_document: DocumentId,

    /// The webrender renderer.
    pub(crate) webrender: Option<webrender::Renderer>,

    /// The GL bindings for webrender
    webrender_gl: Rc<dyn gleam::gl::Gl>,

    /// The last position in the rendered view that the mouse moved over. This becomes `None`
    /// when the mouse leaves the rendered view.
    pub(crate) last_mouse_move_position: Option<DevicePoint>,

    /// A [`ScreenshotTaker`] responsible for handling all screenshot requests.
    pub(crate) screenshot_taker: ScreenshotTaker,

    /// A [`FrameRequestDelayer`] which is used to wait for canvas image updates to
    /// arrive before requesting a new frame, as these happen asynchronously with
    /// `ScriptThread` display list construction.
    pub(crate) frame_delayer: FrameDelayer,

    /// The channel on which messages can be sent to the constellation.
    embedder_to_constellation_sender: Sender<EmbedderToConstellationMessage>,

    /// Calculater for largest-contentful-paint.
    lcp_calculator: LargestContentfulPaintCalculator,
}

impl Drop for Painter {
    fn drop(&mut self) {
        if let Err(error) = self.rendering_context.make_current() {
            warn!("Failed to make the rendering context current: {error:?}");
        }

        self.webrender_api.stop_render_backend();
        self.webrender_api.shut_down(true);

        if let Some(webrender) = self.webrender.take() {
            webrender.deinit();
        }
    }
}

impl Painter {
    pub(crate) fn new(rendering_context: Rc<dyn RenderingContext>, paint: &Paint) -> Self {
        let webrender_gl = rendering_context.gleam_gl_api();

        // Make sure the gl context is made current.
        if let Err(err) = rendering_context.make_current() {
            warn!("Failed to make the rendering context current: {:?}", err);
        }
        debug_assert_eq!(webrender_gl.get_error(), gleam::gl::NO_ERROR,);

        let id_manager = paint.webrender_external_image_id_manager();
        let mut external_image_handlers = Box::new(WebRenderExternalImageHandlers::new(id_manager));

        // Set WebRender external image handler for WebGL textures.
        let image_handler = Box::new(WebGLExternalImages::new(
            paint.webgl_threads(),
            rendering_context.clone(),
            paint.swap_chains.clone(),
            paint.busy_webgl_contexts_map.clone(),
        ));
        external_image_handlers.set_handler(image_handler, WebRenderImageHandlerType::WebGl);

        #[cfg(feature = "webgpu")]
        external_image_handlers.set_handler(
            Box::new(webgpu::WebGpuExternalImages::new(paint.webgpu_image_map())),
            WebRenderImageHandlerType::WebGpu,
        );

        WindowGLContext::initialize_image_handler(&mut external_image_handlers);

        let embedder_to_constellation_sender = paint.embedder_to_constellation_sender.clone();
        let refresh_driver = Rc::new(BaseRefreshDriver::new(
            paint.event_loop_waker.clone_box(),
            rendering_context.refresh_driver(),
        ));
        let animation_refresh_driver_observer = Rc::new(AnimationRefreshDriverObserver::new(
            embedder_to_constellation_sender.clone(),
        ));

        rendering_context.prepare_for_rendering();
        let clear_color = servo_config::pref!(shell_background_color_rgba);
        let clear_color = ColorF::new(
            clear_color[0] as f32,
            clear_color[1] as f32,
            clear_color[2] as f32,
            clear_color[3] as f32,
        );

        // Use same texture upload method as Gecko with ANGLE:
        // https://searchfox.org/mozilla-central/source/gfx/webrender_bindings/src/bindings.rs#1215-1219
        let upload_method = if webrender_gl.get_string(RENDERER).starts_with("ANGLE") {
            UploadMethod::Immediate
        } else {
            UploadMethod::PixelBuffer(ONE_TIME_USAGE_HINT)
        };
        let worker_threads = std::thread::available_parallelism()
            .map(|i| i.get())
            .unwrap_or(pref!(threadpools_fallback_worker_num) as usize)
            .min(pref!(threadpools_webrender_workers_max).max(1) as usize);
        let workers = Some(Arc::new(
            rayon::ThreadPoolBuilder::new()
                .num_threads(worker_threads)
                .thread_name(|idx| format!("WRWorker#{}", idx))
                .build()
                .expect("Unable to initialize WebRender worker pool."),
        ));

        let painter_id = PainterId::next();
        let (mut webrender, webrender_api_sender) = webrender::create_webrender_instance(
            webrender_gl.clone(),
            Box::new(RenderNotifier::new(painter_id, paint.paint_proxy.clone())),
            webrender::WebRenderOptions {
                // We force the use of optimized shaders here because rendering is broken
                // on Android emulators with unoptimized shaders. This is due to a known
                // issue in the emulator's OpenGL emulation layer.
                // See: https://github.com/servo/servo/issues/31726
                use_optimized_shaders: true,
                resource_override_path: opts::get().shaders_path.clone(),
                debug_flags: webrender::DebugFlags::empty(),
                precache_flags: if pref!(gfx_precache_shaders) {
                    ShaderPrecacheFlags::FULL_COMPILE
                } else {
                    ShaderPrecacheFlags::empty()
                },
                enable_aa: pref!(gfx_text_antialiasing_enabled),
                enable_subpixel_aa: pref!(gfx_subpixel_text_antialiasing_enabled),
                allow_texture_swizzling: pref!(gfx_texture_swizzling_enabled),
                clear_color,
                upload_method,
                workers,
                size_of_op: Some(servo_allocator::usable_size),
                // This ensures that we can use the `PainterId` as the `IdNamespace`, which allows mapping
                // from `FontKey`, `FontInstanceKey`, and `ImageKey` back to `PainterId`.
                namespace_alloc_by_client: true,
                shared_font_namespace: Some(painter_id.into()),
                ..Default::default()
            },
            None,
        )
        .expect("Unable to initialize WebRender.");

        webrender.set_external_image_handler(external_image_handlers);

        let webrender_api = webrender_api_sender.create_api_by_client(painter_id.into());
        let webrender_document = webrender_api.add_document(rendering_context.size2d().to_i32());

        let gl_renderer = webrender_gl.get_string(gleam::gl::RENDERER);
        let gl_version = webrender_gl.get_string(gleam::gl::VERSION);
        info!("Running on {gl_renderer} with OpenGL version {gl_version}");

        let painter = Painter {
            painter_id,
            embedder_to_constellation_sender,
            webview_renderers: Default::default(),
            rendering_context,
            needs_repaint: Cell::default(),
            pending_frames: Default::default(),
            screenshot_taker: Default::default(),
            refresh_driver,
            animation_refresh_driver_observer,
            webrender: Some(webrender),
            webrender_api,
            webrender_document,
            webrender_gl,
            last_mouse_move_position: None,
            frame_delayer: Default::default(),
            lcp_calculator: LargestContentfulPaintCalculator::new(),
        };
        painter.assert_gl_framebuffer_complete();
        painter.clear_background();
        painter
    }

    pub(crate) fn perform_updates(&mut self) {
        // The WebXR thread may make a different context current
        if let Err(err) = self.rendering_context.make_current() {
            warn!("Failed to make the rendering context current: {:?}", err);
        }

        let mut need_zoom = false;
        let scroll_offset_updates: Vec<_> = self
            .webview_renderers
            .values_mut()
            .filter_map(|webview_renderer| {
                let (zoom, scroll_result) = webview_renderer
                    .process_pending_scroll_and_pinch_zoom_events(&self.webrender_api);
                need_zoom = need_zoom || (zoom == PinchZoomResult::DidPinchZoom);
                scroll_result
            })
            .collect();

        self.send_zoom_and_scroll_offset_updates(need_zoom, scroll_offset_updates);
    }

    #[track_caller]
    fn assert_no_gl_error(&self) {
        debug_assert_eq!(self.webrender_gl.get_error(), gleam::gl::NO_ERROR);
    }

    #[track_caller]
    fn assert_gl_framebuffer_complete(&self) {
        debug_assert_eq!(
            (
                self.webrender_gl.get_error(),
                self.webrender_gl
                    .check_frame_buffer_status(gleam::gl::FRAMEBUFFER)
            ),
            (gleam::gl::NO_ERROR, gleam::gl::FRAMEBUFFER_COMPLETE)
        );
    }

    pub(crate) fn webview_renderer(&self, webview_id: WebViewId) -> Option<&WebViewRenderer> {
        self.webview_renderers.get(&webview_id)
    }

    pub(crate) fn webview_renderer_mut(
        &mut self,
        webview_id: WebViewId,
    ) -> Option<&mut WebViewRenderer> {
        self.webview_renderers.get_mut(&webview_id)
    }

    /// Whether or not the renderer is waiting on a frame, either because it has been sent
    /// to WebRender and is not ready yet or because the [`FrameDelayer`] is delaying a frame
    /// waiting for asynchronous (canvas) image updates to complete.
    pub(crate) fn has_pending_frames(&self) -> bool {
        self.pending_frames.get() != 0 || self.frame_delayer.pending_frame
    }

    pub(crate) fn set_needs_repaint(&self, reason: RepaintReason) {
        let mut needs_repaint = self.needs_repaint.get();
        needs_repaint.insert(reason);
        self.needs_repaint.set(needs_repaint);
    }

    pub(crate) fn needs_repaint(&self) -> bool {
        let repaint_reason = self.needs_repaint.get();
        if repaint_reason.is_empty() {
            return false;
        }

        !self.refresh_driver.wait_to_paint()
    }

    /// Returns true if any animation callbacks (ie `requestAnimationFrame`) are waiting for a response.
    pub(crate) fn animation_callbacks_running(&self) -> bool {
        self.webview_renderers
            .values()
            .any(WebViewRenderer::animation_callbacks_running)
    }

    pub(crate) fn animating_webviews(&self) -> Vec<WebViewId> {
        self.webview_renderers
            .values()
            .filter_map(|webview_renderer| {
                if webview_renderer.animating() {
                    Some(webview_renderer.id)
                } else {
                    None
                }
            })
            .collect()
    }

    pub(crate) fn send_to_constellation(&self, message: EmbedderToConstellationMessage) {
        if let Err(error) = self.embedder_to_constellation_sender.send(message) {
            warn!("Could not send message to constellation ({error:?})");
        }
    }

    #[servo_tracing::instrument(skip_all)]
    pub(crate) fn render(&mut self, time_profiler_channel: &ProfilerChan) {
        let refresh_driver = self.refresh_driver.clone();
        refresh_driver.notify_will_paint(self);

        if let Err(err) = self.rendering_context.make_current() {
            warn!("Failed to make the rendering context current: {:?}", err);
        }
        self.assert_no_gl_error();

        self.rendering_context.prepare_for_rendering();

        time_profile!(
            ProfilerCategory::Painting,
            None,
            time_profiler_channel.clone(),
            || {
                if let Some(webrender) = self.webrender.as_mut() {
                    webrender.update();
                }

                // Paint the scene.
                // TODO(gw): Take notice of any errors the renderer returns!
                self.clear_background();
                if let Some(webrender) = self.webrender.as_mut() {
                    let size = self.rendering_context.size2d().to_i32();
                    webrender.render(size, 0 /* buffer_age */).ok();
                }
            }
        );

        // We've painted the default target, which means that from the embedder's perspective,
        // the scene no longer needs to be repainted.
        self.needs_repaint.set(RepaintReason::empty());

        self.screenshot_taker.maybe_take_screenshots(self);
        self.send_pending_paint_metrics_messages_after_composite();
    }

    fn clear_background(&self) {
        self.assert_gl_framebuffer_complete();

        // Always clear the entire RenderingContext, regardless of how many WebViews there are
        // or where they are positioned. This is so WebView actually clears even before the
        // first WebView is ready.
        let color = servo_config::pref!(shell_background_color_rgba);
        self.webrender_gl.clear_color(
            color[0] as f32,
            color[1] as f32,
            color[2] as f32,
            color[3] as f32,
        );
        self.webrender_gl.clear(gleam::gl::COLOR_BUFFER_BIT);
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
        for webview_renderer in self.webview_renderers.values() {
            for (pipeline_id, pipeline) in webview_renderer.pipelines.iter() {
                let Some(current_epoch) = self
                    .webrender
                    .as_ref()
                    .and_then(|wr| wr.current_epoch(self.webrender_document, pipeline_id.into()))
                else {
                    continue;
                };

                match pipeline.first_paint_metric.get() {
                    // We need to check whether the current epoch is later, because
                    // CrossProcessPaintMessage::SendInitialTransaction sends an
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

                        self.send_to_constellation(EmbedderToConstellationMessage::PaintMetric(
                            *pipeline_id,
                            PaintMetricEvent::FirstPaint(paint_time, first_reflow),
                        ));

                        pipeline.first_paint_metric.set(PaintMetricState::Sent);
                    },
                    _ => {},
                }

                match pipeline.first_contentful_paint_metric.get() {
                    PaintMetricState::Seen(epoch, first_reflow) if epoch <= current_epoch => {
                        #[cfg(feature = "tracing")]
                        tracing::info!(
                            name: "FirstContentfulPaint",
                            servo_profiling = true,
                            epoch = ?epoch,
                            paint_time = ?paint_time,
                            pipeline_id = ?pipeline_id,
                        );
                        self.send_to_constellation(EmbedderToConstellationMessage::PaintMetric(
                            *pipeline_id,
                            PaintMetricEvent::FirstContentfulPaint(paint_time, first_reflow),
                        ));
                        pipeline
                            .first_contentful_paint_metric
                            .set(PaintMetricState::Sent);
                    },
                    _ => {},
                }

                match pipeline.largest_contentful_paint_metric.get() {
                    PaintMetricState::Seen(epoch, _) if epoch <= current_epoch => {
                        if let Some(lcp) = self
                            .lcp_calculator
                            .calculate_largest_contentful_paint(paint_time, pipeline_id.into())
                        {
                            #[cfg(feature = "tracing")]
                            tracing::info!(
                                name: "LargestContentfulPaint",
                                servo_profiling = true,
                                paint_time = ?paint_time,
                                area = ?lcp.area,
                                lcp_type = ?lcp.lcp_type,
                                pipeline_id = ?pipeline_id,
                            );
                            self.send_to_constellation(
                                EmbedderToConstellationMessage::PaintMetric(
                                    *pipeline_id,
                                    PaintMetricEvent::LargestContentfulPaint(
                                        lcp.paint_time,
                                        lcp.area,
                                        lcp.lcp_type,
                                    ),
                                ),
                            );
                        }
                        pipeline
                            .largest_contentful_paint_metric
                            .set(PaintMetricState::Sent);
                    },
                    _ => {},
                }
            }
        }
    }

    /// Queue a new frame in the transaction and increase the pending frames count.
    pub(crate) fn generate_frame(&self, transaction: &mut Transaction, reason: RenderReasons) {
        transaction.generate_frame(0, true /* present */, false /* tracked */, reason);
        self.pending_frames.set(self.pending_frames.get() + 1);
    }

    pub(crate) fn hit_test_at_point_with_api_and_document(
        webrender_api: &RenderApi,
        webrender_document: DocumentId,
        point: DevicePoint,
    ) -> Vec<PaintHitTestResult> {
        // DevicePoint and WorldPoint are the same for us.
        let world_point = WorldPoint::from_untyped(point.to_untyped());
        let results = webrender_api.hit_test(webrender_document, world_point);

        results
            .items
            .iter()
            .map(|item| {
                let pipeline_id = item.pipeline.into();
                let external_scroll_id = ExternalScrollId(item.tag.0, item.pipeline);
                PaintHitTestResult {
                    pipeline_id,
                    point_in_viewport: Point2D::from_untyped(item.point_in_viewport.to_untyped()),
                    external_scroll_id,
                }
            })
            .collect()
    }

    pub(crate) fn send_transaction(&mut self, transaction: Transaction) {
        let _ = self.rendering_context.make_current();
        self.webrender_api
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

        let root_reference_frame = SpatialId::root_reference_frame(root_pipeline);

        let viewport_size = self.rendering_context.size2d().to_f32().to_untyped();
        let viewport_rect = LayoutRect::from_origin_and_size(
            LayoutPoint::zero(),
            LayoutSize::from_untyped(viewport_size),
        );

        let root_clip_id = builder.define_clip_rect(root_reference_frame, viewport_rect);
        let clip_chain_id = builder.define_clip_chain(None, [root_clip_id]);
        for webview_renderer in self.webview_renderers.values() {
            if webview_renderer.hidden() {
                continue;
            }
            let Some(pipeline_id) = webview_renderer.root_pipeline_id else {
                continue;
            };

            let pinch_zoom_transform = webview_renderer.pinch_zoom().transform().to_untyped();
            let device_pixels_per_page_pixel_not_including_pinch_zoom = webview_renderer
                .device_pixels_per_page_pixel_not_including_pinch_zoom()
                .get();

            let transform = LayoutTransform::scale(
                device_pixels_per_page_pixel_not_including_pinch_zoom,
                device_pixels_per_page_pixel_not_including_pinch_zoom,
                1.0,
            )
            .then(&LayoutTransform::from_untyped(
                &pinch_zoom_transform.to_3d(),
            ));

            let webview_reference_frame = builder.push_reference_frame(
                LayoutPoint::zero(),
                root_reference_frame,
                TransformStyle::Flat,
                PropertyBinding::Value(transform),
                ReferenceFrameKind::Transform {
                    is_2d_scale_translation: true,
                    should_snap: true,
                    paired_with_perspective: false,
                },
                webview_renderer.id.into(),
            );

            let scaled_webview_rect = webview_renderer.rect /
                webview_renderer.device_pixels_per_page_pixel_not_including_pinch_zoom();
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

    /// Set the root pipeline for our WebRender scene to a display list that consists of an iframe
    /// for each visible top-level browsing context, applying a transformation on the root for
    /// pinch zoom, page zoom, and HiDPI scaling.
    fn send_root_pipeline_display_list(&mut self) {
        let mut transaction = Transaction::new();
        self.send_root_pipeline_display_list_in_transaction(&mut transaction);
        self.generate_frame(&mut transaction, RenderReasons::SCENE);
        self.send_transaction(transaction);
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
        for webview_renderer in self.webview_renderers.values() {
            for details in webview_renderer.pipelines.values() {
                for node in details.scroll_tree.nodes.iter() {
                    let (Some(offset), Some(external_id)) = (node.offset(), node.external_id())
                    else {
                        continue;
                    };
                    // Skip scroll offsets that are zero, as they are the default.
                    if offset == LayoutVector2D::zero() {
                        continue;
                    }
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

    fn send_zoom_and_scroll_offset_updates(
        &mut self,
        need_zoom: bool,
        scroll_offset_updates: Vec<ScrollResult>,
    ) {
        if !need_zoom && scroll_offset_updates.is_empty() {
            return;
        }

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
        self.send_transaction(transaction);
    }

    pub(crate) fn toggle_webrender_debug(&mut self, option: WebRenderDebugOption) {
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
        self.send_transaction(txn);
    }

    pub(crate) fn decrement_pending_frames(&self) {
        self.pending_frames.set(self.pending_frames.get() - 1);
    }

    pub(crate) fn report_memory(&self) -> MemoryReport {
        self.webrender_api
            .report_memory(MallocSizeOfOps::new(servo_allocator::usable_size, None))
    }

    pub(crate) fn change_running_animations_state(
        &mut self,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        animation_state: embedder_traits::AnimationState,
    ) {
        let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) else {
            return;
        };
        if !webview_renderer.change_pipeline_running_animations_state(pipeline_id, animation_state)
        {
            return;
        }
        if !self
            .animation_refresh_driver_observer
            .notify_animation_state_changed(webview_renderer)
        {
            return;
        }

        self.refresh_driver
            .add_observer(self.animation_refresh_driver_observer.clone());
    }

    pub(crate) fn set_frame_tree_for_webview(&mut self, frame_tree: &SendableFrameTree) {
        debug!("{}: Setting frame tree for webview", frame_tree.pipeline.id);

        let webview_id = frame_tree.pipeline.webview_id;
        let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) else {
            warn!(
                "Attempted to set frame tree on unknown WebView (perhaps closed?): {webview_id:?}"
            );
            return;
        };

        webview_renderer.set_frame_tree(frame_tree);
        self.send_root_pipeline_display_list();
    }

    pub(crate) fn set_throttled(
        &mut self,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        throttled: bool,
    ) {
        let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) else {
            return;
        };
        if !webview_renderer.set_throttled(pipeline_id, throttled) {
            return;
        }

        if self
            .animation_refresh_driver_observer
            .notify_animation_state_changed(webview_renderer)
        {
            self.refresh_driver
                .add_observer(self.animation_refresh_driver_observer.clone());
        }
    }

    pub(crate) fn notify_pipeline_exited(
        &mut self,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        pipeline_exit_source: PipelineExitSource,
    ) {
        debug!("Paint got pipeline exited: {webview_id:?} {pipeline_id:?}",);
        if let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) {
            webview_renderer.pipeline_exited(pipeline_id, pipeline_exit_source);
        }
        self.lcp_calculator
            .remove_lcp_candidates_for_pipeline(pipeline_id.into());
    }

    pub(crate) fn send_initial_pipeline_transaction(
        &mut self,
        webview_id: WebViewId,
        pipeline_id: WebRenderPipelineId,
    ) {
        let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) else {
            return warn!("Could not find WebView for incoming display list");
        };

        let starting_epoch = Epoch(0);
        let details = webview_renderer.ensure_pipeline_details(pipeline_id.into());
        details.display_list_epoch = Some(starting_epoch);

        let mut txn = Transaction::new();
        txn.set_display_list(starting_epoch.into(), (pipeline_id, Default::default()));

        self.generate_frame(&mut txn, RenderReasons::SCENE);
        self.send_transaction(txn);
    }

    pub(crate) fn scroll_node_by_delta(
        &mut self,
        webview_id: WebViewId,
        pipeline_id: WebRenderPipelineId,
        offset: LayoutVector2D,
        external_scroll_id: webrender_api::ExternalScrollId,
    ) {
        let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) else {
            return;
        };

        let pipeline_id = pipeline_id.into();
        let Some(pipeline_details) = webview_renderer.pipelines.get_mut(&pipeline_id) else {
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

        let mut transaction = Transaction::new();
        transaction.set_scroll_offsets(
            external_scroll_id,
            vec![SampledScrollOffset {
                offset,
                generation: 0,
            }],
        );

        self.generate_frame(&mut transaction, RenderReasons::APZ);
        self.send_transaction(transaction);
    }

    pub(crate) fn scroll_viewport_by_delta(
        &mut self,
        webview_id: WebViewId,
        delta: LayoutVector2D,
    ) {
        let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) else {
            return;
        };
        let (pinch_zoom_result, scroll_results) = webview_renderer.scroll_viewport_by_delta(delta);
        self.send_zoom_and_scroll_offset_updates(
            pinch_zoom_result == PinchZoomResult::DidPinchZoom,
            scroll_results,
        );
    }

    pub(crate) fn update_epoch(
        &mut self,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        epoch: Epoch,
    ) {
        let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) else {
            return warn!("Could not find WebView for Epoch update.");
        };
        webview_renderer
            .ensure_pipeline_details(pipeline_id)
            .display_list_epoch = Some(Epoch(epoch.0));
    }

    pub(crate) fn handle_new_display_list(
        &mut self,
        webview_id: WebViewId,
        display_list_descriptor: BuiltDisplayListDescriptor,
        display_list_receiver: IpcBytesReceiver,
    ) {
        // This must match the order from the sender, currently in `shared/script/lib.rs`.
        let display_list_info = match display_list_receiver.recv() {
            Ok(display_list_info) => display_list_info,
            Err(error) => {
                return warn!("Could not receive display list info: {error}");
            },
        };
        let display_list_info: PaintDisplayListInfo = match bincode::deserialize(&display_list_info)
        {
            Ok(display_list_info) => display_list_info,
            Err(error) => {
                return warn!("Could not deserialize display list info: {error}");
            },
        };
        let items_data = match display_list_receiver.recv() {
            Ok(display_list_data) => display_list_data,
            Err(error) => {
                return warn!("Could not receive WebRender display list items data: {error}");
            },
        };
        let cache_data = match display_list_receiver.recv() {
            Ok(display_list_data) => display_list_data,
            Err(error) => {
                return warn!("Could not receive WebRender display list cache data: {error}");
            },
        };
        let spatial_tree = match display_list_receiver.recv() {
            Ok(display_list_data) => display_list_data,
            Err(error) => {
                return warn!("Could not receive WebRender display list spatial tree: {error}.");
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
        let _span = profile_traits::trace_span!("PaintMessage::SendDisplayList",).entered();
        let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) else {
            return warn!("Could not find WebView for incoming display list");
        };

        let old_scale = webview_renderer.device_pixels_per_page_pixel();
        let pipeline_id = display_list_info.pipeline_id;
        let details = webview_renderer.ensure_pipeline_details(pipeline_id.into());

        details.install_new_scroll_tree(display_list_info.scroll_tree);
        details.viewport_scale = Some(display_list_info.viewport_details.hidpi_scale_factor);

        let epoch = display_list_info.epoch.into();
        let first_reflow = display_list_info.first_reflow;
        if details.first_paint_metric.get() == PaintMetricState::Waiting {
            details
                .first_paint_metric
                .set(PaintMetricState::Seen(epoch, first_reflow));
        }

        if details.first_contentful_paint_metric.get() == PaintMetricState::Waiting &&
            display_list_info.is_contentful
        {
            details
                .first_contentful_paint_metric
                .set(PaintMetricState::Seen(epoch, first_reflow));
        }

        let mut transaction = Transaction::new();
        let is_root_pipeline = Some(pipeline_id.into()) == webview_renderer.root_pipeline_id;
        if is_root_pipeline && old_scale != webview_renderer.device_pixels_per_page_pixel() {
            self.send_root_pipeline_display_list_in_transaction(&mut transaction);
        }

        transaction.set_display_list(epoch, (pipeline_id, built_display_list));

        self.update_transaction_with_all_scroll_offsets(&mut transaction);
        self.send_transaction(transaction);
    }

    pub(crate) fn generate_frame_for_script(&mut self) {
        self.frame_delayer.set_pending_frame(true);

        if !self.frame_delayer.needs_new_frame() {
            return;
        }

        let mut transaction = Transaction::new();
        self.generate_frame(&mut transaction, RenderReasons::SCENE);
        self.send_transaction(transaction);

        let waiting_pipelines = self.frame_delayer.take_waiting_pipelines();

        self.send_to_constellation(
            EmbedderToConstellationMessage::NoLongerWaitingOnAsynchronousImageUpdates(
                waiting_pipelines,
            ),
        );

        self.frame_delayer.set_pending_frame(false);
        self.screenshot_taker
            .prepare_screenshot_requests_for_render(self)
    }

    pub(crate) fn update_images(&mut self, updates: SmallVec<[ImageUpdate; 1]>) {
        let mut txn = Transaction::new();
        for update in updates {
            match update {
                ImageUpdate::AddImage(key, desc, data) => {
                    txn.add_image(key, desc, data.into(), None)
                },
                ImageUpdate::DeleteImage(key) => {
                    txn.delete_image(key);
                    self.frame_delayer.delete_image(key);
                },
                ImageUpdate::UpdateImage(key, desc, data, epoch) => {
                    if let Some(epoch) = epoch {
                        self.frame_delayer.update_image(key, epoch);
                    }
                    txn.update_image(key, desc, data.into(), &DirtyRect::All)
                },
            }
        }

        if self.frame_delayer.needs_new_frame() {
            self.frame_delayer.set_pending_frame(false);
            self.generate_frame(&mut txn, RenderReasons::SCENE);
            let waiting_pipelines = self.frame_delayer.take_waiting_pipelines();

            self.send_to_constellation(
                EmbedderToConstellationMessage::NoLongerWaitingOnAsynchronousImageUpdates(
                    waiting_pipelines,
                ),
            );

            self.screenshot_taker
                .prepare_screenshot_requests_for_render(&*self);
        }

        self.send_transaction(txn);
    }

    pub(crate) fn delay_new_frames_for_canvas(
        &mut self,
        pipeline_id: PipelineId,
        canvas_epoch: Epoch,
        image_keys: Vec<ImageKey>,
    ) {
        self.frame_delayer
            .add_delay(pipeline_id, canvas_epoch, image_keys);
    }

    pub(crate) fn add_font(
        &mut self,
        font_key: FontKey,
        data: Arc<GenericSharedMemory>,
        index: u32,
    ) {
        let mut transaction = Transaction::new();
        transaction.add_raw_font(font_key, (**data).into(), index);
        self.send_transaction(transaction);
    }

    pub(crate) fn add_system_font(&mut self, font_key: FontKey, native_handle: NativeFontHandle) {
        let mut transaction = Transaction::new();
        transaction.add_native_font(font_key, native_handle);
        self.send_transaction(transaction);
    }

    pub(crate) fn add_font_instance(
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

        self.send_transaction(transaction);
    }

    pub(crate) fn remove_fonts(&mut self, keys: Vec<FontKey>, instance_keys: Vec<FontInstanceKey>) {
        let mut transaction = Transaction::new();

        for instance in instance_keys.into_iter() {
            transaction.delete_font_instance(instance);
        }
        for key in keys.into_iter() {
            transaction.delete_font(key);
        }

        self.send_transaction(transaction);
    }

    pub(crate) fn set_viewport_description(
        &mut self,
        webview_id: WebViewId,
        viewport_description: ViewportDescription,
    ) {
        if let Some(webview) = self.webview_renderers.get_mut(&webview_id) {
            webview.set_viewport_description(viewport_description);
        }
    }

    pub(crate) fn handle_screenshot_readiness_reply(
        &self,
        webview_id: WebViewId,
        expected_epochs: FxHashMap<PipelineId, Epoch>,
    ) {
        self.screenshot_taker
            .handle_screenshot_readiness_reply(webview_id, expected_epochs, self);
    }

    pub(crate) fn add_webview(
        &mut self,
        webview: Box<dyn WebViewTrait>,
        viewport_details: ViewportDetails,
    ) {
        self.webview_renderers
            .entry(webview.id())
            .or_insert(WebViewRenderer::new(
                webview,
                viewport_details,
                self.embedder_to_constellation_sender.clone(),
                self.refresh_driver.clone(),
                self.webrender_document,
            ));
    }

    pub(crate) fn remove_webview(&mut self, webview_id: WebViewId) {
        if self.webview_renderers.remove(&webview_id).is_none() {
            warn!("Tried removing unknown WebView: {webview_id:?}");
            return;
        };

        self.send_root_pipeline_display_list();
        self.lcp_calculator.note_webview_removed(webview_id);
    }

    pub(crate) fn is_empty(&mut self) -> bool {
        self.webview_renderers.is_empty()
    }

    pub(crate) fn set_webview_hidden(
        &mut self,
        webview_id: WebViewId,
        hidden: bool,
    ) -> Result<(), UnknownWebView> {
        debug!("Setting WebView visiblity for {webview_id:?} to hidden={hidden}");
        let Some(webview_renderer) = self.webview_renderer_mut(webview_id) else {
            return Err(UnknownWebView(webview_id));
        };
        if !webview_renderer.set_hidden(hidden) {
            return Ok(());
        }
        self.send_root_pipeline_display_list();
        Ok(())
    }

    pub(crate) fn set_hidpi_scale_factor(
        &mut self,
        webview_id: WebViewId,
        new_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    ) {
        let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) else {
            return;
        };
        if !webview_renderer.set_hidpi_scale_factor(new_scale_factor) {
            return;
        }

        self.send_root_pipeline_display_list();
        self.set_needs_repaint(RepaintReason::Resize);
    }

    pub(crate) fn resize_rendering_context(&mut self, new_size: PhysicalSize<u32>) {
        if self.rendering_context.size() == new_size {
            return;
        }

        self.rendering_context.resize(new_size);

        let new_size = Size2D::new(new_size.width as f32, new_size.height as f32);
        let new_viewport_rect = Rect::from(new_size).to_box2d();
        for webview_renderer in self.webview_renderers.values_mut() {
            webview_renderer.set_rect(new_viewport_rect);
        }

        let mut transaction = Transaction::new();
        transaction.set_document_view(new_viewport_rect.to_i32());
        self.send_transaction(transaction);

        self.send_root_pipeline_display_list();
        self.set_needs_repaint(RepaintReason::Resize);
    }

    pub(crate) fn set_page_zoom(&mut self, webview_id: WebViewId, new_zoom: f32) {
        if let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) {
            webview_renderer.set_page_zoom(Scale::new(new_zoom));
        }
    }

    pub(crate) fn page_zoom(&self, webview_id: WebViewId) -> f32 {
        self.webview_renderers
            .get(&webview_id)
            .map(|webview_renderer| webview_renderer.page_zoom.get())
            .unwrap_or_default()
    }

    pub(crate) fn notify_input_event(&mut self, webview_id: WebViewId, event: InputEventAndId) {
        if let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) {
            match &event.event {
                InputEvent::MouseMove(event) => {
                    // We only track the last mouse move position for non-touch events.
                    if !event.is_compatibility_event_for_touch {
                        let event_point = event
                            .point
                            .as_device_point(webview_renderer.device_pixels_per_page_pixel());
                        self.last_mouse_move_position = Some(event_point);
                    }
                },
                InputEvent::MouseLeftViewport(_) => {
                    self.last_mouse_move_position = None;
                },
                _ => {},
            }

            webview_renderer.notify_input_event(&self.webrender_api, &self.needs_repaint, event);
        }
        self.disable_lcp_calculation_for_webview(webview_id);
    }

    pub(crate) fn notify_scroll_event(
        &mut self,
        webview_id: WebViewId,
        scroll: Scroll,
        point: WebViewPoint,
    ) {
        if let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) {
            webview_renderer.notify_scroll_event(scroll, point);
        }
        self.disable_lcp_calculation_for_webview(webview_id);
    }

    pub(crate) fn pinch_zoom(
        &mut self,
        webview_id: WebViewId,
        pinch_zoom_delta: f32,
        center: DevicePoint,
    ) {
        if let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) {
            webview_renderer.adjust_pinch_zoom(pinch_zoom_delta, center);
        }
    }

    pub(crate) fn device_pixels_per_page_pixel(
        &self,
        webview_id: WebViewId,
    ) -> Scale<f32, CSSPixel, DevicePixel> {
        self.webview_renderers
            .get(&webview_id)
            .map(WebViewRenderer::device_pixels_per_page_pixel)
            .unwrap_or_default()
    }

    pub(crate) fn request_screenshot(
        &self,
        webview_id: WebViewId,
        rect: Option<WebViewRect>,
        callback: Box<dyn FnOnce(Result<RgbaImage, ScreenshotCaptureError>) + 'static>,
    ) {
        let Some(webview) = self.webview_renderers.get(&webview_id) else {
            return;
        };

        let rect = rect.map(|rect| rect.as_device_rect(webview.device_pixels_per_page_pixel()));
        self.screenshot_taker
            .request_screenshot(webview_id, rect, callback);
        self.send_to_constellation(EmbedderToConstellationMessage::RequestScreenshotReadiness(
            webview_id,
        ));
    }

    pub(crate) fn notify_input_event_handled(
        &mut self,
        webview_id: WebViewId,
        input_event_id: InputEventId,
        result: InputEventResult,
    ) {
        let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) else {
            warn!("Handled input event for unknown webview: {webview_id}");
            return;
        };
        webview_renderer.notify_input_event_handled(
            &self.webrender_api,
            &self.needs_repaint,
            input_event_id,
            result,
        );
    }

    pub(crate) fn refresh_cursor(&self) {
        let Some(last_mouse_move_position) = self.last_mouse_move_position else {
            return;
        };

        let Some(hit_test_result) = Self::hit_test_at_point_with_api_and_document(
            &self.webrender_api,
            self.webrender_document,
            last_mouse_move_position,
        )
        .first()
        .cloned() else {
            return;
        };

        if let Err(error) = self.embedder_to_constellation_sender.send(
            EmbedderToConstellationMessage::RefreshCursor(hit_test_result.pipeline_id),
        ) {
            warn!("Sending event to constellation failed ({:?}).", error);
        }
    }

    pub(crate) fn handle_new_webrender_frame_ready(&self, repaint_needed: bool) {
        if repaint_needed {
            self.refresh_cursor()
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

    pub(crate) fn webviews_needing_repaint(&self) -> Vec<WebViewId> {
        if self.needs_repaint() {
            self.webview_renderers
                .values()
                .map(|webview_renderer| webview_renderer.id)
                .collect()
        } else {
            Vec::new()
        }
    }

    pub(crate) fn scroll_trees_memory_usage(
        &self,
        ops: &mut malloc_size_of::MallocSizeOfOps,
    ) -> usize {
        self.webview_renderers
            .values()
            .map(|renderer| renderer.scroll_trees_memory_usage(ops))
            .sum::<usize>()
    }

    pub(crate) fn append_lcp_candidate(
        &mut self,
        lcp_candidate: LCPCandidate,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        epoch: Epoch,
    ) {
        if self
            .lcp_calculator
            .append_lcp_candidate(webview_id, pipeline_id.into(), lcp_candidate)
        {
            if let Some(webview_renderer) = self.webview_renderers.get_mut(&webview_id) {
                webview_renderer
                    .ensure_pipeline_details(pipeline_id)
                    .largest_contentful_paint_metric
                    .set(PaintMetricState::Seen(epoch.into(), false));
            }
        };
    }

    /// Disable LCP feature when the user interacts with the page.
    fn disable_lcp_calculation_for_webview(&mut self, webview_id: WebViewId) {
        self.lcp_calculator.disable_for_webview(webview_id);
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
pub(crate) struct FrameDelayer {
    /// The latest [`Epoch`] of canvas images that have been sent to WebRender. Note
    /// that this only records the `Epoch`s for canvases and only ones that are involved
    /// in "update the rendering".
    image_epochs: FxHashMap<ImageKey, Epoch>,
    /// A map of all pending canvas images
    pending_canvas_images: FxHashMap<ImageKey, Epoch>,
    /// Whether or not we have a pending frame.
    pub(crate) pending_frame: bool,
    /// A list of pipelines that should be notified when we are no longer waiting for
    /// canvas images.
    waiting_pipelines: FxHashSet<PipelineId>,
}

impl FrameDelayer {
    pub(crate) fn delete_image(&mut self, image_key: ImageKey) {
        self.image_epochs.remove(&image_key);
        self.pending_canvas_images.remove(&image_key);
    }

    pub(crate) fn update_image(&mut self, image_key: ImageKey, epoch: Epoch) {
        self.image_epochs.insert(image_key, epoch);
        let Entry::Occupied(entry) = self.pending_canvas_images.entry(image_key) else {
            return;
        };
        if *entry.get() <= epoch {
            entry.remove();
        }
    }

    pub(crate) fn add_delay(
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

    pub(crate) fn needs_new_frame(&self) -> bool {
        self.pending_frame && self.pending_canvas_images.is_empty()
    }

    pub(crate) fn set_pending_frame(&mut self, value: bool) {
        self.pending_frame = value;
    }

    pub(crate) fn take_waiting_pipelines(&mut self) -> Vec<PipelineId> {
        self.waiting_pipelines.drain().collect()
    }
}

/// The paint status of a particular pipeline in a [`Painter`]. This is used to trigger metrics
/// in script (via the constellation) when display lists are received.
///
/// See <https://w3c.github.io/paint-timing/#first-contentful-paint>.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum PaintMetricState {
    /// The painter is still waiting to process a display list which triggers this metric.
    Waiting,
    /// The painter has processed the display list which will trigger this event, marked the Servo
    /// instance ready to paint, and is waiting for the given epoch to actually be rendered.
    Seen(WebRenderEpoch, bool /* first_reflow */),
    /// The metric has been sent to the constellation and no more work needs to be done.
    Sent,
}
