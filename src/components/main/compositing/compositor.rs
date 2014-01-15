/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use constellation::SendableFrameTree;
use compositing::compositor_layer::CompositorLayer;
use compositing::*;

use platform::{Application, Window};

use windowing::{WindowEvent, WindowMethods,
                WindowNavigateMsg,
                IdleWindowEvent, RefreshWindowEvent, ResizeWindowEvent, LoadUrlWindowEvent,
                MouseWindowEventClass,ScrollWindowEvent, ZoomWindowEvent, NavigationWindowEvent,
                FinishedWindowEvent, QuitWindowEvent,
                MouseWindowEvent, MouseWindowClickEvent, MouseWindowMouseDownEvent, MouseWindowMouseUpEvent};


use azure::azure_hl::{SourceSurfaceMethods, Color};
use azure::azure_hl;
use extra::time::precise_time_s;
use geom::matrix::identity;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::opts::Opts;
use layers::layers::{ContainerLayer, ContainerLayerKind};
use layers::platform::surface::NativeCompositingGraphicsContext;
use layers::rendergl;
use layers::rendergl::RenderContext;
use layers::scene::Scene;
use opengles::gl2;
use png;
use servo_msg::compositor_msg::{Epoch, IdleRenderState, LayerBufferSet, RenderState};
use servo_msg::constellation_msg::{ConstellationChan, ExitMsg, NavigateMsg, ResizedWindowMsg, LoadUrlMsg, PipelineId};
use servo_msg::constellation_msg;
use servo_util::time::{profile, ProfilerChan, Timer};
use servo_util::{time, url};
use std::comm::Port;
use std::num::Orderable;
use std::path::Path;


pub struct IOCompositor {
    /// The application window.
    window: @mut Window,

    /// The port on which we receive messages.
    port: Port<Msg>,

    /// The render context.
    context: RenderContext,

    /// The root ContainerLayer.
    root_layer: @mut ContainerLayer,

    /// The canvas to paint a page.
    scene: Scene,

    /// The application window size.
    window_size: Size2D<uint>,

    /// The platform-specific graphics context.
    graphics_context: NativeCompositingGraphicsContext,

    /// Tracks whether the renderer has finished its first rendering
    composite_ready: bool,

    /// Tracks whether we should close compositor.
    done: bool,

    /// Tracks whether we need to re-composite a page.
    recomposite: bool,

    /// Keeps track of the current zoom factor.
    world_zoom: f32,

    /// Tracks whether the zoom action has happend recently.
    zoom_action: bool,

    /// The time of the last zoom action has started.
    zoom_time: f64,

    /// The command line option flags.
    opts: Opts,

    /// The root CompositorLayer
    compositor_layer: Option<CompositorLayer>,

    /// The channel on which messages can be sent to the constellation.
    constellation_chan: ConstellationChan,

    /// The channel on which messages can be sent to the profiler.
    profiler_chan: ProfilerChan,

    /// Pending scroll to fragment event, if any 
    fragment_point: Option<Point2D<f32>>
}

impl IOCompositor {

    pub fn new(app: &Application,
               opts: Opts,
               port: Port<Msg>,
               constellation_chan: ConstellationChan,
               profiler_chan: ProfilerChan) -> IOCompositor {
        let window: @mut Window = WindowMethods::new(app);

        // Create an initial layer tree.
        //
        // TODO: There should be no initial layer tree until the renderer creates one from the display
        // list. This is only here because we don't have that logic in the renderer yet.
        let root_layer = @mut ContainerLayer();
        let window_size = window.size();

        IOCompositor {
            window: window,
            port: port,
            opts: opts,
            context: rendergl::init_render_context(),
            root_layer: root_layer,
            scene: Scene(ContainerLayerKind(root_layer), window_size, identity()),
            window_size: Size2D(window_size.width as uint, window_size.height as uint),
            graphics_context: CompositorTask::create_graphics_context(),
            composite_ready: false,
            done: false,
            recomposite: false,
            world_zoom: 1f32,
            zoom_action: false,
            zoom_time: 0f64,
            compositor_layer: None,
            constellation_chan: constellation_chan,
            profiler_chan: profiler_chan,
            fragment_point: None
        }
    }

    pub fn create(app: &Application,
                  opts: Opts,
                  port: Port<Msg>,
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
        self.constellation_chan.send(ResizedWindowMsg(self.window_size));

        // Enter the main event loop.
        while !self.done {
            // Check for new messages coming from the rendering task.
            self.handle_message();

            if (self.done) {
                // We have exited the compositor and passing window
                // messages to script may crash.
                debug!("Exiting the compositor due to a request from script.");
                break;
            }

            // Check for messages coming from the windowing system.
            self.handle_window_message(self.window.recv());

            // If asked to recomposite and renderer has run at least once
            if self.recomposite && self.composite_ready {
                self.recomposite = false;
                self.composite();
            }

            Timer::sleep(10);

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
        while self.port.try_recv().is_some() {}
    }

    fn handle_message(&mut self) {
        loop {
            match self.port.try_recv() {
                None => break,

                Some(Exit(chan)) => {
                    debug!("shutting down the constellation");
                    self.constellation_chan.send(ExitMsg);
                    chan.send(());
                }

                Some(ShutdownComplete) => {
                    debug!("constellation completed shutdown");
                    self.done = true;
                }

                Some(ChangeReadyState(ready_state)) => {
                    self.window.set_ready_state(ready_state);
                }

                Some(ChangeRenderState(render_state)) => {
                    self.change_render_state(render_state);
                }

                Some(SetUnRenderedColor(_id, color)) => {
                    self.set_unrendered_color(_id, color);
                }


                Some(SetIds(frame_tree, response_chan, new_constellation_chan)) => {
                    self.set_ids(frame_tree, response_chan, new_constellation_chan);
                }

                Some(GetGraphicsMetadata(chan)) => {
                    chan.send(Some(azure_hl::current_graphics_metadata()));
                }

                Some(NewLayer(_id, new_size)) => {
                    self.create_new_layer(_id, new_size);
                }

                Some(SetLayerPageSize(id, new_size, epoch)) => {
                    self.set_layer_page_size(id, new_size, epoch);
                }

                Some(SetLayerClipRect(id, new_rect)) => {
                    self.set_layer_clip_rect(id, new_rect);
                }

                Some(DeleteLayer(id)) => {
                    self.delete_layer(id);
                }

                Some(Paint(id, new_layer_buffer_set, epoch)) => {
                    self.paint(id, new_layer_buffer_set, epoch);
                }

                Some(InvalidateRect(id, rect)) => {
                    self.invalidate_rect(id, rect);
                }

                Some(ScrollFragmentPoint(id, point)) => {
                    self.scroll_fragment_to_point(id, point);
                }
            }
        }
    }

    fn change_render_state(&mut self, render_state: RenderState) {
        self.window.set_render_state(render_state);
        if render_state == IdleRenderState {
            self.composite_ready = true;
        }
    }

    fn set_unrendered_color(&mut self, _id: PipelineId, color: Color) {
        match self.compositor_layer {
            Some(ref mut layer) => {
                layer.unrendered_color = color;
            }
            None => {}
        }
    }

    fn set_ids(&mut self,
               frame_tree: SendableFrameTree,
               response_chan: Chan<()>,
               new_constellation_chan: ConstellationChan) {
        response_chan.send(());

        // This assumes there is at most one child, which should be the case.
        match self.root_layer.first_child {
            Some(old_layer) => self.root_layer.remove_child(old_layer),
            None => {}
        }

        let layer = CompositorLayer::from_frame_tree(frame_tree,
                                                     self.opts.tile_size,
                                                     Some(10000000u),
                                                     self.opts.cpu_painting);
        self.root_layer.add_child_start(ContainerLayerKind(layer.root_layer));

        // If there's already a root layer, destroy it cleanly.
        match self.compositor_layer {
            None => {}
            Some(ref mut compositor_layer) => compositor_layer.clear_all(),
        }

        self.compositor_layer = Some(layer);

        // Initialize the new constellation channel by sending it the root window size.
        let window_size = self.window.size();
        let window_size = Size2D(window_size.width as uint,
                                 window_size.height as uint);
        new_constellation_chan.send(ResizedWindowMsg(window_size));

        self.constellation_chan = new_constellation_chan;
    }

    fn create_new_layer(&mut self, _id: PipelineId, new_size: Size2D<f32>) {
        // FIXME: This should create an additional layer instead of replacing the current one.
        // Once ResizeLayer messages are set up, we can switch to the new functionality.

        let p = match self.compositor_layer {
            Some(ref compositor_layer) => compositor_layer.pipeline.clone(),
            None => fail!("Compositor: Received new layer without initialized pipeline"),
        };
        let page_size = Size2D(new_size.width as f32, new_size.height as f32);
        let new_layer = CompositorLayer::new(p,
                                             Some(page_size),
                                             self.opts.tile_size,
                                             Some(10000000u),
                                             self.opts.cpu_painting);

        let current_child = self.root_layer.first_child;
        // This assumes there is at most one child, which should be the case.
        match current_child {
            Some(old_layer) => self.root_layer.remove_child(old_layer),
            None => {}
        }
        self.root_layer.add_child_start(ContainerLayerKind(new_layer.root_layer));
        self.compositor_layer = Some(new_layer);

        self.ask_for_tiles();
    }

    fn set_layer_page_size(&mut self,
                           id: PipelineId,
                           new_size: Size2D<f32>,
                           epoch: Epoch) {
        let (ask, move): (bool, bool) = match self.compositor_layer {
            Some(ref mut layer) => {
                let window_size = &self.window_size;
                let world_zoom = self.world_zoom;
                let page_window = Size2D(window_size.width as f32 / world_zoom,
                                         window_size.height as f32 / world_zoom);
                assert!(layer.resize(id, new_size, page_window, epoch));
                let move = self.fragment_point.take().map_default(false, |point| layer.move(point, page_window));

                (true, move)
            }
            None => (false, false)
        };

        if ask {
            self.recomposite_if(move);
            self.ask_for_tiles();
        }
    }

    fn set_layer_clip_rect(&mut self, id: PipelineId, new_rect: Rect<f32>) {
        let ask: bool = match self.compositor_layer {
            Some(ref mut layer) => {
                assert!(layer.set_clipping_rect(id, new_rect));
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
             id: PipelineId,
             new_layer_buffer_set: ~LayerBufferSet,
             epoch: Epoch) {
        debug!("osmain: received new frame");

        // From now on, if we destroy the buffers, they will leak.
        let mut new_layer_buffer_set = new_layer_buffer_set;
        new_layer_buffer_set.mark_will_leak();

        match self.compositor_layer {
            Some(ref mut layer) => {
                assert!(layer.add_buffers(&self.graphics_context,
                                          id,
                                          new_layer_buffer_set,
                                          epoch).is_none());
                self.recomposite = true;
            }
            None => {
                fail!("Compositor: given paint command with no CompositorLayer initialized");
            }
        }
        // TODO: Recycle the old buffers; send them back to the renderer to reuse if
        // it wishes.
    }

    fn invalidate_rect(&mut self, id: PipelineId, rect: Rect<uint>) {
        let ask: bool = match self.compositor_layer {
            Some(ref mut layer) => {
                let point = Point2D(rect.origin.x as f32,
                                    rect.origin.y as f32);
                let size = Size2D(rect.size.width as f32,
                                  rect.size.height as f32);
                layer.invalidate_rect(id, Rect(point, size));
                true
            }
            None => {
                // Nothing to do
                false
            }
        };

        if ask {
            self.ask_for_tiles();
        }
    }

    fn scroll_fragment_to_point(&mut self, id: PipelineId, point: Point2D<f32>) {
        let world_zoom = self.world_zoom;
        let page_window = Size2D(self.window_size.width as f32 / world_zoom,
                                 self.window_size.height as f32 / world_zoom);
        let (ask, move): (bool, bool) = match self.compositor_layer {
            Some(ref mut layer) if layer.pipeline.id == id && !layer.hidden => {

                (true, layer.move(point, page_window))
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
                    self.constellation_chan.send(ExitMsg);
                }
            }

            QuitWindowEvent => {
                debug!("shutting down the constellation for QuitWindowEvent");
                self.constellation_chan.send(ExitMsg);
            }
        }
    }

    fn on_resize_window_event(&mut self, width: uint, height: uint) {
        let new_size = Size2D(width, height);
        if self.window_size != new_size {
            debug!("osmain: window resized to {:u}x{:u}", width, height);
            self.window_size = new_size;
            self.constellation_chan.send(ResizedWindowMsg(new_size))
        } else {
            debug!("osmain: dropping window resize since size is still {:u}x{:u}", width, height);
        }
    }

    fn on_load_url_window_event(&self, url_string: ~str) {
        debug!("osmain: loading URL `{:s}`", url_string);
        let root_pipeline_id = match self.compositor_layer {
            Some(ref layer) => layer.pipeline.id.clone(),
            None => fail!("Compositor: Received LoadUrlWindowEvent without initialized compositor layers"),
        };

        let msg = LoadUrlMsg(root_pipeline_id, url::make_url(url_string.to_str(), None));
        self.constellation_chan.send(msg);
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
            scroll = layer.scroll(page_delta, page_cursor, page_window) || scroll;
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
        self.world_zoom = (self.world_zoom * magnification).max(&1.0);
        let world_zoom = self.world_zoom;

        self.root_layer.common.set_transform(identity().scale(world_zoom, world_zoom, 1f32));

        // Scroll as needed
        let page_delta = Point2D(window_size.width as f32 * (1.0 / world_zoom - 1.0 / old_world_zoom) * 0.5,
                                 window_size.height as f32 * (1.0 / world_zoom - 1.0 / old_world_zoom) * 0.5);
        // TODO: modify delta to snap scroll to pixels.
        let page_cursor = Point2D(-1f32, -1f32); // Make sure this hits the base layer
        let page_window = Size2D(window_size.width as f32 / world_zoom,
                                 window_size.height as f32 / world_zoom);
        for layer in self.compositor_layer.mut_iter() {
            layer.scroll(page_delta, page_cursor, page_window);
        }

        self.recomposite = true;
    }

    fn on_navigation_window_event(&self, direction: WindowNavigateMsg) {
        let direction = match direction {
            windowing::Forward => constellation_msg::Forward,
            windowing::Back => constellation_msg::Back,
        };
        self.constellation_chan.send(NavigateMsg(direction))
    }

    /// Get BufferRequests from each layer.
    fn ask_for_tiles(&mut self) {
        let world_zoom = self.world_zoom;
        let window_size_page = Size2D(self.window_size.width as f32 / world_zoom,
                                      self.window_size.height as f32 / world_zoom);
        for layer in self.compositor_layer.mut_iter() {
            if !layer.hidden {
                let rect = Rect(Point2D(0f32, 0f32), window_size_page);
                let recomposite = layer.get_buffer_request(&self.graphics_context, rect, world_zoom) ||
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
        let write_png = self.opts.output_file.is_some();
        if write_png {
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
            self.constellation_chan.send(ExitMsg);
        }

        self.window.present();

        let exit = self.opts.exit_after_load;
        if exit {
            debug!("shutting down the constellation for exit_after_load");
            self.constellation_chan.send(ExitMsg);
        }
    }

    fn recomposite_if(&mut self, result: bool) {
        self.recomposite = result || self.recomposite;
    }
}
