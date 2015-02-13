/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_layer::{CompositorData, CompositorLayer, WantsScrollEventsFlag};
use compositor_task::{CompositorEventListener, CompositorProxy, CompositorReceiver};
use compositor_task::{CompositorTask, LayerProperties, Msg};
use constellation::{FrameId, SendableFrameTree};
use pipeline::CompositionPipeline;
use scrolling::ScrollingTimerProxy;
use windowing;
use windowing::{MouseWindowEvent, WindowEvent, WindowMethods, WindowNavigateMsg};

use std::cmp;
use std::mem;
use geom::point::{Point2D, TypedPoint2D};
use geom::rect::{Rect, TypedRect};
use geom::size::TypedSize2D;
use geom::scale_factor::ScaleFactor;
use gfx::color;
use gfx::paint_task::Msg as PaintMsg;
use gfx::paint_task::PaintRequest;
use layers::geometry::{DevicePixel, LayerPixel};
use layers::layers::{BufferRequest, Layer, LayerBuffer, LayerBufferSet};
use layers::rendergl;
use layers::rendergl::RenderContext;
use layers::scene::Scene;
use png;
use gleam::gl::types::{GLint, GLsizei};
use gleam::gl;
use script_traits::{ConstellationControlMsg, ScriptControlChan};
use msg::compositor_msg::{Epoch, LayerId};
use msg::compositor_msg::{ReadyState, PaintState, ScrollPolicy};
use msg::constellation_msg::{ConstellationChan, NavigationDirection};
use msg::constellation_msg::Msg as ConstellationMsg;
use msg::constellation_msg::{Key, KeyModifiers, KeyState, LoadData};
use msg::constellation_msg::{PipelineId, WindowSizeData};
use util::geometry::{PagePx, ScreenPx, ViewportPx};
use util::memory::MemoryProfilerChan;
use util::opts;
use util::time::{TimeProfilerCategory, profile, TimeProfilerChan};
use util::{memory, time};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::old_path::Path;
use std::num::Float;
use std::rc::Rc;
use std::slice::bytes::copy_memory;
use std::sync::mpsc::Sender;
use time::{precise_time_ns, precise_time_s};
use url::Url;

/// NB: Never block on the constellation, because sometimes the constellation blocks on us.
pub struct IOCompositor<Window: WindowMethods> {
    /// The application window.
    window: Rc<Window>,

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

    /// Whether the page being rendered has loaded completely.
    /// Differs from ReadyState because we can finish loading (ready)
    /// many times for a single page.
    got_load_complete_message: bool,

    /// Whether we have received a `SetFrameTree` message.
    got_set_frame_tree_message: bool,

    /// The channel on which messages can be sent to the constellation.
    constellation_chan: ConstellationChan,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: TimeProfilerChan,

    /// The channel on which messages can be sent to the memory profiler.
    memory_profiler_chan: MemoryProfilerChan,

    /// Pending scroll to fragment event, if any
    fragment_point: Option<Point2D<f32>>,

    /// Pending scroll events.
    pending_scroll_events: Vec<ScrollEvent>,
}

pub struct ScrollEvent {
    delta: TypedPoint2D<DevicePixel,f32>,
    cursor: TypedPoint2D<DevicePixel,i32>,
}

#[derive(PartialEq)]
enum CompositionRequest {
    NoCompositingNecessary,
    CompositeOnScrollTimeout(u64),
    CompositeNow,
}

#[derive(Copy, PartialEq, Debug)]
enum ShutdownState {
    NotShuttingDown,
    ShuttingDown,
    FinishedShuttingDown,
}

struct HitTestResult {
    layer: Rc<Layer<CompositorData>>,
    point: TypedPoint2D<LayerPixel, f32>,
}

struct PipelineDetails {
    /// The pipeline associated with this PipelineDetails object.
    pipeline: Option<CompositionPipeline>,

    /// The status of this pipeline's ScriptTask.
    ready_state: ReadyState,

    /// The status of this pipeline's PaintTask.
    paint_state: PaintState,
}

impl PipelineDetails {
    fn new() -> PipelineDetails {
        PipelineDetails {
            pipeline: None,
            ready_state: ReadyState::Blank,
            paint_state: PaintState::Painting,
        }
    }
}

impl<Window: WindowMethods> IOCompositor<Window> {
    fn new(window: Rc<Window>,
           sender: Box<CompositorProxy+Send>,
           receiver: Box<CompositorReceiver>,
           constellation_chan: ConstellationChan,
           time_profiler_chan: TimeProfilerChan,
           memory_profiler_chan: MemoryProfilerChan)
           -> IOCompositor<Window> {
        // Create an initial layer tree.
        //
        // TODO: There should be no initial layer tree until the painter creates one from the
        // display list. This is only here because we don't have that logic in the painter yet.
        let window_size = window.framebuffer_size();
        let hidpi_factor = window.hidpi_factor();
        IOCompositor {
            window: window,
            port: receiver,
            context: None,
            root_pipeline: None,
            pipeline_details: HashMap::new(),
            scene: Scene::new(Rect {
                origin: Point2D::zero(),
                size: window_size.as_f32(),
            }),
            window_size: window_size,
            hidpi_factor: hidpi_factor,
            scrolling_timer: ScrollingTimerProxy::new(sender),
            composition_request: CompositionRequest::NoCompositingNecessary,
            pending_scroll_events: Vec::new(),
            shutdown_state: ShutdownState::NotShuttingDown,
            page_zoom: ScaleFactor(1.0),
            viewport_zoom: ScaleFactor(1.0),
            zoom_action: false,
            zoom_time: 0f64,
            got_load_complete_message: false,
            got_set_frame_tree_message: false,
            constellation_chan: constellation_chan,
            time_profiler_chan: time_profiler_chan,
            memory_profiler_chan: memory_profiler_chan,
            fragment_point: None,
            outstanding_paint_msgs: 0,
            last_composite_time: 0,
        }
    }

    pub fn create(window: Rc<Window>,
                  sender: Box<CompositorProxy+Send>,
                  receiver: Box<CompositorReceiver>,
                  constellation_chan: ConstellationChan,
                  time_profiler_chan: TimeProfilerChan,
                  memory_profiler_chan: MemoryProfilerChan)
                  -> IOCompositor<Window> {
        let mut compositor = IOCompositor::new(window,
                                               sender,
                                               receiver,
                                               constellation_chan,
                                               time_profiler_chan,
                                               memory_profiler_chan);

        // Set the size of the root layer.
        compositor.update_zoom_transform();

        // Tell the constellation about the initial window size.
        compositor.send_window_size();

        compositor
    }

    fn handle_browser_message(&mut self, msg: Msg) -> bool {
        match (msg, self.shutdown_state) {
            (_, ShutdownState::FinishedShuttingDown) =>
                panic!("compositor shouldn't be handling messages after shutting down"),

            (Msg::Exit(chan), _) => {
                debug!("shutting down the constellation");
                let ConstellationChan(ref con_chan) = self.constellation_chan;
                con_chan.send(ConstellationMsg::Exit).unwrap();
                chan.send(()).unwrap();
                self.shutdown_state = ShutdownState::ShuttingDown;
            }

            (Msg::ShutdownComplete, _) => {
                debug!("constellation completed shutdown");
                self.shutdown_state = ShutdownState::FinishedShuttingDown;
                return false;
            }

            (Msg::ChangeReadyState(pipeline_id, ready_state), ShutdownState::NotShuttingDown) => {
                self.change_ready_state(pipeline_id, ready_state);
            }

            (Msg::ChangePaintState(pipeline_id, paint_state), ShutdownState::NotShuttingDown) => {
                self.change_paint_state(pipeline_id, paint_state);
            }

            (Msg::ChangePageTitle(pipeline_id, title), ShutdownState::NotShuttingDown) => {
                self.change_page_title(pipeline_id, title);
            }

            (Msg::ChangePageLoadData(frame_id, load_data), ShutdownState::NotShuttingDown) => {
                self.change_page_load_data(frame_id, load_data);
            }

            (Msg::PaintMsgDiscarded, ShutdownState::NotShuttingDown) => {
                self.remove_outstanding_paint_msg();
            }

            (Msg::SetFrameTree(frame_tree, response_chan, new_constellation_chan),
             ShutdownState::NotShuttingDown) => {
                self.set_frame_tree(&frame_tree,
                                    response_chan,
                                    new_constellation_chan);
                self.send_viewport_rects_for_all_layers();
            }

            (Msg::ChangeLayerPipelineAndRemoveChildren(old_pipeline, new_pipeline, response_channel),
             ShutdownState::NotShuttingDown) => {
                self.handle_change_layer_pipeline_and_remove_children(old_pipeline, new_pipeline);
                response_channel.send(()).unwrap();
            }

            (Msg::CreateRootLayerForPipeline(parent_pipeline, pipeline, rect, response_channel),
             ShutdownState::NotShuttingDown) => {
                self.handle_create_root_layer_for_pipeline(parent_pipeline, pipeline, rect);
                response_channel.send(()).unwrap();
            }

            (Msg::CreateOrUpdateBaseLayer(layer_properties), ShutdownState::NotShuttingDown) => {
                self.create_or_update_base_layer(layer_properties);
            }

            (Msg::CreateOrUpdateDescendantLayer(layer_properties),
             ShutdownState::NotShuttingDown) => {
                self.create_or_update_descendant_layer(layer_properties);
            }

            (Msg::GetGraphicsMetadata(chan), ShutdownState::NotShuttingDown) => {
                chan.send(Some(self.window.native_metadata())).unwrap();
            }

            (Msg::SetLayerOrigin(pipeline_id, layer_id, origin),
             ShutdownState::NotShuttingDown) => {
                self.set_layer_origin(pipeline_id, layer_id, origin);
            }

            (Msg::AssignPaintedBuffers(pipeline_id, epoch, replies), ShutdownState::NotShuttingDown) => {
                for (layer_id, new_layer_buffer_set) in replies.into_iter() {
                    self.assign_painted_buffers(pipeline_id, layer_id, new_layer_buffer_set, epoch);
                }
                self.remove_outstanding_paint_msg();
            }

            (Msg::ScrollFragmentPoint(pipeline_id, layer_id, point),
             ShutdownState::NotShuttingDown) => {
                self.scroll_fragment_to_point(pipeline_id, layer_id, point);
            }

            (Msg::LoadComplete, ShutdownState::NotShuttingDown) => {
                self.got_load_complete_message = true;

                // If we're painting in headless mode, schedule a recomposite.
                if opts::get().output_file.is_some() {
                    self.composite_if_necessary();
                }

                // Inform the embedder that the load has finished.
                //
                // TODO(pcwalton): Specify which frame's load completed.
                self.window.load_end();
            }

            (Msg::ScrollTimeout(timestamp), ShutdownState::NotShuttingDown) => {
                debug!("scroll timeout, drawing unpainted content!");
                match self.composition_request {
                    CompositionRequest::CompositeOnScrollTimeout(this_timestamp) => {
                        if timestamp == this_timestamp {
                            self.composition_request = CompositionRequest::CompositeNow
                        }
                    }
                    _ => {}
                }
            }

            (Msg::KeyEvent(key, state, modified), ShutdownState::NotShuttingDown) => {
                if state == KeyState::Pressed {
                    self.window.handle_key(key, modified);
                }
            }

            (Msg::SetCursor(cursor), ShutdownState::NotShuttingDown) => {
                self.window.set_cursor(cursor)
            }

            (Msg::PaintTaskExited(pipeline_id), ShutdownState::NotShuttingDown) => {
                if self.pipeline_details.remove(&pipeline_id).is_none() {
                    panic!("Saw PaintTaskExited message from an unknown pipeline!");
                }
            }

            // When we are shutting_down, we need to avoid performing operations
            // such as Paint that may crash because we have begun tearing down
            // the rest of our resources.
            (_, ShutdownState::ShuttingDown) => { }
        }

        true
    }

    fn change_ready_state(&mut self, pipeline_id: PipelineId, ready_state: ReadyState) {
        self.get_or_create_pipeline_details(pipeline_id).ready_state = ready_state;
        self.window.set_ready_state(self.get_earliest_pipeline_ready_state());

        // If we're painting in headless mode, schedule a recomposite.
        if opts::get().output_file.is_some() {
            self.composite_if_necessary()
        }
    }

    fn get_earliest_pipeline_ready_state(&self) -> ReadyState {
        if self.pipeline_details.len() == 0 {
            return ReadyState::Blank;
        }
        return self.pipeline_details.values().fold(ReadyState::FinishedLoading,
                                                   |v, ref details| {
                                                       cmp::min(v, details.ready_state)
                                                   });
    }

    fn change_paint_state(&mut self, pipeline_id: PipelineId, paint_state: PaintState) {
        self.get_or_create_pipeline_details(pipeline_id).paint_state = paint_state;
        self.window.set_paint_state(paint_state);
    }

    pub fn get_or_create_pipeline_details<'a>(&'a mut self,
                                              pipeline_id: PipelineId)
                                              -> &'a mut PipelineDetails {
        if !self.pipeline_details.contains_key(&pipeline_id) {
            self.pipeline_details.insert(pipeline_id, PipelineDetails::new());
        }
        return self.pipeline_details.get_mut(&pipeline_id).unwrap();
    }

    pub fn get_pipeline<'a>(&'a self, pipeline_id: PipelineId) -> &'a CompositionPipeline {
        match self.pipeline_details.get(&pipeline_id) {
            Some(ref details) => {
                match details.pipeline {
                    Some(ref pipeline) => pipeline,
                    None => panic!("Compositor layer has an unitialized pipeline ({:?}).",
                                   pipeline_id),

                }
            }
            None => panic!("Compositor layer has an unknown pipeline ({:?}).", pipeline_id),
        }
    }

    fn change_page_title(&mut self, _: PipelineId, title: Option<String>) {
        self.window.set_page_title(title);
    }

    fn change_page_load_data(&mut self, _: FrameId, load_data: LoadData) {
        self.window.set_page_load_data(load_data);
    }

    fn all_pipelines_in_idle_paint_state(&self) -> bool {
        if self.pipeline_details.len() == 0 {
            return false;
        }
        return self.pipeline_details.values().all(|ref details| {
                                                     details.paint_state == PaintState::Idle
                                                  });
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
        debug!("add_outstanding_paint_msg {:?}", self.outstanding_paint_msgs);
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
        response_chan.send(()).unwrap();

        self.root_pipeline = Some(frame_tree.pipeline.clone());

        // If we have an old root layer, release all old tiles before replacing it.
        match self.scene.root {
            Some(ref layer) => layer.clear_all_tiles(self),
            None => { }
        }
        self.scene.root = Some(self.create_frame_tree_root_layers(frame_tree, None));
        self.scene.set_root_layer_size(self.window_size.as_f32());

        // Initialize the new constellation channel by sending it the root window size.
        self.constellation_chan = new_constellation_chan;
        self.send_window_size();

        self.got_set_frame_tree_message = true;
        self.composite_if_necessary();
    }

    fn create_root_layer_for_pipeline_and_rect(&mut self,
                                               pipeline: &CompositionPipeline,
                                               frame_rect: Option<TypedRect<PagePx, f32>>)
                                               -> Rc<Layer<CompositorData>> {
        let layer_properties = LayerProperties {
            pipeline_id: pipeline.id,
            epoch: Epoch(0),
            id: LayerId::null(),
            rect: Rect::zero(),
            background_color: color::transparent_black(),
            scroll_policy: ScrollPolicy::Scrollable,
        };

        let root_layer = CompositorData::new_layer(layer_properties,
                                                   WantsScrollEventsFlag::WantsScrollEvents,
                                                   opts::get().tile_size);

        self.get_or_create_pipeline_details(pipeline.id).pipeline = Some(pipeline.clone());

        // All root layers mask to bounds.
        *root_layer.masks_to_bounds.borrow_mut() = true;

        if let Some(ref frame_rect) = frame_rect {
            let frame_rect = frame_rect.to_untyped();
            *root_layer.bounds.borrow_mut() = Rect::from_untyped(&frame_rect);
        }

        return root_layer;
    }

    fn create_frame_tree_root_layers(&mut self,
                                     frame_tree: &SendableFrameTree,
                                     frame_rect: Option<TypedRect<PagePx, f32>>)
                                     -> Rc<Layer<CompositorData>> {
        let root_layer = self.create_root_layer_for_pipeline_and_rect(&frame_tree.pipeline,
                                                                      frame_rect);
        for kid in frame_tree.children.iter() {
            root_layer.add_child(self.create_frame_tree_root_layers(&kid.frame_tree, kid.rect));
        }
        return root_layer;
    }

    fn handle_change_layer_pipeline_and_remove_children(&mut self,
                                                        old_pipeline: CompositionPipeline,
                                                        new_pipeline: CompositionPipeline) {
        let root_layer = match self.find_pipeline_root_layer(old_pipeline.id) {
            Some(root_layer) => root_layer,
            None => {
                debug!("Ignoring ChangeLayerPipelineAndRemoveChildren message \
                        for pipeline ({:?}) shutting down.",
                       old_pipeline.id);
                return;
            }
        };

        root_layer.clear_all_tiles(self);
        root_layer.children().clear();

        debug_assert!(root_layer.extra_data.borrow().pipeline_id == old_pipeline.id);
        root_layer.extra_data.borrow_mut().pipeline_id = new_pipeline.id;

        let new_pipeline_id = new_pipeline.id;
        self.get_or_create_pipeline_details(new_pipeline_id).pipeline = Some(new_pipeline);
    }

    fn handle_create_root_layer_for_pipeline(&mut self,
                                             parent_pipeline: CompositionPipeline,
                                             new_pipeline: CompositionPipeline,
                                             frame_rect: Option<TypedRect<PagePx, f32>>) {
        let root_layer = self.create_root_layer_for_pipeline_and_rect(&new_pipeline, frame_rect);
        match frame_rect {
            Some(ref frame_rect) => {
                *root_layer.masks_to_bounds.borrow_mut() = true;

                let frame_rect = frame_rect.to_untyped();
                *root_layer.bounds.borrow_mut() = Rect::from_untyped(&frame_rect);
            }
            None => {}
        }

        let pipeline_id = parent_pipeline.id;
        let parent_layer = match self.find_pipeline_root_layer(pipeline_id) {
            Some(root_layer) => root_layer,
            None => {
                debug!("Ignoring FrameTreeUpdate message for pipeline ({:?}) \
                        shutting down.",
                       pipeline_id);
                return;
            }
        };
        parent_layer.add_child(root_layer);
    }

    fn find_pipeline_root_layer(&self, pipeline_id: PipelineId) -> Option<Rc<Layer<CompositorData>>> {
        if !self.pipeline_details.contains_key(&pipeline_id) {
            panic!("Tried to create or update layer for unknown pipeline")
        }
        self.find_layer_with_pipeline_and_layer_id(pipeline_id, LayerId::null())
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

    fn create_or_update_base_layer(&mut self, layer_properties: LayerProperties) {
        let pipeline_id = layer_properties.pipeline_id;
        let root_layer = match self.find_pipeline_root_layer(pipeline_id) {
            Some(root_layer) => root_layer,
            None => {
                debug!("Ignoring CreateOrUpdateBaseLayer message for pipeline \
                        ({:?}) shutting down.",
                       pipeline_id);
                return;
            }
        };

        let need_new_base_layer = !self.update_layer_if_exists(layer_properties);
        if need_new_base_layer {
            root_layer.update_layer_except_bounds(layer_properties);

            let base_layer = CompositorData::new_layer(
                layer_properties,
                WantsScrollEventsFlag::DoesntWantScrollEvents,
                opts::get().tile_size);

            // Add the base layer to the front of the child list, so that child
            // iframe layers are painted on top of the base layer. These iframe
            // layers were added previously when creating the layer tree
            // skeleton in create_frame_tree_root_layers.
            root_layer.children().insert(0, base_layer);
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
        let root_layer = match self.find_pipeline_root_layer(layer_properties.pipeline_id) {
            Some(root_layer) => root_layer,
            None => return, // This pipeline is in the process of shutting down.
        };

        let new_layer = CompositorData::new_layer(layer_properties,
                                                  WantsScrollEventsFlag::DoesntWantScrollEvents,
                                                  root_layer.tile_size);
        root_layer.add_child(new_layer);
    }

    fn send_window_size(&self) {
        let dppx = self.page_zoom * self.device_pixels_per_screen_px();
        let initial_viewport = self.window_size.as_f32() / dppx;
        let visible_viewport = initial_viewport / self.viewport_zoom;

        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(ConstellationMsg::ResizedWindow(WindowSizeData {
            device_pixel_ratio: dppx,
            initial_viewport: initial_viewport,
            visible_viewport: visible_viewport,
        })).unwrap()
    }

    pub fn move_layer(&self,
                      pipeline_id: PipelineId,
                      layer_id: LayerId,
                      origin: TypedPoint2D<LayerPixel, f32>)
                      -> bool {
        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, layer_id) {
            Some(ref layer) => {
                if layer.wants_scroll_events() == WantsScrollEventsFlag::WantsScrollEvents {
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
            CompositionRequest::CompositeNow | CompositionRequest::CompositeOnScrollTimeout(_) =>
                return,
            CompositionRequest::NoCompositingNecessary => {}
        }

        let timestamp = precise_time_ns();
        self.scrolling_timer.scroll_event_processed(timestamp);
        self.composition_request = CompositionRequest::CompositeOnScrollTimeout(timestamp);
    }

    fn set_layer_origin(&mut self,
                        pipeline_id: PipelineId,
                        layer_id: LayerId,
                        new_origin: Point2D<f32>) {
        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, layer_id) {
            Some(ref layer) => {
                layer.bounds.borrow_mut().origin = Point2D::from_untyped(&new_origin)
            }
            None => panic!("Compositor received SetLayerOrigin for nonexistent \
                            layer: {:?}", pipeline_id),
        };

        self.send_buffer_requests_for_all_layers();
    }

    fn assign_painted_buffers(&mut self,
                              pipeline_id: PipelineId,
                              layer_id: LayerId,
                              new_layer_buffer_set: Box<LayerBufferSet>,
                              epoch: Epoch) {
        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, layer_id) {
            Some(layer) => {
                self.assign_painted_buffers_to_layer(layer, new_layer_buffer_set, epoch);
                return;
            }
            None => {}
        }

        let pipeline = self.get_pipeline(pipeline_id);
        let message = PaintMsg::UnusedBuffer(new_layer_buffer_set.buffers);
        let _ = pipeline.paint_chan.send(message);
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
        assert!(layer.add_buffers(self, new_layer_buffer_set, epoch));
        self.composite_if_necessary();
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

    fn handle_window_message(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Idle => {}

            WindowEvent::Refresh => {
                self.composite();
            }

            WindowEvent::InitializeCompositing => {
                self.initialize_compositing();
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

            WindowEvent::Scroll(delta, cursor) => {
                self.on_scroll_window_event(delta, cursor);
            }

            WindowEvent::Zoom(magnification) => {
                self.on_zoom_window_event(magnification);
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
                debug!("shutting down the constellation for WindowEvent::Quit");
                let ConstellationChan(ref chan) = self.constellation_chan;
                chan.send(ConstellationMsg::Exit).unwrap();
                self.shutdown_state = ShutdownState::ShuttingDown;
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
        let root_pipeline_id = match self.scene.root {
            Some(ref layer) => layer.get_pipeline_id(),
            None => panic!("Compositor: Received WindowEvent::LoadUrl without initialized compositor \
                           layers"),
        };

        let msg = ConstellationMsg::LoadUrl(root_pipeline_id,
            LoadData::new(Url::parse(url_string.as_slice()).unwrap()));
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(msg).unwrap()
    }

    fn on_mouse_window_event_class(&self, mouse_window_event: MouseWindowEvent) {
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

    fn on_mouse_window_move_event_class(&self, cursor: TypedPoint2D<DevicePixel, f32>) {
        match self.find_topmost_layer_at_point(cursor / self.scene.scale) {
            Some(result) => result.layer.send_mouse_move_event(self, result.point),
            None => {},
        }
    }

    fn on_scroll_window_event(&mut self,
                              delta: TypedPoint2D<DevicePixel, f32>,
                              cursor: TypedPoint2D<DevicePixel, i32>) {
        self.pending_scroll_events.push(ScrollEvent {
            delta: delta,
            cursor: cursor,
        });

        self.composite_if_necessary();
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
            self.send_viewport_rects_for_all_layers();
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

    fn on_zoom_window_event(&mut self, magnification: f32) {
        self.page_zoom = ScaleFactor((self.page_zoom.get() * magnification).max(1.0));
        self.update_zoom_transform();
        self.send_window_size();
    }

    // TODO(pcwalton): I think this should go through the same queuing as scroll events do.
    fn on_pinch_zoom_window_event(&mut self, magnification: f32) {
        self.zoom_action = true;
        self.zoom_time = precise_time_s();
        let old_viewport_zoom = self.viewport_zoom;

        self.viewport_zoom = ScaleFactor((self.viewport_zoom.get() * magnification).max(1.0));
        let viewport_zoom = self.viewport_zoom;

        self.update_zoom_transform();

        // Scroll as needed
        let window_size = self.window_size.as_f32();
        let page_delta: TypedPoint2D<LayerPixel, f32> = TypedPoint2D(
            window_size.width.get() * (viewport_zoom.inv() - old_viewport_zoom.inv()).get() * 0.5,
            window_size.height.get() * (viewport_zoom.inv() - old_viewport_zoom.inv()).get() * 0.5);

        let cursor = TypedPoint2D(-1f32, -1f32);  // Make sure this hits the base layer.
        match self.scene.root {
            Some(ref mut layer) => {
                layer.handle_scroll_event(page_delta, cursor);
            }
            None => { }
        }

        self.send_viewport_rects_for_all_layers();
        self.composite_if_necessary();
    }

    fn on_navigation_window_event(&self, direction: WindowNavigateMsg) {
        let direction = match direction {
            windowing::WindowNavigateMsg::Forward => NavigationDirection::Forward,
            windowing::WindowNavigateMsg::Back => NavigationDirection::Back,
        };
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(ConstellationMsg::Navigate(direction)).unwrap()
    }

    fn on_key_event(&self, key: Key, state: KeyState, modifiers: KeyModifiers) {
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(ConstellationMsg::KeyEvent(key, state, modifiers)).unwrap()
    }

    fn convert_buffer_requests_to_pipeline_requests_map(&self,
                                                        requests: Vec<(Rc<Layer<CompositorData>>,
                                                                       Vec<BufferRequest>)>)
                                                        -> HashMap<PipelineId, Vec<PaintRequest>> {
        let scale = self.device_pixels_per_page_px();
        let mut results: HashMap<PipelineId, Vec<PaintRequest>> = HashMap::new();

        for (layer, mut layer_requests) in requests.into_iter() {
            let vec = match results.entry(layer.get_pipeline_id()) {
                Occupied(mut entry) => {
                    *entry.get_mut() = Vec::new();
                    entry.into_mut()
                }
                Vacant(entry) => {
                    entry.insert(Vec::new())
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

    fn send_back_unused_buffers(&mut self,
                                unused_buffers: Vec<(Rc<Layer<CompositorData>>,
                                                     Vec<Box<LayerBuffer>>)>) {
        for (layer, buffers) in unused_buffers.into_iter() {
            let pipeline = self.get_pipeline(layer.get_pipeline_id());
            let _ = pipeline.paint_chan.send_opt(PaintMsg::UnusedBuffer(buffers));
        }
    }

    fn send_viewport_rect_for_layer(&self, layer: Rc<Layer<CompositorData>>) {
        if layer.extra_data.borrow().id == LayerId::null() {
            let layer_rect = Rect(-layer.extra_data.borrow().scroll_offset.to_untyped(),
                                  layer.bounds.borrow().size.to_untyped());
            let pipeline = self.get_pipeline(layer.get_pipeline_id());
            let ScriptControlChan(ref chan) = pipeline.script_chan;
            chan.send(ConstellationControlMsg::Viewport(pipeline.id.clone(), layer_rect)).unwrap();
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
        let mut unused_buffers = Vec::new();
        self.scene.get_buffer_requests(&mut layers_and_requests, &mut unused_buffers);

        // Return unused tiles first, so that they can be reused by any new BufferRequests.
        self.send_back_unused_buffers(unused_buffers);

        if layers_and_requests.len() == 0 {
            return false;
        }

        // We want to batch requests for each pipeline to avoid race conditions
        // when handling the resulting BufferRequest responses.
        let pipeline_requests =
            self.convert_buffer_requests_to_pipeline_requests_map(layers_and_requests);

        let mut num_paint_msgs_sent = 0;
        for (pipeline_id, requests) in pipeline_requests.into_iter() {
            num_paint_msgs_sent += 1;
            let _ = self.get_pipeline(pipeline_id).paint_chan.send(PaintMsg::Paint(requests));
        }

        self.add_outstanding_paint_msg(num_paint_msgs_sent);
        true
    }

    fn is_ready_to_paint_image_output(&self) -> bool {
        if !self.got_load_complete_message {
            return false;
        }

        if self.get_earliest_pipeline_ready_state() != ReadyState::FinishedLoading {
            return false;
        }

        if self.has_outstanding_paint_msgs() {
            return false;
        }

        if !self.all_pipelines_in_idle_paint_state() {
            return false;
        }

        if !self.got_set_frame_tree_message {
            return false;
        }

        return true;
    }

    fn composite(&mut self) {
        if !self.window.prepare_for_composite() {
            return
        }

        let output_image = opts::get().output_file.is_some() &&
                            self.is_ready_to_paint_image_output();

        let mut framebuffer_ids = vec!();
        let mut texture_ids = vec!();
        let (width, height) = (self.window_size.width.get() as usize, self.window_size.height.get() as usize);

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

        profile(TimeProfilerCategory::Compositing, None, self.time_profiler_chan.clone(), || {
            debug!("compositor: compositing");
            // Adjust the layer dimensions as necessary to correspond to the size of the window.
            self.scene.viewport = Rect {
                origin: Point2D::zero(),
                size: self.window_size.as_f32(),
            };
            // paint the scene.
            match self.scene.root {
                Some(ref layer) => {
                    match self.context {
                        None => {
                            debug!("compositor: not compositing because context not yet set up")
                        }
                        Some(context) => {
                            rendergl::render_scene(layer.clone(), context, &self.scene);
                        }
                    }
                }
                None => {}
            }
        });

        if output_image {
            let path: Path =
                opts::get().output_file.as_ref().unwrap().as_slice().parse().unwrap();
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
                let src_slice = &orig_pixels[src_start .. src_start + stride];
                copy_memory(&mut pixels[dst_start .. dst_start + stride],
                            &src_slice[..stride]);
            }
            let mut img = png::Image {
                width: width as u32,
                height: height as u32,
                pixels: png::PixelsByColorType::RGB8(pixels),
            };
            let res = png::store_png(&mut img, &path);
            assert!(res.is_ok());

            debug!("shutting down the constellation after generating an output file");
            let ConstellationChan(ref chan) = self.constellation_chan;
            chan.send(ConstellationMsg::Exit).unwrap();
            self.shutdown_state = ShutdownState::ShuttingDown;
        }

        // Perform the page flip. This will likely block for a while.
        self.window.present();

        self.last_composite_time = precise_time_ns();

        self.composition_request = CompositionRequest::NoCompositingNecessary;
        self.process_pending_scroll_events();
    }

    fn composite_if_necessary(&mut self) {
        if self.composition_request == CompositionRequest::NoCompositingNecessary {
            self.composition_request = CompositionRequest::CompositeNow
        }
    }

    fn initialize_compositing(&mut self) {
        let context = CompositorTask::create_graphics_context(&self.window.native_metadata());
        let show_debug_borders = opts::get().show_debug_borders;
        self.context = Some(rendergl::RenderContext::new(context, show_debug_borders))
    }

    fn find_topmost_layer_at_point_for_layer(&self,
                                             layer: Rc<Layer<CompositorData>>,
                                             point: TypedPoint2D<LayerPixel, f32>,
                                             clip_rect: &TypedRect<LayerPixel, f32>)
                                             -> Option<HitTestResult> {
        let layer_bounds = *layer.bounds.borrow();
        let masks_to_bounds = *layer.masks_to_bounds.borrow();
        if layer_bounds.is_empty() && masks_to_bounds {
            return None;
        }

        let clipped_layer_bounds = match clip_rect.intersection(&layer_bounds) {
            Some(rect) => rect,
            None => return None,
        };

        let clip_rect_for_children = if masks_to_bounds {
            Rect(Point2D::zero(), clipped_layer_bounds.size)
        } else {
            clipped_layer_bounds.translate(&clip_rect.origin)
        };

        let child_point = point - layer_bounds.origin;
        for child in layer.children().iter().rev() {
            // Translate the clip rect into the child's coordinate system.
            let clip_rect_for_child =
                clip_rect_for_children.translate(&-*child.content_offset.borrow());
            let result = self.find_topmost_layer_at_point_for_layer(child.clone(),
                                                                    child_point,
                                                                    &clip_rect_for_child);
            if result.is_some() {
                return result;
            }
        }

        let point = point - *layer.content_offset.borrow();
        if !clipped_layer_bounds.contains(&point) {
            return None;
        }

        return Some(HitTestResult { layer: layer, point: point });
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
}

fn find_layer_with_pipeline_and_layer_id_for_layer(layer: Rc<Layer<CompositorData>>,
                                                   pipeline_id: PipelineId,
                                                   layer_id: LayerId)
                                                   -> Option<Rc<Layer<CompositorData>>> {
    if layer.extra_data.borrow().pipeline_id == pipeline_id &&
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

impl<Window> CompositorEventListener for IOCompositor<Window> where Window: WindowMethods {
    fn handle_event(&mut self, msg: WindowEvent) -> bool {
        // Check for new messages coming from the other tasks in the system.
        loop {
            match self.port.try_recv_compositor_msg() {
                None => break,
                Some(msg) => {
                    if !self.handle_browser_message(msg) {
                        break
                    }
                }
            }
        }

        if self.shutdown_state == ShutdownState::FinishedShuttingDown {
            // We have exited the compositor and passing window
            // messages to script may crash.
            debug!("Exiting the compositor due to a request from script.");
            return false;
        }

        // Handle the message coming from the windowing system.
        self.handle_window_message(msg);

        // If a pinch-zoom happened recently, ask for tiles at the new resolution
        if self.zoom_action && precise_time_s() - self.zoom_time > 0.3 {
            self.zoom_action = false;
            self.scene.mark_layer_contents_as_changed_recursively();
            self.send_buffer_requests_for_all_layers();
        }

        match self.composition_request {
            CompositionRequest::NoCompositingNecessary | CompositionRequest::CompositeOnScrollTimeout(_) => {}
            CompositionRequest::CompositeNow => self.composite(),
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

    fn shutdown(&mut self) {
        // Clear out the compositor layers so that painting tasks can destroy the buffers.
        match self.scene.root {
            None => {}
            Some(ref layer) => layer.forget_all_tiles(),
        }

        // Drain compositor port, sometimes messages contain channels that are blocking
        // another task from finishing (i.e. SetFrameTree).
        while self.port.try_recv_compositor_msg().is_some() {}

        // Tell the profiler, memory profiler, and scrolling timer to shut down.
        let TimeProfilerChan(ref time_profiler_chan) = self.time_profiler_chan;
        time_profiler_chan.send(time::TimeProfilerMsg::Exit).unwrap();

        let MemoryProfilerChan(ref memory_profiler_chan) = self.memory_profiler_chan;
        memory_profiler_chan.send(memory::MemoryProfilerMsg::Exit).unwrap();

        self.scrolling_timer.shutdown();
    }

    fn pinch_zoom_level(&self) -> f32 {
        self.viewport_zoom.get() as f32
    }

    fn get_title_for_main_frame(&self) {
        let root_pipeline_id = match self.root_pipeline {
            None => return,
            Some(ref root_pipeline) => root_pipeline.id,
        };
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(ConstellationMsg::GetPipelineTitle(root_pipeline_id)).unwrap();
    }
}
