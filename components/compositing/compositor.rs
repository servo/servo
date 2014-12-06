/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_layer::{CompositorData, CompositorLayer, DoesntWantScrollEvents};
use compositor_layer::WantsScrollEvents;
use compositor_task::{ChangePaintState, ChangeReadyState, CompositorEventListener};
use compositor_task::{CompositorReceiver, CompositorTask};
use compositor_task::{CreateOrUpdateDescendantLayer, CreateOrUpdateRootLayer, Exit};
use compositor_task::{FrameTreeUpdateMsg, GetGraphicsMetadata, InitialCompositorState};
use compositor_task::{LayerProperties, LoadComplete, Msg, Paint, PaintMsgDiscarded, PinchZoom};
use compositor_task::{Refresh, Resize, Scroll, ScrollFragmentPoint, ScrollTimeout, SendMouseEvent};
use compositor_task::{SendMouseMoveEvent, SetIds, SetLayerOrigin, ShutdownComplete};
use compositor_task::{SynchronousRefresh, Zoom};
use constellation::{SendableFrameTree, FrameTreeDiff};
use main_thread::MainThreadProxy;
use pipeline::CompositionPipeline;
use scrolling::ScrollingTimerProxy;
use windowing::{CompositorSupport, MouseWindowEvent, MouseWindowClickEvent};
use windowing::{MouseWindowMouseDownEvent, MouseWindowMouseUpEvent, SetReadyStateWindowEvent};
use windowing::{SetRenderStateWindowEvent};

use azure::azure_hl;
use std::cmp;
use std::num::Zero;
use geom::point::{Point2D, TypedPoint2D};
use geom::rect::{Rect, TypedRect};
use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use gfx::paint_task::{PaintChan, PaintMsg, RenderRequest, UnusedBufferMsg};
use layers::geometry::{DevicePixel, LayerPixel};
use layers::layers::{BufferRequest, Layer, LayerBufferSet};
use layers::platform::surface::NativeGraphicsMetadata;
use layers::rendergl;
use layers::rendergl::RenderContext;
use layers::scene::Scene;
use png;
use gleam::gl::types::{GLint, GLsizei};
use gleam::gl;
use script_traits::{ViewportMsg, ScriptControlChan};
use servo_msg::compositor_msg::{Blank, Epoch, FinishedLoading, IdleRenderState, LayerId};
use servo_msg::compositor_msg::{PaintingPaintState, PaintState, ReadyState, Scrollable};
use servo_msg::constellation_msg::{ConstellationChan, ExitMsg, PipelineId, ResizedWindowMsg};
use servo_msg::constellation_msg::{WindowSizeData};
use servo_util::geometry::{PagePx, ScreenPx, ViewportPx};
use servo_util::memory::MemoryProfilerChan;
use servo_util::opts;
use servo_util::time::{profile, TimeProfilerChan};
use servo_util::{memory, time};
use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::mem;
use std::path::Path;
use std::rc::Rc;
use std::slice::bytes::copy_memory;
use time as std_time;
use time::precise_time_ns;

pub struct IOCompositor {
    /// The channel on which messages can be sent to the main thread.
    main_thread_proxy: Box<MainThreadProxy + 'static>,

    /// The port on which we receive messages.
    port: CompositorReceiver,

    /// The render context.
    context: RenderContext,

    /// The native graphics metadata.
    native_graphics_metadata: Option<NativeGraphicsMetadata>,

    /// The compositor support object, which allows us to present the rendered contents to the
    /// screen.
    compositor_support: Box<CompositorSupport + Send>,

    /// The root pipeline.
    root_pipeline: Option<CompositionPipeline>,

    /// The canvas to paint a page.
    scene: Scene<CompositorData>,

    /// The application window size.
    window_size: TypedSize2D<DevicePixel, uint>,

    /// "Mobile-style" zoom that does not reflow the page.
    viewport_zoom: ScaleFactor<PagePx, ViewportPx, f32>,

    /// "Desktop-style" zoom that resizes the viewport to fit the window.
    /// See `ViewportPx` docs in util/geom.rs for details.
    page_zoom: ScaleFactor<ViewportPx, ScreenPx, f32>,

    /// The device pixel ratio for this window.
    hidpi_factor: ScaleFactor<ScreenPx, DevicePixel, f32>,

    /// A handle to the scrolling timer.
    scrolling_timer: ScrollingTimerProxy,

    /// Tracks whether we should composite this frame.
    composition_request: CompositionRequest,

    /// Tracks whether we are in the process of shutting down, or have shut down and should close
    /// the compositor.
    shutdown_state: ShutdownState,

    /// Tracks outstanding paint_msg's sent to the paint tasks.
    outstanding_paint_msgs: uint,

    /// Tracks the last composite time.
    last_composite_time: u64,

    /// Tracks whether the zoom action has happened recently.
    zoom_action: bool,

    /// The time of the last zoom action has started.
    zoom_time: f64,

    /// Current display/reflow status of each pipeline.
    ready_states: HashMap<PipelineId, ReadyState>,

    /// Current paint status of each pipeline.
    paint_states: HashMap<PipelineId, PaintState>,

    /// Whether the page being rendered has loaded completely.
    /// Differs from ReadyState because we can finish loading (ready)
    /// many times for a single page.
    got_load_complete_message: bool,

    /// Whether we have gotten a `SetIds` message.
    got_set_ids_message: bool,

    /// The channel on which messages can be sent to the constellation.
    constellation_chan: ConstellationChan,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: TimeProfilerChan,

    /// The channel on which messages can be sent to the memory profiler.
    memory_profiler_chan: MemoryProfilerChan,

    /// Pending scroll to fragment event, if any.
    fragment_point: Option<Point2D<f32>>,

    /// Pending scroll events.
    pending_scroll_events: Vec<ScrollEvent>,

    /// The sender corresponding to the pending synchronous refresh event, if any. When a paint
    /// comes in, a message is sent on this sender.
    pending_synchronous_refresh_sender: Option<Sender<()>>,
}

#[deriving(PartialEq)]
enum CompositionRequest {
    NoCompositingNecessary,
    CompositeOnScrollTimeout(u64),
    CompositeNow,
}

#[deriving(PartialEq, Show)]
enum ShutdownState {
    NotShuttingDown,
    ShuttingDown,
    FinishedShuttingDown,
}

struct HitTestResult {
    layer: Rc<Layer<CompositorData>>,
    point: TypedPoint2D<LayerPixel, f32>,
}

impl IOCompositor {
    #[allow(unused_variables)]
    fn new(mut state: InitialCompositorState) -> IOCompositor {
        // Get the native graphics system set up. This must be done first.
        state.compositor_support.initialize();

        // Create an initial layer tree.
        //
        // TODO: There should be no initial layer tree until the painter creates one from the
        // display list. This is only here because we don't have that logic in the painter yet.
        let window_size = state.window_framebuffer_size;
        let hidpi_factor = state.hidpi_factor;
        let context = CompositorTask::create_graphics_context(state.native_graphics_metadata
                                                                   .as_ref()
                                                                   .unwrap());

        let show_debug_borders = opts::get().show_debug_borders;
        IOCompositor {
            main_thread_proxy: state.main_thread_proxy,
            port: state.receiver,
            context: rendergl::RenderContext::new(context, show_debug_borders),
            compositor_support: state.compositor_support,
            native_graphics_metadata: state.native_graphics_metadata,
            root_pipeline: None,
            scene: Scene::new(Rect {
                origin: Zero::zero(),
                size: window_size.as_f32(),
            }),
            window_size: window_size,
            hidpi_factor: hidpi_factor,
            scrolling_timer: ScrollingTimerProxy::new(state.sender),
            composition_request: NoCompositingNecessary,
            shutdown_state: NotShuttingDown,
            page_zoom: ScaleFactor(1.0),
            viewport_zoom: ScaleFactor(1.0),
            zoom_action: false,
            zoom_time: 0f64,
            ready_states: HashMap::new(),
            paint_states: HashMap::new(),
            got_load_complete_message: false,
            got_set_ids_message: false,
            constellation_chan: state.constellation_sender,
            time_profiler_chan: state.time_profiler_sender,
            memory_profiler_chan: state.memory_profiler_sender,
            fragment_point: None,
            outstanding_paint_msgs: 0,
            last_composite_time: 0,
            pending_scroll_events: Vec::new(),
            pending_synchronous_refresh_sender: None,
        }
    }

    pub fn create(state: InitialCompositorState) -> IOCompositor {
        let mut compositor = IOCompositor::new(state);

        // Set the size of the root layer.
        compositor.update_zoom_transform();

        // Tell the constellation about the initial window size.
        compositor.send_window_size();

        compositor
    }

    fn handle_browser_message(&mut self, msg: Msg) {
        match (msg, self.shutdown_state) {
            (_, FinishedShuttingDown) =>
                panic!("compositor shouldn't be handling messages after shutting down"),

            (Exit(chan), _) => {
                // The constellation might have already shut down if it was the one that initiated
                // this exit request. So use `send_opt` to avoid a panic.
                debug!("shutting down the constellation");
                let ConstellationChan(ref con_chan) = self.constellation_chan;
                drop(con_chan.send_opt(ExitMsg));

                chan.send(());
                debug!("constellation completed shutdown");
                self.shutdown_state = FinishedShuttingDown;
            }

            (ShutdownComplete, _) => {
                // XXX(pcwalton)
            }

            (ChangeReadyState(pipeline_id, ready_state), NotShuttingDown) => {
                self.change_ready_state(pipeline_id, ready_state);
            }

            (ChangePaintState(pipeline_id, paint_state), NotShuttingDown) => {
                self.change_paint_state(pipeline_id, paint_state);
            }

            (PaintMsgDiscarded, NotShuttingDown) => {
                self.remove_outstanding_paint_msg();
            }

            (SetIds(frame_tree, response_chan, new_constellation_chan), NotShuttingDown) => {
                self.set_frame_tree(&frame_tree,
                                    response_chan,
                                    new_constellation_chan);
                self.send_viewport_rects_for_all_layers();
            }

            (FrameTreeUpdateMsg(frame_tree_diff, response_channel), NotShuttingDown) => {
                self.update_frame_tree(&frame_tree_diff);
                response_channel.send(());
            }

            (CreateOrUpdateRootLayer(layer_properties), NotShuttingDown) => {
                self.create_or_update_root_layer(layer_properties);
            }

            (CreateOrUpdateDescendantLayer(layer_properties), NotShuttingDown) => {
                self.create_or_update_descendant_layer(layer_properties);
            }

            (GetGraphicsMetadata(chan), NotShuttingDown) => {
                chan.send(self.native_graphics_metadata.clone());
            }

            (SetLayerOrigin(pipeline_id, layer_id, origin), NotShuttingDown) => {
                self.set_layer_origin(pipeline_id, layer_id, origin);
            }

            (Paint(pipeline_id, epoch, replies), NotShuttingDown) => {
                for (layer_id, new_layer_buffer_set) in replies.into_iter() {
                    self.paint(pipeline_id, layer_id, new_layer_buffer_set, epoch);
                }
                self.remove_outstanding_paint_msg();

                // If someone is waiting on a synchronous refresh, composite now and let them know.
                match mem::replace(&mut self.pending_synchronous_refresh_sender, None) {
                    None => {}
                    Some(sender) => {
                        self.composite();
                        sender.send(())
                    }
                }
            }

            (ScrollFragmentPoint(pipeline_id, layer_id, point), NotShuttingDown) => {
                self.scroll_fragment_to_point(pipeline_id, layer_id, point);
            }

            (LoadComplete(..), NotShuttingDown) => {
                self.got_load_complete_message = true;

                // If we're painting in headless mode, schedule a recomposite.
                if opts::get().output_file.is_some() {
                    self.composite_if_necessary();
                }
            }

            (ScrollTimeout(timestamp), NotShuttingDown) => {
                debug!("scroll timeout, drawing unpainted content!");
                match self.composition_request {
                    CompositeOnScrollTimeout(this_timestamp) if timestamp == this_timestamp => {
                        self.composition_request = CompositeNow
                    }
                    _ => {}
                }
            }

            (Refresh, NotShuttingDown) => {
                self.composite_if_necessary()
            }

            (SynchronousRefresh(sender), NotShuttingDown) => {
                debug_assert!(self.pending_synchronous_refresh_sender.is_none());
                self.pending_synchronous_refresh_sender = Some(sender)
            }

            (Resize(new_size, new_hidpi_factor), NotShuttingDown) => {
                self.resize(new_size, new_hidpi_factor)
            }

            (Scroll(delta, cursor), NotShuttingDown) => self.scroll(delta, cursor),
            (SendMouseEvent(mouse_window_event), NotShuttingDown) => {
                self.handle_mouse_event(mouse_window_event)
            }
            (SendMouseMoveEvent(point), NotShuttingDown) => self.handle_mouse_move_event(point),
            (PinchZoom(magnification), NotShuttingDown) => self.pinch_zoom(magnification),
            (Zoom(magnification), NotShuttingDown) => self.zoom(magnification),

            // When we are shutting_down, we need to avoid performing operations
            // such as Paint that may crash because we have begun tearing down
            // the rest of our resources.
            (_, ShuttingDown) => { }
        }
    }

    fn change_ready_state(&mut self, pipeline_id: PipelineId, ready_state: ReadyState) {
        match self.ready_states.entry(pipeline_id) {
            Occupied(entry) => {
                *entry.into_mut() = ready_state;
            }
            Vacant(entry) => {
                entry.set(ready_state);
            }
        }

        let ready_state = self.get_earliest_pipeline_ready_state();
        self.main_thread_proxy.send(SetReadyStateWindowEvent(ready_state));

        // If we're painting in headless mode, schedule a recomposite.
        if opts::get().output_file.is_some() {
            self.composite_if_necessary()
        }
    }

    fn get_earliest_pipeline_ready_state(&self) -> ReadyState {
        if self.ready_states.len() == 0 {
            return Blank;
        }
        return self.ready_states.values().fold(FinishedLoading, |a, &b| cmp::min(a, b));

    }

    fn change_paint_state(&mut self, pipeline_id: PipelineId, paint_state: PaintState) {
        match self.paint_states.entry(pipeline_id) {
            Occupied(entry) => {
                *entry.into_mut() = paint_state;
            }
            Vacant(entry) => {
                entry.set(paint_state);
            }
        }

        self.main_thread_proxy.send(SetPaintStateWindowEvent(paint_state));
    }

    fn all_pipelines_in_idle_paint_state(&self) -> bool {
        if self.ready_states.len() == 0 {
            return false;
        }
        return self.paint_states.values().all(|&value| value == IdlePaintState);
    }

    fn has_paint_msg_tracking(&self) -> bool {
        // only track PaintMsg's if the compositor outputs to a file.
        opts::get().output_file.is_some()
    }

    fn has_outstanding_paint_msgs(&self) -> bool {
        self.has_paint_msg_tracking() && self.outstanding_paint_msgs > 0
    }

    fn add_outstanding_paint_msg(&mut self, count: uint) {
        // return early if not tracking paint_msg's
        if !self.has_paint_msg_tracking() {
            return;
        }
        debug!("add_outstanding_paint_msg {}", self.outstanding_paint_msgs);
        self.outstanding_paint_msgs += count;
    }

    fn remove_outstanding_paint_msg(&mut self) {
        if !self.has_paint_msg_tracking() {
            return;
        }
        if self.outstanding_paint_msgs > 0 {
            self.outstanding_paint_msgs -= 1;
        } else {
            debug!("too many repaint msgs completed");
        }
    }

    fn set_frame_tree(&mut self,
                      frame_tree: &SendableFrameTree,
                      response_chan: Sender<()>,
                      new_constellation_chan: ConstellationChan) {
        response_chan.send(());

        self.root_pipeline = Some(frame_tree.pipeline.clone());

        // If we have an old root layer, release all old tiles before replacing it.
        match self.scene.root {
            Some(ref mut layer) => layer.clear_all_tiles(),
            None => { }
        }
        self.scene.root = Some(self.create_frame_tree_root_layers(frame_tree, None));
        self.scene.set_root_layer_size(self.window_size.as_f32());

        // Initialize the new constellation channel by sending it the root window size.
        self.constellation_chan = new_constellation_chan;
        self.send_window_size();

        self.got_set_ids_message = true;
        self.composite_if_necessary();
    }

    fn create_frame_tree_root_layers(&mut self,
                                     frame_tree: &SendableFrameTree,
                                     frame_rect: Option<TypedRect<PagePx, f32>>)
                                     -> Rc<Layer<CompositorData>> {
        // Initialize the ReadyState and PaintState for this pipeline.
        self.ready_states.insert(frame_tree.pipeline.id, Blank);
        self.paint_states.insert(frame_tree.pipeline.id, PaintingPaintState);

        let root_layer = create_root_layer_for_pipeline_and_rect(&frame_tree.pipeline, frame_rect);
        for kid in frame_tree.children.iter() {
            root_layer.add_child(self.create_frame_tree_root_layers(&kid.frame_tree, kid.rect));
        }
        return root_layer;
    }

    fn update_frame_tree(&mut self, frame_tree_diff: &FrameTreeDiff) {
        let parent_layer = self.find_pipeline_root_layer(frame_tree_diff.parent_pipeline.id);
        parent_layer.add_child(
            create_root_layer_for_pipeline_and_rect(&frame_tree_diff.pipeline,
                                                    frame_tree_diff.rect));
    }

    fn find_pipeline_root_layer(&self, pipeline_id: PipelineId) -> Rc<Layer<CompositorData>> {
        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, LayerId::null()) {
            Some(ref layer) => layer.clone(),
            None => panic!("Tried to create or update layer for unknown pipeline"),
        }
    }

    fn update_layer_if_exists(&mut self, properties: LayerProperties) -> bool {
        match self.find_layer_with_pipeline_and_layer_id(properties.pipeline_id, properties.id) {
            Some(existing_layer) => {
                existing_layer.update_layer(properties);
                true
            }
            None => false,
        }
    }

    fn create_or_update_root_layer(&mut self, layer_properties: LayerProperties) {
        let need_new_root_layer = !self.update_layer_if_exists(layer_properties);
        if need_new_root_layer {
            let root_layer = self.find_pipeline_root_layer(layer_properties.pipeline_id);
            root_layer.update_layer_except_size(layer_properties);

            let root_layer_pipeline = root_layer.extra_data.borrow().pipeline.clone();
            let first_child = CompositorData::new_layer(root_layer_pipeline.clone(),
                                                        layer_properties,
                                                        DoesntWantScrollEvents,
                                                        opts::get().tile_size);

            // Add the first child / base layer to the front of the child list, so that
            // child iframe layers are painted on top of the base layer. These iframe
            // layers were added previously when creating the layer tree skeleton in
            // create_frame_tree_root_layers.
            root_layer.children().insert(0, first_child);
        }

        self.scroll_layer_to_fragment_point_if_necessary(layer_properties.pipeline_id,
                                                         layer_properties.id);
        self.send_buffer_requests_for_all_layers();
    }

    fn create_or_update_descendant_layer(&mut self, layer_properties: LayerProperties) {
        if !self.update_layer_if_exists(layer_properties) {
            self.create_descendant_layer(layer_properties);
        }
        self.scroll_layer_to_fragment_point_if_necessary(layer_properties.pipeline_id,
                                                         layer_properties.id);
        self.send_buffer_requests_for_all_layers();
    }

    fn create_descendant_layer(&self, layer_properties: LayerProperties) {
        let root_layer = self.find_pipeline_root_layer(layer_properties.pipeline_id);
        let root_layer_pipeline = root_layer.extra_data.borrow().pipeline.clone();
        let new_layer = CompositorData::new_layer(root_layer_pipeline,
                                                  layer_properties,
                                                  DoesntWantScrollEvents,
                                                  root_layer.tile_size);
        root_layer.add_child(new_layer);
    }

    fn send_window_size(&self) {
        let dppx = self.page_zoom * self.device_pixels_per_screen_px();
        let initial_viewport = self.window_size.as_f32() / dppx;
        let visible_viewport = initial_viewport / self.viewport_zoom;

        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(ResizedWindowMsg(WindowSizeData {
            device_pixel_ratio: dppx,
            initial_viewport: initial_viewport,
            visible_viewport: visible_viewport,
        }));
    }

    pub fn move_layer(&self,
                      pipeline_id: PipelineId,
                      layer_id: LayerId,
                      origin: TypedPoint2D<LayerPixel, f32>)
                      -> bool {
        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, layer_id) {
            Some(ref layer) => {
                if layer.extra_data.borrow().wants_scroll_events == WantsScrollEvents {
                    layer.clamp_scroll_offset_and_scroll_layer(TypedPoint2D(0f32, 0f32) - origin);
                }
                true
            }
            None => false,
        }
    }

    fn scroll_layer_to_fragment_point_if_necessary(&mut self,
                                                   pipeline_id: PipelineId,
                                                   layer_id: LayerId) {
        match self.fragment_point.take() {
            Some(point) => {
                if !self.move_layer(pipeline_id, layer_id, Point2D::from_untyped(&point)) {
                    panic!("Compositor: Tried to scroll to fragment with unknown layer.");
                }

                self.start_scrolling_timer_if_necessary();
            }
            None => {}
        }
    }

    fn start_scrolling_timer_if_necessary(&mut self) {
        match self.composition_request {
            CompositeNow | CompositeOnScrollTimeout(_) => return,
            NoCompositingNecessary => {}
        }

        let timestamp = precise_time_ns();
        self.scrolling_timer.scroll_event_processed(timestamp);
        self.composition_request = CompositeOnScrollTimeout(timestamp);
    }

    fn set_layer_origin(&mut self,
                        pipeline_id: PipelineId,
                        layer_id: LayerId,
                        new_origin: Point2D<f32>) {
        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, layer_id) {
            Some(ref layer) => {
                layer.bounds.borrow_mut().origin = Point2D::from_untyped(&new_origin)
            }
            None => panic!("Compositor received SetLayerOrigin for nonexistent layer"),
        };

        self.send_buffer_requests_for_all_layers();
    }

    fn paint(&mut self,
             pipeline_id: PipelineId,
             layer_id: LayerId,
             new_layer_buffer_set: Box<LayerBufferSet>,
             epoch: Epoch) {
        debug!("compositor received new frame at size {}x{}",
               self.window_size.width.get(),
               self.window_size.height.get());

        // From now on, if we destroy the buffers, they will leak.
        let mut new_layer_buffer_set = new_layer_buffer_set;
        new_layer_buffer_set.mark_will_leak();

        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, layer_id) {
            Some(ref layer) => {
                // FIXME(pcwalton): This is going to cause problems with inconsistent frames since
                // we only composite one layer at a time.
                assert!(layer.add_buffers(new_layer_buffer_set, epoch));
                self.composite_if_necessary();
            }
            None => {
                // FIXME: This may potentially be triggered by a race condition where a
                // buffers are being painted but the layer is removed before painting
                // completes.
                panic!("compositor given paint command for non-existent layer");
            }
        }
    }

    fn scroll_fragment_to_point(&mut self,
                                pipeline_id: PipelineId,
                                layer_id: LayerId,
                                point: Point2D<f32>) {
        if self.move_layer(pipeline_id, layer_id, Point2D::from_untyped(&point)) {
            if self.send_buffer_requests_for_all_layers() {
                self.start_scrolling_timer_if_necessary();
            }
        } else {
            self.fragment_point = Some(point);
        }
    }

    fn device_pixels_per_screen_px(&self) -> ScaleFactor<ScreenPx, DevicePixel, f32> {
        match opts::get().device_pixels_per_px {
            Some(device_pixels_per_px) => device_pixels_per_px,
            None => match opts::get().output_file {
                Some(_) => ScaleFactor(1.0),
                None => self.hidpi_factor
            }
        }
    }

    fn device_pixels_per_page_px(&self) -> ScaleFactor<PagePx, DevicePixel, f32> {
        self.viewport_zoom * self.page_zoom * self.device_pixels_per_screen_px()
    }

    fn update_zoom_transform(&mut self) {
        let scale = self.device_pixels_per_page_px();
        self.scene.scale = ScaleFactor(scale.get());

        // We need to set the size of the root layer again, since the window size
        // has changed in unscaled layer pixels.
        self.scene.set_root_layer_size(self.window_size.as_f32());
    }

    fn convert_buffer_requests_to_pipeline_requests_map(&self,
                                                        requests: Vec<(Rc<Layer<CompositorData>>,
                                                                       Vec<BufferRequest>)>) ->
                                                        HashMap<PipelineId, (PaintChan,
                                                                             Vec<PaintRequest>)> {
        let scale = self.device_pixels_per_page_px();
        let mut results:
            HashMap<PipelineId, (PaintChan, Vec<PaintRequest>)> = HashMap::new();

        for (layer, mut layer_requests) in requests.into_iter() {
            let &(_, ref mut vec) =
                match results.entry(layer.extra_data.borrow().pipeline.id) {
                    Occupied(mut entry) => {
                        *entry.get_mut() =
                            (layer.extra_data.borrow().pipeline.paint_chan.clone(), vec!());
                        entry.into_mut()
                    }
                    Vacant(entry) => {
                        entry.set((layer.extra_data.borrow().pipeline.paint_chan.clone(), vec!()))
                    }
                };

            // All the BufferRequests are in layer/device coordinates, but the paint task
            // wants to know the page coordinates. We scale them before sending them.
            for request in layer_requests.iter_mut() {
                request.page_rect = request.page_rect / scale.get();
            }

            vec.push(PaintRequest {
                buffer_requests: layer_requests,
                scale: scale.get(),
                layer_id: layer.extra_data.borrow().id,
                epoch: layer.extra_data.borrow().epoch,
            });
        }

        return results;
    }

    fn send_back_unused_buffers(&mut self) {
        match self.root_pipeline {
            Some(ref pipeline) => {
                let unused_buffers = self.scene.collect_unused_buffers();
                if unused_buffers.len() != 0 {
                    let message = UnusedBufferMsg(unused_buffers);
                    let _ = pipeline.paint_chan.send_opt(message);
                }
            },
            None => {}
        }
    }

    fn send_viewport_rect_for_layer(&self, layer: Rc<Layer<CompositorData>>) {
        if layer.extra_data.borrow().id == LayerId::null() {
            let layer_rect = Rect(-layer.extra_data.borrow().scroll_offset.to_untyped(),
                                  layer.bounds.borrow().size.to_untyped());
            let pipeline = &layer.extra_data.borrow().pipeline;
            let ScriptControlChan(ref chan) = pipeline.script_chan;
            chan.send(ViewportMsg(pipeline.id.clone(), layer_rect));
        }

        for kid in layer.children().iter() {
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
        let mut layers_and_requests = Vec::new();
        self.scene.get_buffer_requests(&mut layers_and_requests,
                                       Rect(TypedPoint2D(0f32, 0f32), self.window_size.as_f32()));

        // Return unused tiles first, so that they can be reused by any new BufferRequests.
        self.send_back_unused_buffers();

        if layers_and_requests.len() == 0 {
            return false;
        }

        // We want to batch requests for each pipeline to avoid race conditions
        // when handling the resulting BufferRequest responses.
        let pipeline_requests =
            self.convert_buffer_requests_to_pipeline_requests_map(layers_and_requests);

        let mut num_paint_msgs_sent = 0;
        for (_pipeline_id, (chan, requests)) in pipeline_requests.into_iter() {
            num_paint_msgs_sent += 1;
            let _ = chan.send_opt(PaintMsg(requests));
        }

        self.add_outstanding_paint_msg(num_paint_msgs_sent);
        true
    }

    fn is_ready_to_paint_image_output(&self) -> bool {
        if !self.got_load_complete_message {
            return false;
        }

        if self.get_earliest_pipeline_ready_state() != FinishedLoading {
            return false;
        }

        if self.has_outstanding_paint_msgs() {
            return false;
        }

        if !self.all_pipelines_in_idle_paint_state() {
            return false;
        }

        if !self.got_set_ids_message {
            return false;
        }

        return true;
    }

    fn composite(&mut self) {
        let output_image = opts::get().output_file.is_some() &&
                            self.is_ready_to_paint_image_output();

        let mut framebuffer_ids = vec!();
        let mut texture_ids = vec!();
        let (width, height) = (self.window_size.width.get(), self.window_size.height.get());

        if output_image {
            framebuffer_ids = gl::gen_framebuffers(1);
            gl::bind_framebuffer(gl::FRAMEBUFFER, framebuffer_ids[0]);

            texture_ids = gl::gen_textures(1);
            gl::bind_texture(gl::TEXTURE_2D, texture_ids[0]);

            gl::tex_image_2d(gl::TEXTURE_2D, 0, gl::RGB as GLint, width as GLsizei,
                             height as GLsizei, 0, gl::RGB, gl::UNSIGNED_BYTE, None);
            gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);

            gl::framebuffer_texture_2d(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D,
                                       texture_ids[0], 0);

            gl::bind_texture(gl::TEXTURE_2D, 0);
        }

        profile(time::CompositingCategory, None, self.time_profiler_chan.clone(), || {
            debug!("compositor: compositing");
            // Adjust the layer dimensions as necessary to correspond to the size of the window.
            self.scene.viewport = Rect {
                origin: Zero::zero(),
                size: self.window_size.as_f32(),
            };
            // paint the scene.
            match self.scene.root {
                Some(ref layer) => {
                    rendergl::render_scene(layer.clone(), self.context, &self.scene);
                }
                None => {}
            }
        });

        if output_image {
            let path =
                from_str::<Path>(opts::get().output_file.as_ref().unwrap().as_slice()).unwrap();
            let mut pixels = gl::read_pixels(0, 0,
                                             width as gl::GLsizei,
                                             height as gl::GLsizei,
                                             gl::RGB, gl::UNSIGNED_BYTE);

            gl::bind_framebuffer(gl::FRAMEBUFFER, 0);

            gl::delete_buffers(texture_ids.as_slice());
            gl::delete_frame_buffers(framebuffer_ids.as_slice());

            // flip image vertically (texture is upside down)
            let orig_pixels = pixels.clone();
            let stride = width * 3;
            for y in range(0, height) {
                let dst_start = y * stride;
                let src_start = (height - y - 1) * stride;
                let src_slice = orig_pixels.slice(src_start, src_start + stride);
                copy_memory(pixels.slice_mut(dst_start, dst_start + stride),
                            src_slice.slice_to(stride));
            }
            let mut img = png::Image {
                width: width as u32,
                height: height as u32,
                pixels: png::RGB8(pixels),
            };
            let res = png::store_png(&mut img, &path);
            assert!(res.is_ok());

            debug!("shutting down the constellation after generating an output file");
            let ConstellationChan(ref chan) = self.constellation_chan;
            chan.send(ExitMsg);
            self.shutdown_state = ShuttingDown;
        }

        // Perform the page flip. This will likely block for a while.
        self.compositor_support.present();

        self.last_composite_time = precise_time_ns();

        self.composition_request = NoCompositingNecessary;
        self.process_pending_scroll_events();
    }

    fn composite_if_necessary(&mut self) {
        if self.composition_request == NoCompositingNecessary {
            self.composition_request = CompositeNow
        }
    }

    fn find_topmost_layer_at_point_for_layer(&self,
                                             layer: Rc<Layer<CompositorData>>,
                                             point: TypedPoint2D<LayerPixel, f32>)
                                             -> Option<HitTestResult> {
        let child_point = point - layer.bounds.borrow().origin;
        for child in layer.children().iter().rev() {
            let result = self.find_topmost_layer_at_point_for_layer(child.clone(), child_point);
            if result.is_some() {
                return result;
            }
        }

        let point = point - *layer.content_offset.borrow();
        if !layer.bounds.borrow().contains(&point) {
            return None;
        }

        return Some(HitTestResult { layer: layer, point: point });
    }

    fn find_topmost_layer_at_point(&self,
                                   point: TypedPoint2D<LayerPixel, f32>)
                                   -> Option<HitTestResult> {
        match self.scene.root {
            Some(ref layer) => self.find_topmost_layer_at_point_for_layer(layer.clone(), point),
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

    fn resize(&mut self,
              new_size: TypedSize2D<DevicePixel,uint>,
              new_hidpi_factor: ScaleFactor<ScreenPx,DevicePixel,f32>) {
        // A size change could also mean a resolution change.
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

    fn scroll(&mut self,
              delta: TypedPoint2D<DevicePixel,f32>,
              cursor: TypedPoint2D<DevicePixel,i32>) {
        self.pending_scroll_events.push(ScrollEvent {
            delta: delta,
            cursor: cursor,
        });

        self.composite_if_necessary();
    }

    fn zoom(&mut self, magnification: f32) {
        self.page_zoom = ScaleFactor((self.page_zoom.get() * magnification).max(1.0));
        self.update_zoom_transform();
        self.send_window_size();
    }

    fn pinch_zoom(&mut self, magnification: f32) {
        self.zoom_action = true;
        self.zoom_time = std_time::precise_time_s();
        let old_viewport_zoom = self.viewport_zoom;

        self.viewport_zoom = ScaleFactor((self.viewport_zoom.get() * magnification).max(1.0));
        let viewport_zoom = self.viewport_zoom;

        self.update_zoom_transform();

        // Scroll as needed
        let window_size = self.window_size.as_f32();
        let page_delta: TypedPoint2D<DevicePixel, f32> = TypedPoint2D(
            window_size.width.get() * (viewport_zoom.inv() - old_viewport_zoom.inv()).get() * 0.5,
            window_size.height.get() * (viewport_zoom.inv() - old_viewport_zoom.inv()).get() * 0.5);

        let cursor = TypedPoint2D(-1, -1);  // Make sure this hits the base layer.
        self.scroll(page_delta, cursor);

        self.send_viewport_rects_for_all_layers();
        self.composite_if_necessary();
    }

    fn handle_mouse_event(&mut self, mouse_window_event: MouseWindowEvent) {
        let point = match mouse_window_event {
            MouseWindowClickEvent(_, p) => p,
            MouseWindowMouseDownEvent(_, p) => p,
            MouseWindowMouseUpEvent(_, p) => p,
        };
        match self.find_topmost_layer_at_point(point / self.scene.scale) {
            Some(result) => result.layer.send_mouse_event(mouse_window_event, result.point),
            None => {},
        }
    }

    fn handle_mouse_move_event(&mut self, cursor: TypedPoint2D<DevicePixel,f32>) {
        match self.find_topmost_layer_at_point(cursor / self.scene.scale) {
            Some(result) => result.layer.send_mouse_move_event(result.point),
            None => {},
        }
    }

    fn process_pending_scroll_events(&mut self) {
        let had_scroll_events = self.pending_scroll_events.len() > 0;
        for scroll_event in mem::replace(&mut self.pending_scroll_events, Vec::new()).into_iter() {
            let delta = scroll_event.delta / self.scene.scale;
            let cursor = scroll_event.cursor.as_f32() / self.scene.scale;

            match self.scene.root {
                Some(ref mut layer) => {
                    layer.handle_scroll_event(delta, cursor);
                }
                None => {}
            }

            self.start_scrolling_timer_if_necessary();
            self.send_buffer_requests_for_all_layers();
        }

        if had_scroll_events {
            self.send_viewport_rects_for_all_layers()
        }
    }
}

fn find_layer_with_pipeline_and_layer_id_for_layer(layer: Rc<Layer<CompositorData>>,
                                                   pipeline_id: PipelineId,
                                                   layer_id: LayerId)
                                                   -> Option<Rc<Layer<CompositorData>>> {
    if layer.extra_data.borrow().pipeline.id == pipeline_id &&
       layer.extra_data.borrow().id == layer_id {
        return Some(layer);
    }

    for kid in layer.children().iter() {
        let result = find_layer_with_pipeline_and_layer_id_for_layer(kid.clone(),
                                                                     pipeline_id,
                                                                     layer_id);
        if result.is_some() {
            return result;
        }
    }

    return None;
}

fn create_root_layer_for_pipeline_and_rect(pipeline: &CompositionPipeline,
                                           frame_rect: Option<TypedRect<PagePx, f32>>)
                                           -> Rc<Layer<CompositorData>> {
    let layer_properties = LayerProperties {
        pipeline_id: pipeline.id,
        epoch: Epoch(0),
        id: LayerId::null(),
        rect: Rect::zero(),
        background_color: azure_hl::Color::new(0., 0., 0., 0.),
        scroll_policy: Scrollable,
    };

    let root_layer = CompositorData::new_layer(pipeline.clone(),
                                               layer_properties,
                                               WantsScrollEvents,
                                               opts::get().tile_size);

    match frame_rect {
        Some(ref frame_rect) => {
            *root_layer.masks_to_bounds.borrow_mut() = true;

            let frame_rect = frame_rect.to_untyped();
            *root_layer.bounds.borrow_mut() = Rect::from_untyped(&frame_rect);
        }
        None => {}
    }

    return root_layer;
}

impl CompositorEventListener for IOCompositor {
    fn handle_events(&mut self) -> bool {
        // Wait for a message and handle it. This will block.
        let msg = self.port.recv();
        self.handle_browser_message(msg);

        // Drain all messages. (If we don't do this, we can only process one message per frame due
        // to the buffer swap below -- obviously not good!)
        loop {
            match self.port.try_recv() {
                Err(_) => break,
                Ok(msg) => self.handle_browser_message(msg),
            }
        }

        // Exit now if we need to. We do this now because we risk panicking due to sending to a
        // dead script task if we proceed.
        //
        // FIXME(pcwalton): Is this still true? I don't think we send to the script task below.
        if self.shutdown_state == FinishedShuttingDown {
            debug!("Exiting the compositor due to a request from script.");
            return false;
        }

        // If a pinch-zoom happened recently, ask for tiles at the new resolution.
        if self.zoom_action && std_time::precise_time_s() - self.zoom_time > 0.3 {
            self.zoom_action = false;
            self.scene.mark_layer_contents_as_changed_recursively();
            self.send_buffer_requests_for_all_layers();
        }

        // Composite if requested. This may block on a buffer swap.
        match self.composition_request {
            NoCompositingNecessary | CompositeOnScrollTimeout(_) => {}
            CompositeNow => self.composite(),
        }

        self.shutdown_state != FinishedShuttingDown
    }

    fn shutdown(&mut self) {
        // Clear out the compositor layers so that painting tasks can destroy the buffers.
        match self.scene.root {
            None => {}
            Some(ref layer) => layer.forget_all_tiles(),
        }

        // Drain compositor port, sometimes messages contain channels that are blocking
        // another task from finishing (i.e. SetIds)
        while self.port.try_recv().is_ok() {}

        // Tell the profiler, memory profiler, and scrolling timer to shut down.
        let TimeProfilerChan(ref time_profiler_chan) = self.time_profiler_chan;
        time_profiler_chan.send(time::ExitMsg);

        let MemoryProfilerChan(ref memory_profiler_chan) = self.memory_profiler_chan;
        memory_profiler_chan.send(memory::ExitMsg);

        self.scrolling_timer.shutdown();
    }
}

/// A record of a stored scroll event.
pub struct ScrollEvent {
    /// The amount that the user scrolled.
    delta: TypedPoint2D<DevicePixel,f32>,
    /// The position that the scroll event was directed to.
    cursor: TypedPoint2D<DevicePixel,i32>,
}

