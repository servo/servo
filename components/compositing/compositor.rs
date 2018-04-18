/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use CompositionPipeline;
use SendableFrameTree;
use compositor_thread::{CompositorProxy, CompositorReceiver};
use compositor_thread::{InitialCompositorState, Msg};
use euclid::{TypedPoint2D, TypedVector2D, TypedScale};
use gfx_traits::Epoch;
#[cfg(feature = "gleam")]
use gl;
#[cfg(feature = "gleam")]
use image::{DynamicImage, ImageFormat};
use ipc_channel::ipc;
use libc::c_void;
use msg::constellation_msg::{PipelineId, PipelineIndex, PipelineNamespaceId};
use net_traits::image::base::Image;
#[cfg(feature = "gleam")]
use net_traits::image::base::PixelFormat;
use nonzero::NonZeroU32;
use profile_traits::time::{self, ProfilerCategory, profile};
use script_traits::{AnimationState, AnimationTickType, ConstellationMsg, LayoutControlMsg};
use script_traits::{MouseButton, MouseEventType, ScrollState, TouchEventType, TouchId};
use script_traits::{UntrustedNodeAddress, WindowSizeData, WindowSizeType};
use script_traits::CompositorEvent::{MouseMoveEvent, MouseButtonEvent, TouchEvent};
use servo_config::opts;
use servo_geometry::DeviceIndependentPixel;
use std::collections::HashMap;
use std::env;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::time::Instant;
use style_traits::{CSSPixel, DevicePixel, PinchZoomFactor};
use style_traits::cursor::CursorKind;
use style_traits::viewport::ViewportConstraints;
use time::{now, precise_time_ns, precise_time_s};
use touch::{TouchHandler, TouchAction};
use webrender;
use webrender_api::{self, DeviceIntPoint, DevicePoint, HitTestFlags, HitTestResult};
use webrender_api::{LayoutVector2D, ScrollLocation};
use windowing::{self, EmbedderCoordinates, MouseWindowEvent, WebRenderDebugOption, WindowMethods};


#[derive(Debug, PartialEq)]
enum UnableToComposite {
    WindowUnprepared,
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

trait ConvertPipelineIdFromWebRender {
    fn from_webrender(&self) -> PipelineId;
}

impl ConvertPipelineIdFromWebRender for webrender_api::PipelineId {
    fn from_webrender(&self) -> PipelineId {
        PipelineId {
            namespace_id: PipelineNamespaceId(self.0),
            index: PipelineIndex(NonZeroU32::new(self.1).expect("Webrender pipeline zero?")),
        }
    }
}

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

/// One pixel in layer coordinate space.
///
/// This unit corresponds to a "pixel" in layer coordinate space, which after scaling and
/// transformation becomes a device pixel.
#[derive(Clone, Copy, Debug)]
enum LayerPixel {}

/// NB: Never block on the constellation, because sometimes the constellation blocks on us.
pub struct IOCompositor<Window: WindowMethods> {
    /// The application window.
    pub window: Rc<Window>,

    /// The port on which we receive messages.
    port: CompositorReceiver,

    /// The root pipeline.
    root_pipeline: Option<CompositionPipeline>,

    /// Tracks details about each active pipeline that the compositor knows about.
    pipeline_details: HashMap<PipelineId, PipelineDetails>,

    /// The scene scale, to allow for zooming and high-resolution painting.
    scale: TypedScale<f32, LayerPixel, DevicePixel>,

    /// "Mobile-style" zoom that does not reflow the page.
    viewport_zoom: PinchZoomFactor,

    /// Viewport zoom constraints provided by @viewport.
    min_viewport_zoom: Option<PinchZoomFactor>,
    max_viewport_zoom: Option<PinchZoomFactor>,

    /// "Desktop-style" zoom that resizes the viewport to fit the window.
    page_zoom: TypedScale<f32, CSSPixel, DeviceIndependentPixel>,

    /// The type of composition to perform
    composite_target: CompositeTarget,

    /// Tracks whether we should composite this frame.
    composition_request: CompositionRequest,

    /// Tracks whether we are in the process of shutting down, or have shut down and should close
    /// the compositor.
    pub shutdown_state: ShutdownState,

    /// Tracks the last composite time.
    last_composite_time: u64,

    /// Tracks whether the zoom action has happened recently.
    zoom_action: bool,

    /// The time of the last zoom action has started.
    zoom_time: f64,

    /// The current frame tree ID (used to reject old paint buffers)
    frame_tree_id: FrameTreeId,

    /// The channel on which messages can be sent to the constellation.
    constellation_chan: Sender<ConstellationMsg>,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: time::ProfilerChan,

    /// Touch input state machine
    touch_handler: TouchHandler,

    /// Pending scroll/zoom events.
    pending_scroll_zoom_events: Vec<ScrollZoomEvent>,

    /// Whether we're waiting on a recomposite after dispatching a scroll.
    waiting_for_results_of_scroll: bool,

    /// Used by the logic that determines when it is safe to output an
    /// image for the reftest framework.
    ready_to_save_state: ReadyState,

    /// Whether a scroll is in progress; i.e. whether the user's fingers are down.
    scroll_in_progress: bool,

    in_scroll_transaction: Option<Instant>,

    /// The webrender renderer.
    webrender: webrender::Renderer,

    /// The active webrender document.
    webrender_document: webrender_api::DocumentId,

    /// The webrender interface, if enabled.
    webrender_api: webrender_api::RenderApi,

    /// Map of the pending paint metrics per layout thread.
    /// The layout thread for each specific pipeline expects the compositor to
    /// paint frames with specific given IDs (epoch). Once the compositor paints
    /// these frames, it records the paint time for each of them and sends the
    /// metric to the corresponding layout thread.
    pending_paint_metrics: HashMap<PipelineId, Epoch>,

    /// The coordinates of the native window, its view and the screen.
    embedder_coordinates: EmbedderCoordinates,
}

#[derive(Clone, Copy)]
struct ScrollZoomEvent {
    /// Change the pinch zoom level by this factor
    magnification: f32,
    /// Scroll by this offset, or to Start or End
    scroll_location: ScrollLocation,
    /// Apply changes to the frame at this location
    cursor: DeviceIntPoint,
    /// The number of OS events that have been coalesced together into this one event.
    event_count: u32,
}

#[derive(Debug, PartialEq)]
enum CompositionRequest {
    NoCompositingNecessary,
    CompositeNow(CompositingReason),
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

    /// Whether animations are running
    animations_running: bool,

    /// Whether there are animation callbacks
    animation_callbacks_running: bool,

    /// Whether this pipeline is visible
    visible: bool,
}

impl PipelineDetails {
    fn new() -> PipelineDetails {
        PipelineDetails {
            pipeline: None,
            animations_running: false,
            animation_callbacks_running: false,
            visible: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CompositeTarget {
    /// Normal composition to a window
    Window,

    /// Compose as normal, but also return a PNG of the composed output
    WindowAndPng,

    /// Compose to a PNG, write it to disk, and then exit the browser (used for reftests)
    PngFile
}

#[derive(Clone)]
pub struct RenderNotifier {
    compositor_proxy: CompositorProxy,
}

impl RenderNotifier {
    pub fn new(compositor_proxy: CompositorProxy) -> RenderNotifier {
        RenderNotifier {
            compositor_proxy: compositor_proxy,
        }
    }
}

impl webrender_api::RenderNotifier for RenderNotifier {
    fn clone(&self) -> Box<webrender_api::RenderNotifier> {
        Box::new(RenderNotifier::new(self.compositor_proxy.clone()))
    }

    fn wake_up(&self) {
        self.compositor_proxy.recomposite(CompositingReason::NewWebRenderFrame);
    }

    fn new_frame_ready(
        &self,
        _document_id: webrender_api::DocumentId,
        scrolled: bool,
        composite_needed: bool,
    ) {
        if scrolled {
            self.compositor_proxy.send(Msg::NewScrollFrameReady(composite_needed));
        } else {
            self.wake_up();
        }
    }
}

impl<Window: WindowMethods> IOCompositor<Window> {
    fn new(window: Rc<Window>, state: InitialCompositorState) -> Self {
        let composite_target = match opts::get().output_file {
            Some(_) => CompositeTarget::PngFile,
            None => CompositeTarget::Window
        };

        IOCompositor {
            embedder_coordinates: window.get_coordinates(),
            window,
            port: state.receiver,
            root_pipeline: None,
            pipeline_details: HashMap::new(),
            scale: TypedScale::new(1.0),
            composition_request: CompositionRequest::NoCompositingNecessary,
            touch_handler: TouchHandler::new(),
            pending_scroll_zoom_events: Vec::new(),
            waiting_for_results_of_scroll: false,
            composite_target,
            shutdown_state: ShutdownState::NotShuttingDown,
            page_zoom: TypedScale::new(1.0),
            viewport_zoom: PinchZoomFactor::new(1.0),
            min_viewport_zoom: None,
            max_viewport_zoom: None,
            zoom_action: false,
            zoom_time: 0f64,
            frame_tree_id: FrameTreeId(0),
            constellation_chan: state.constellation_chan,
            time_profiler_chan: state.time_profiler_chan,
            last_composite_time: 0,
            ready_to_save_state: ReadyState::Unknown,
            scroll_in_progress: false,
            in_scroll_transaction: None,
            webrender: state.webrender,
            webrender_document: state.webrender_document,
            webrender_api: state.webrender_api,
            pending_paint_metrics: HashMap::new(),
        }
    }

    pub fn create(window: Rc<Window>, state: InitialCompositorState) -> Self {
        let mut compositor = IOCompositor::new(window, state);

        // Set the size of the root layer.
        compositor.update_zoom_transform();

        // Tell the constellation about the initial window size.
        compositor.send_window_size(WindowSizeType::Initial);

        compositor
    }

    pub fn deinit(self) {
        self.webrender.deinit();
    }

    pub fn maybe_start_shutting_down(&mut self) {
        if self.shutdown_state == ShutdownState::NotShuttingDown {
            debug!("Shutting down the constellation for WindowEvent::Quit");
            self.start_shutting_down();
        }
    }

    fn start_shutting_down(&mut self) {
        debug!("Compositor sending Exit message to Constellation");
        if let Err(e) = self.constellation_chan.send(ConstellationMsg::Exit) {
            warn!("Sending exit message to constellation failed ({}).", e);
        }

        self.shutdown_state = ShutdownState::ShuttingDown;
    }

    fn finish_shutting_down(&mut self) {
        debug!("Compositor received message that constellation shutdown is complete");

        // Drain compositor port, sometimes messages contain channels that are blocking
        // another thread from finishing (i.e. SetFrameTree).
        while self.port.try_recv_compositor_msg().is_some() {}

        // Tell the profiler, memory profiler, and scrolling timer to shut down.
        if let Ok((sender, receiver)) = ipc::channel() {
            self.time_profiler_chan.send(time::ProfilerMsg::Exit(sender));
            let _ = receiver.recv();
        }

        self.shutdown_state = ShutdownState::FinishedShuttingDown;
    }

    fn handle_browser_message(&mut self, msg: Msg) -> bool {
        match (msg, self.shutdown_state) {
            (_, ShutdownState::FinishedShuttingDown) => {
                error!("compositor shouldn't be handling messages after shutting down");
                return false
            }

            (Msg::Exit, _) => {
                self.start_shutting_down();
            }

            (Msg::ShutdownComplete, _) => {
                self.finish_shutting_down();
                return false;
            }

            (Msg::ChangeRunningAnimationsState(pipeline_id, animation_state),
             ShutdownState::NotShuttingDown) => {
                self.change_running_animations_state(pipeline_id, animation_state);
            }

            (Msg::SetFrameTree(frame_tree),
             ShutdownState::NotShuttingDown) => {
                self.set_frame_tree(&frame_tree);
                self.send_viewport_rects();
            }

            (Msg::Recomposite(reason), ShutdownState::NotShuttingDown) => {
                self.composition_request = CompositionRequest::CompositeNow(reason)
            }


            (Msg::TouchEventProcessed(result), ShutdownState::NotShuttingDown) => {
                self.touch_handler.on_event_processed(result);
            }

            (Msg::CreatePng(reply), ShutdownState::NotShuttingDown) => {
                let res = self.composite_specific_target(CompositeTarget::WindowAndPng);
                if let Err(ref e) = res {
                    info!("Error retrieving PNG: {:?}", e);
                }
                let img = res.unwrap_or(None);
                if let Err(e) = reply.send(img) {
                    warn!("Sending reply to create png failed ({}).", e);
                }
            }

            (Msg::ViewportConstrained(pipeline_id, constraints),
             ShutdownState::NotShuttingDown) => {
                self.constrain_viewport(pipeline_id, constraints);
            }

            (Msg::IsReadyToSaveImageReply(is_ready), ShutdownState::NotShuttingDown) => {
                assert_eq!(self.ready_to_save_state, ReadyState::WaitingForConstellationReply);
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

            (Msg::PipelineVisibilityChanged(pipeline_id, visible), ShutdownState::NotShuttingDown) => {
                self.pipeline_details(pipeline_id).visible = visible;
                if visible {
                    self.process_animations();
                }
            }

            (Msg::PipelineExited(pipeline_id, sender), _) => {
                debug!("Compositor got pipeline exited: {:?}", pipeline_id);
                self.remove_pipeline_root_layer(pipeline_id);
                let _ = sender.send(());
            }

            (Msg::NewScrollFrameReady(recomposite_needed), ShutdownState::NotShuttingDown) => {
                self.waiting_for_results_of_scroll = false;
                if recomposite_needed {
                    self.composition_request = CompositionRequest::CompositeNow(
                        CompositingReason::NewWebRenderScrollFrame);
                }
            }

            (Msg::Dispatch(func), ShutdownState::NotShuttingDown) => {
                // The functions sent here right now are really dumb, so they can't panic.
                // But if we start running more complex code here, we should really catch panic here.
                func();
            }

            (Msg::LoadComplete(_), ShutdownState::NotShuttingDown) => {
                // If we're painting in headless mode, schedule a recomposite.
                if opts::get().output_file.is_some() || opts::get().exit_after_load {
                    self.composite_if_necessary(CompositingReason::Headless);
                }
            },

            (Msg::PendingPaintMetric(pipeline_id, epoch), _) => {
                self.pending_paint_metrics.insert(pipeline_id, epoch);
            }

            (Msg::GetClientWindow(req), ShutdownState::NotShuttingDown) => {
                if let Err(e) = req.send(self.embedder_coordinates.window) {
                    warn!("Sending response to get client window failed ({}).", e);
                }
            }

            (Msg::GetScreenSize(req), ShutdownState::NotShuttingDown) => {
                if let Err(e) = req.send(self.embedder_coordinates.screen) {
                    warn!("Sending response to get screen size failed ({}).", e);
                }
            }

            (Msg::GetScreenAvailSize(req), ShutdownState::NotShuttingDown) => {
                if let Err(e) = req.send(self.embedder_coordinates.screen_avail) {
                    warn!("Sending response to get screen avail size failed ({}).", e);
                }
            }

            // When we are shutting_down, we need to avoid performing operations
            // such as Paint that may crash because we have begun tearing down
            // the rest of our resources.
            (_, ShutdownState::ShuttingDown) => {}
        }

        true
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
                let visible = self.pipeline_details(pipeline_id).visible;
                self.pipeline_details(pipeline_id).animations_running = true;
                if visible {
                    self.composite_if_necessary(CompositingReason::Animation);
                }
            }
            AnimationState::AnimationCallbacksPresent => {
                let visible = self.pipeline_details(pipeline_id).visible;
                self.pipeline_details(pipeline_id).animation_callbacks_running = true;
                if visible {
                    self.tick_animations_for_pipeline(pipeline_id);
                }
            }
            AnimationState::NoAnimationsPresent => {
                self.pipeline_details(pipeline_id).animations_running = false;
            }
            AnimationState::NoAnimationCallbacksPresent => {
                self.pipeline_details(pipeline_id).animation_callbacks_running = false;
            }
        }
    }

    fn pipeline_details(&mut self, pipeline_id: PipelineId) -> &mut PipelineDetails {
        if !self.pipeline_details.contains_key(&pipeline_id) {
            self.pipeline_details.insert(pipeline_id, PipelineDetails::new());
        }
        self.pipeline_details.get_mut(&pipeline_id).expect("Insert then get failed!")
    }

    pub fn pipeline(&self, pipeline_id: PipelineId) -> Option<&CompositionPipeline> {
        match self.pipeline_details.get(&pipeline_id) {
            Some(ref details) => details.pipeline.as_ref(),
            None => {
                warn!("Compositor layer has an unknown pipeline ({:?}).", pipeline_id);
                None
            }
        }
    }

    fn set_frame_tree(&mut self, frame_tree: &SendableFrameTree) {
        debug!("Setting the frame tree for pipeline {}", frame_tree.pipeline.id);

        self.root_pipeline = Some(frame_tree.pipeline.clone());

        let pipeline_id = frame_tree.pipeline.id.to_webrender();
        let mut txn = webrender_api::Transaction::new();
        txn.set_root_pipeline(pipeline_id);
        txn.generate_frame();
        self.webrender_api.send_transaction(self.webrender_document, txn);

        self.create_pipeline_details_for_frame_tree(&frame_tree);

        self.send_window_size(WindowSizeType::Initial);

        self.frame_tree_id.next();
    }

    fn create_pipeline_details_for_frame_tree(&mut self, frame_tree: &SendableFrameTree) {
        self.pipeline_details(frame_tree.pipeline.id).pipeline = Some(frame_tree.pipeline.clone());

        for kid in &frame_tree.children {
            self.create_pipeline_details_for_frame_tree(kid);
        }
    }

    fn remove_pipeline_root_layer(&mut self, pipeline_id: PipelineId) {
        self.pipeline_details.remove(&pipeline_id);
    }

    fn send_window_size(&self, size_type: WindowSizeType) {
        let dppx = self.page_zoom * self.embedder_coordinates.hidpi_factor;

        self.webrender_api.set_window_parameters(self.webrender_document,
                                                 self.embedder_coordinates.framebuffer,
                                                 self.embedder_coordinates.viewport,
                                                 self.embedder_coordinates.hidpi_factor.get());

        let initial_viewport = self.embedder_coordinates.viewport.size.to_f32() / dppx;

        let data = WindowSizeData {
            device_pixel_ratio: dppx,
            initial_viewport: initial_viewport,
        };

        let top_level_browsing_context_id = self.root_pipeline.as_ref().map(|pipeline| {
            pipeline.top_level_browsing_context_id
        });

        let msg = ConstellationMsg::WindowSize(top_level_browsing_context_id, data, size_type);

        if let Err(e) = self.constellation_chan.send(msg) {
            warn!("Sending window resize to constellation failed ({}).", e);
        }
    }

    pub fn on_resize_window_event(&mut self) {
        debug!("compositor resize requested");

        let old_coords = self.embedder_coordinates;
        self.embedder_coordinates = self.window.get_coordinates();

        // A size change could also mean a resolution change.
        if self.embedder_coordinates.hidpi_factor != old_coords.hidpi_factor {
            self.update_zoom_transform();
        }

        if self.embedder_coordinates.viewport == old_coords.viewport &&
           self.embedder_coordinates.framebuffer == old_coords.framebuffer {
            return;
        }

        self.send_window_size(WindowSizeType::Resize);
    }

    pub fn on_mouse_window_event_class(&mut self, mouse_window_event: MouseWindowEvent) {
        if opts::get().convert_mouse_to_touch {
            match mouse_window_event {
                MouseWindowEvent::Click(_, _) => {}
                MouseWindowEvent::MouseDown(_, p) => self.on_touch_down(TouchId(0), p),
                MouseWindowEvent::MouseUp(_, p) => self.on_touch_up(TouchId(0), p),
            }
            return
        }

        self.dispatch_mouse_window_event_class(mouse_window_event);
    }

    fn dispatch_mouse_window_event_class(&mut self, mouse_window_event: MouseWindowEvent) {
        let point = match mouse_window_event {
            MouseWindowEvent::Click(_, p) => p,
            MouseWindowEvent::MouseDown(_, p) => p,
            MouseWindowEvent::MouseUp(_, p) => p,
        };

        let results = self.hit_test_at_point(point);
        let result = match results.items.first() {
            Some(result) => result,
            None => return,
        };

        let (button, event_type) = match mouse_window_event {
            MouseWindowEvent::Click(button, _) => (button, MouseEventType::Click),
            MouseWindowEvent::MouseDown(button, _) => (button, MouseEventType::MouseDown),
            MouseWindowEvent::MouseUp(button, _) => (button, MouseEventType::MouseUp),
        };

        let event_to_send = MouseButtonEvent(
            event_type,
            button,
            result.point_in_viewport.to_untyped(),
            Some(UntrustedNodeAddress(result.tag.0 as *const c_void)),
            Some(result.point_relative_to_item.to_untyped()),
        );

        let pipeline_id = PipelineId::from_webrender(result.pipeline);
        let msg = ConstellationMsg::ForwardEvent(pipeline_id, event_to_send);
        if let Err(e) = self.constellation_chan.send(msg) {
            warn!("Sending event to constellation failed ({}).", e);
        }
    }

    fn hit_test_at_point(&self, point: DevicePoint) -> HitTestResult {
        let dppx = self.page_zoom * self.hidpi_factor();
        let scaled_point = (point / dppx).to_untyped();

        let world_cursor = webrender_api::WorldPoint::from_untyped(&scaled_point);
        self.webrender_api.hit_test(
            self.webrender_document,
            None,
            world_cursor,
            HitTestFlags::empty()
        )

    }

    pub fn on_mouse_window_move_event_class(&mut self, cursor: DevicePoint) {
        if opts::get().convert_mouse_to_touch {
            self.on_touch_move(TouchId(0), cursor);
            return
        }

        self.dispatch_mouse_window_move_event_class(cursor);
    }

    fn dispatch_mouse_window_move_event_class(&mut self, cursor: DevicePoint) {
        let root_pipeline_id = match self.get_root_pipeline_id() {
            Some(root_pipeline_id) => root_pipeline_id,
            None => return,
        };
        if self.pipeline(root_pipeline_id).is_none() {
            return;
        }

        let results = self.hit_test_at_point(cursor);
        if let Some(item) = results.items.first() {
            let node_address = Some(UntrustedNodeAddress(item.tag.0 as *const c_void));
            let event = MouseMoveEvent(Some(item.point_in_viewport.to_untyped()), node_address);
            let pipeline_id = PipelineId::from_webrender(item.pipeline);
            let msg = ConstellationMsg::ForwardEvent(pipeline_id, event);
            if let Err(e) = self.constellation_chan.send(msg) {
                warn!("Sending event to constellation failed ({}).", e);
            }

            if let Some(cursor) =  CursorKind::from_u8(item.tag.1 as _).ok() {
                let msg = ConstellationMsg::SetCursor(cursor);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending event to constellation failed ({}).", e);
                }
            }
        }
    }

    fn send_touch_event(
        &self,
        event_type: TouchEventType,
        identifier: TouchId,
        point: DevicePoint)
    {
        let results = self.hit_test_at_point(point);
        if let Some(item) = results.items.first() {
            let event = TouchEvent(
                event_type,
                identifier,
                item.point_in_viewport.to_untyped(),
                Some(UntrustedNodeAddress(item.tag.0 as *const c_void)),
            );
            let pipeline_id = PipelineId::from_webrender(item.pipeline);
            let msg = ConstellationMsg::ForwardEvent(pipeline_id, event);
            if let Err(e) = self.constellation_chan.send(msg) {
                warn!("Sending event to constellation failed ({}).", e);
            }
        }
    }

    pub fn on_touch_event(&mut self,
                          event_type: TouchEventType,
                          identifier: TouchId,
                          location: DevicePoint) {
        match event_type {
            TouchEventType::Down => self.on_touch_down(identifier, location),
            TouchEventType::Move => self.on_touch_move(identifier, location),
            TouchEventType::Up => self.on_touch_up(identifier, location),
            TouchEventType::Cancel => self.on_touch_cancel(identifier, location),
        }
    }

    fn on_touch_down(&mut self, identifier: TouchId, point: DevicePoint) {
        self.touch_handler.on_touch_down(identifier, point);
        self.send_touch_event(TouchEventType::Down, identifier, point);
    }

    fn on_touch_move(&mut self, identifier: TouchId, point: DevicePoint) {
        match self.touch_handler.on_touch_move(identifier, point) {
            TouchAction::Scroll(delta) => {
                match point.cast() {
                    Some(point) => self.on_scroll_window_event(
                        ScrollLocation::Delta(
                            LayoutVector2D::from_untyped(&delta.to_untyped())
                        ),
                        point
                    ),
                    None => error!("Point cast failed."),
                }
            }
            TouchAction::Zoom(magnification, scroll_delta) => {
                let cursor = TypedPoint2D::new(-1, -1);  // Make sure this hits the base layer.
                self.pending_scroll_zoom_events.push(ScrollZoomEvent {
                    magnification: magnification,
                    scroll_location: ScrollLocation::Delta(webrender_api::LayoutVector2D::from_untyped(
                                                           &scroll_delta.to_untyped())),
                    cursor: cursor,
                    event_count: 1,
                });
            }
            TouchAction::DispatchEvent => {
                self.send_touch_event(TouchEventType::Move, identifier, point);
            }
            _ => {}
        }
    }

    fn on_touch_up(&mut self, identifier: TouchId, point: DevicePoint) {
        self.send_touch_event(TouchEventType::Up, identifier, point);

        if let TouchAction::Click = self.touch_handler.on_touch_up(identifier, point) {
            self.simulate_mouse_click(point);
        }
    }

    fn on_touch_cancel(&mut self, identifier: TouchId, point: DevicePoint) {
        // Send the event to script.
        self.touch_handler.on_touch_cancel(identifier, point);
        self.send_touch_event(TouchEventType::Cancel, identifier, point);
    }

    /// <http://w3c.github.io/touch-events/#mouse-events>
    fn simulate_mouse_click(&mut self, p: DevicePoint) {
        let button = MouseButton::Left;
        self.dispatch_mouse_window_move_event_class(p);
        self.dispatch_mouse_window_event_class(MouseWindowEvent::MouseDown(button, p));
        self.dispatch_mouse_window_event_class(MouseWindowEvent::MouseUp(button, p));
        self.dispatch_mouse_window_event_class(MouseWindowEvent::Click(button, p));
    }

    pub fn on_scroll_event(&mut self,
                           delta: ScrollLocation,
                           cursor: DeviceIntPoint,
                           phase: TouchEventType) {
        match phase {
            TouchEventType::Move => self.on_scroll_window_event(delta, cursor),
            TouchEventType::Up | TouchEventType::Cancel => {
                self.on_scroll_end_window_event(delta, cursor);
            }
            TouchEventType::Down => {
                self.on_scroll_start_window_event(delta, cursor);
            }
        }
    }

    fn on_scroll_window_event(&mut self,
                              scroll_location: ScrollLocation,
                              cursor: DeviceIntPoint) {
        self.in_scroll_transaction = Some(Instant::now());
        self.pending_scroll_zoom_events.push(ScrollZoomEvent {
            magnification: 1.0,
            scroll_location: scroll_location,
            cursor: cursor,
            event_count: 1,
        });
    }

    fn on_scroll_start_window_event(&mut self,
                                    scroll_location: ScrollLocation,
                                    cursor: DeviceIntPoint) {
        self.scroll_in_progress = true;
        self.pending_scroll_zoom_events.push(ScrollZoomEvent {
            magnification: 1.0,
            scroll_location: scroll_location,
            cursor: cursor,
            event_count: 1,
        });
    }

    fn on_scroll_end_window_event(&mut self,
                                  scroll_location: ScrollLocation,
                                  cursor: DeviceIntPoint) {
        self.scroll_in_progress = false;
        self.pending_scroll_zoom_events.push(ScrollZoomEvent {
            magnification: 1.0,
            scroll_location: scroll_location,
            cursor: cursor,
            event_count: 1,
        });
    }

    fn process_pending_scroll_events(&mut self) {
        let had_events = self.pending_scroll_zoom_events.len() > 0;

        // Batch up all scroll events into one, or else we'll do way too much painting.
        let mut last_combined_event: Option<ScrollZoomEvent> = None;
        for scroll_event in self.pending_scroll_zoom_events.drain(..) {
            let this_cursor = scroll_event.cursor;

            let this_delta = match scroll_event.scroll_location {
                ScrollLocation::Delta(delta) => delta,
                ScrollLocation::Start | ScrollLocation::End => {
                    // If this is an event which is scrolling to the start or end of the page,
                    // disregard other pending events and exit the loop.
                    last_combined_event = Some(scroll_event);
                    break;
                }
            };

            match &mut last_combined_event {
                last_combined_event @ &mut None => {
                    *last_combined_event = Some(ScrollZoomEvent {
                        magnification: scroll_event.magnification,
                        scroll_location: ScrollLocation::Delta(webrender_api::LayoutVector2D::from_untyped(
                                                               &this_delta.to_untyped())),
                        cursor: this_cursor,
                        event_count: 1,
                    })
                }
                &mut Some(ref mut last_combined_event) => {
                    // Mac OS X sometimes delivers scroll events out of vsync during a
                    // fling. This causes events to get bunched up occasionally, causing
                    // nasty-looking "pops". To mitigate this, during a fling we average
                    // deltas instead of summing them.
                    if let ScrollLocation::Delta(delta) = last_combined_event.scroll_location {
                        let old_event_count =
                            TypedScale::new(last_combined_event.event_count as f32);
                        last_combined_event.event_count += 1;
                        let new_event_count =
                            TypedScale::new(last_combined_event.event_count as f32);
                        last_combined_event.scroll_location = ScrollLocation::Delta(
                            (delta * old_event_count + this_delta) /
                            new_event_count);
                    }
                }
            }
        }

        if let Some(combined_event) = last_combined_event {
            let scroll_location = match combined_event.scroll_location {
                ScrollLocation::Delta(delta) => {
                    let scaled_delta = (TypedVector2D::from_untyped(&delta.to_untyped()) / self.scale)
                                       .to_untyped();
                    let calculated_delta = webrender_api::LayoutVector2D::from_untyped(&scaled_delta);
                                           ScrollLocation::Delta(calculated_delta)
                },
                // Leave ScrollLocation unchanged if it is Start or End location.
                sl @ ScrollLocation::Start | sl @ ScrollLocation::End => sl,
            };
            let cursor = (combined_event.cursor.to_f32() / self.scale).to_untyped();
            let cursor = webrender_api::WorldPoint::from_untyped(&cursor);
            let mut txn = webrender_api::Transaction::new();
            txn.scroll(scroll_location, cursor);
            self.webrender_api.send_transaction(self.webrender_document, txn);
            self.waiting_for_results_of_scroll = true
        }

        if had_events {
            self.send_viewport_rects();
        }
    }

    /// If there are any animations running, dispatches appropriate messages to the constellation.
    fn process_animations(&mut self) {
        let mut pipeline_ids = vec![];
        for (pipeline_id, pipeline_details) in &self.pipeline_details {
            if (pipeline_details.animations_running ||
                pipeline_details.animation_callbacks_running) &&
               pipeline_details.visible {
                   pipeline_ids.push(*pipeline_id);
            }
        }
        let animation_state = if pipeline_ids.is_empty() {
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
        let animation_callbacks_running = self.pipeline_details(pipeline_id).animation_callbacks_running;
        if animation_callbacks_running {
            let msg = ConstellationMsg::TickAnimation(pipeline_id, AnimationTickType::Script);
            if let Err(e) = self.constellation_chan.send(msg) {
                warn!("Sending tick to constellation failed ({}).", e);
            }
        }

        // We may need to tick animations in layout. (See #12749.)
        let animations_running = self.pipeline_details(pipeline_id).animations_running;
        if animations_running {
            let msg = ConstellationMsg::TickAnimation(pipeline_id, AnimationTickType::Layout);
            if let Err(e) = self.constellation_chan.send(msg) {
                warn!("Sending tick to constellation failed ({}).", e);
            }
        }
    }

    fn constrain_viewport(&mut self, pipeline_id: PipelineId, constraints: ViewportConstraints) {
        let is_root = self.root_pipeline.as_ref().map_or(false, |root_pipeline| {
            root_pipeline.id == pipeline_id
        });

        if is_root {
            self.viewport_zoom = constraints.initial_zoom;
            self.min_viewport_zoom = constraints.min_zoom;
            self.max_viewport_zoom = constraints.max_zoom;
            self.update_zoom_transform();
        }
    }

    fn hidpi_factor(&self) -> TypedScale<f32, DeviceIndependentPixel, DevicePixel> {
        match opts::get().device_pixels_per_px {
            Some(device_pixels_per_px) => TypedScale::new(device_pixels_per_px),
            None => match opts::get().output_file {
                Some(_) => TypedScale::new(1.0),
                None => self.embedder_coordinates.hidpi_factor,
            }
        }
    }

    fn device_pixels_per_page_px(&self) -> TypedScale<f32, CSSPixel, DevicePixel> {
        self.page_zoom * self.hidpi_factor()
    }

    fn update_zoom_transform(&mut self) {
        let scale = self.device_pixels_per_page_px();
        self.scale = TypedScale::new(scale.get());
    }

    pub fn on_zoom_reset_window_event(&mut self) {
        self.page_zoom = TypedScale::new(1.0);
        self.update_zoom_transform();
        self.send_window_size(WindowSizeType::Resize);
        self.update_page_zoom_for_webrender();
    }

    pub fn on_zoom_window_event(&mut self, magnification: f32) {
        self.page_zoom = TypedScale::new((self.page_zoom.get() * magnification)
                                          .max(MIN_ZOOM).min(MAX_ZOOM));
        self.update_zoom_transform();
        self.send_window_size(WindowSizeType::Resize);
        self.update_page_zoom_for_webrender();
    }

    fn update_page_zoom_for_webrender(&mut self) {
        let page_zoom = webrender_api::ZoomFactor::new(self.page_zoom.get());

        let mut txn = webrender_api::Transaction::new();
        txn.set_page_zoom(page_zoom);
        self.webrender_api.send_transaction(self.webrender_document, txn);
    }

    /// Simulate a pinch zoom
    pub fn on_pinch_zoom_window_event(&mut self, magnification: f32) {
        self.pending_scroll_zoom_events.push(ScrollZoomEvent {
            magnification: magnification,
            scroll_location: ScrollLocation::Delta(TypedVector2D::zero()), // TODO: Scroll to keep the center in view?
            cursor:  TypedPoint2D::new(-1, -1), // Make sure this hits the base layer.
            event_count: 1,
        });
    }

    fn send_viewport_rects(&self) {
        let mut scroll_states_per_pipeline = HashMap::new();
        for scroll_layer_state in self.webrender_api.get_scroll_node_state(self.webrender_document) {
            let scroll_state = ScrollState {
                scroll_id: scroll_layer_state.id,
                scroll_offset: scroll_layer_state.scroll_offset.to_untyped(),
            };

            scroll_states_per_pipeline
                .entry(scroll_layer_state.id.pipeline_id())
                .or_insert(vec![])
                .push(scroll_state);
        }

        for (pipeline_id, scroll_states) in scroll_states_per_pipeline {
            if let Some(pipeline) = self.pipeline(pipeline_id.from_webrender()) {
                let msg = LayoutControlMsg::SetScrollStates(scroll_states);
                let _ = pipeline.layout_chan.send(msg);
            }
        }
    }

    // Check if any pipelines currently have active animations or animation callbacks.
    fn animations_active(&self) -> bool {
        for (_, details) in &self.pipeline_details {
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
                for (id, _) in &self.pipeline_details {
                    let webrender_pipeline_id = id.to_webrender();
                    if let Some(webrender_api::Epoch(epoch)) = self.webrender
                                                                   .current_epoch(webrender_pipeline_id) {
                        let epoch = Epoch(epoch);
                        pipeline_epochs.insert(*id, epoch);
                    }
                }

                // Pass the pipeline/epoch states to the constellation and check
                // if it's safe to output the image.
                let msg = ConstellationMsg::IsReadyToSaveImage(pipeline_epochs);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending ready to save to constellation failed ({}).", e);
                }
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

    pub fn composite(&mut self) {
        let target = self.composite_target;
        match self.composite_specific_target(target) {
            Ok(_) => if opts::get().output_file.is_some() || opts::get().exit_after_load {
                println!("Shutting down the Constellation after generating an output file or exit flag specified");
                self.start_shutting_down();
            },
            Err(e) => if opts::get().is_running_problem_test {
                if e != UnableToComposite::NotReadyToPaintImage(NotReadyToPaint::WaitingOnConstellation) {
                    println!("not ready to composite: {:?}", e);
                }
            },
        }
    }

    /// Composite either to the screen or to a png image or both.
    /// Returns Ok if composition was performed or Err if it was not possible to composite
    /// for some reason. If CompositeTarget is Window or Png no image data is returned;
    /// in the latter case the image is written directly to a file. If CompositeTarget
    /// is WindowAndPng Ok(Some(png::Image)) is returned.
    fn composite_specific_target(&mut self,
                                 target: CompositeTarget)
                                 -> Result<Option<Image>, UnableToComposite> {
        let width = self.embedder_coordinates.framebuffer.width_typed();
        let height = self.embedder_coordinates.framebuffer.height_typed();
        if !self.window.prepare_for_composite(width, height) {
            return Err(UnableToComposite::WindowUnprepared)
        }

        self.webrender.update();

        let wait_for_stable_image = match target {
            CompositeTarget::WindowAndPng | CompositeTarget::PngFile => true,
            CompositeTarget::Window => opts::get().exit_after_load,
        };

        if wait_for_stable_image {
            // The current image may be ready to output. However, if there are animations active,
            // tick those instead and continue waiting for the image output to be stable AND
            // all active animations to complete.
            if self.animations_active() {
                self.process_animations();
                return Err(UnableToComposite::NotReadyToPaintImage(NotReadyToPaint::AnimationsActive));
            }
            if let Err(result) = self.is_ready_to_paint_image_output() {
                return Err(UnableToComposite::NotReadyToPaintImage(result))
            }
        }

        let rt_info = match target {
            #[cfg(feature = "gleam")]
            CompositeTarget::Window => {
                gl::RenderTargetInfo::default()
            }
            #[cfg(feature = "gleam")]
            CompositeTarget::WindowAndPng |
            CompositeTarget::PngFile => {
                gl::initialize_png(&*self.window.gl(), width, height)
            }
            #[cfg(not(feature = "gleam"))]
            _ => ()
        };

        profile(ProfilerCategory::Compositing, None, self.time_profiler_chan.clone(), || {
            debug!("compositor: compositing");

            // Paint the scene.
            // TODO(gw): Take notice of any errors the renderer returns!
            self.webrender.render(self.embedder_coordinates.framebuffer).ok();
        });

        // If there are pending paint metrics, we check if any of the painted epochs is
        // one of the ones that the paint metrics recorder is expecting . In that case,
        // we get the current time, inform the layout thread about it and remove the
        // pending metric from the list.
        if !self.pending_paint_metrics.is_empty() {
            let paint_time = precise_time_ns();
            let mut to_remove = Vec::new();
            // For each pending paint metrics pipeline id
            for (id, pending_epoch) in &self.pending_paint_metrics {
                // we get the last painted frame id from webrender
                if let Some(webrender_api::Epoch(epoch)) = self.webrender.current_epoch(id.to_webrender()) {
                    // and check if it is the one the layout thread is expecting,
                    let epoch = Epoch(epoch);
                    if *pending_epoch != epoch {
                        continue;
                    }
                    // in which case, we remove it from the list of pending metrics,
                    to_remove.push(id.clone());
                    if let Some(pipeline) = self.pipeline(*id) {
                        // and inform the layout thread with the measured paint time.
                        let msg = LayoutControlMsg::PaintMetric(epoch, paint_time);
                        if let Err(e)  = pipeline.layout_chan.send(msg) {
                            warn!("Sending PaintMetric message to layout failed ({}).", e);
                        }
                    }
                }
            }
            for id in to_remove.iter() {
                self.pending_paint_metrics.remove(id);
            }
        }

        let rv = match target {
            CompositeTarget::Window => None,
            #[cfg(feature = "gleam")]
            CompositeTarget::WindowAndPng => {
                let img = gl::draw_img(&*self.window.gl(), rt_info, width, height);
                Some(Image {
                    width: img.width(),
                    height: img.height(),
                    format: PixelFormat::RGB8,
                    bytes: ipc::IpcSharedMemory::from_bytes(&*img),
                    id: None,
                })
            }
            #[cfg(feature = "gleam")]
            CompositeTarget::PngFile => {
                let gl = &*self.window.gl();
                profile(ProfilerCategory::ImageSaving, None, self.time_profiler_chan.clone(), || {
                    match opts::get().output_file.as_ref() {
                        Some(path) => match File::create(path) {
                            Ok(mut file) => {
                                let img = gl::draw_img(gl, rt_info, width, height);
                                let dynamic_image = DynamicImage::ImageRgb8(img);
                                if let Err(e) = dynamic_image.save(&mut file, ImageFormat::PNG) {
                                    error!("Failed to save {} ({}).", path, e);
                                }
                            },
                            Err(e) => error!("Failed to create {} ({}).", path, e),
                        },
                        None => error!("No file specified."),
                    }
                });
                None
            }
            #[cfg(not(feature = "gleam"))]
            _ => None,
        };

        // Perform the page flip. This will likely block for a while.
        self.window.present();

        self.last_composite_time = precise_time_ns();

        self.composition_request = CompositionRequest::NoCompositingNecessary;

        self.process_animations();
        self.waiting_for_results_of_scroll = false;

        Ok(rv)
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

    fn get_root_pipeline_id(&self) -> Option<PipelineId> {
        self.root_pipeline.as_ref().map(|pipeline| pipeline.id)
    }

    pub fn receive_messages(&mut self) -> bool {
        // Check for new messages coming from the other threads in the system.
        let mut compositor_messages = vec![];
        let mut found_recomposite_msg = false;
        while let Some(msg) = self.port.try_recv_compositor_msg() {
            match msg {
                Msg::Recomposite(_) if found_recomposite_msg => {}
                Msg::Recomposite(_) => {
                    found_recomposite_msg = true;
                    compositor_messages.push(msg)
                }
                _ => compositor_messages.push(msg),
            }
        }
        for msg in compositor_messages {
            if !self.handle_browser_message(msg) {
                return false
            }
        }
        true
    }

    pub fn perform_updates(&mut self) -> bool {
        if self.shutdown_state == ShutdownState::FinishedShuttingDown {
            return false;
        }

        // If a pinch-zoom happened recently, ask for tiles at the new resolution
        if self.zoom_action && precise_time_s() - self.zoom_time > 0.3 {
            self.zoom_action = false;
        }

        match self.composition_request {
            CompositionRequest::NoCompositingNecessary => {}
            CompositionRequest::CompositeNow(_) => {
                self.composite()
            }
        }

        if !self.pending_scroll_zoom_events.is_empty() && !self.waiting_for_results_of_scroll {
            self.process_pending_scroll_events()
        }
        self.shutdown_state != ShutdownState::FinishedShuttingDown
    }

    /// Repaints and recomposites synchronously. You must be careful when calling this, as if a
    /// paint is not scheduled the compositor will hang forever.
    ///
    /// This is used when resizing the window.
    pub fn repaint_synchronously(&mut self) {
        while self.shutdown_state != ShutdownState::ShuttingDown {
            let msg = self.port.recv_compositor_msg();
            let need_recomposite = match msg {
                Msg::Recomposite(_) => true,
                _ => false,
            };
            let keep_going = self.handle_browser_message(msg);
            if need_recomposite {
                self.composite();
                break
            }
            if !keep_going {
                break
            }
        }
    }

    pub fn pinch_zoom_level(&self) -> f32 {
        // TODO(gw): Access via WR.
        1.0
    }

    pub fn toggle_webrender_debug(&mut self, option: WebRenderDebugOption) {
        let mut flags = self.webrender.get_debug_flags();
        let flag = match option {
            WebRenderDebugOption::Profiler => {
                webrender::DebugFlags::PROFILER_DBG |
                webrender::DebugFlags::GPU_TIME_QUERIES |
                webrender::DebugFlags::GPU_SAMPLE_QUERIES
            }
            WebRenderDebugOption::TextureCacheDebug => {
                webrender::DebugFlags::TEXTURE_CACHE_DBG
            }
            WebRenderDebugOption::RenderTargetDebug => {
                webrender::DebugFlags::RENDER_TARGET_DBG
            }
        };
        flags.toggle(flag);
        self.webrender.set_debug_flags(flags);

        let mut txn = webrender_api::Transaction::new();
        txn.generate_frame();
        self.webrender_api.send_transaction(self.webrender_document, txn);
    }

    pub fn capture_webrender(&mut self) {
        match env::current_dir() {
            Ok(current_dir) => {
                let capture_id = now().to_timespec().sec.to_string();
                let capture_path = current_dir.join("capture_webrender").join(capture_id);
                let revision_file_path = capture_path.join("wr.txt");

                if let Err(err) = create_dir_all(&capture_path) {
                    eprintln!("Unable to create path '{:?}' for capture: {:?}", capture_path, err);
                    return
                }

                self.webrender_api.save_capture(capture_path, webrender_api::CaptureBits::all());

                match File::create(revision_file_path) {
                    Ok(mut file) => {
                        let revision = include!(concat!(env!("OUT_DIR"), "/webrender_revision.rs"));
                        if let Err(err) = write!(&mut file, "{}", revision) {
                            eprintln!("Unable to write webrender revision: {:?}", err)
                        }
                    }
                    Err(err) => eprintln!("Capture triggered, creating webrender revision info skipped: {:?}", err)
                }
            },
            Err(err) => eprintln!("Unable to locate path to save captures: {:?}", err)
        }
    }
}

/// Why we performed a composite. This is used for debugging.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompositingReason {
    /// We hit the delayed composition timeout. (See `delayed_composition.rs`.)
    DelayedCompositeTimeout,
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
    /// A new WebRender frame has arrived.
    NewWebRenderFrame,
    /// WebRender has processed a scroll event and has generated a new frame.
    NewWebRenderScrollFrame,
}
