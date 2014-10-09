/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_data::{CompositorData, DoesntWantScrollEvents, WantsScrollEvents};
use compositor_task::{Msg, CompositorTask, Exit, ChangeReadyState, SetIds, LayerProperties};
use compositor_task::{GetGraphicsMetadata, CreateOrUpdateRootLayer, CreateOrUpdateDescendantLayer};
use compositor_task::{SetLayerOrigin, Paint, ScrollFragmentPoint, LoadComplete};
use compositor_task::{ShutdownComplete, ChangeRenderState, RenderMsgDiscarded};
use constellation::SendableFrameTree;
use events;
use events::ScrollPositionChanged;
use pipeline::CompositionPipeline;
use platform::{Application, Window};
use windowing;
use windowing::{FinishedWindowEvent, IdleWindowEvent, LoadUrlWindowEvent, MouseWindowClickEvent};
use windowing::{MouseWindowEvent, MouseWindowEventClass, MouseWindowMouseDownEvent};
use windowing::{MouseWindowMouseUpEvent, MouseWindowMoveEventClass, NavigationWindowEvent};
use windowing::{QuitWindowEvent, RefreshWindowEvent, ResizeWindowEvent, ScrollWindowEvent};
use windowing::{WindowEvent, WindowMethods, WindowNavigateMsg, ZoomWindowEvent};
use windowing::PinchZoomWindowEvent;

use azure::azure_hl;
use std::cmp;
use std::num::Zero;
use std::time::duration::Duration;
use geom::point::{Point2D, TypedPoint2D};
use geom::rect::{Rect, TypedRect};
use geom::size::TypedSize2D;
use geom::scale_factor::ScaleFactor;
use gfx::render_task::{RenderChan, RenderMsg, RenderRequest, UnusedBufferMsg};
use layers::geometry::{DevicePixel, LayerPixel};
use layers::layers::{BufferRequest, Layer, LayerBufferSet};
use layers::rendergl;
use layers::rendergl::RenderContext;
use layers::scene::Scene;
use opengles::gl2;
use png;
use servo_msg::compositor_msg::{Blank, Epoch, FinishedLoading, IdleRenderState, LayerId};
use servo_msg::compositor_msg::{ReadyState, RenderingRenderState, RenderState, Scrollable};
use servo_msg::constellation_msg::{ConstellationChan, ExitMsg, LoadUrlMsg, NavigateMsg};
use servo_msg::constellation_msg::{LoadData, PipelineId, ResizedWindowMsg, WindowSizeData};
use servo_msg::constellation_msg;
use servo_util::geometry::{PagePx, ScreenPx, ViewportPx};
use servo_util::memory::MemoryProfilerChan;
use servo_util::opts::Opts;
use servo_util::time::{profile, TimeProfilerChan};
use servo_util::{memory, time};
use std::io::timer::sleep;
use std::collections::hashmap::HashMap;
use std::path::Path;
use std::rc::Rc;
use time::precise_time_s;
use url::Url;


pub struct IOCompositor {
    /// The application window.
    window: Rc<Window>,

    /// The port on which we receive messages.
    port: Receiver<Msg>,

    /// The render context.
    context: RenderContext,

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

    /// Tracks whether the renderer has finished its first rendering
    composite_ready: bool,

    /// Tracks whether we are in the process of shutting down, or have shut down and should close
    /// the compositor.
    shutdown_state: ShutdownState,

    /// Tracks whether we need to re-composite a page.
    recomposite: bool,

    /// Tracks outstanding render_msg's sent to the render tasks.
    outstanding_render_msgs: uint,

    /// Tracks whether the zoom action has happend recently.
    zoom_action: bool,

    /// The time of the last zoom action has started.
    zoom_time: f64,

    /// Current display/reflow status of each pipeline.
    ready_states: HashMap<PipelineId, ReadyState>,

    /// Current render status of each pipeline.
    render_states: HashMap<PipelineId, RenderState>,

    /// Whether the page being rendered has loaded completely.
    /// Differs from ReadyState because we can finish loading (ready)
    /// many times for a single page.
    got_load_complete_message: bool,

    /// The command line option flags.
    opts: Opts,

    /// The channel on which messages can be sent to the constellation.
    constellation_chan: ConstellationChan,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: TimeProfilerChan,

    /// The channel on which messages can be sent to the memory profiler.
    memory_profiler_chan: MemoryProfilerChan,

    /// Pending scroll to fragment event, if any
    fragment_point: Option<Point2D<f32>>
}

#[deriving(PartialEq)]
enum ShutdownState {
    NotShuttingDown,
    ShuttingDown,
    FinishedShuttingDown,
}

impl IOCompositor {
    fn new(app: &Application,
               opts: Opts,
               port: Receiver<Msg>,
               constellation_chan: ConstellationChan,
               time_profiler_chan: TimeProfilerChan,
               memory_profiler_chan: MemoryProfilerChan) -> IOCompositor {

        let scale_factor = match opts.device_pixels_per_px {
            Some(device_pixels_per_px) => device_pixels_per_px,
            None => ScaleFactor(1.0),
        };
        let framebuffer_size = opts.initial_window_size.as_f32() * scale_factor;

        let window: Rc<Window> = WindowMethods::new(app, opts.output_file.is_none(),
                                                    framebuffer_size.as_uint());

        // Create an initial layer tree.
        //
        // TODO: There should be no initial layer tree until the renderer creates one from the
        // display list. This is only here because we don't have that logic in the renderer yet.
        let window_size = window.framebuffer_size();
        let hidpi_factor = window.hidpi_factor();

        let show_debug_borders = opts.show_debug_borders;
        IOCompositor {
            window: window,
            port: port,
            opts: opts,
            context: rendergl::RenderContext::new(CompositorTask::create_graphics_context(),
                                                  show_debug_borders),
            root_pipeline: None,
            scene: Scene::new(Rect {
                origin: Zero::zero(),
                size: window_size.as_f32(),
            }),
            window_size: window_size,
            hidpi_factor: hidpi_factor,
            composite_ready: false,
            shutdown_state: NotShuttingDown,
            recomposite: false,
            page_zoom: ScaleFactor(1.0),
            viewport_zoom: ScaleFactor(1.0),
            zoom_action: false,
            zoom_time: 0f64,
            ready_states: HashMap::new(),
            render_states: HashMap::new(),
            got_load_complete_message: false,
            constellation_chan: constellation_chan,
            time_profiler_chan: time_profiler_chan,
            memory_profiler_chan: memory_profiler_chan,
            fragment_point: None,
            outstanding_render_msgs: 0,
        }
    }

    pub fn create(app: &Application,
                  opts: Opts,
                  port: Receiver<Msg>,
                  constellation_chan: ConstellationChan,
                  time_profiler_chan: TimeProfilerChan,
                  memory_profiler_chan: MemoryProfilerChan) {
        let mut compositor = IOCompositor::new(app,
                                               opts,
                                               port,
                                               constellation_chan,
                                               time_profiler_chan,
                                               memory_profiler_chan);
        compositor.update_zoom_transform();

        // Starts the compositor, which listens for messages on the specified port.
        compositor.run();
    }

    fn run (&mut self) {
        // Tell the constellation about the initial window size.
        self.send_window_size();

        // Enter the main event loop.
        while self.shutdown_state != FinishedShuttingDown {
            // Check for new messages coming from the rendering task.
            self.handle_message();

            if self.shutdown_state == FinishedShuttingDown {
                // We have exited the compositor and passing window
                // messages to script may crash.
                debug!("Exiting the compositor due to a request from script.");
                break;
            }

            // Check for messages coming from the windowing system.
            let msg = self.window.recv();
            self.handle_window_message(msg);

            // If asked to recomposite and renderer has run at least once
            if self.recomposite && self.composite_ready {
                self.recomposite = false;
                self.composite();
            }

            sleep(Duration::milliseconds(10));

            // If a pinch-zoom happened recently, ask for tiles at the new resolution
            if self.zoom_action && precise_time_s() - self.zoom_time > 0.3 {
                self.zoom_action = false;
                self.scene.mark_layer_contents_as_changed_recursively();
                self.send_buffer_requests_for_all_layers();
            }

        }

        // Clear out the compositor layers so that painting tasks can destroy the buffers.
        match self.scene.root {
            None => {}
            Some(ref layer) => CompositorData::forget_all_tiles(layer.clone()),
        }

        // Drain compositor port, sometimes messages contain channels that are blocking
        // another task from finishing (i.e. SetIds)
        loop {
            match self.port.try_recv() {
                Err(_) => break,
                Ok(_) => {},
            }
        }

        // Tell the profiler and memory profiler to shut down.
        let TimeProfilerChan(ref time_profiler_chan) = self.time_profiler_chan;
        time_profiler_chan.send(time::ExitMsg);

        let MemoryProfilerChan(ref memory_profiler_chan) = self.memory_profiler_chan;
        memory_profiler_chan.send(memory::ExitMsg);
    }

    fn handle_message(&mut self) {
        loop {
            match (self.port.try_recv(), self.shutdown_state) {
                (_, FinishedShuttingDown) =>
                    fail!("compositor shouldn't be handling messages after shutting down"),

                (Err(_), _) => break,

                (Ok(Exit(chan)), _) => {
                    debug!("shutting down the constellation");
                    let ConstellationChan(ref con_chan) = self.constellation_chan;
                    con_chan.send(ExitMsg);
                    chan.send(());
                    self.shutdown_state = ShuttingDown;
                }

                (Ok(ShutdownComplete), _) => {
                    debug!("constellation completed shutdown");
                    self.shutdown_state = FinishedShuttingDown;
                    break;
                }

                (Ok(ChangeReadyState(pipeline_id, ready_state)), NotShuttingDown) => {
                    self.change_ready_state(pipeline_id, ready_state);
                }

                (Ok(ChangeRenderState(pipeline_id, render_state)), NotShuttingDown) => {
                    self.change_render_state(pipeline_id, render_state);
                }

                (Ok(RenderMsgDiscarded), NotShuttingDown) => {
                    self.remove_outstanding_render_msg();
                }

                (Ok(SetIds(frame_tree, response_chan, new_constellation_chan)), _) => {
                    self.set_frame_tree(&frame_tree,
                                        response_chan,
                                        new_constellation_chan);
                }

                (Ok(GetGraphicsMetadata(chan)), NotShuttingDown) => {
                    chan.send(Some(azure_hl::current_graphics_metadata()));
                }

                (Ok(CreateOrUpdateRootLayer(layer_properties)), NotShuttingDown) => {
                    self.create_or_update_root_layer(layer_properties);
                }

                (Ok(CreateOrUpdateDescendantLayer(layer_properties)), NotShuttingDown) => {
                    self.create_or_update_descendant_layer(layer_properties);
                }

                (Ok(SetLayerOrigin(pipeline_id, layer_id, origin)), NotShuttingDown) => {
                    self.set_layer_origin(pipeline_id, layer_id, origin);
                }

                (Ok(Paint(pipeline_id, epoch, replies)), NotShuttingDown) => {
                    for (layer_id, new_layer_buffer_set) in replies.into_iter() {
                        self.paint(pipeline_id, layer_id, new_layer_buffer_set, epoch);
                    }
                    self.remove_outstanding_render_msg();
                }

                (Ok(ScrollFragmentPoint(pipeline_id, layer_id, point)), NotShuttingDown) => {
                    self.scroll_fragment_to_point(pipeline_id, layer_id, point);
                }

                (Ok(LoadComplete(..)), NotShuttingDown) => {
                    self.got_load_complete_message = true;
                }

                // When we are shutting_down, we need to avoid performing operations
                // such as Paint that may crash because we have begun tearing down
                // the rest of our resources.
                (_, ShuttingDown) => { }
            }
        }
    }

    fn change_ready_state(&mut self, pipeline_id: PipelineId, ready_state: ReadyState) {
        self.ready_states.insert_or_update_with(pipeline_id,
                                                ready_state,
                                                |_key, value| *value = ready_state);
        self.window.set_ready_state(self.get_earliest_pipeline_ready_state());
    }

    fn get_earliest_pipeline_ready_state(&self) -> ReadyState {
        if self.ready_states.len() == 0 {
            return Blank;
        }
        return self.ready_states.values().fold(FinishedLoading, |a, &b| cmp::min(a, b));

    }

    fn change_render_state(&mut self, pipeline_id: PipelineId, render_state: RenderState) {
        self.render_states.insert_or_update_with(pipeline_id,
                                                 render_state,
                                                 |_key, value| *value = render_state);
        self.window.set_render_state(render_state);
        if render_state == IdleRenderState {
            self.composite_ready = true;
        }
    }

    fn all_pipelines_in_idle_render_state(&self) -> bool {
        if self.ready_states.len() == 0 {
            return false;
        }
        return self.render_states.values().all(|&value| value == IdleRenderState);
    }

    fn has_render_msg_tracking(&self) -> bool {
        // only track RenderMsg's if the compositor outputs to a file.
        self.opts.output_file.is_some()
    }

    fn has_outstanding_render_msgs(&self) -> bool {
        self.has_render_msg_tracking() && self.outstanding_render_msgs > 0
    }

    fn add_outstanding_render_msg(&mut self, count: uint) {
        // return early if not tracking render_msg's
        if !self.has_render_msg_tracking() {
            return;
        }
        debug!("add_outstanding_render_msg {}", self.outstanding_render_msgs);
        self.outstanding_render_msgs += count;
    }

    fn remove_outstanding_render_msg(&mut self) {
        if !self.has_render_msg_tracking() {
            return;
        }
        if self.outstanding_render_msgs > 0 {
            self.outstanding_render_msgs -= 1;
        } else {
            debug!("too many rerender msgs completed");
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
            Some(ref mut layer) => CompositorData::clear_all_tiles(layer.clone()),
            None => { }
        }
        self.scene.root = Some(self.create_frame_tree_root_layers(frame_tree, None));
        self.scene.set_root_layer_size(self.window_size.as_f32());

        // Initialize the new constellation channel by sending it the root window size.
        self.constellation_chan = new_constellation_chan;
        self.send_window_size();
    }

    fn create_frame_tree_root_layers(&mut self,
                                     frame_tree: &SendableFrameTree,
                                     frame_rect: Option<TypedRect<PagePx, f32>>)
                                     -> Rc<Layer<CompositorData>> {
        // Initialize the ReadyState and RenderState for this pipeline.
        self.ready_states.insert(frame_tree.pipeline.id, Blank);
        self.render_states.insert(frame_tree.pipeline.id, RenderingRenderState);

        let layer_properties = LayerProperties {
            pipeline_id: frame_tree.pipeline.id,
            epoch: Epoch(0),
            id: LayerId::null(),
            rect: Rect::zero(),
            background_color: azure_hl::Color::new(0., 0., 0., 0.),
            scroll_policy: Scrollable,
        };
        let root_layer = CompositorData::new_layer(frame_tree.pipeline.clone(),
                                                   layer_properties,
                                                   WantsScrollEvents,
                                                   self.opts.tile_size);

        match frame_rect {
            Some(ref frame_rect) => {
                *root_layer.masks_to_bounds.borrow_mut() = true;

                let frame_rect = frame_rect.to_untyped();
                *root_layer.bounds.borrow_mut() = Rect::from_untyped(&frame_rect);
            }
            None => {}
        }

        for kid in frame_tree.children.iter() {
            root_layer.add_child(self.create_frame_tree_root_layers(&kid.frame_tree, kid.rect));
        }
        return root_layer;
    }

    fn find_layer_with_pipeline_and_layer_id(&self,
                                             pipeline_id: PipelineId,
                                             layer_id: LayerId)
                                             -> Option<Rc<Layer<CompositorData>>> {
        match self.scene.root {
            Some(ref root_layer) => {
                CompositorData::find_layer_with_pipeline_and_layer_id(root_layer.clone(),
                                                                      pipeline_id,
                                                                      layer_id)
            }
            None => None,
        }

    }

    fn find_pipeline_root_layer(&self, pipeline_id: PipelineId) -> Rc<Layer<CompositorData>> {
        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, LayerId::null()) {
            Some(ref layer) => layer.clone(),
            None => fail!("Tried to create or update layer for unknown pipeline"),
        }
    }

    fn update_layer_if_exists(&mut self, properties: LayerProperties) -> bool {
        match self.find_layer_with_pipeline_and_layer_id(properties.pipeline_id, properties.id) {
            Some(existing_layer) => {
                CompositorData::update_layer(existing_layer.clone(), properties);
                true
            }
            None => false,
        }
    }

    fn create_or_update_root_layer(&mut self, layer_properties: LayerProperties) {
        let need_new_root_layer = !self.update_layer_if_exists(layer_properties);
        if need_new_root_layer {
            let root_layer = self.find_pipeline_root_layer(layer_properties.pipeline_id);
            CompositorData::update_layer_except_size(root_layer.clone(), layer_properties);

            let root_layer_pipeline = root_layer.extra_data.borrow().pipeline.clone();
            let first_child = CompositorData::new_layer(root_layer_pipeline.clone(),
                                                        layer_properties,
                                                        DoesntWantScrollEvents,
                                                        self.opts.tile_size);

            // Add the first child / base layer to the front of the child list, so that
            // child iframe layers are rendered on top of the base layer. These iframe
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
                    events::clamp_scroll_offset_and_scroll_layer(layer.clone(),
                                                                 TypedPoint2D(0f32, 0f32) - origin);
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
                    fail!("Compositor: Tried to scroll to fragment with unknown layer.");
                }

                self.recomposite = true;
            }
            None => {}
        };
    }

    fn set_layer_origin(&mut self,
                        pipeline_id: PipelineId,
                        layer_id: LayerId,
                        new_origin: Point2D<f32>) {
        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, layer_id) {
            Some(ref layer) => {
                layer.bounds.borrow_mut().origin = Point2D::from_untyped(&new_origin)
            }
            None => fail!("Compositor received SetLayerOrigin for nonexistent layer"),
        };

        self.send_buffer_requests_for_all_layers();
    }

    fn paint(&mut self,
             pipeline_id: PipelineId,
             layer_id: LayerId,
             new_layer_buffer_set: Box<LayerBufferSet>,
             epoch: Epoch) {
        debug!("compositor received new frame");

        // From now on, if we destroy the buffers, they will leak.
        let mut new_layer_buffer_set = new_layer_buffer_set;
        new_layer_buffer_set.mark_will_leak();

        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, layer_id) {
            Some(ref layer) => {
                assert!(CompositorData::add_buffers(layer.clone(), new_layer_buffer_set, epoch));
                self.recomposite = true;
            }
            None => {
                // FIXME: This may potentially be triggered by a race condition where a
                // buffers are being rendered but the layer is removed before rendering
                // completes.
                fail!("compositor given paint command for non-existent layer");
            }
        }
    }

    fn scroll_fragment_to_point(&mut self,
                                pipeline_id: PipelineId,
                                layer_id: LayerId,
                                point: Point2D<f32>) {
        if self.move_layer(pipeline_id, layer_id, Point2D::from_untyped(&point)) {
            self.recomposite = true;
            self.send_buffer_requests_for_all_layers();
        } else {
            self.fragment_point = Some(point);
        }
    }

    fn handle_window_message(&mut self, event: WindowEvent) {
        match event {
            IdleWindowEvent => {}

            RefreshWindowEvent => {
                self.recomposite = true;
            }

            ResizeWindowEvent(size) => {
                self.on_resize_window_event(size);
            }

            LoadUrlWindowEvent(url_string) => {
                self.on_load_url_window_event(url_string);
            }

            MouseWindowEventClass(mouse_window_event) => {
                self.on_mouse_window_event_class(mouse_window_event);
            }

            MouseWindowMoveEventClass(cursor) => {
                self.on_mouse_window_move_event_class(cursor);
            }

            ScrollWindowEvent(delta, cursor) => {
                self.on_scroll_window_event(delta, cursor);
            }

            ZoomWindowEvent(magnification) => {
                self.on_zoom_window_event(magnification);
            }

            PinchZoomWindowEvent(magnification) => {
                self.on_pinch_zoom_window_event(magnification);
            }

            NavigationWindowEvent(direction) => {
                self.on_navigation_window_event(direction);
            }

            FinishedWindowEvent => {
                let exit = self.opts.exit_after_load;
                if exit {
                    debug!("shutting down the constellation for FinishedWindowEvent");
                    let ConstellationChan(ref chan) = self.constellation_chan;
                    chan.send(ExitMsg);
                    self.shutdown_state = ShuttingDown;
                }
            }

            QuitWindowEvent => {
                debug!("shutting down the constellation for QuitWindowEvent");
                let ConstellationChan(ref chan) = self.constellation_chan;
                chan.send(ExitMsg);
                self.shutdown_state = ShuttingDown;
            }
        }
    }

    fn on_resize_window_event(&mut self, new_size: TypedSize2D<DevicePixel, uint>) {
        // A size change could also mean a resolution change.
        let new_hidpi_factor = self.window.hidpi_factor();
        if self.hidpi_factor != new_hidpi_factor {
            self.hidpi_factor = new_hidpi_factor;
            self.update_zoom_transform();
        }

        if self.window_size == new_size {
            return;
        }

        debug!("osmain: window resized to {:?}", new_size);
        self.window_size = new_size;
        self.scene.set_root_layer_size(new_size.as_f32());
        self.send_window_size();
    }

    fn on_load_url_window_event(&mut self, url_string: String) {
        debug!("osmain: loading URL `{:s}`", url_string);
        self.got_load_complete_message = false;
        let root_pipeline_id = match self.scene.root {
            Some(ref layer) => layer.extra_data.borrow().pipeline.id.clone(),
            None => fail!("Compositor: Received LoadUrlWindowEvent without initialized compositor \
                           layers"),
        };

        let msg = LoadUrlMsg(root_pipeline_id, LoadData::new(Url::parse(url_string.as_slice()).unwrap()));
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(msg);
    }

    fn on_mouse_window_event_class(&self, mouse_window_event: MouseWindowEvent) {
        let point = match mouse_window_event {
            MouseWindowClickEvent(_, p) => p,
            MouseWindowMouseDownEvent(_, p) => p,
            MouseWindowMouseUpEvent(_, p) => p,
        };
        for layer in self.scene.root.iter() {
            events::send_mouse_event(layer.clone(), mouse_window_event, point / self.scene.scale);
        }
    }

    fn on_mouse_window_move_event_class(&self, cursor: TypedPoint2D<DevicePixel, f32>) {
        for layer in self.scene.root.iter() {
            events::send_mouse_move_event(layer.clone(), cursor / self.scene.scale);
        }
    }

    fn on_scroll_window_event(&mut self,
                              delta: TypedPoint2D<DevicePixel, f32>,
                              cursor: TypedPoint2D<DevicePixel, i32>) {
        let delta = delta / self.scene.scale;
        let cursor = cursor.as_f32() / self.scene.scale;

        let mut scroll = false;
        match self.scene.root {
            Some(ref mut layer) => {
                scroll = events::handle_scroll_event(layer.clone(),
                                                     delta,
                                                     cursor) == ScrollPositionChanged;
            }
            None => { }
        }
        self.recomposite_if(scroll);
        self.send_buffer_requests_for_all_layers();
    }

    fn device_pixels_per_screen_px(&self) -> ScaleFactor<ScreenPx, DevicePixel, f32> {
        match self.opts.device_pixels_per_px {
            Some(device_pixels_per_px) => device_pixels_per_px,
            None => match self.opts.output_file {
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
                events::handle_scroll_event(layer.clone(),
                                            page_delta,
                                            cursor);
            }
            None => { }
        }

        self.recomposite = true;
    }

    fn on_navigation_window_event(&self, direction: WindowNavigateMsg) {
        let direction = match direction {
            windowing::Forward => constellation_msg::Forward,
            windowing::Back => constellation_msg::Back,
        };
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(NavigateMsg(direction))
    }

    fn convert_buffer_requests_to_pipeline_requests_map(&self,
                                                        requests: Vec<(Rc<Layer<CompositorData>>,
                                                                       Vec<BufferRequest>)>) ->
                                                        HashMap<PipelineId, (RenderChan,
                                                                             Vec<RenderRequest>)> {
        let scale = self.device_pixels_per_page_px();
        let mut results:
            HashMap<PipelineId, (RenderChan, Vec<RenderRequest>)> = HashMap::new();

        for (layer, mut layer_requests) in requests.into_iter() {
            let pipeline_id = layer.extra_data.borrow().pipeline.id;
            let &(_, ref mut vec) = results.find_or_insert_with(pipeline_id, |_| {
                (layer.extra_data.borrow().pipeline.render_chan.clone(), Vec::new())
            });

            // All the BufferRequests are in layer/device coordinates, but the render task
            // wants to know the page coordinates. We scale them before sending them.
            for request in layer_requests.iter_mut() {
                request.page_rect = request.page_rect / scale.get();
            }

            vec.push(RenderRequest {
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
                let have_unused_buffers = unused_buffers.len() > 0;
                self.recomposite = self.recomposite || have_unused_buffers;
                if have_unused_buffers {
                    let message = UnusedBufferMsg(unused_buffers);
                    let _ = pipeline.render_chan.send_opt(message);
                }
            },
            None => {}
        }
    }

    fn send_buffer_requests_for_all_layers(&mut self) {
        let mut layers_and_requests = Vec::new();
        self.scene.get_buffer_requests(&mut layers_and_requests,
                                       Rect(TypedPoint2D(0f32, 0f32), self.window_size.as_f32()));

        // Return unused tiles first, so that they can be reused by any new BufferRequests.
        self.send_back_unused_buffers();

        if layers_and_requests.len() == 0 {
            return;
        }

        // We want to batch requests for each pipeline to avoid race conditions
        // when handling the resulting BufferRequest responses.
        let pipeline_requests =
            self.convert_buffer_requests_to_pipeline_requests_map(layers_and_requests);

        let mut num_render_msgs_sent = 0;
        for (_pipeline_id, (chan, requests)) in pipeline_requests.into_iter() {
            num_render_msgs_sent += 1;
            let _ = chan.send_opt(RenderMsg(requests));
        }

        self.add_outstanding_render_msg(num_render_msgs_sent);
    }

    fn is_ready_to_render_image_output(&self) -> bool {
        if !self.got_load_complete_message {
            return false;
        }

        if self.get_earliest_pipeline_ready_state() != FinishedLoading {
            return false;
        }

        if self.has_outstanding_render_msgs() {
            return false;
        }

        if !self.all_pipelines_in_idle_render_state() {
            return false;
        }
        return true;
    }

    fn composite(&mut self) {
        let output_image = self.opts.output_file.is_some() &&
                            self.is_ready_to_render_image_output();

        let mut framebuffer_ids = vec!();
        let mut texture_ids = vec!();
        let (width, height) = (self.window_size.width.get(), self.window_size.height.get());

        if output_image {
            framebuffer_ids = gl2::gen_framebuffers(1);
            gl2::bind_framebuffer(gl2::FRAMEBUFFER, framebuffer_ids[0]);

            texture_ids = gl2::gen_textures(1);
            gl2::bind_texture(gl2::TEXTURE_2D, texture_ids[0]);

            gl2::tex_image_2d(gl2::TEXTURE_2D, 0, gl2::RGB as gl2::GLint, width as gl2::GLsizei,
                                height as gl2::GLsizei, 0, gl2::RGB, gl2::UNSIGNED_BYTE, None);
            gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_MAG_FILTER, gl2::NEAREST as gl2::GLint);
            gl2::tex_parameter_i(gl2::TEXTURE_2D, gl2::TEXTURE_MIN_FILTER, gl2::NEAREST as gl2::GLint);

            gl2::framebuffer_texture_2d(gl2::FRAMEBUFFER, gl2::COLOR_ATTACHMENT0, gl2::TEXTURE_2D,
                                        texture_ids[0], 0);

            gl2::bind_texture(gl2::TEXTURE_2D, 0);
        }

        profile(time::CompositingCategory, None, self.time_profiler_chan.clone(), || {
            debug!("compositor: compositing");
            // Adjust the layer dimensions as necessary to correspond to the size of the window.
            self.scene.viewport = Rect {
                origin: Zero::zero(),
                size: self.window_size.as_f32(),
            };
            // Render the scene.
            match self.scene.root {
                Some(ref layer) => {
                    self.scene.background_color.r = layer.extra_data.borrow().background_color.r;
                    self.scene.background_color.g = layer.extra_data.borrow().background_color.g;
                    self.scene.background_color.b = layer.extra_data.borrow().background_color.b;
                    self.scene.background_color.a = layer.extra_data.borrow().background_color.a;
                    rendergl::render_scene(layer.clone(), self.context, &self.scene);
                }
                None => {}
            }
        });

        if output_image {
            let path = from_str::<Path>(self.opts.output_file.as_ref().unwrap().as_slice()).unwrap();
            let mut pixels = gl2::read_pixels(0, 0,
                                              width as gl2::GLsizei,
                                              height as gl2::GLsizei,
                                              gl2::RGB, gl2::UNSIGNED_BYTE);

            gl2::bind_framebuffer(gl2::FRAMEBUFFER, 0);

            gl2::delete_buffers(texture_ids.as_slice());
            gl2::delete_frame_buffers(framebuffer_ids.as_slice());

            // flip image vertically (texture is upside down)
            let orig_pixels = pixels.clone();
            let stride = width * 3;
            for y in range(0, height) {
                let dst_start = y * stride;
                let src_start = (height - y - 1) * stride;
                unsafe {
                    let src_slice = orig_pixels.slice(src_start, src_start + stride);
                    pixels.slice_mut(dst_start, dst_start + stride)
                          .copy_memory(src_slice.slice_to(stride));
                }
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

        self.window.present();

        let exit = self.opts.exit_after_load;
        if exit {
            debug!("shutting down the constellation for exit_after_load");
            let ConstellationChan(ref chan) = self.constellation_chan;
            chan.send(ExitMsg);
        }
    }

    fn recomposite_if(&mut self, result: bool) {
        self.recomposite = result || self.recomposite;
    }
}
