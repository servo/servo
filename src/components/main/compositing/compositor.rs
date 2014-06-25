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
use windowing::PinchZoomWindowEvent;

use azure::azure_hl::{SourceSurfaceMethods, Color};
use azure::azure_hl;
use geom::matrix::identity;
use geom::point::{Point2D, TypedPoint2D};
use geom::rect::Rect;
use geom::size::{Size2D, TypedSize2D};
use geom::scale_factor::ScaleFactor;
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
use servo_msg::constellation_msg::{PipelineId, ResizedWindowMsg, WindowSizeData};
use servo_msg::constellation_msg;
use servo_util::geometry::{DevicePixel, PagePx, ScreenPx, ViewportPx};
use servo_util::opts::Opts;
use servo_util::time::{profile, ProfilerChan};
use servo_util::{time, url};
use std::io::timer::sleep;
use std::path::Path;
use std::rc::Rc;
use time::precise_time_s;


pub struct IOCompositor {
    /// The application window.
    window: Rc<Window>,

    /// The port on which we receive messages.
    port: Receiver<Msg>,

    /// The render context.
    context: RenderContext,

    /// The root ContainerLayer.
    root_layer: Rc<ContainerLayer>,

    /// The root pipeline.
    root_pipeline: Option<CompositionPipeline>,

    /// The canvas to paint a page.
    scene: Scene,

    /// The application window size.
    window_size: TypedSize2D<DevicePixel, uint>,

    /// "Mobile-style" zoom that does not reflow the page.
    viewport_zoom: ScaleFactor<PagePx, ViewportPx, f32>,

    /// "Desktop-style" zoom that resizes the viewport to fit the window.
    /// See `ViewportPx` docs in util/geom.rs for details.
    page_zoom: ScaleFactor<ViewportPx, ScreenPx, f32>,

    /// The device pixel ratio for this window.
    hidpi_factor: ScaleFactor<ScreenPx, DevicePixel, f32>,

    /// The platform-specific graphics context.
    graphics_context: NativeCompositingGraphicsContext,

    /// Tracks whether the renderer has finished its first rendering
    composite_ready: bool,

    /// Tracks whether we are in the process of shutting down.
    shutting_down: bool,

    /// Tracks whether we should close compositor.
    done: bool,

    /// Tracks whether we need to re-composite a page.
    recomposite: bool,

    /// Tracks whether the zoom action has happend recently.
    zoom_action: bool,

    /// The time of the last zoom action has started.
    zoom_time: f64,

    /// Current display/reflow status of the page
    ready_state: ReadyState,

    /// Whether the page being rendered has loaded completely.
    /// Differs from ReadyState because we can finish loading (ready)
    /// many times for a single page.
    load_complete: bool,

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
    fn new(app: &Application,
               opts: Opts,
               port: Receiver<Msg>,
               constellation_chan: ConstellationChan,
               profiler_chan: ProfilerChan) -> IOCompositor {
        let window: Rc<Window> = WindowMethods::new(app, opts.output_file.is_none());

        // Create an initial layer tree.
        //
        // TODO: There should be no initial layer tree until the renderer creates one from the display
        // list. This is only here because we don't have that logic in the renderer yet.
        let root_layer = Rc::new(ContainerLayer());
        let window_size = window.framebuffer_size();
        let hidpi_factor = window.hidpi_factor();

        IOCompositor {
            window: window,
            port: port,
            opts: opts,
            context: rendergl::init_render_context(),
            root_layer: root_layer.clone(),
            root_pipeline: None,
            scene: Scene(ContainerLayerKind(root_layer), window_size.as_f32().to_untyped(), identity()),
            window_size: window_size,
            hidpi_factor: hidpi_factor,
            graphics_context: CompositorTask::create_graphics_context(),
            composite_ready: false,
            shutting_down: false,
            done: false,
            recomposite: false,
            page_zoom: ScaleFactor(1.0),
            viewport_zoom: ScaleFactor(1.0),
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
        compositor.update_zoom_transform();

        // Starts the compositor, which listens for messages on the specified port.
        compositor.run();
    }

    fn run (&mut self) {
        // Tell the constellation about the initial window size.
        self.send_window_size();

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
                Err(_) => break,
                Ok(_) => {},
            }
        }

        // Tell the profiler to shut down.
        let ProfilerChan(ref chan) = self.profiler_chan;
        chan.send(time::ExitMsg);
    }

    fn handle_message(&mut self) {
        loop {
            match (self.port.try_recv(), self.shutting_down) {
                (Err(_), _) => break,

                (Ok(Exit(chan)), _) => {
                    debug!("shutting down the constellation");
                    let ConstellationChan(ref con_chan) = self.constellation_chan;
                    con_chan.send(ExitMsg);
                    chan.send(());
                    self.shutting_down = true;
                }

                (Ok(ShutdownComplete), _) => {
                    debug!("constellation completed shutdown");
                    self.done = true;
                }

                (Ok(ChangeReadyState(ready_state)), false) => {
                    self.window.set_ready_state(ready_state);
                    self.ready_state = ready_state;
                }

                (Ok(ChangeRenderState(render_state)), false) => {
                    self.change_render_state(render_state);
                }

                (Ok(SetUnRenderedColor(pipeline_id, layer_id, color)), false) => {
                    self.set_unrendered_color(pipeline_id, layer_id, color);
                }

                (Ok(SetIds(frame_tree, response_chan, new_constellation_chan)), _) => {
                    self.set_ids(frame_tree, response_chan, new_constellation_chan);
                }

                (Ok(GetGraphicsMetadata(chan)), false) => {
                    chan.send(Some(azure_hl::current_graphics_metadata()));
                }

                (Ok(CreateRootCompositorLayerIfNecessary(pipeline_id, layer_id, size, color)),
                 false) => {
                    self.create_root_compositor_layer_if_necessary(pipeline_id, layer_id, size, color);
                }

                (Ok(CreateDescendantCompositorLayerIfNecessary(pipeline_id,
                                                                 layer_id,
                                                                 rect,
                                                                 scroll_behavior)),
                 false) => {
                    self.create_descendant_compositor_layer_if_necessary(pipeline_id,
                                                                         layer_id,
                                                                         rect,
                                                                         scroll_behavior);
                }

                (Ok(SetLayerPageSize(pipeline_id, layer_id, new_size, epoch)), false) => {
                    self.set_layer_page_size(pipeline_id, layer_id, new_size, epoch);
                }

                (Ok(SetLayerClipRect(pipeline_id, layer_id, new_rect)), false) => {
                    self.set_layer_clip_rect(pipeline_id, layer_id, new_rect);
                }

                (Ok(Paint(pipeline_id, layer_id, new_layer_buffer_set, epoch)), false) => {
                    self.paint(pipeline_id, layer_id, new_layer_buffer_set, epoch);
                }

                (Ok(ScrollFragmentPoint(pipeline_id, layer_id, point)), false) => {
                    self.scroll_fragment_to_point(pipeline_id, layer_id, point);
                }

                (Ok(LoadComplete(..)), false) => {
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

    fn set_unrendered_color(&mut self, pipeline_id: PipelineId, layer_id: LayerId, color: Color) {
        match self.compositor_layer {
            Some(ref mut layer) => layer.set_unrendered_color(pipeline_id, layer_id, color),
            None => false,
        };
    }

    fn set_ids(&mut self,
               frame_tree: SendableFrameTree,
               response_chan: Sender<()>,
               new_constellation_chan: ConstellationChan) {
        response_chan.send(());

        self.root_pipeline = Some(frame_tree.pipeline.clone());

        // Initialize the new constellation channel by sending it the root window size.
        self.constellation_chan = new_constellation_chan;
        self.send_window_size();
    }

    fn create_root_compositor_layer_if_necessary(&mut self,
                                                 id: PipelineId,
                                                 layer_id: LayerId,
                                                 size: Size2D<f32>,
                                                 unrendered_color: Color) {
        let (root_pipeline, root_layer_id) = match self.compositor_layer {
            Some(ref compositor_layer) if compositor_layer.pipeline.id == id => {
                (compositor_layer.pipeline.clone(), compositor_layer.id_of_first_child())
            }
            _ => {
                match self.root_pipeline {
                    Some(ref root_pipeline) => {
                        (root_pipeline.clone(), LayerId::null())
                    },
                    _ => fail!("Compositor: Received new layer without initialized pipeline"),
                }
            }
        };

        if layer_id != root_layer_id {
            let root_pipeline_id = root_pipeline.id;
            let mut new_layer = CompositorLayer::new_root(root_pipeline,
                                                          size,
                                                          self.opts.tile_size,
                                                          self.opts.cpu_painting);
            new_layer.unrendered_color = unrendered_color;

            self.root_layer.remove_all_children();

            let new_layer_id = new_layer.id;
            assert!(new_layer.add_child_if_necessary(self.root_layer.clone(),
                                                     root_pipeline_id,
                                                     new_layer_id,
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
                let compositor_layer_id = compositor_layer.id;
                let page_size = compositor_layer.page_size.unwrap();
                assert!(compositor_layer.add_child_if_necessary(self.root_layer.clone(),
                                                                pipeline_id,
                                                                compositor_layer_id,
                                                                layer_id,
                                                                rect,
                                                                page_size,
                                                                scroll_policy))
            }
            None => fail!("Compositor: Received new layer without initialized pipeline"),
        };

        self.ask_for_tiles();
    }

    /// The size of the content area in CSS px at the current zoom level
    fn page_window(&self) -> TypedSize2D<PagePx, f32> {
        self.window_size.as_f32() / self.device_pixels_per_page_px()
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

    fn set_layer_page_size(&mut self,
                           pipeline_id: PipelineId,
                           layer_id: LayerId,
                           new_size: Size2D<f32>,
                           epoch: Epoch) {
        let page_window = self.page_window();
        let (ask, move): (bool, bool) = match self.compositor_layer {
            Some(ref mut layer) => {
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

    fn paint(&mut self,
             pipeline_id: PipelineId,
             layer_id: LayerId,
             new_layer_buffer_set: Box<LayerBufferSet>,
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
        let page_window = self.page_window();
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

    fn on_resize_window_event(&mut self, new_size: TypedSize2D<DevicePixel, uint>) {
        // A size change could also mean a resolution change.
        let new_hidpi_factor = self.window.hidpi_factor();
        if self.hidpi_factor != new_hidpi_factor {
            self.hidpi_factor = new_hidpi_factor;
            self.update_zoom_transform();
        }
        if self.window_size != new_size {
            debug!("osmain: window resized to {:?}", new_size);
            self.window_size = new_size;
            self.send_window_size();
        } else {
            debug!("osmain: dropping window resize since size is still {:?}", new_size);
        }
    }

    fn on_load_url_window_event(&mut self, url_string: String) {
        debug!("osmain: loading URL `{:s}`", url_string);
        self.load_complete = false;
        let root_pipeline_id = match self.compositor_layer {
            Some(ref layer) => layer.pipeline.id.clone(),
            None => fail!("Compositor: Received LoadUrlWindowEvent without initialized compositor layers"),
        };

        let msg = LoadUrlMsg(root_pipeline_id, url::parse_url(url_string.as_slice(), None));
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(msg);
    }

    fn on_mouse_window_event_class(&self, mouse_window_event: MouseWindowEvent) {
        let scale = self.device_pixels_per_page_px();
        let point = match mouse_window_event {
            MouseWindowClickEvent(_, p) => p / scale,
            MouseWindowMouseDownEvent(_, p) => p / scale,
            MouseWindowMouseUpEvent(_, p) => p / scale,
        };
        for layer in self.compositor_layer.iter() {
            layer.send_mouse_event(mouse_window_event, point);
        }
    }

    fn on_mouse_window_move_event_class(&self, cursor: TypedPoint2D<DevicePixel, f32>) {
        let scale = self.device_pixels_per_page_px();
        for layer in self.compositor_layer.iter() {
            layer.send_mouse_move_event(cursor / scale);
        }
    }

    fn on_scroll_window_event(&mut self,
                              delta: TypedPoint2D<DevicePixel, f32>,
                              cursor: TypedPoint2D<DevicePixel, i32>) {
        let scale = self.device_pixels_per_page_px();
        // TODO: modify delta to snap scroll to pixels.
        let page_delta = delta / scale;
        let page_cursor = cursor.as_f32() / scale;
        let page_window = self.page_window();
        let mut scroll = false;
        for layer in self.compositor_layer.mut_iter() {
            scroll = layer.handle_scroll_event(page_delta, page_cursor, page_window) || scroll;
        }
        self.recomposite_if(scroll);
        self.ask_for_tiles();
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
        self.root_layer.common.borrow_mut().set_transform(identity().scale(scale.get(), scale.get(), 1f32));
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
        let window_size = self.window_size.as_f32();

        self.viewport_zoom = ScaleFactor((self.viewport_zoom.get() * magnification).max(1.0));
        let viewport_zoom = self.viewport_zoom;

        self.update_zoom_transform();

        // Scroll as needed
        let page_delta = TypedPoint2D(
            window_size.width.get() * (viewport_zoom.inv() - old_viewport_zoom.inv()).get() * 0.5,
            window_size.height.get() * (viewport_zoom.inv() - old_viewport_zoom.inv()).get() * 0.5);
        // TODO: modify delta to snap scroll to pixels.
        let page_cursor = TypedPoint2D(-1f32, -1f32); // Make sure this hits the base layer
        let page_window = self.page_window();

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
        let scale = self.device_pixels_per_page_px();
        let page_window = self.page_window();
        for layer in self.compositor_layer.mut_iter() {
            if !layer.hidden {
                let rect = Rect(Point2D(0f32, 0f32), page_window.to_untyped());
                let recomposite = layer.get_buffer_request(&self.graphics_context,
                                                           rect,
                                                           scale.get()) ||
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
            self.scene.size = self.window_size.as_f32().to_untyped();
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
            let (width, height) = (self.window_size.width.get(), self.window_size.height.get());
            let path = from_str::<Path>(self.opts.output_file.get_ref().as_slice()).unwrap();
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

