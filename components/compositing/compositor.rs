/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use CompositorMsg as ConstellationMsg;
use app_units::Au;
use compositor_layer::{CompositorData, CompositorLayer, RcCompositorLayer, WantsScrollEventsFlag};
use compositor_thread::{CompositorEventListener, CompositorProxy};
use compositor_thread::{CompositorReceiver, InitialCompositorState, Msg};
use constellation::SendableFrameTree;
use euclid::point::TypedPoint2D;
use euclid::rect::TypedRect;
use euclid::scale_factor::ScaleFactor;
use euclid::size::TypedSize2D;
use euclid::{Matrix4, Point2D, Rect, Size2D};
use gfx::paint_thread::{ChromeToPaintMsg, PaintRequest};
use gfx_traits::{color, Epoch, FrameTreeId, LayerId, LayerKind, LayerProperties, ScrollPolicy};
use gleam::gl;
use gleam::gl::types::{GLint, GLsizei};
use image::{DynamicImage, ImageFormat, RgbImage};
use ipc_channel::ipc::{self, IpcSender, IpcSharedMemory};
use ipc_channel::router::ROUTER;
use layers::geometry::{DevicePixel, LayerPixel};
use layers::layers::{BufferRequest, Layer, LayerBuffer, LayerBufferSet};
use layers::platform::surface::NativeDisplay;
use layers::rendergl;
use layers::rendergl::RenderContext;
use layers::scene::Scene;
use layout_traits::LayoutControlChan;
use msg::constellation_msg::{Image, PixelFormat};
use msg::constellation_msg::{Key, KeyModifiers, KeyState, LoadData};
use msg::constellation_msg::{NavigationDirection, PipelineId, WindowSizeData};
use pipeline::CompositionPipeline;
use profile_traits::mem::{self, ReportKind, Reporter, ReporterRequest};
use profile_traits::time::{self, ProfilerCategory, profile};
use script_traits::CompositorEvent::{MouseMoveEvent, TouchEvent};
use script_traits::{AnimationState, ConstellationControlMsg, LayoutControlMsg};
use script_traits::{MouseButton, TouchEventType, TouchId};
use scrolling::ScrollingTimerProxy;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::mem as std_mem;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use style_traits::viewport::ViewportConstraints;
use surface_map::SurfaceMap;
use time::{precise_time_ns, precise_time_s};
use touch::{TouchHandler, TouchAction};
use url::Url;
use util::geometry::{PagePx, ScreenPx, ViewportPx};
use util::opts;
use util::print_tree::PrintTree;
use windowing::{self, MouseWindowEvent, WindowEvent, WindowMethods, WindowNavigateMsg};

#[derive(Debug, PartialEq)]
pub enum UnableToComposite {
    NoContext,
    WindowUnprepared,
    NotReadyToPaintImage(NotReadyToPaint),
}

#[derive(Debug, PartialEq)]
pub enum NotReadyToPaint {
    LayerHasOutstandingPaintMessages,
    MissingRoot,
    PendingSubpages(usize),
    AnimationsRunning,
    AnimationCallbacksRunning,
    JustNotifiedConstellation,
    WaitingOnConstellation,
}

const BUFFER_MAP_SIZE: usize = 10000000;

// Default viewport constraints
const MAX_ZOOM: f32 = 8.0;
const MIN_ZOOM: f32 = 0.1;

/// Holds the state when running reftests that determines when it is
/// safe to save the output image.
#[derive(Copy, Clone, PartialEq)]
enum ReadyState {
    Unknown,
    WaitingForConstellationReply,
    ReadyToSaveImage,
}

/// NB: Never block on the constellation, because sometimes the constellation blocks on us.
pub struct IOCompositor<Window: WindowMethods> {
    /// The application window.
    window: Rc<Window>,

    /// The display this compositor targets.
    native_display: NativeDisplay,

    /// The port on which we receive messages.
    port: Box<CompositorReceiver>,

    /// The render context. This will be `None` if the windowing system has not yet sent us a
    /// `PrepareRenderingEvent`.
    context: Option<RenderContext>,

    /// The root pipeline.
    root_pipeline: Option<CompositionPipeline>,

    /// Tracks details about each active pipeline that the compositor knows about.
    pipeline_details: HashMap<PipelineId, PipelineDetails>,

    /// The canvas to paint a page.
    scene: Scene<CompositorData>,

    /// The application window size.
    window_size: TypedSize2D<DevicePixel, u32>,

    /// The overridden viewport.
    viewport: Option<(TypedPoint2D<DevicePixel, u32>, TypedSize2D<DevicePixel, u32>)>,

    /// "Mobile-style" zoom that does not reflow the page.
    viewport_zoom: ScaleFactor<PagePx, ViewportPx, f32>,

    /// Viewport zoom constraints provided by @viewport.
    min_viewport_zoom: Option<ScaleFactor<PagePx, ViewportPx, f32>>,
    max_viewport_zoom: Option<ScaleFactor<PagePx, ViewportPx, f32>>,

    /// "Desktop-style" zoom that resizes the viewport to fit the window.
    /// See `ViewportPx` docs in util/geom.rs for details.
    page_zoom: ScaleFactor<ViewportPx, ScreenPx, f32>,

    /// The device pixel ratio for this window.
    hidpi_factor: ScaleFactor<ScreenPx, DevicePixel, f32>,

    channel_to_self: Box<CompositorProxy + Send>,

    /// A handle to the scrolling timer.
    scrolling_timer: ScrollingTimerProxy,

    /// The type of composition to perform
    composite_target: CompositeTarget,

    /// Tracks whether we should composite this frame.
    composition_request: CompositionRequest,

    /// Tracks whether we are in the process of shutting down, or have shut down and should close
    /// the compositor.
    shutdown_state: ShutdownState,

    /// Tracks the last composite time.
    last_composite_time: u64,

    /// Tracks whether the zoom action has happened recently.
    zoom_action: bool,

    /// The time of the last zoom action has started.
    zoom_time: f64,

    /// Whether the page being rendered has loaded completely.
    /// Differs from ReadyState because we can finish loading (ready)
    /// many times for a single page.
    got_load_complete_message: bool,

    /// The current frame tree ID (used to reject old paint buffers)
    frame_tree_id: FrameTreeId,

    /// The channel on which messages can be sent to the constellation.
    constellation_chan: Sender<ConstellationMsg>,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: time::ProfilerChan,

    /// The channel on which messages can be sent to the memory profiler.
    mem_profiler_chan: mem::ProfilerChan,

    /// Pending scroll to fragment event, if any
    fragment_point: Option<Point2D<f32>>,

    /// Touch input state machine
    touch_handler: TouchHandler,

    /// Pending scroll/zoom events.
    pending_scroll_zoom_events: Vec<ScrollZoomEvent>,

    /// Used by the logic that determines when it is safe to output an
    /// image for the reftest framework.
    ready_to_save_state: ReadyState,

    /// A data structure to cache unused NativeSurfaces.
    surface_map: SurfaceMap,

    /// Pipeline IDs of subpages that the compositor has seen in a layer tree but which have not
    /// yet been painted.
    pending_subpages: HashSet<PipelineId>,

    /// The id of the pipeline that was last sent a mouse move event, if any.
    last_mouse_move_recipient: Option<PipelineId>,
}

pub struct ScrollZoomEvent {
    /// Change the pinch zoom level by this factor
    magnification: f32,
    /// Scroll by this offset
    delta: TypedPoint2D<DevicePixel, f32>,
    /// Apply changes to the frame at this location
    cursor: TypedPoint2D<DevicePixel, i32>,
}

#[derive(PartialEq, Debug)]
enum CompositionRequest {
    NoCompositingNecessary,
    CompositeOnScrollTimeout(u64),
    CompositeNow(CompositingReason),
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ShutdownState {
    NotShuttingDown,
    ShuttingDown,
    FinishedShuttingDown,
}

struct HitTestResult {
    /// The topmost layer containing the requested point
    layer: Rc<Layer<CompositorData>>,
    /// The point in client coordinates of the innermost window or frame containing `layer`
    point: TypedPoint2D<LayerPixel, f32>,
}

pub struct PipelineDetails {
    /// The pipeline associated with this PipelineDetails object.
    pipeline: Option<CompositionPipeline>,

    /// The current layout epoch that this pipeline wants to draw.
    current_epoch: Epoch,

    /// Whether animations are running
    animations_running: bool,

    /// Whether there are animation callbacks
    animation_callbacks_running: bool,
}

impl PipelineDetails {
    fn new() -> PipelineDetails {
        PipelineDetails {
            pipeline: None,
            current_epoch: Epoch(0),
            animations_running: false,
            animation_callbacks_running: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CompositeTarget {
    /// Normal composition to a window
    Window,

    /// Compose as normal, but also return a PNG of the composed output
    WindowAndPng,

    /// Compose to a PNG, write it to disk, and then exit the browser (used for reftests)
    PngFile
}

fn initialize_png(width: usize, height: usize) -> (Vec<gl::GLuint>, Vec<gl::GLuint>) {
    let framebuffer_ids = gl::gen_framebuffers(1);
    gl::bind_framebuffer(gl::FRAMEBUFFER, framebuffer_ids[0]);

    let texture_ids = gl::gen_textures(1);
    gl::bind_texture(gl::TEXTURE_2D, texture_ids[0]);

    gl::tex_image_2d(gl::TEXTURE_2D, 0, gl::RGB as GLint, width as GLsizei,
                     height as GLsizei, 0, gl::RGB, gl::UNSIGNED_BYTE, None);
    gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
    gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);

    gl::framebuffer_texture_2d(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D,
                               texture_ids[0], 0);

    gl::bind_texture(gl::TEXTURE_2D, 0);

    (framebuffer_ids, texture_ids)
}

pub fn reporter_name() -> String {
    "compositor-reporter".to_owned()
}

impl<Window: WindowMethods> IOCompositor<Window> {
    fn new(window: Rc<Window>, state: InitialCompositorState)
           -> IOCompositor<Window> {
        // Register this thread as a memory reporter, via its own channel.
        let (reporter_sender, reporter_receiver) = ipc::channel().unwrap();
        let compositor_proxy_for_memory_reporter = state.sender.clone_compositor_proxy();
        ROUTER.add_route(reporter_receiver.to_opaque(), box move |reporter_request| {
            let reporter_request: ReporterRequest = reporter_request.to().unwrap();
            compositor_proxy_for_memory_reporter.send(Msg::CollectMemoryReports(
                    reporter_request.reports_channel));
        });
        let reporter = Reporter(reporter_sender);
        state.mem_profiler_chan.send(
            mem::ProfilerMsg::RegisterReporter(reporter_name(), reporter));

        let window_size = window.framebuffer_size();
        let hidpi_factor = window.hidpi_factor();
        let composite_target = match opts::get().output_file {
            Some(_) => CompositeTarget::PngFile,
            None => CompositeTarget::Window
        };
        let native_display = window.native_display();
        IOCompositor {
            window: window,
            native_display: native_display,
            port: state.receiver,
            context: None,
            root_pipeline: None,
            pipeline_details: HashMap::new(),
            scene: Scene::new(Rect {
                origin: Point2D::zero(),
                size: window_size.as_f32(),
            }),
            window_size: window_size,
            viewport: None,
            hidpi_factor: hidpi_factor,
            channel_to_self: state.sender.clone_compositor_proxy(),
            scrolling_timer: ScrollingTimerProxy::new(state.sender),
            composition_request: CompositionRequest::NoCompositingNecessary,
            touch_handler: TouchHandler::new(),
            pending_scroll_zoom_events: Vec::new(),
            composite_target: composite_target,
            shutdown_state: ShutdownState::NotShuttingDown,
            page_zoom: ScaleFactor::new(1.0),
            viewport_zoom: ScaleFactor::new(1.0),
            min_viewport_zoom: None,
            max_viewport_zoom: None,
            zoom_action: false,
            zoom_time: 0f64,
            got_load_complete_message: false,
            frame_tree_id: FrameTreeId(0),
            constellation_chan: state.constellation_chan,
            time_profiler_chan: state.time_profiler_chan,
            mem_profiler_chan: state.mem_profiler_chan,
            fragment_point: None,
            last_composite_time: 0,
            ready_to_save_state: ReadyState::Unknown,
            surface_map: SurfaceMap::new(BUFFER_MAP_SIZE),
            pending_subpages: HashSet::new(),
            last_mouse_move_recipient: None,
        }
    }

    pub fn create(window: Rc<Window>, state: InitialCompositorState)
                  -> IOCompositor<Window> {
        let mut compositor = IOCompositor::new(window, state);

        // Set the size of the root layer.
        compositor.update_zoom_transform();

        // Tell the constellation about the initial window size.
        compositor.send_window_size();

        compositor
    }

    pub fn start_shutting_down(&mut self) {
        debug!("Compositor sending Exit message to Constellation");
        self.constellation_chan.send(ConstellationMsg::Exit).unwrap();

        self.mem_profiler_chan.send(mem::ProfilerMsg::UnregisterReporter(reporter_name()));

        self.shutdown_state = ShutdownState::ShuttingDown;
    }

    pub fn finish_shutting_down(&mut self) {
        debug!("Compositor received message that constellation shutdown is complete");

        // Clear out the compositor layers so that painting threads can destroy the buffers.
        if let Some(ref root_layer) = self.scene.root {
            root_layer.forget_all_tiles();
        }

        // Drain compositor port, sometimes messages contain channels that are blocking
        // another thread from finishing (i.e. SetFrameTree).
        while self.port.try_recv_compositor_msg().is_some() {}

        // Tell the profiler, memory profiler, and scrolling timer to shut down.
        self.time_profiler_chan.send(time::ProfilerMsg::Exit);
        self.mem_profiler_chan.send(mem::ProfilerMsg::Exit);
        self.scrolling_timer.shutdown();

        self.shutdown_state = ShutdownState::FinishedShuttingDown;
    }

    fn handle_browser_message(&mut self, msg: Msg) -> bool {
        match (msg, self.shutdown_state) {
            (_, ShutdownState::FinishedShuttingDown) =>
                panic!("compositor shouldn't be handling messages after shutting down"),

            (Msg::Exit(channel), _) => {
                self.start_shutting_down();
                channel.send(()).unwrap();
            }

            (Msg::ShutdownComplete, _) => {
                self.finish_shutting_down();
                return false;
            }

            (Msg::ChangeRunningAnimationsState(pipeline_id, animation_state),
             ShutdownState::NotShuttingDown) => {
                self.change_running_animations_state(pipeline_id, animation_state);
            }

            (Msg::ChangePageTitle(pipeline_id, title), ShutdownState::NotShuttingDown) => {
                self.change_page_title(pipeline_id, title);
            }

            (Msg::ChangePageUrl(pipeline_id, url), ShutdownState::NotShuttingDown) => {
                self.change_page_url(pipeline_id, url);
            }

            (Msg::SetFrameTree(frame_tree, response_chan, new_constellation_chan),
             ShutdownState::NotShuttingDown) => {
                self.set_frame_tree(&frame_tree, response_chan, new_constellation_chan);
                self.send_viewport_rects_for_all_layers();
                self.title_for_main_frame();
            }

            (Msg::InitializeLayersForPipeline(pipeline_id, epoch, properties),
             ShutdownState::NotShuttingDown) => {
                self.pipeline_details(pipeline_id).current_epoch = epoch;

                self.collect_old_layers(pipeline_id, &properties);
                for (index, layer_properties) in properties.iter().enumerate() {
                    if index == 0 {
                        self.create_or_update_base_layer(pipeline_id, *layer_properties);
                    } else {
                        self.create_or_update_descendant_layer(pipeline_id, *layer_properties);
                    }
                }

                self.send_buffer_requests_for_all_layers();
                self.dump_layer_tree();
            }

            (Msg::GetNativeDisplay(chan),
             ShutdownState::NotShuttingDown) => {
                chan.send(Some(self.native_display.clone())).unwrap();
            }

            (Msg::AssignPaintedBuffers(pipeline_id, epoch, replies, frame_tree_id),
             ShutdownState::NotShuttingDown) => {
                self.pending_subpages.remove(&pipeline_id);

                for (layer_id, new_layer_buffer_set) in replies {
                    self.assign_painted_buffers(pipeline_id,
                                                layer_id,
                                                new_layer_buffer_set,
                                                epoch,
                                                frame_tree_id);
                }
            }

            (Msg::ReturnUnusedNativeSurfaces(native_surfaces),
             ShutdownState::NotShuttingDown) => {
                self.surface_map.insert_surfaces(&self.native_display, native_surfaces);
            }

            (Msg::ScrollFragmentPoint(pipeline_id, layer_id, point, _),
             ShutdownState::NotShuttingDown) => {
                self.scroll_fragment_to_point(pipeline_id, layer_id, point);
            }

            (Msg::MoveTo(point),
             ShutdownState::NotShuttingDown) => {
                self.window.set_position(point);
            }

            (Msg::ResizeTo(size),
             ShutdownState::NotShuttingDown) => {
                self.window.set_inner_size(size);
            }

            (Msg::GetClientWindow(send),
             ShutdownState::NotShuttingDown) => {
                let rect = self.window.client_window();
                send.send(rect).unwrap();
            }

            (Msg::Status(message), ShutdownState::NotShuttingDown) => {
                self.window.status(message);
            }

            (Msg::LoadStart(back, forward), ShutdownState::NotShuttingDown) => {
                self.window.load_start(back, forward);
            }

            (Msg::LoadComplete(back, forward), ShutdownState::NotShuttingDown) => {
                self.got_load_complete_message = true;

                // If we're painting in headless mode, schedule a recomposite.
                if opts::get().output_file.is_some() || opts::get().exit_after_load {
                    self.composite_if_necessary(CompositingReason::Headless);
                }

                // Inform the embedder that the load has finished.
                //
                // TODO(pcwalton): Specify which frame's load completed.
                self.window.load_end(back, forward);
            }

            (Msg::ScrollTimeout(timestamp), ShutdownState::NotShuttingDown) => {
                debug!("scroll timeout, drawing unpainted content!");
                if let CompositionRequest::CompositeOnScrollTimeout(this_timestamp) =
                    self.composition_request {
                    if timestamp == this_timestamp {
                        self.composition_request = CompositionRequest::CompositeNow(
                            CompositingReason::HitScrollTimeout)
                    }
                }
            }

            (Msg::RecompositeAfterScroll, ShutdownState::NotShuttingDown) => {
                self.composition_request =
                    CompositionRequest::CompositeNow(CompositingReason::ContinueScroll)
            }

            (Msg::KeyEvent(key, state, modified), ShutdownState::NotShuttingDown) => {
                if state == KeyState::Pressed {
                    self.window.handle_key(key, modified);
                }
            }

            (Msg::TouchEventProcessed(result), ShutdownState::NotShuttingDown) => {
                self.touch_handler.on_event_processed(result);
            }

            (Msg::SetCursor(cursor), ShutdownState::NotShuttingDown) => {
                self.window.set_cursor(cursor)
            }

            (Msg::CreatePng(reply), ShutdownState::NotShuttingDown) => {
                let res = self.composite_specific_target(CompositeTarget::WindowAndPng);
                let img = res.unwrap_or(None);
                reply.send(img).unwrap();
            }

            (Msg::PaintThreadExited(pipeline_id), ShutdownState::NotShuttingDown) => {
                self.remove_pipeline_root_layer(pipeline_id);
            }

            (Msg::ViewportConstrained(pipeline_id, constraints),
             ShutdownState::NotShuttingDown) => {
                self.constrain_viewport(pipeline_id, constraints);
            }

            (Msg::IsReadyToSaveImageReply(is_ready), ShutdownState::NotShuttingDown) => {
                assert!(self.ready_to_save_state == ReadyState::WaitingForConstellationReply);
                if is_ready {
                    self.ready_to_save_state = ReadyState::ReadyToSaveImage;
                    if opts::get().is_running_problem_test {
                        println!("ready to save image!");
                    }
                } else {
                    self.ready_to_save_state = ReadyState::Unknown;
                    if opts::get().is_running_problem_test {
                        println!("resetting ready_to_save_state!");
                    }
                }
                self.composite_if_necessary(CompositingReason::Headless);
            }

            (Msg::NewFavicon(url), ShutdownState::NotShuttingDown) => {
                self.window.set_favicon(url);
            }

            (Msg::HeadParsed, ShutdownState::NotShuttingDown) => {
                self.window.head_parsed();
            }

            (Msg::CollectMemoryReports(reports_chan), ShutdownState::NotShuttingDown) => {
                let name = "compositor-thread";
                // These are both `ExplicitUnknownLocationSize` because the memory might be in the
                // GPU or on the heap.
                let reports = vec![mem::Report {
                    path: path![name, "surface-map"],
                    kind: ReportKind::ExplicitUnknownLocationSize,
                    size: self.surface_map.mem(),
                }, mem::Report {
                    path: path![name, "layer-tree"],
                    kind: ReportKind::ExplicitUnknownLocationSize,
                    size: self.scene.get_memory_usage(),
                }];
                reports_chan.send(reports);
            }

            (Msg::PipelineExited(pipeline_id), _) => {
                self.pending_subpages.remove(&pipeline_id);
            }

            // When we are shutting_down, we need to avoid performing operations
            // such as Paint that may crash because we have begun tearing down
            // the rest of our resources.
            (_, ShutdownState::ShuttingDown) => { }
        }

        true
    }

    /// Sets or unsets the animations-running flag for the given pipeline, and schedules a
    /// recomposite if necessary.
    fn change_running_animations_state(&mut self,
                                       pipeline_id: PipelineId,
                                       animation_state: AnimationState) {
        match animation_state {
            AnimationState::AnimationsPresent => {
                self.pipeline_details(pipeline_id).animations_running = true;
                self.composite_if_necessary(CompositingReason::Animation);
            }
            AnimationState::AnimationCallbacksPresent => {
                if self.pipeline_details(pipeline_id).animation_callbacks_running {
                    return
                }
                self.pipeline_details(pipeline_id).animation_callbacks_running =
                    true;
                self.tick_animations_for_pipeline(pipeline_id);
                self.composite_if_necessary(CompositingReason::Animation);
            }
            AnimationState::NoAnimationsPresent => {
                self.pipeline_details(pipeline_id).animations_running = false;
            }
            AnimationState::NoAnimationCallbacksPresent => {
                self.pipeline_details(pipeline_id).animation_callbacks_running = false;
            }
        }
    }

    pub fn pipeline_details (&mut self,
                                              pipeline_id: PipelineId)
                                              -> &mut PipelineDetails {
        if !self.pipeline_details.contains_key(&pipeline_id) {
            self.pipeline_details.insert(pipeline_id, PipelineDetails::new());
        }
        self.pipeline_details.get_mut(&pipeline_id).unwrap()
    }

    pub fn pipeline(&self, pipeline_id: PipelineId) -> Option<&CompositionPipeline> {
        match self.pipeline_details.get(&pipeline_id) {
            Some(ref details) => details.pipeline.as_ref(),
            None => panic!("Compositor layer has an unknown pipeline ({:?}).", pipeline_id),
        }
    }

    fn change_page_title(&mut self, pipeline_id: PipelineId, title: Option<String>) {
        let set_title = self.root_pipeline.as_ref().map_or(false, |root_pipeline| {
            root_pipeline.id == pipeline_id
        });
        if set_title {
            self.window.set_page_title(title);
        }
    }

    fn change_page_url(&mut self, _: PipelineId, url: Url) {
        self.window.set_page_url(url);
    }

    fn set_frame_tree(&mut self,
                      frame_tree: &SendableFrameTree,
                      response_chan: IpcSender<()>,
                      new_constellation_chan: Sender<ConstellationMsg>) {
        response_chan.send(()).unwrap();

        // There are now no more pending iframes.
        self.pending_subpages.clear();

        self.root_pipeline = Some(frame_tree.pipeline.clone());

        // If we have an old root layer, release all old tiles before replacing it.
        let old_root_layer = self.scene.root.take();
        if let Some(ref old_root_layer) = old_root_layer {
            old_root_layer.clear_all_tiles(self)
        }

        self.scene.root = Some(self.create_root_layer_for_pipeline_and_size(&frame_tree.pipeline,
                                                                            None));
        self.scene.set_root_layer_size(self.window_size.as_f32());

        self.create_pipeline_details_for_frame_tree(&frame_tree);

        // Initialize the new constellation channel by sending it the root window size.
        self.constellation_chan = new_constellation_chan;
        self.send_window_size();

        self.frame_tree_id.next();
        self.composite_if_necessary(CompositingReason::NewFrameTree);
    }

    fn create_root_layer_for_pipeline_and_size(&mut self,
                                               pipeline: &CompositionPipeline,
                                               frame_size: Option<TypedSize2D<PagePx, f32>>)
                                               -> Rc<Layer<CompositorData>> {
        let layer_properties = LayerProperties {
            id: LayerId::null(),
            parent_id: None,
            rect: Rect::zero(),
            background_color: color::transparent(),
            scroll_policy: ScrollPolicy::Scrollable,
            transform: Matrix4::identity(),
            perspective: Matrix4::identity(),
            subpage_pipeline_id: None,
            establishes_3d_context: true,
            scrolls_overflow_area: false,
        };

        let root_layer = CompositorData::new_layer(pipeline.id,
                                                   layer_properties,
                                                   WantsScrollEventsFlag::WantsScrollEvents,
                                                   opts::get().tile_size);

        self.pipeline_details(pipeline.id).pipeline = Some(pipeline.clone());

        // All root layers mask to bounds.
        *root_layer.masks_to_bounds.borrow_mut() = true;

        if let Some(ref frame_size) = frame_size {
            let frame_size = frame_size.to_untyped();
            root_layer.bounds.borrow_mut().size = Size2D::from_untyped(&frame_size);
        }

        root_layer
    }

    fn create_pipeline_details_for_frame_tree(&mut self, frame_tree: &SendableFrameTree) {
        self.pipeline_details(frame_tree.pipeline.id).pipeline = Some(frame_tree.pipeline.clone());

        for kid in &frame_tree.children {
            self.create_pipeline_details_for_frame_tree(kid);
        }
    }

    fn find_pipeline_root_layer(&self, pipeline_id: PipelineId)
                                -> Option<Rc<Layer<CompositorData>>> {
        if !self.pipeline_details.contains_key(&pipeline_id) {
            panic!("Tried to create or update layer for unknown pipeline")
        }
        self.find_layer_with_pipeline_and_layer_id(pipeline_id, LayerId::null())
    }

    fn collect_old_layers(&mut self,
                          pipeline_id: PipelineId,
                          new_layers: &[LayerProperties]) {
        let root_layer = match self.scene.root {
            Some(ref root_layer) => root_layer.clone(),
            None => return,
        };

        let mut pipelines_removed = Vec::new();
        root_layer.collect_old_layers(self, pipeline_id, new_layers, &mut pipelines_removed);

        for pipeline_removed in pipelines_removed.into_iter() {
            self.pending_subpages.remove(&pipeline_removed);
        }
    }

    fn remove_pipeline_root_layer(&mut self, pipeline_id: PipelineId) {
        let root_layer = match self.scene.root {
            Some(ref root_layer) => root_layer.clone(),
            None => return,
        };

        // Remove all the compositor layers for this pipeline and recache
        // any buffers that they owned.
        root_layer.remove_root_layer_with_pipeline_id(self, pipeline_id);
        self.pipeline_details.remove(&pipeline_id);
    }

    fn update_layer_if_exists(&mut self,
                              pipeline_id: PipelineId,
                              properties: LayerProperties)
                              -> bool {
        if let Some(subpage_id) = properties.subpage_pipeline_id {
            match self.find_layer_with_pipeline_and_layer_id(subpage_id, LayerId::null()) {
                Some(layer) => {
                    *layer.bounds.borrow_mut() = Rect::from_untyped(
                        &Rect::new(Point2D::zero(), properties.rect.size));
                }
                None => warn!("Tried to update non-existent subpage root layer: {:?}", subpage_id),
            }
        }

        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, properties.id) {
            Some(existing_layer) => {
                // If this layer contains a subpage, then create the root layer for that subpage
                // now.
                if properties.subpage_pipeline_id.is_some() {
                    self.create_root_layer_for_subpage_if_necessary(properties,
                                                                    existing_layer.clone())
                }

                existing_layer.update_layer(properties);
                true
            }
            None => false,
        }
    }

    fn create_or_update_base_layer(&mut self,
                                   pipeline_id: PipelineId,
                                   layer_properties: LayerProperties) {
        debug_assert!(layer_properties.parent_id.is_none());

        let root_layer = match self.find_pipeline_root_layer(pipeline_id) {
            Some(root_layer) => root_layer,
            None => {
                debug!("Ignoring CreateOrUpdateBaseLayer message for pipeline \
                        ({:?}) shutting down.",
                       pipeline_id);
                return;
            }
        };

        let need_new_base_layer = !self.update_layer_if_exists(pipeline_id, layer_properties);
        if need_new_base_layer {
            root_layer.update_layer_except_bounds(layer_properties);

            let base_layer = CompositorData::new_layer(
                pipeline_id,
                layer_properties,
                WantsScrollEventsFlag::DoesntWantScrollEvents,
                opts::get().tile_size);

            // Add the base layer to the front of the child list, so that child
            // iframe layers are painted on top of the base layer. These iframe
            // layers were added previously when creating the layer tree
            // skeleton in create_frame_tree_root_layers.
            root_layer.children().insert(0, base_layer);
        }

        self.scroll_layer_to_fragment_point_if_necessary(pipeline_id,
                                                         layer_properties.id);
    }

    fn create_or_update_descendant_layer(&mut self,
                                         pipeline_id: PipelineId,
                                         layer_properties: LayerProperties) {
        debug_assert!(layer_properties.parent_id.is_some());

        if !self.update_layer_if_exists(pipeline_id, layer_properties) {
            self.create_descendant_layer(pipeline_id, layer_properties);
        }
        self.update_subpage_size_if_necessary(&layer_properties);
        self.scroll_layer_to_fragment_point_if_necessary(pipeline_id,
                                                         layer_properties.id);
    }

    fn create_descendant_layer(&mut self,
                               pipeline_id: PipelineId,
                               layer_properties: LayerProperties) {
        let parent_id = layer_properties.parent_id.unwrap();
        if let Some(parent_layer) = self.find_layer_with_pipeline_and_layer_id(pipeline_id,
                                                                               parent_id) {
            let wants_scroll_events = if layer_properties.scrolls_overflow_area {
                WantsScrollEventsFlag::WantsScrollEvents
            } else {
                WantsScrollEventsFlag::DoesntWantScrollEvents
            };

            let new_layer = CompositorData::new_layer(pipeline_id,
                                                      layer_properties,
                                                      wants_scroll_events,
                                                      parent_layer.tile_size);

            if layer_properties.scrolls_overflow_area {
                *new_layer.masks_to_bounds.borrow_mut() = true
            }

            // If this layer contains a subpage, then create the root layer for that subpage now.
            if layer_properties.subpage_pipeline_id.is_some() {
                self.create_root_layer_for_subpage_if_necessary(layer_properties,
                                                                new_layer.clone())
            }

            parent_layer.add_child(new_layer.clone());
        }

        self.dump_layer_tree();
    }

    fn create_root_layer_for_subpage_if_necessary(&mut self,
                                                  layer_properties: LayerProperties,
                                                  parent_layer: Rc<Layer<CompositorData>>) {
        if parent_layer.children
                       .borrow()
                       .iter()
                       .any(|child| child.extra_data.borrow().subpage_info.is_some()) {
            return
        }

        let subpage_pipeline_id =
            layer_properties.subpage_pipeline_id
                            .expect("create_root_layer_for_subpage() called for non-subpage?!");
        let subpage_layer_properties = LayerProperties {
            id: LayerId::null(),
            parent_id: None,
            rect: Rect::new(Point2D::zero(), layer_properties.rect.size),
            background_color: layer_properties.background_color,
            scroll_policy: ScrollPolicy::Scrollable,
            transform: Matrix4::identity(),
            perspective: Matrix4::identity(),
            subpage_pipeline_id: Some(subpage_pipeline_id),
            establishes_3d_context: true,
            scrolls_overflow_area: true,
        };

        let wants_scroll_events = if subpage_layer_properties.scrolls_overflow_area {
            WantsScrollEventsFlag::WantsScrollEvents
        } else {
            WantsScrollEventsFlag::DoesntWantScrollEvents
        };
        let subpage_layer = CompositorData::new_layer(subpage_pipeline_id,
                                                      subpage_layer_properties,
                                                      wants_scroll_events,
                                                      parent_layer.tile_size);
        *subpage_layer.masks_to_bounds.borrow_mut() = true;
        parent_layer.add_child(subpage_layer);
        self.pending_subpages.insert(subpage_pipeline_id);
    }

    fn send_window_size(&self) {
        let dppx = self.page_zoom * self.device_pixels_per_screen_px();
        let initial_viewport = self.window_size.as_f32() / dppx;
        let visible_viewport = initial_viewport / self.viewport_zoom;

        self.constellation_chan.send(ConstellationMsg::ResizedWindow(WindowSizeData {
            device_pixel_ratio: dppx,
            initial_viewport: initial_viewport,
            visible_viewport: visible_viewport,
        })).unwrap()
    }

    /// Sends the size of the given subpage up to the constellation. This will often trigger a
    /// reflow of that subpage.
    fn update_subpage_size_if_necessary(&self, layer_properties: &LayerProperties) {
        let subpage_pipeline_id = match layer_properties.subpage_pipeline_id {
            Some(ref subpage_pipeline_id) => subpage_pipeline_id,
            None => return,
        };

        self.constellation_chan.send(ConstellationMsg::FrameSize(
            *subpage_pipeline_id,
            layer_properties.rect.size)).unwrap();
    }

    pub fn move_layer(&self,
                      pipeline_id: PipelineId,
                      layer_id: LayerId,
                      origin: TypedPoint2D<LayerPixel, f32>)
                      -> bool {
        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, layer_id) {
            Some(ref layer) => {
                if layer.wants_scroll_events() == WantsScrollEventsFlag::WantsScrollEvents {
                    layer.clamp_scroll_offset_and_scroll_layer(Point2D::typed(0f32, 0f32) - origin);
                }
                true
            }
            None => false,
        }
    }

    fn scroll_layer_to_fragment_point_if_necessary(&mut self,
                                                   pipeline_id: PipelineId,
                                                   layer_id: LayerId) {
        if let Some(point) = self.fragment_point.take() {
            if !self.move_layer(pipeline_id, layer_id, Point2D::from_untyped(&point)) {
                panic!("Compositor: Tried to scroll to fragment with unknown layer.");
            }

            self.perform_updates_after_scroll();
        }
    }

    fn start_scrolling_timer_if_necessary(&mut self) {
        match self.composition_request {
            CompositionRequest::CompositeNow(_) |
            CompositionRequest::CompositeOnScrollTimeout(_) => return,
            CompositionRequest::NoCompositingNecessary => {}
        }

        let timestamp = precise_time_ns();
        self.scrolling_timer.scroll_event_processed(timestamp);
        self.composition_request = CompositionRequest::CompositeOnScrollTimeout(timestamp);
    }

    fn assign_painted_buffers(&mut self,
                              pipeline_id: PipelineId,
                              layer_id: LayerId,
                              new_layer_buffer_set: Box<LayerBufferSet>,
                              epoch: Epoch,
                              frame_tree_id: FrameTreeId) {
        // If the frame tree id has changed since this paint request was sent,
        // reject the buffers and send them back to the paint thread. If this isn't handled
        // correctly, the content_age in the tile grid can get out of sync when iframes are
        // loaded and the frame tree changes. This can result in the compositor thinking it
        // has already drawn the most recently painted buffer, and missing a frame.
        if frame_tree_id == self.frame_tree_id {
            if let Some(layer) = self.find_layer_with_pipeline_and_layer_id(pipeline_id,
                                                                            layer_id) {
                let requested_epoch = layer.extra_data.borrow().requested_epoch;
                if requested_epoch == epoch {
                    self.assign_painted_buffers_to_layer(layer, new_layer_buffer_set, epoch);
                    return
                } else {
                    debug!("assign_painted_buffers epoch mismatch {:?} {:?} req={:?} actual={:?}",
                           pipeline_id,
                           layer_id,
                           requested_epoch,
                           epoch);
                }
            }
        }

        self.cache_unused_buffers(new_layer_buffer_set.buffers);
    }

    fn assign_painted_buffers_to_layer(&mut self,
                                       layer: Rc<Layer<CompositorData>>,
                                       new_layer_buffer_set: Box<LayerBufferSet>,
                                       epoch: Epoch) {
        debug!("compositor received new frame at size {:?}x{:?}",
               self.window_size.width.get(),
               self.window_size.height.get());

        // From now on, if we destroy the buffers, they will leak.
        let mut new_layer_buffer_set = new_layer_buffer_set;
        new_layer_buffer_set.mark_will_leak();

        // FIXME(pcwalton): This is going to cause problems with inconsistent frames since
        // we only composite one layer at a time.
        layer.add_buffers(self, new_layer_buffer_set, epoch);
        self.composite_if_necessary(CompositingReason::NewPaintedBuffers);
    }

    fn scroll_fragment_to_point(&mut self,
                                pipeline_id: PipelineId,
                                layer_id: LayerId,
                                point: Point2D<f32>) {
        if self.move_layer(pipeline_id, layer_id, Point2D::from_untyped(&point)) {
            self.perform_updates_after_scroll()
        } else {
            self.fragment_point = Some(point)
        }
    }

    fn handle_window_message(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Idle => {}

            WindowEvent::Refresh => {
                self.composite();
            }

            WindowEvent::InitializeCompositing => {
                self.initialize_compositing();
            }

            WindowEvent::Viewport(point, size) => {
              self.viewport = Some((point, size));
            }

            WindowEvent::Resize(size) => {
                self.on_resize_window_event(size);
            }

            WindowEvent::LoadUrl(url_string) => {
                self.on_load_url_window_event(url_string);
            }

            WindowEvent::MouseWindowEventClass(mouse_window_event) => {
                self.on_mouse_window_event_class(mouse_window_event);
            }

            WindowEvent::MouseWindowMoveEventClass(cursor) => {
                self.on_mouse_window_move_event_class(cursor);
            }

            WindowEvent::Touch(event_type, identifier, location) => {
                match event_type {
                    TouchEventType::Down => self.on_touch_down(identifier, location),
                    TouchEventType::Move => self.on_touch_move(identifier, location),
                    TouchEventType::Up => self.on_touch_up(identifier, location),
                    TouchEventType::Cancel => self.on_touch_cancel(identifier, location),
                }
            }

            WindowEvent::Scroll(delta, cursor) => {
                self.on_scroll_window_event(delta, cursor);
            }

            WindowEvent::Zoom(magnification) => {
                self.on_zoom_window_event(magnification);
            }

            WindowEvent::ResetZoom => {
                self.on_zoom_reset_window_event();
            }

            WindowEvent::PinchZoom(magnification) => {
                self.on_pinch_zoom_window_event(magnification);
            }

            WindowEvent::Navigation(direction) => {
                self.on_navigation_window_event(direction);
            }

            WindowEvent::KeyEvent(key, state, modifiers) => {
                self.on_key_event(key, state, modifiers);
            }

            WindowEvent::Quit => {
                if self.shutdown_state == ShutdownState::NotShuttingDown {
                    debug!("Shutting down the constellation for WindowEvent::Quit");
                    self.start_shutting_down();
                }
            }
        }
    }

    fn on_resize_window_event(&mut self, new_size: TypedSize2D<DevicePixel, u32>) {
        debug!("compositor resizing to {:?}", new_size.to_untyped());

        // A size change could also mean a resolution change.
        let new_hidpi_factor = self.window.hidpi_factor();
        if self.hidpi_factor != new_hidpi_factor {
            self.hidpi_factor = new_hidpi_factor;
            self.update_zoom_transform();
        }

        if self.window_size == new_size {
            return;
        }

        self.window_size = new_size;

        self.scene.set_root_layer_size(new_size.as_f32());
        self.send_window_size();
    }

    fn on_load_url_window_event(&mut self, url_string: String) {
        debug!("osmain: loading URL `{}`", url_string);
        self.got_load_complete_message = false;
        let url = Url::parse(&url_string).unwrap();
        self.window.set_page_url(url.clone());
        let msg = match self.scene.root {
            Some(ref layer) => ConstellationMsg::LoadUrl(layer.pipeline_id(), LoadData::new(url)),
            None => ConstellationMsg::InitLoadUrl(url)
        };

        self.constellation_chan.send(msg).unwrap()
    }

    fn on_mouse_window_event_class(&mut self, mouse_window_event: MouseWindowEvent) {
        if opts::get().convert_mouse_to_touch {
            match mouse_window_event {
                MouseWindowEvent::Click(_, _) => {}
                MouseWindowEvent::MouseDown(_, p) => self.on_touch_down(TouchId(0), p),
                MouseWindowEvent::MouseUp(_, p) => self.on_touch_up(TouchId(0), p),
            }
            return
        }

        let point = match mouse_window_event {
            MouseWindowEvent::Click(_, p) => p,
            MouseWindowEvent::MouseDown(_, p) => p,
            MouseWindowEvent::MouseUp(_, p) => p,
        };
        match self.find_topmost_layer_at_point(point / self.scene.scale) {
            Some(result) => result.layer.send_mouse_event(self, mouse_window_event, result.point),
            None => {},
        }
    }

    fn on_mouse_window_move_event_class(&mut self, cursor: TypedPoint2D<DevicePixel, f32>) {
        if opts::get().convert_mouse_to_touch {
            self.on_touch_move(TouchId(0), cursor);
            return
        }

        match self.find_topmost_layer_at_point(cursor / self.scene.scale) {
            Some(result) => {
                // In the case that the mouse was previously over a different layer,
                // that layer must update its state.
                if let Some(last_pipeline_id) = self.last_mouse_move_recipient {
                    if last_pipeline_id != result.layer.pipeline_id() {
                        if let Some(pipeline) = self.pipeline(last_pipeline_id) {
                            let _ = pipeline.script_chan
                                            .send(ConstellationControlMsg::SendEvent(
                                                last_pipeline_id.clone(),
                                                MouseMoveEvent(None)));
                        }
                    }
                }

                self.last_mouse_move_recipient = Some(result.layer.pipeline_id());
                result.layer.send_mouse_move_event(self, result.point);
            }
            None => {}
        }
    }

    fn on_touch_down(&mut self, identifier: TouchId, point: TypedPoint2D<DevicePixel, f32>) {
        self.touch_handler.on_touch_down(identifier, point);
        if let Some(result) = self.find_topmost_layer_at_point(point / self.scene.scale) {
            result.layer.send_event(self, TouchEvent(TouchEventType::Down, identifier,
                                                     result.point.to_untyped()));
        }
    }

    fn on_touch_move(&mut self, identifier: TouchId, point: TypedPoint2D<DevicePixel, f32>) {
        match self.touch_handler.on_touch_move(identifier, point) {
            TouchAction::Scroll(delta) => {
                self.on_scroll_window_event(delta, point.cast().unwrap());
            }
            TouchAction::Zoom(magnification, scroll_delta) => {
                let cursor = Point2D::typed(-1, -1);  // Make sure this hits the base layer.
                self.pending_scroll_zoom_events.push(ScrollZoomEvent {
                    magnification: magnification,
                    delta: scroll_delta,
                    cursor: cursor,
                });
                self.composite_if_necessary(CompositingReason::Zoom);
            }
            TouchAction::DispatchEvent => {
                if let Some(result) = self.find_topmost_layer_at_point(point / self.scene.scale) {
                    result.layer.send_event(self, TouchEvent(TouchEventType::Move, identifier,
                                                             result.point.to_untyped()));
                }
            }
            _ => {}
        }
    }

    fn on_touch_up(&mut self, identifier: TouchId, point: TypedPoint2D<DevicePixel, f32>) {
        if let Some(result) = self.find_topmost_layer_at_point(point / self.scene.scale) {
            result.layer.send_event(self, TouchEvent(TouchEventType::Up, identifier,
                                                     result.point.to_untyped()));
        }
        if let TouchAction::Click = self.touch_handler.on_touch_up(identifier, point) {
            self.simulate_mouse_click(point);
        }
    }

    fn on_touch_cancel(&mut self, identifier: TouchId, point: TypedPoint2D<DevicePixel, f32>) {
        // Send the event to script.
        self.touch_handler.on_touch_cancel(identifier, point);
        if let Some(result) = self.find_topmost_layer_at_point(point / self.scene.scale) {
            result.layer.send_event(self, TouchEvent(TouchEventType::Cancel, identifier,
                                                     result.point.to_untyped()));
        }
    }

    /// http://w3c.github.io/touch-events/#mouse-events
    fn simulate_mouse_click(&self, p: TypedPoint2D<DevicePixel, f32>) {
        match self.find_topmost_layer_at_point(p / self.scene.scale) {
            Some(HitTestResult { layer, point }) => {
                let button = MouseButton::Left;
                layer.send_mouse_move_event(self, point);
                layer.send_mouse_event(self, MouseWindowEvent::MouseDown(button, p), point);
                layer.send_mouse_event(self, MouseWindowEvent::MouseUp(button, p), point);
                layer.send_mouse_event(self, MouseWindowEvent::Click(button, p), point);
            }
            None => {},
        }
    }

    fn on_scroll_window_event(&mut self,
                              delta: TypedPoint2D<DevicePixel, f32>,
                              cursor: TypedPoint2D<DevicePixel, i32>) {
        self.pending_scroll_zoom_events.push(ScrollZoomEvent {
            magnification: 1.0,
            delta: delta,
            cursor: cursor,
        });

        self.composite_if_necessary(CompositingReason::Scroll);
    }

    fn process_pending_scroll_events(&mut self) {
        let had_events = self.pending_scroll_zoom_events.len() > 0;
        for event in std_mem::replace(&mut self.pending_scroll_zoom_events,
                                             Vec::new()) {
            let delta = event.delta / self.scene.scale;
            let cursor = event.cursor.as_f32() / self.scene.scale;

            if let Some(ref mut layer) = self.scene.root {
                layer.handle_scroll_event(delta, cursor);
            }

            if event.magnification != 1.0 {
                self.zoom_action = true;
                self.zoom_time = precise_time_s();
                self.viewport_zoom = ScaleFactor::new(
                    (self.viewport_zoom.get() * event.magnification)
                    .min(self.max_viewport_zoom.as_ref().map_or(MAX_ZOOM, ScaleFactor::get))
                    .max(self.min_viewport_zoom.as_ref().map_or(MIN_ZOOM, ScaleFactor::get)));
                self.update_zoom_transform();
            }

            self.perform_updates_after_scroll();
        }

        if had_events {
            self.send_viewport_rects_for_all_layers();
        }
    }

    /// Computes new display ports for each layer, taking the scroll position into account, and
    /// sends them to layout as necessary. This ultimately triggers a rerender of the content.
    fn send_updated_display_ports_to_layout(&mut self) {
        fn process_layer(layer: &Layer<CompositorData>,
                         window_size: &TypedSize2D<LayerPixel, f32>,
                         new_display_ports: &mut HashMap<PipelineId, Vec<(LayerId, Rect<Au>)>>) {
            let visible_rect =
                Rect::new(Point2D::zero(), *window_size).translate(&-*layer.content_offset.borrow())
                                                        .intersection(&*layer.bounds.borrow())
                                                        .unwrap_or(Rect::zero())
                                                        .to_untyped();
            let visible_rect = Rect::new(Point2D::new(Au::from_f32_px(visible_rect.origin.x),
                                                      Au::from_f32_px(visible_rect.origin.y)),
                                         Size2D::new(Au::from_f32_px(visible_rect.size.width),
                                                     Au::from_f32_px(visible_rect.size.height)));

            let extra_layer_data = layer.extra_data.borrow();
            if !new_display_ports.contains_key(&extra_layer_data.pipeline_id) {
                new_display_ports.insert(extra_layer_data.pipeline_id, Vec::new());
            }
            new_display_ports.get_mut(&extra_layer_data.pipeline_id)
                             .unwrap()
                             .push((extra_layer_data.id, visible_rect));

            for kid in &*layer.children.borrow() {
                process_layer(&*kid, window_size, new_display_ports)
            }
        }

        let dppx = self.page_zoom * self.device_pixels_per_screen_px();
        let window_size = self.window_size.as_f32() / dppx * ScaleFactor::new(1.0);
        let mut new_visible_rects = HashMap::new();
        if let Some(ref layer) = self.scene.root {
            process_layer(&**layer, &window_size, &mut new_visible_rects)
        }

        for (pipeline_id, new_visible_rects) in &new_visible_rects {
            if let Some(pipeline_details) = self.pipeline_details.get(&pipeline_id) {
                if let Some(ref pipeline) = pipeline_details.pipeline {
                    let LayoutControlChan(ref sender) = pipeline.layout_chan;
                    sender.send(LayoutControlMsg::SetVisibleRects((*new_visible_rects).clone()))
                          .unwrap()
                }
            }
        }
    }

    /// Performs buffer requests and starts the scrolling timer or schedules a recomposite as
    /// necessary.
    fn perform_updates_after_scroll(&mut self) {
        self.send_updated_display_ports_to_layout();
        if self.send_buffer_requests_for_all_layers() {
            self.start_scrolling_timer_if_necessary();
        } else {
            self.channel_to_self.send(Msg::RecompositeAfterScroll);
        }
    }

    /// If there are any animations running, dispatches appropriate messages to the constellation.
    fn process_animations(&mut self) {
        for (pipeline_id, pipeline_details) in &self.pipeline_details {
            if pipeline_details.animations_running ||
               pipeline_details.animation_callbacks_running {
                self.tick_animations_for_pipeline(*pipeline_id)
            }
        }
    }

    fn tick_animations_for_pipeline(&self, pipeline_id: PipelineId) {
        self.constellation_chan.send(ConstellationMsg::TickAnimation(pipeline_id)).unwrap()
    }

    fn constrain_viewport(&mut self, pipeline_id: PipelineId, constraints: ViewportConstraints) {
        let is_root = self.root_pipeline.as_ref().map_or(false, |root_pipeline| {
            root_pipeline.id == pipeline_id
        });

        if is_root {
            // TODO: actual viewport size

            self.viewport_zoom = constraints.initial_zoom;
            self.min_viewport_zoom = constraints.min_zoom;
            self.max_viewport_zoom = constraints.max_zoom;
            self.update_zoom_transform();
        }
    }

    fn device_pixels_per_screen_px(&self) -> ScaleFactor<ScreenPx, DevicePixel, f32> {
        match opts::get().device_pixels_per_px {
            Some(device_pixels_per_px) => ScaleFactor::new(device_pixels_per_px),
            None => match opts::get().output_file {
                Some(_) => ScaleFactor::new(1.0),
                None => self.hidpi_factor
            }
        }
    }

    fn device_pixels_per_page_px(&self) -> ScaleFactor<PagePx, DevicePixel, f32> {
        self.viewport_zoom * self.page_zoom * self.device_pixels_per_screen_px()
    }

    fn update_zoom_transform(&mut self) {
        let scale = self.device_pixels_per_page_px();
        self.scene.scale = ScaleFactor::new(scale.get());

        // We need to set the size of the root layer again, since the window size
        // has changed in unscaled layer pixels.
        self.scene.set_root_layer_size(self.window_size.as_f32());
    }

    fn on_zoom_reset_window_event(&mut self) {
        self.page_zoom = ScaleFactor::new(1.0);
        self.update_zoom_transform();
        self.send_window_size();
    }

    fn on_zoom_window_event(&mut self, magnification: f32) {
        self.page_zoom = ScaleFactor::new((self.page_zoom.get() * magnification)
                                          .max(MIN_ZOOM).min(MAX_ZOOM));
        self.update_zoom_transform();
        self.send_window_size();
    }

    /// Simulate a pinch zoom
    fn on_pinch_zoom_window_event(&mut self, magnification: f32) {
        self.pending_scroll_zoom_events.push(ScrollZoomEvent {
            magnification: magnification,
            delta: Point2D::typed(0.0, 0.0), // TODO: Scroll to keep the center in view?
            cursor:  Point2D::typed(-1, -1), // Make sure this hits the base layer.
        });
        self.composite_if_necessary(CompositingReason::Zoom);
    }

    fn on_navigation_window_event(&self, direction: WindowNavigateMsg) {
        let direction = match direction {
            windowing::WindowNavigateMsg::Forward => NavigationDirection::Forward,
            windowing::WindowNavigateMsg::Back => NavigationDirection::Back,
        };
        self.constellation_chan.send(ConstellationMsg::Navigate(None, direction)).unwrap()
    }

    fn on_key_event(&self, key: Key, state: KeyState, modifiers: KeyModifiers) {
        self.constellation_chan.send(ConstellationMsg::KeyEvent(key, state, modifiers)).unwrap()
    }

    fn fill_paint_request_with_cached_layer_buffers(&mut self, paint_request: &mut PaintRequest) {
        for buffer_request in &mut paint_request.buffer_requests {
            if self.surface_map.mem() == 0 {
                return;
            }

            let size = Size2D::new(buffer_request.screen_rect.size.width as i32,
                                   buffer_request.screen_rect.size.height as i32);
            if let Some(mut native_surface) = self.surface_map.find(size) {
                native_surface.mark_wont_leak();
                buffer_request.native_surface = Some(native_surface);
            }
        }
    }

    fn convert_buffer_requests_to_pipeline_requests_map(&mut self,
                                                        requests: Vec<(Rc<Layer<CompositorData>>,
                                                                       Vec<BufferRequest>)>)
                                                        -> HashMap<PipelineId, Vec<PaintRequest>> {
        let scale = self.device_pixels_per_page_px();
        let mut results: HashMap<PipelineId, Vec<PaintRequest>> = HashMap::new();

        for (layer, mut layer_requests) in requests {
            let pipeline_id = layer.pipeline_id();
            let current_epoch = self.pipeline_details(pipeline_id).current_epoch;
            layer.extra_data.borrow_mut().requested_epoch = current_epoch;
            let vec = match results.entry(pipeline_id) {
                Occupied(entry) => {
                    entry.into_mut()
                }
                Vacant(entry) => {
                    entry.insert(Vec::new())
                }
            };

            // All the BufferRequests are in layer/device coordinates, but the paint thread
            // wants to know the page coordinates. We scale them before sending them.
            for request in &mut layer_requests {
                request.page_rect = request.page_rect / scale.get();
            }

            let layer_kind = if layer.transform_state.borrow().has_transform {
                LayerKind::HasTransform
            } else {
                LayerKind::NoTransform
            };

            let mut paint_request = PaintRequest {
                buffer_requests: layer_requests,
                scale: scale.get(),
                layer_id: layer.extra_data.borrow().id,
                epoch: layer.extra_data.borrow().requested_epoch,
                layer_kind: layer_kind,
            };
            self.fill_paint_request_with_cached_layer_buffers(&mut paint_request);
            vec.push(paint_request);
        }

        results
    }

    fn send_viewport_rect_for_layer(&self, layer: Rc<Layer<CompositorData>>) {
        if layer.extra_data.borrow().id == LayerId::null() {
            let layer_rect = Rect::new(-layer.extra_data.borrow().scroll_offset.to_untyped(),
                                       layer.bounds.borrow().size.to_untyped());
            if let Some(pipeline) = self.pipeline(layer.pipeline_id()) {
                pipeline.script_chan.send(ConstellationControlMsg::Viewport(pipeline.id.clone(),
                                                                            layer_rect)).unwrap();
            }
        }

        for kid in &*layer.children() {
            self.send_viewport_rect_for_layer(kid.clone());
        }
    }

    fn send_viewport_rects_for_all_layers(&self) {
        match self.scene.root {
            Some(ref root) => self.send_viewport_rect_for_layer(root.clone()),
            None => {},
        }
    }

    /// Returns true if any buffer requests were sent or false otherwise.
    fn send_buffer_requests_for_all_layers(&mut self) -> bool {
        if let Some(ref root_layer) = self.scene.root {
            root_layer.update_transform_state(&Matrix4::identity(),
                                              &Matrix4::identity(),
                                              &Point2D::zero());
        }

        let mut layers_and_requests = Vec::new();
        let mut unused_buffers = Vec::new();
        self.scene.get_buffer_requests(&mut layers_and_requests, &mut unused_buffers);

        // Return unused tiles first, so that they can be reused by any new BufferRequests.
        self.cache_unused_buffers(unused_buffers);

        if layers_and_requests.is_empty() {
            return false;
        }

        // We want to batch requests for each pipeline to avoid race conditions
        // when handling the resulting BufferRequest responses.
        let pipeline_requests =
            self.convert_buffer_requests_to_pipeline_requests_map(layers_and_requests);

        for (pipeline_id, requests) in pipeline_requests {
            let msg = ChromeToPaintMsg::Paint(requests, self.frame_tree_id);
            if let Some(pipeline) = self.pipeline(pipeline_id) {
                pipeline.chrome_to_paint_chan.send(msg).unwrap();
            }
        }

        true
    }

    /// Check if a layer (or its children) have any outstanding paint
    /// results to arrive yet.
    fn does_layer_have_outstanding_paint_messages(&self, layer: &Rc<Layer<CompositorData>>)
                                                  -> bool {
        let layer_data = layer.extra_data.borrow();
        let current_epoch = match self.pipeline_details.get(&layer_data.pipeline_id) {
            None => return false,
            Some(ref details) => details.current_epoch,
        };

        // Only check layers that have requested the current epoch, as there may be
        // layers that are not visible in the current viewport, and therefore
        // have not requested a paint of the current epoch.
        // If a layer has sent a request for the current epoch, but it hasn't
        // arrived yet then this layer is waiting for a paint message.
        //
        // Also don't check the root layer, because the paint thread won't paint
        // anything for it after first layout.
        if layer_data.id != LayerId::null() &&
                layer_data.requested_epoch == current_epoch &&
                layer_data.painted_epoch != current_epoch {
            return true;
        }

        for child in &*layer.children() {
            if self.does_layer_have_outstanding_paint_messages(child) {
                return true;
            }
        }

        false
    }

    /// Query the constellation to see if the current compositor
    /// output matches the current frame tree output, and if the
    /// associated script threads are idle.
    fn is_ready_to_paint_image_output(&mut self) -> Result<(), NotReadyToPaint> {
        match self.ready_to_save_state {
            ReadyState::Unknown => {
                // Unsure if the output image is stable.

                // Check if any layers are waiting for paints to complete
                // of their current epoch request. If so, early exit
                // from this check.
                match self.scene.root {
                    Some(ref root_layer) => {
                        if self.does_layer_have_outstanding_paint_messages(root_layer) {
                            return Err(NotReadyToPaint::LayerHasOutstandingPaintMessages);
                        }
                    }
                    None => {
                        return Err(NotReadyToPaint::MissingRoot);
                    }
                }

                // Check if there are any pending frames. If so, the image is not stable yet.
                if self.pending_subpages.len() > 0 {
                    return Err(NotReadyToPaint::PendingSubpages(self.pending_subpages.len()));
                }

                // Collect the currently painted epoch of each pipeline that is
                // complete (i.e. has *all* layers painted to the requested epoch).
                // This gets sent to the constellation for comparison with the current
                // frame tree.
                let mut pipeline_epochs = HashMap::new();
                for (id, details) in &self.pipeline_details {
                    // If animations are currently running, then don't bother checking
                    // with the constellation if the output image is stable.
                    if details.animations_running {
                        return Err(NotReadyToPaint::AnimationsRunning);
                    }
                    if details.animation_callbacks_running {
                        return Err(NotReadyToPaint::AnimationCallbacksRunning);
                    }

                    pipeline_epochs.insert(*id, details.current_epoch);
                }

                // Pass the pipeline/epoch states to the constellation and check
                // if it's safe to output the image.
                self.constellation_chan.send(ConstellationMsg::IsReadyToSaveImage(pipeline_epochs)).unwrap();
                self.ready_to_save_state = ReadyState::WaitingForConstellationReply;
                Err(NotReadyToPaint::JustNotifiedConstellation)
            }
            ReadyState::WaitingForConstellationReply => {
                // If waiting on a reply from the constellation to the last
                // query if the image is stable, then assume not ready yet.
                Err(NotReadyToPaint::WaitingOnConstellation)
            }
            ReadyState::ReadyToSaveImage => {
                // Constellation has replied at some point in the past
                // that the current output image is stable and ready
                // for saving.
                // Reset the flag so that we check again in the future
                // TODO: only reset this if we load a new document?
                if opts::get().is_running_problem_test {
                    println!("was ready to save, resetting ready_to_save_state");
                }
                self.ready_to_save_state = ReadyState::Unknown;
                Ok(())
            }
        }
    }

    fn composite(&mut self) {
        let target = self.composite_target;
        let composited = self.composite_specific_target(target);
        if composited.is_ok() &&
            (opts::get().output_file.is_some() || opts::get().exit_after_load) {
            println!("Shutting down the Constellation after generating an output file or exit flag specified");
            self.start_shutting_down();
        } else if composited.is_err() &&
            opts::get().is_running_problem_test &&
            composited.as_ref().err().unwrap() != &UnableToComposite::NotReadyToPaintImage(
                NotReadyToPaint::WaitingOnConstellation) {
            println!("not ready to composite: {:?}", composited.err().unwrap());
        }
    }

    /// Composite either to the screen or to a png image or both.
    /// Returns Ok if composition was performed or Err if it was not possible to composite
    /// for some reason. If CompositeTarget is Window or Png no image data is returned;
    /// in the latter case the image is written directly to a file. If CompositeTarget
    /// is WindowAndPng Ok(Some(png::Image)) is returned.
    pub fn composite_specific_target(&mut self, target: CompositeTarget) -> Result<Option<Image>, UnableToComposite> {

        if !self.context.is_some() {
            return Err(UnableToComposite::NoContext)
        }
        let (width, height) =
            (self.window_size.width.get() as usize, self.window_size.height.get() as usize);
        if !self.window.prepare_for_composite(width, height) {
            return Err(UnableToComposite::WindowUnprepared)
        }

        match target {
            CompositeTarget::WindowAndPng | CompositeTarget::PngFile => {
                if let Err(result) = self.is_ready_to_paint_image_output() {
                    return Err(UnableToComposite::NotReadyToPaintImage(result))
                }
            }
            CompositeTarget::Window => {
                if opts::get().exit_after_load {
                    if let Err(result) = self.is_ready_to_paint_image_output() {
                        return Err(UnableToComposite::NotReadyToPaintImage(result))
                    }
                }
            }
        }

        let (framebuffer_ids, texture_ids) = match target {
            CompositeTarget::Window => (vec!(), vec!()),
            _ => initialize_png(width, height)
        };

        profile(ProfilerCategory::Compositing, None, self.time_profiler_chan.clone(), || {
            debug!("compositor: compositing");
            self.dump_layer_tree();
            // Adjust the layer dimensions as necessary to correspond to the size of the window.
            self.scene.viewport = match self.viewport {
                Some((point, size)) => Rect {
                    origin: point.as_f32(),
                    size:   size.as_f32(),
                },

                None => Rect {
                    origin: Point2D::zero(),
                    size: self.window_size.as_f32(),
                }
            };

            // Paint the scene.
            if let Some(ref layer) = self.scene.root {
                match self.context {
                    Some(context) => {
                        if let Some((point, size)) = self.viewport {
                            let point = point.to_untyped();
                            let size  = size.to_untyped();

                            gl::scissor(point.x as GLint, point.y as GLint,
                                        size.width as GLsizei, size.height as GLsizei);

                            gl::enable(gl::SCISSOR_TEST);
                            rendergl::render_scene(layer.clone(), context, &self.scene);
                            gl::disable(gl::SCISSOR_TEST);

                        }
                        else {
                            rendergl::render_scene(layer.clone(), context, &self.scene);
                        }
                    }

                    None => {
                        debug!("compositor: not compositing because context not yet set up")
                    }
                }
            }
        });

        let rv = match target {
            CompositeTarget::Window => None,
            CompositeTarget::WindowAndPng => {
                let img = self.draw_img(framebuffer_ids, texture_ids, width, height);
                Some(Image {
                    width: img.width(),
                    height: img.height(),
                    format: PixelFormat::RGB8,
                    bytes: IpcSharedMemory::from_bytes(&*img),
                })
            }
            CompositeTarget::PngFile => {
                let img = self.draw_img(framebuffer_ids, texture_ids, width, height);
                let path = opts::get().output_file.as_ref().unwrap();
                let mut file = File::create(path).unwrap();
                DynamicImage::ImageRgb8(img).save(&mut file, ImageFormat::PNG).unwrap();
                None
            }
        };

        // Perform the page flip. This will likely block for a while.
        self.window.present();

        self.last_composite_time = precise_time_ns();

        self.composition_request = CompositionRequest::NoCompositingNecessary;
        self.process_pending_scroll_events();
        self.process_animations();

        Ok(rv)
    }

    fn draw_img(&self,
                framebuffer_ids: Vec<gl::GLuint>,
                texture_ids: Vec<gl::GLuint>,
                width: usize,
                height: usize)
                -> RgbImage {
        let mut pixels = gl::read_pixels(0, 0,
                                         width as gl::GLsizei,
                                         height as gl::GLsizei,
                                         gl::RGB, gl::UNSIGNED_BYTE);

        gl::bind_framebuffer(gl::FRAMEBUFFER, 0);

        gl::delete_buffers(&texture_ids);
        gl::delete_frame_buffers(&framebuffer_ids);

        // flip image vertically (texture is upside down)
        let orig_pixels = pixels.clone();
        let stride = width * 3;
        for y in 0..height {
            let dst_start = y * stride;
            let src_start = (height - y - 1) * stride;
            let src_slice = &orig_pixels[src_start .. src_start + stride];
            (&mut pixels[dst_start .. dst_start + stride]).clone_from_slice(&src_slice[..stride]);
        }
        RgbImage::from_raw(width as u32, height as u32, pixels).unwrap()
    }

    fn composite_if_necessary(&mut self, reason: CompositingReason) {
        if self.composition_request == CompositionRequest::NoCompositingNecessary {
            if opts::get().is_running_problem_test {
                println!("updating composition_request ({:?})", reason);
            }
            self.composition_request = CompositionRequest::CompositeNow(reason)
        } else if opts::get().is_running_problem_test {
            println!("composition_request is already {:?}", self.composition_request);
        }
    }

    fn initialize_compositing(&mut self) {
        let show_debug_borders = opts::get().show_debug_borders;
        self.context = Some(rendergl::RenderContext::new(self.native_display.clone(),
                                                         show_debug_borders,
                                                         opts::get().output_file.is_some()))
    }

    fn find_topmost_layer_at_point_for_layer(&self,
                                             layer: Rc<Layer<CompositorData>>,
                                             point_in_parent_layer: TypedPoint2D<LayerPixel, f32>,
                                             clip_rect_in_parent_layer: &TypedRect<LayerPixel, f32>)
                                             -> Option<HitTestResult> {
        let layer_bounds = *layer.bounds.borrow();
        let masks_to_bounds = *layer.masks_to_bounds.borrow();
        if layer_bounds.is_empty() && masks_to_bounds {
            return None;
        }
        let scroll_offset = layer.extra_data.borrow().scroll_offset;

        // Total offset from parent coordinates to this layer's coordinates.
        // FIXME: This offset is incorrect for fixed-position layers.
        let layer_offset = scroll_offset + layer_bounds.origin;

        let clipped_layer_bounds = match clip_rect_in_parent_layer.intersection(&layer_bounds) {
            Some(rect) => rect,
            None => return None,
        };

        let clip_rect_for_children = if masks_to_bounds {
            &clipped_layer_bounds
        } else {
            clip_rect_in_parent_layer
        }.translate(&-layer_offset);

        let child_point = point_in_parent_layer - layer_offset;
        for child in layer.children().iter().rev() {
            // Translate the clip rect into the child's coordinate system.
            let result = self.find_topmost_layer_at_point_for_layer(child.clone(),
                                                                    child_point,
                                                                    &clip_rect_for_children);
            if let Some(mut result) = result {
                // Return the point in layer coordinates of the topmost frame containing the point.
                let pipeline_id = layer.extra_data.borrow().pipeline_id;
                let child_pipeline_id = result.layer.extra_data.borrow().pipeline_id;
                if pipeline_id == child_pipeline_id {
                    result.point = result.point + layer_offset;
                }
                return Some(result);
            }
        }

        if !clipped_layer_bounds.contains(&point_in_parent_layer) {
            return None;
        }

        Some(HitTestResult { layer: layer, point: point_in_parent_layer })
    }

    fn find_topmost_layer_at_point(&self,
                                   point: TypedPoint2D<LayerPixel, f32>)
                                   -> Option<HitTestResult> {
        match self.scene.root {
            Some(ref layer) => {
                self.find_topmost_layer_at_point_for_layer(layer.clone(),
                                                           point,
                                                           &*layer.bounds.borrow())
            }

            None => None,
        }
    }

    fn find_layer_with_pipeline_and_layer_id(&self,
                                             pipeline_id: PipelineId,
                                             layer_id: LayerId)
                                             -> Option<Rc<Layer<CompositorData>>> {
        match self.scene.root {
            Some(ref layer) =>
                find_layer_with_pipeline_and_layer_id_for_layer(layer.clone(),
                                                                pipeline_id,
                                                                layer_id),

            None => None,
        }
    }

    pub fn cache_unused_buffers<B>(&mut self, buffers: B)
        where B: IntoIterator<Item=Box<LayerBuffer>>
    {
        let surfaces = buffers.into_iter().map(|buffer| buffer.native_surface);
        self.surface_map.insert_surfaces(&self.native_display, surfaces);
    }

    #[allow(dead_code)]
    fn dump_layer_tree(&self) {
        if !opts::get().dump_layer_tree {
            return;
        }

        let mut print_tree = PrintTree::new("Layer tree".to_owned());
        if let Some(ref layer) = self.scene.root {
            self.dump_layer_tree_layer(&**layer, &mut print_tree);
        }
    }

    #[allow(dead_code)]
    fn dump_layer_tree_layer(&self, layer: &Layer<CompositorData>, print_tree: &mut PrintTree) {
        let data = layer.extra_data.borrow();
        let layer_string = if data.id == LayerId::null() {
            format!("Root Layer (pipeline={})", data.pipeline_id)
        } else {
            "Layer".to_owned()
        };

        let masks_string = if *layer.masks_to_bounds.borrow() {
            " (masks children)"
        } else {
            ""
        };

        let establishes_3d_context_string = if layer.establishes_3d_context {
            " (3D context)"
        } else {
            ""
        };

        let fixed_string = if data.scroll_policy == ScrollPolicy::FixedPosition {
            " (fixed)"
        } else {
            ""
        };

        let layer_string = format!("{} ({:?}) ({},{} at {},{}){}{}{}",
                                   layer_string,
                                   layer.extra_data.borrow().id,
                                   (*layer.bounds.borrow()).size.to_untyped().width,
                                   (*layer.bounds.borrow()).size.to_untyped().height,
                                   (*layer.bounds.borrow()).origin.to_untyped().x,
                                   (*layer.bounds.borrow()).origin.to_untyped().y,
                                   masks_string,
                                   establishes_3d_context_string,
                                   fixed_string);

        let children = layer.children();
        if !children.is_empty() {
            print_tree.new_level(layer_string);
            for kid in &*children {
                self.dump_layer_tree_layer(&**kid, print_tree);
            }
            print_tree.end_level();
        } else {
            print_tree.add_item(layer_string);
        }
    }
}

fn find_layer_with_pipeline_and_layer_id_for_layer(layer: Rc<Layer<CompositorData>>,
                                                   pipeline_id: PipelineId,
                                                   layer_id: LayerId)
                                                   -> Option<Rc<Layer<CompositorData>>> {
    if layer.extra_data.borrow().pipeline_id == pipeline_id &&
       layer.extra_data.borrow().id == layer_id {
        return Some(layer);
    }

    for kid in &*layer.children() {
        let result = find_layer_with_pipeline_and_layer_id_for_layer(kid.clone(),
                                                                     pipeline_id,
                                                                     layer_id);
        if result.is_some() {
            return result;
        }
    }

    None
}

impl<Window> CompositorEventListener for IOCompositor<Window> where Window: WindowMethods {
    fn handle_events(&mut self, messages: Vec<WindowEvent>) -> bool {
        // Check for new messages coming from the other threads in the system.
        while let Some(msg) = self.port.try_recv_compositor_msg() {
            if !self.handle_browser_message(msg) {
                break
            }
        }

        if self.shutdown_state == ShutdownState::FinishedShuttingDown {
            return false;
        }

        // Handle any messages coming from the windowing system.
        for message in messages {
            self.handle_window_message(message);
        }

        // If a pinch-zoom happened recently, ask for tiles at the new resolution
        if self.zoom_action && precise_time_s() - self.zoom_time > 0.3 {
            self.zoom_action = false;
            self.scene.mark_layer_contents_as_changed_recursively();
            self.send_buffer_requests_for_all_layers();
        }

        match self.composition_request {
            CompositionRequest::NoCompositingNecessary |
            CompositionRequest::CompositeOnScrollTimeout(_) => {}
            CompositionRequest::CompositeNow(_) => {
                self.composite()
            }
        }

        self.shutdown_state != ShutdownState::FinishedShuttingDown
    }

    /// Repaints and recomposites synchronously. You must be careful when calling this, as if a
    /// paint is not scheduled the compositor will hang forever.
    ///
    /// This is used when resizing the window.
    fn repaint_synchronously(&mut self) {
        while self.shutdown_state != ShutdownState::ShuttingDown {
            let msg = self.port.recv_compositor_msg();
            let received_new_buffers = match msg {
                Msg::AssignPaintedBuffers(..) => true,
                _ => false,
            };
            let keep_going = self.handle_browser_message(msg);
            if received_new_buffers {
                self.composite();
                break
            }
            if !keep_going {
                break
            }
        }
    }

    fn pinch_zoom_level(&self) -> f32 {
        self.viewport_zoom.get() as f32
    }

    fn title_for_main_frame(&self) {
        let root_pipeline_id = match self.root_pipeline {
            None => return,
            Some(ref root_pipeline) => root_pipeline.id,
        };
        self.constellation_chan.send(ConstellationMsg::GetPipelineTitle(root_pipeline_id)).unwrap();
    }
}

/// Why we performed a composite. This is used for debugging.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CompositingReason {
    /// We hit the scroll timeout and are therefore drawing unrendered content.
    HitScrollTimeout,
    /// The window has been scrolled and we're starting the first recomposite.
    Scroll,
    /// A scroll has continued and we need to recomposite again.
    ContinueScroll,
    /// We're performing the single composite in headless mode.
    Headless,
    /// We're performing a composite to run an animation.
    Animation,
    /// A new frame tree has been loaded.
    NewFrameTree,
    /// New painted buffers have been received.
    NewPaintedBuffers,
    /// The window has been zoomed.
    Zoom,
}
