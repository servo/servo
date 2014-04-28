/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use constellation::SendableFrameTree;
use compositing::compositor_layer::CompositorLayer;
use compositing::*;
use pipeline::CompositionPipeline;
use platform::{Application, Window};
use windowing::{FinishedWindowEvent, IdleWindowEvent, LoadUrlWindowEvent, MouseWindowClickEvent};
use windowing::{MouseWindowEvent, MouseWindowEventClass, MouseWindowMouseDownEvent};
use windowing::{MouseWindowMouseUpEvent, MouseWindowMoveEventClass, NavigationWindowEvent};
use windowing::{QuitWindowEvent, RefreshWindowEvent, ResizeWindowEvent, ScrollWindowEvent};
use windowing::{WindowEvent, WindowMethods, WindowNavigateMsg, ZoomWindowEvent};

use azure::azure_hl::{SourceSurfaceMethods, Color};
use azure::azure_hl;
use geom::matrix::identity;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use layers::layers::{ContainerLayer, ContainerLayerKind};
use layers::platform::surface::NativeCompositingGraphicsContext;
use layers::rendergl;
use layers::rendergl::RenderContext;
use layers::scene::Scene;
use opengles::gl2;
use png;
use servo_msg::compositor_msg::{Blank, Epoch, FinishedLoading, IdleRenderState, LayerBufferSet};
use servo_msg::compositor_msg::{LayerId, ReadyState, RenderState, ScrollPolicy, Scrollable};
use servo_msg::constellation_msg::{ConstellationChan, ExitMsg, LoadUrlMsg, NavigateMsg};
use servo_msg::constellation_msg::{PipelineId, ResizedWindowMsg};
use servo_msg::constellation_msg;
use servo_util::opts::Opts;
use servo_util::time::{profile, ProfilerChan};
use servo_util::{time, url};
use std::comm::{Empty, Disconnected, Data, Sender, Receiver};
use std::io::timer::sleep;
use std::path::Path;
use std::rc::Rc;
use time::precise_time_s;


pub struct IOCompositor {
    /// The application window.
    pub window: Rc<Window>,

    /// The port on which we receive messages.
    pub port: Receiver<Msg>,

    /// The render context.
    pub context: RenderContext,

    /// The root ContainerLayer.
    pub root_layer: Rc<ContainerLayer>,

    /// The root pipeline.
    pub root_pipeline: Option<CompositionPipeline>,

    /// The canvas to paint a page.
    pub scene: Scene,

    /// The application window size.
    pub window_size: Size2D<uint>,

    /// The platform-specific graphics context.
    pub graphics_context: NativeCompositingGraphicsContext,

    /// Tracks whether the renderer has finished its first rendering
    pub composite_ready: bool,

    /// Tracks whether we are in the process of shutting down.
    pub shutting_down: bool,

    /// Tracks whether we should close compositor.
    pub done: bool,

    /// Tracks whether we need to re-composite a page.
    pub recomposite: bool,

    /// Keeps track of the current zoom factor.
    pub world_zoom: f32,

    /// Tracks whether the zoom action has happend recently.
    pub zoom_action: bool,

    /// The time of the last zoom action has started.
    pub zoom_time: f64,

    /// Current display/reflow status of the page
    pub ready_state: ReadyState,

    /// Whether the page being rendered has loaded completely.
    /// Differs from ReadyState because we can finish loading (ready)
    /// many times for a single page.
    pub load_complete: bool,

    /// The command line option flags.
    pub opts: Opts,

    /// The root CompositorLayer
    pub compositor_layer: Option<CompositorLayer>,

    /// The channel on which messages can be sent to the constellation.
    pub constellation_chan: ConstellationChan,

    /// The channel on which messages can be sent to the profiler.
    pub profiler_chan: ProfilerChan,

    /// Pending scroll to fragment event, if any
    pub fragment_point: Option<Point2D<f32>>
}

impl IOCompositor {
    pub fn new(app: &Application,
               opts: Opts,
               port: Receiver<Msg>,
               constellation_chan: ConstellationChan,
               profiler_chan: ProfilerChan) -> IOCompositor {
        let window: Rc<Window> = WindowMethods::new(app);

        // Create an initial layer tree.
        //
        // TODO: There should be no initial layer tree until the renderer creates one from the display
        // list. This is only here because we don't have that logic in the renderer yet.
        let root_layer = Rc::new(ContainerLayer());
        let window_size = window.size();

        let hidpi_factor = window.hidpi_factor();
        root_layer.common.borrow_mut().set_transform(identity().scale(hidpi_factor, hidpi_factor, 1f32));

        IOCompositor {
            window: window,
            port: port,
            opts: opts,
            context: rendergl::init_render_context(),
            root_layer: root_layer.clone(),
            root_pipeline: None,
            scene: Scene(ContainerLayerKind(root_layer), window_size, identity()),
            window_size: Size2D(window_size.width as uint, window_size.height as uint),
            graphics_context: CompositorTask::create_graphics_context(),
            composite_ready: false,
            shutting_down: false,
            done: false,
            recomposite: false,
            world_zoom: hidpi_factor,
            zoom_action: false,
            zoom_time: 0f64,
            ready_state: Blank,
            load_complete: false,
            compositor_layer: None,
            constellation_chan: constellation_chan,
            profiler_chan: profiler_chan,
            fragment_point: None
        }
    }

    pub fn create(app: &Application,
                  opts: Opts,
                  port: Receiver<Msg>,
                  constellation_chan: ConstellationChan,
                  profiler_chan: ProfilerChan) {
        let mut compositor = IOCompositor::new(app,
                                               opts,
                                               port,
                                               constellation_chan,
                                               profiler_chan);

        // Starts the compositor, which listens for messages on the specified port.
        compositor.run();
    }

    fn run (&mut self) {
        // Tell the constellation about the initial window size.
        {
            let ConstellationChan(ref chan) = self.constellation_chan;
            chan.send(ResizedWindowMsg(self.window_size));
        }

        // Enter the main event loop.
        while !self.done {
            // Check for new messages coming from the rendering task.
            self.handle_message();

            if self.done {
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

            sleep(10);

            // If a pinch-zoom happened recently, ask for tiles at the new resolution
            if self.zoom_action && precise_time_s() - self.zoom_time > 0.3 {
                self.zoom_action = false;
                self.ask_for_tiles();
            }

        }

        // Clear out the compositor layers so that painting tasks can destroy the buffers.
        match self.compositor_layer {
            None => {}
            Some(ref mut layer) => layer.forget_all_tiles(),
        }

        // Drain compositor port, sometimes messages contain channels that are blocking
        // another task from finishing (i.e. SetIds)
        loop {
            match self.port.try_recv() {
                Empty | Disconnected => break,
                Data(_) => {},
            }
        }

        // Tell the profiler to shut down.
        let ProfilerChan(ref chan) = self.profiler_chan;
        chan.send(time::ExitMsg);
    }

    fn handle_message(&mut self) {
        loop {
            match (self.port.try_recv(), self.shutting_down) {
                (Empty, _) => break,

                (Disconnected, _) => break,

                (Data(Exit(chan)), _) => {
                    debug!("shutting down the constellation");
                    let ConstellationChan(ref con_chan) = self.constellation_chan;
                    con_chan.send(ExitMsg);
                    chan.send(());
                    self.shutting_down = true;
                }

                (Data(ShutdownComplete), _) => {
                    debug!("constellation completed shutdown");
                    self.done = true;
                }

                (Data(ChangeReadyState(ready_state)), false) => {
                    self.window.set_ready_state(ready_state);
                    self.ready_state = ready_state;
                }

                (Data(ChangeRenderState(render_state)), false) => {
                    self.change_render_state(render_state);
                }

                (Data(SetUnRenderedColor(pipeline_id, layer_id, color)), false) => {
                    self.set_unrendered_color(pipeline_id, layer_id, color);
                }

                (Data(SetIds(frame_tree, response_chan, new_constellation_chan)), _) => {
                    self.set_ids(frame_tree, response_chan, new_constellation_chan);
                }

                (Data(GetGraphicsMetadata(chan)), false) => {
                    chan.send(Some(azure_hl::current_graphics_metadata()));
                }

                (Data(CreateRootCompositorLayerIfNecessary(pipeline_id, layer_id, size)),
                 false) => {
                    self.create_root_compositor_layer_if_necessary(pipeline_id, layer_id, size);
                }

                (Data(CreateDescendantCompositorLayerIfNecessary(pipeline_id,
                                                                 layer_id,
                                                                 rect,
                                                                 scroll_behavior)),
                 false) => {
                    self.create_descendant_compositor_layer_if_necessary(pipeline_id,
                                                                         layer_id,
                                                                         rect,
                                                                         scroll_behavior);
                }

                (Data(SetLayerPageSize(pipeline_id, layer_id, new_size, epoch)), false) => {
                    self.set_layer_page_size(pipeline_id, layer_id, new_size, epoch);
                }

                (Data(SetLayerClipRect(pipeline_id, layer_id, new_rect)), false) => {
                    self.set_layer_clip_rect(pipeline_id, layer_id, new_rect);
                }

                (Data(DeleteLayerGroup(id)), _) => {
                    self.delete_layer(id);
                }

                (Data(Paint(pipeline_id, layer_id, new_layer_buffer_set, epoch)), false) => {
                    self.paint(pipeline_id, layer_id, new_layer_buffer_set, epoch);
                }

                (Data(ScrollFragmentPoint(pipeline_id, layer_id, point)), false) => {
                    self.scroll_fragment_to_point(pipeline_id, layer_id, point);
                }

                (Data(LoadComplete(..)), false) => {
                    self.load_complete = true;
                }

                // When we are shutting_down, we need to avoid performing operations
                // such as Paint that may crash because we have begun tearing down
                // the rest of our resources.
                (_, true) => { }
            }
        }
    }

    fn change_render_state(&mut self, render_state: RenderState) {
        self.window.set_render_state(render_state);
        if render_state == IdleRenderState {
            self.composite_ready = true;
        }
    }

    // FIXME(#2004, pcwalton): Take the pipeline ID and layer ID into account.
    fn set_unrendered_color(&mut self, _: PipelineId, _: LayerId, color: Color) {
        match self.compositor_layer {
            Some(ref mut layer) => layer.unrendered_color = color,
            None => {}
        }
    }

    fn set_ids(&mut self,
               frame_tree: SendableFrameTree,
               response_chan: Sender<()>,
               new_constellation_chan: ConstellationChan) {
        response_chan.send(());

        self.root_pipeline = Some(frame_tree.pipeline.clone());

        // Initialize the new constellation channel by sending it the root window size.
        let window_size = self.window.size();
        let window_size = Size2D(window_size.width as uint,
                                 window_size.height as uint);
        {
            let ConstellationChan(ref chan) = new_constellation_chan;
            chan.send(ResizedWindowMsg(window_size));
        }

        self.constellation_chan = new_constellation_chan;
    }

    fn create_root_compositor_layer_if_necessary(&mut self,
                                                 id: PipelineId,
                                                 layer_id: LayerId,
                                                 size: Size2D<f32>) {
        let (root_pipeline, root_layer_id) = match self.compositor_layer {
            Some(ref compositor_layer) if compositor_layer.pipeline.id == id => {
                (compositor_layer.pipeline.clone(), compositor_layer.id_of_first_child())
            }
            _ => {
                match self.root_pipeline {
                    Some(ref root_pipeline) => (root_pipeline.clone(), LayerId::null()),
                    None => fail!("Compositor: Received new layer without initialized pipeline"),
                }
            }
        };

        if layer_id != root_layer_id {
            let root_pipeline_id = root_pipeline.id;
            let mut new_layer = CompositorLayer::new_root(root_pipeline,
                                                          size,
                                                          self.opts.tile_size,
                                                          self.opts.cpu_painting);

            let first_child = self.root_layer.first_child.borrow().clone();
            match first_child {
                None => {}
                Some(old_layer) => {
                    ContainerLayer::remove_child(self.root_layer.clone(), old_layer)
                }
            }

            assert!(new_layer.add_child_if_necessary(self.root_layer.clone(),
                                                     root_pipeline_id,
                                                     new_layer.id,
                                                     layer_id,
                                                     Rect(Point2D(0f32, 0f32), size),
                                                     size,
                                                     Scrollable));

            ContainerLayer::add_child_start(self.root_layer.clone(),
                                            ContainerLayerKind(new_layer.root_layer.clone()));

            // Release all tiles from the layer before dropping it.
            for layer in self.compositor_layer.mut_iter() {
                layer.clear_all_tiles();
            }
            self.compositor_layer = Some(new_layer);
        }

        self.ask_for_tiles();
    }

    fn create_descendant_compositor_layer_if_necessary(&mut self,
                                                       pipeline_id: PipelineId,
                                                       layer_id: LayerId,
                                                       rect: Rect<f32>,
                                                       scroll_policy: ScrollPolicy) {
        match self.compositor_layer {
            Some(ref mut compositor_layer) => {
                assert!(compositor_layer.add_child_if_necessary(self.root_layer.clone(),
                                                                pipeline_id,
                                                                compositor_layer.id,
                                                                layer_id,
                                                                rect,
                                                                compositor_layer.page_size
                                                                                .unwrap(),
                                                                scroll_policy))
            }
            None => fail!("Compositor: Received new layer without initialized pipeline"),
        };

        self.ask_for_tiles();
    }

    fn set_layer_page_size(&mut self,
                           pipeline_id: PipelineId,
                           layer_id: LayerId,
                           new_size: Size2D<f32>,
                           epoch: Epoch) {
        let (ask, move): (bool, bool) = match self.compositor_layer {
            Some(ref mut layer) => {
                let window_size = &self.window_size;
                let world_zoom = self.world_zoom;
                let page_window = Size2D(window_size.width as f32 / world_zoom,
                                         window_size.height as f32 / world_zoom);
                layer.resize(pipeline_id, layer_id, new_size, page_window, epoch);
                let move = self.fragment_point.take().map_or(false, |point| {
                    layer.move(pipeline_id, layer_id, point, page_window)
                });

                (true, move)
            }
            None => (false, false)
        };

        if ask {
            self.recomposite_if(move);
            self.ask_for_tiles();
        }
    }

    fn set_layer_clip_rect(&mut self,
                           pipeline_id: PipelineId,
                           layer_id: LayerId,
                           new_rect: Rect<f32>) {
        let ask: bool = match self.compositor_layer {
            Some(ref mut layer) => {
                assert!(layer.set_clipping_rect(pipeline_id, layer_id, new_rect));
                true
            }
            None => {
                false
            }
        };

        if ask {
            self.ask_for_tiles();
        }
    }

    fn delete_layer(&mut self, id: PipelineId) {
        let ask: bool = match self.compositor_layer {
            Some(ref mut layer) => {
                assert!(layer.delete(&self.graphics_context, id));
                true
            }
            None => {
                false
            }
        };

        if ask {
            self.ask_for_tiles();
        }
    }

    fn paint(&mut self,
             pipeline_id: PipelineId,
             layer_id: LayerId,
             new_layer_buffer_set: ~LayerBufferSet,
             epoch: Epoch) {
        debug!("compositor received new frame");

        // From now on, if we destroy the buffers, they will leak.
        let mut new_layer_buffer_set = new_layer_buffer_set;
        new_layer_buffer_set.mark_will_leak();

        match self.compositor_layer {
            Some(ref mut layer) => {
                assert!(layer.add_buffers(&self.graphics_context,
                                          pipeline_id,
                                          layer_id,
                                          new_layer_buffer_set,
                                          epoch).is_none());
                self.recomposite = true;
            }
            None => {
                fail!("compositor given paint command with no CompositorLayer initialized");
            }
        }

        // TODO: Recycle the old buffers; send them back to the renderer to reuse if
        // it wishes.
    }

    fn scroll_fragment_to_point(&mut self,
                                pipeline_id: PipelineId,
                                layer_id: LayerId,
                                point: Point2D<f32>) {
        let world_zoom = self.world_zoom;
        let page_window = Size2D(self.window_size.width as f32 / world_zoom,
                                 self.window_size.height as f32 / world_zoom);

        let (ask, move): (bool, bool) = match self.compositor_layer {
            Some(ref mut layer) if layer.pipeline.id == pipeline_id && !layer.hidden => {

                (true, layer.move(pipeline_id, layer_id, point, page_window))
            }
            Some(_) | None => {
                self.fragment_point = Some(point);

                (false, false)
            }
        };

        if ask {
            self.recomposite_if(move);
            self.ask_for_tiles();
        }
    }

    fn handle_window_message(&mut self, event: WindowEvent) {
        match event {
            IdleWindowEvent => {}

            RefreshWindowEvent => {
                self.recomposite = true;
            }

            ResizeWindowEvent(width, height) => {
                self.on_resize_window_event(width, height);
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

            NavigationWindowEvent(direction) => {
                self.on_navigation_window_event(direction);
            }

            FinishedWindowEvent => {
                let exit = self.opts.exit_after_load;
                if exit {
                    debug!("shutting down the constellation for FinishedWindowEvent");
                    let ConstellationChan(ref chan) = self.constellation_chan;
                    chan.send(ExitMsg);
                    self.shutting_down = true;
                }
            }

            QuitWindowEvent => {
                debug!("shutting down the constellation for QuitWindowEvent");
                let ConstellationChan(ref chan) = self.constellation_chan;
                chan.send(ExitMsg);
                self.shutting_down = true;
            }
        }
    }

    fn on_resize_window_event(&mut self, width: uint, height: uint) {
        let new_size = Size2D(width, height);
        if self.window_size != new_size {
            debug!("osmain: window resized to {:u}x{:u}", width, height);
            self.window_size = new_size;
            let ConstellationChan(ref chan) = self.constellation_chan;
            chan.send(ResizedWindowMsg(new_size))
        } else {
            debug!("osmain: dropping window resize since size is still {:u}x{:u}", width, height);
        }
    }

    fn on_load_url_window_event(&mut self, url_string: ~str) {
        debug!("osmain: loading URL `{:s}`", url_string);
        self.load_complete = false;
        let root_pipeline_id = match self.compositor_layer {
            Some(ref layer) => layer.pipeline.id.clone(),
            None => fail!("Compositor: Received LoadUrlWindowEvent without initialized compositor layers"),
        };

        let msg = LoadUrlMsg(root_pipeline_id, url::parse_url(url_string, None));
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(msg);
    }

    fn on_mouse_window_event_class(&self, mouse_window_event: MouseWindowEvent) {
        let world_zoom = self.world_zoom;
        let point = match mouse_window_event {
            MouseWindowClickEvent(_, p) => Point2D(p.x / world_zoom, p.y / world_zoom),
            MouseWindowMouseDownEvent(_, p) => Point2D(p.x / world_zoom, p.y / world_zoom),
            MouseWindowMouseUpEvent(_, p) => Point2D(p.x / world_zoom, p.y / world_zoom),
        };
        for layer in self.compositor_layer.iter() {
            layer.send_mouse_event(mouse_window_event, point);
        }
    }

    fn on_mouse_window_move_event_class(&self, cursor: Point2D<f32>) {
        for layer in self.compositor_layer.iter() {
            layer.send_mouse_move_event(cursor);
        }
    }

    fn on_scroll_window_event(&mut self, delta: Point2D<f32>, cursor: Point2D<i32>) {
        let world_zoom = self.world_zoom;
        // TODO: modify delta to snap scroll to pixels.
        let page_delta = Point2D(delta.x as f32 / world_zoom, delta.y as f32 / world_zoom);
        let page_cursor: Point2D<f32> = Point2D(cursor.x as f32 / world_zoom,
                                                cursor.y as f32 / world_zoom);
        let page_window = Size2D(self.window_size.width as f32 / world_zoom,
                                 self.window_size.height as f32 / world_zoom);
        let mut scroll = false;
        for layer in self.compositor_layer.mut_iter() {
            scroll = layer.handle_scroll_event(page_delta, page_cursor, page_window) || scroll;
        }
        self.recomposite_if(scroll);
        self.ask_for_tiles();
    }

    fn on_zoom_window_event(&mut self, magnification: f32) {
        self.zoom_action = true;
        self.zoom_time = precise_time_s();
        let old_world_zoom = self.world_zoom;
        let window_size = &self.window_size;

        // Determine zoom amount
        self.world_zoom = (self.world_zoom * magnification).max(1.0);
        let world_zoom = self.world_zoom;

        {
            self.root_layer.common.borrow_mut().set_transform(identity().scale(world_zoom, world_zoom, 1f32));
        }

        // Scroll as needed
        let page_delta = Point2D(window_size.width as f32 * (1.0 / world_zoom - 1.0 / old_world_zoom) * 0.5,
                                 window_size.height as f32 * (1.0 / world_zoom - 1.0 / old_world_zoom) * 0.5);
        // TODO: modify delta to snap scroll to pixels.
        let page_cursor = Point2D(-1f32, -1f32); // Make sure this hits the base layer
        let page_window = Size2D(window_size.width as f32 / world_zoom,
                                 window_size.height as f32 / world_zoom);
        for layer in self.compositor_layer.mut_iter() {
            layer.handle_scroll_event(page_delta, page_cursor, page_window);
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

    /// Get BufferRequests from each layer.
    fn ask_for_tiles(&mut self) {
        let world_zoom = self.world_zoom;
        let window_size_page = Size2D(self.window_size.width as f32 / world_zoom,
                                      self.window_size.height as f32 / world_zoom);
        for layer in self.compositor_layer.mut_iter() {
            if !layer.hidden {
                let rect = Rect(Point2D(0f32, 0f32), window_size_page);
                let recomposite = layer.get_buffer_request(&self.graphics_context,
                                                           rect,
                                                           world_zoom) ||
                                  self.recomposite;
                self.recomposite = recomposite;
            } else {
                debug!("Compositor: root layer is hidden!");
            }
        }
    }

    fn composite(&mut self) {
        profile(time::CompositingCategory, self.profiler_chan.clone(), || {
            debug!("compositor: compositing");
            // Adjust the layer dimensions as necessary to correspond to the size of the window.
            self.scene.size = self.window.size();
            // Render the scene.
            match self.compositor_layer {
                Some(ref mut layer) => {
                    self.scene.background_color.r = layer.unrendered_color.r;
                    self.scene.background_color.g = layer.unrendered_color.g;
                    self.scene.background_color.b = layer.unrendered_color.b;
                    self.scene.background_color.a = layer.unrendered_color.a;
                }
                None => {}
            }
            rendergl::render_scene(self.context, &self.scene);
        });

        // Render to PNG. We must read from the back buffer (ie, before
        // self.window.present()) as OpenGL ES 2 does not have glReadBuffer().
        if self.load_complete && self.ready_state == FinishedLoading
            && self.opts.output_file.is_some() {
            let (width, height) = (self.window_size.width as uint, self.window_size.height as uint);
            let path = from_str::<Path>(*self.opts.output_file.get_ref()).unwrap();
            let mut pixels = gl2::read_pixels(0, 0,
                                              width as gl2::GLsizei,
                                              height as gl2::GLsizei,
                                              gl2::RGB, gl2::UNSIGNED_BYTE);
            // flip image vertically (texture is upside down)
            let orig_pixels = pixels.clone();
            let stride = width * 3;
            for y in range(0, height) {
                let dst_start = y * stride;
                let src_start = (height - y - 1) * stride;
                unsafe {
                    pixels.mut_slice(dst_start, dst_start + stride)
                        .copy_memory(orig_pixels.slice(src_start, src_start + stride).slice_to(stride));
                }
            }
            let img = png::Image {
                width: width as u32,
                height: height as u32,
                color_type: png::RGB8,
                pixels: pixels,
            };
            let res = png::store_png(&img, &path);
            assert!(res.is_ok());

            debug!("shutting down the constellation after generating an output file");
            let ConstellationChan(ref chan) = self.constellation_chan;
            chan.send(ExitMsg);
            self.shutting_down = true;
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

