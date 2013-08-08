/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use platform::{Application, Window};
use script::dom::event::ResizeEvent;
use script::script_task::{LoadMsg, NavigateMsg, SendEventMsg};

pub use windowing;
use windowing::{ApplicationMethods, WindowEvent, WindowMethods};
use windowing::{IdleWindowEvent, ResizeWindowEvent, LoadUrlWindowEvent, MouseWindowEventClass};
use windowing::{ScrollWindowEvent, ZoomWindowEvent, NavigationWindowEvent, FinishedWindowEvent};
use windowing::{QuitWindowEvent, MouseWindowClickEvent, MouseWindowMouseDownEvent, MouseWindowMouseUpEvent};

use servo_msg::compositor_msg::{RenderListener, LayerBufferSet, RenderState};
use servo_msg::compositor_msg::{ReadyState, ScriptListener};
use servo_msg::constellation_msg::PipelineId;
use servo_msg::constellation_msg;
use gfx::opts::Opts;

use azure::azure_hl::{DataSourceSurface, DrawTarget, SourceSurfaceMethods, current_gl_context};
use azure::azure::AzGLContext;
use std::cell::Cell;
use std::comm;
use std::comm::{Chan, SharedChan, Port};
use std::num::Orderable;
use std::task;
use std::uint;
use std::vec;
use extra::uv_global_loop;
use extra::timer;
use geom::matrix::identity;
use geom::point::Point2D;
use geom::size::Size2D;
use geom::rect::Rect;
use layers::layers::{ARGB32Format, ContainerLayer, ContainerLayerKind, Format};
use layers::layers::{ImageData, WithDataFn};
use layers::rendergl;
use layers::scene::Scene;
use opengles::gl2;
use png;
use servo_util::{time, url};
use servo_util::time::profile;
use servo_util::time::ProfilerChan;

use extra::time::precise_time_s;
use extra::arc;

use constellation::SendableFrameTree;
use pipeline::Pipeline;
use compositing::compositor_layer::CompositorLayer;

mod quadtree;
mod compositor_layer;


/// The implementation of the layers-based compositor.
#[deriving(Clone)]
pub struct CompositorChan {
    /// A channel on which messages can be sent to the compositor.
    chan: SharedChan<Msg>,
}

/// Implementation of the abstract `ScriptListener` interface.
impl ScriptListener for CompositorChan {

    fn set_ready_state(&self, ready_state: ReadyState) {
        let msg = ChangeReadyState(ready_state);
        self.chan.send(msg);
    }

}

/// Implementation of the abstract `RenderListener` interface.
impl RenderListener for CompositorChan {

    fn get_gl_context(&self) -> AzGLContext {
        let (port, chan) = comm::stream();
        self.chan.send(GetGLContext(chan));
        port.recv()
    }

    fn paint(&self, id: PipelineId, layer_buffer_set: arc::ARC<LayerBufferSet>) {
        self.chan.send(Paint(id, layer_buffer_set))
    }

    fn new_layer(&self, id: PipelineId, page_size: Size2D<uint>) {
        self.chan.send(NewLayer(id, page_size))
    }
    fn resize_layer(&self, id: PipelineId, page_size: Size2D<uint>) {
        self.chan.send(ResizeLayer(id, page_size))
    }
    fn delete_layer(&self, id: PipelineId) {
        self.chan.send(DeleteLayer(id))
    }

    fn set_render_state(&self, render_state: RenderState) {
        self.chan.send(ChangeRenderState(render_state))
    }
}

impl CompositorChan {

    pub fn new(chan: Chan<Msg>) -> CompositorChan {
        CompositorChan {
            chan: SharedChan::new(chan),
        }
    }

    pub fn send(&self, msg: Msg) {
        self.chan.send(msg);
    }

    pub fn get_size(&self) -> Size2D<int> {
        let (port, chan) = comm::stream();
        self.chan.send(GetSize(chan));
        port.recv()
    }
}

/// Messages to the compositor.
pub enum Msg {
    /// Requests that the compositor shut down.
    Exit,
    /// Requests the window size
    GetSize(Chan<Size2D<int>>),
    /// Requests the compositors GL context.
    GetGLContext(Chan<AzGLContext>),

    // TODO: Attach epochs to these messages
    /// Alerts the compositor that there is a new layer to be rendered.
    NewLayer(PipelineId, Size2D<uint>),
    /// Alerts the compositor that the specified layer has changed size.
    ResizeLayer(PipelineId, Size2D<uint>),
    /// Alerts the compositor that the specified layer has been deleted.
    DeleteLayer(PipelineId),

    /// Requests that the compositor paint the given layer buffer set for the given page size.
    Paint(PipelineId, arc::ARC<LayerBufferSet>),
    /// Alerts the compositor to the current status of page loading.
    ChangeReadyState(ReadyState),
    /// Alerts the compositor to the current status of rendering.
    ChangeRenderState(RenderState),
    /// Sets the channel to the current layout and render tasks, along with their id
    SetIds(SendableFrameTree, Chan<()>),
}

/// Azure surface wrapping to work with the layers infrastructure.
struct AzureDrawTargetImageData {
    draw_target: DrawTarget,
    data_source_surface: DataSourceSurface,
    size: Size2D<uint>,
}

impl ImageData for AzureDrawTargetImageData {
    fn size(&self) -> Size2D<uint> {
        self.size
    }
    fn stride(&self) -> uint {
        self.data_source_surface.stride() as uint
    }
    fn format(&self) -> Format {
        // FIXME: This is not always correct. We should query the Azure draw target for the format.
        ARGB32Format
    }
    fn with_data(&self, f: WithDataFn) {
        do self.data_source_surface.with_data |data| {
            f(data);
        }
    }
}

pub struct CompositorTask {
    opts: Opts,
    port: Port<Msg>,
    profiler_chan: ProfilerChan,
    shutdown_chan: SharedChan<()>,
}

impl CompositorTask {
    pub fn new(opts: Opts,
               port: Port<Msg>,
               profiler_chan: ProfilerChan,
               shutdown_chan: Chan<()>)
               -> CompositorTask {
        CompositorTask {
            opts: opts,
            port: port,
            profiler_chan: profiler_chan,
            shutdown_chan: SharedChan::new(shutdown_chan),
        }
    }

    /// Starts the compositor, which listens for messages on the specified port. 
    pub fn create(opts: Opts,
                  port: Port<Msg>,
                  profiler_chan: ProfilerChan,
                  shutdown_chan: Chan<()>) {
        let port = Cell::new(port);
        let shutdown_chan = Cell::new(shutdown_chan);
        let opts = Cell::new(opts);
        do on_osmain {
            let compositor_task = CompositorTask::new(opts.take(),
                                                      port.take(),
                                                      profiler_chan.clone(),
                                                      shutdown_chan.take());
            debug!("preparing to enter main loop");
            compositor_task.run_main_loop();
        };
    }

    fn run_main_loop(&self) {
        let app: Application = ApplicationMethods::new();
        let window: @mut Window = WindowMethods::new(&app);

        // Create an initial layer tree.
        //
        // TODO: There should be no initial layer tree until the renderer creates one from the display
        // list. This is only here because we don't have that logic in the renderer yet.
        let context = rendergl::init_render_context();
        let root_layer = @mut ContainerLayer();
        let window_size = window.size();
        let mut scene = Scene(ContainerLayerKind(root_layer), window_size, identity());
        let mut window_size = Size2D(window_size.width as int, window_size.height as int);
        let mut done = false;
        let mut recomposite = false;

        // Keeps track of the current zoom factor
        let mut world_zoom = 1f32;
        let mut zoom_action = false;
        let mut zoom_time = 0f;

        // Channel to the outermost frame's pipeline.
        // FIXME: Events are only forwarded to this pipeline, but they should be
        // routed to the appropriate pipeline via the constellation.
        let mut pipeline: Option<Pipeline> = None;

        // The root CompositorLayer
        let mut compositor_layer: Option<CompositorLayer> = None;

        // Get BufferRequests from each layer.
        let ask_for_tiles = || {
            let window_size_page = Size2D(window_size.width as f32 / world_zoom,
                                          window_size.height as f32 / world_zoom);
            for compositor_layer.mut_iter().advance |layer| {
                recomposite = layer.get_buffer_request(Rect(Point2D(0f32, 0f32), window_size_page),
                                                       world_zoom) || recomposite;
            }
        };
        
        let check_for_messages: &fn(&Port<Msg>) = |port: &Port<Msg>| {
            // Handle messages
            while port.peek() {
                match port.recv() {
                    Exit => done = true,

                    ChangeReadyState(ready_state) => window.set_ready_state(ready_state),
                    ChangeRenderState(render_state) => window.set_render_state(render_state),

                    SetIds(frame_tree, response_chan) => {
                        pipeline = Some(frame_tree.pipeline);
                        response_chan.send(());
                    }

                    GetSize(chan) => {
                        let size = window.size();
                        chan.send(Size2D(size.width as int, size.height as int));
                    }

                    GetGLContext(chan) => chan.send(current_gl_context()),

                    NewLayer(_id, new_size) => {
                        // FIXME: This should create an additional layer instead of replacing the current one.
                        // Once ResizeLayer messages are set up, we can switch to the new functionality.

                        let p = match pipeline {
                            Some(ref pipeline) => pipeline,
                            None => fail!("Compositor: Received new layer without initialized pipeline"),
                        };
                        let page_size = Size2D(new_size.width as f32, new_size.height as f32);
                        let new_layer = CompositorLayer::new(p.clone(), Some(page_size),
                                                             self.opts.tile_size, Some(10000000u));
                        
                        let current_child = root_layer.first_child;
                        // This assumes there is at most one child, which should be the case.
                        match current_child {
                            Some(old_layer) => root_layer.remove_child(old_layer),
                            None => {}
                        }
                        root_layer.add_child(ContainerLayerKind(new_layer.root_layer));
                        compositor_layer = Some(new_layer);

                        ask_for_tiles();
                    }

                    ResizeLayer(id, new_size) => {
                        match compositor_layer {
                            Some(ref mut layer) => {
                                let page_window = Size2D(window_size.width as f32 / world_zoom,
                                                         window_size.height as f32 / world_zoom);
                                assert!(layer.resize(id, Size2D(new_size.width as f32,
                                                                new_size.height as f32),
                                                     page_window));
                                ask_for_tiles();
                            }
                            None => {}
                        }
                    }

                    DeleteLayer(id) => {
                        match compositor_layer {
                            Some(ref mut layer) => {
                                assert!(layer.delete(id));
                                ask_for_tiles();
                            }
                            None => {}
                        }
                    }

                    Paint(id, new_layer_buffer_set) => {
                        debug!("osmain: received new frame"); 

                        match compositor_layer {
                            Some(ref mut layer) => {
                                assert!(layer.add_buffers(id, new_layer_buffer_set.get()));
                                recomposite = true;
                            }
                            None => {
                                fail!("Compositor: given paint command with no CompositorLayer initialized");
                            }
                        }
                        // TODO: Recycle the old buffers; send them back to the renderer to reuse if
                        // it wishes.
                    }
                }
            }
        };

        let check_for_window_messages: &fn(WindowEvent) = |event| {
            match event {
                IdleWindowEvent => {}

                ResizeWindowEvent(width, height) => {
                    let new_size = Size2D(width as int, height as int);
                    if window_size != new_size {
                        debug!("osmain: window resized to %ux%u", width, height);
                        window_size = new_size;
                        match pipeline {
                            Some(ref pipeline) => pipeline.script_chan.send(SendEventMsg(pipeline.id.clone(), ResizeEvent(width, height))),
                            None => error!("Compositor: Recieved resize event without initialized layout chan"),
                        }
                    } else {
                        debug!("osmain: dropping window resize since size is still %ux%u", width, height);
                    }
                }
                
                LoadUrlWindowEvent(url_string) => {
                    debug!("osmain: loading URL `%s`", url_string);
                    match pipeline {
                        Some(ref pipeline) => pipeline.script_chan.send(LoadMsg(pipeline.id.clone(), url::make_url(url_string.to_str(), None))),
                        None => error!("Compositor: Recieved loadurl event without initialized layout chan"),
                    }
                }
                
                MouseWindowEventClass(mouse_window_event) => {
                    let point = match mouse_window_event {
                        MouseWindowClickEvent(_, p) => Point2D(p.x / world_zoom, p.y / world_zoom),
                        MouseWindowMouseDownEvent(_, p) => Point2D(p.x / world_zoom, p.y / world_zoom),
                        MouseWindowMouseUpEvent(_, p) => Point2D(p.x / world_zoom, p.y / world_zoom),
                    };
                    for compositor_layer.iter().advance |layer| {
                        layer.send_mouse_event(mouse_window_event, point);
                    }
                }
                
                ScrollWindowEvent(delta, cursor) => {
                    // TODO: modify delta to snap scroll to pixels.
                    let page_delta = Point2D(delta.x as f32 / world_zoom, delta.y as f32 / world_zoom);
                    let page_cursor: Point2D<f32> = Point2D(cursor.x as f32 / world_zoom,
                                                            cursor.y as f32 / world_zoom);
                    let page_window = Size2D(window_size.width as f32 / world_zoom,
                                             window_size.height as f32 / world_zoom);
                    for compositor_layer.mut_iter().advance |layer| {
                        recomposite = layer.scroll(page_delta, page_cursor, page_window) || recomposite;
                    }
                    ask_for_tiles();
                }
                
                ZoomWindowEvent(magnification) => {
                    zoom_action = true;
                    zoom_time = precise_time_s();
                    let old_world_zoom = world_zoom;

                    // Determine zoom amount
                    world_zoom = (world_zoom * magnification).max(&1.0);            
                    root_layer.common.set_transform(identity().scale(world_zoom, world_zoom, 1f32));
                    
                    // Scroll as needed
                    let page_delta = Point2D(window_size.width as f32 * (1.0 / world_zoom - 1.0 / old_world_zoom) * 0.5,
                                             window_size.height as f32 * (1.0 / world_zoom - 1.0 / old_world_zoom) * 0.5);
                    // TODO: modify delta to snap scroll to pixels.
                    let page_cursor = Point2D(-1f32, -1f32); // Make sure this hits the base layer
                    let page_window = Size2D(window_size.width as f32 / world_zoom,
                                             window_size.height as f32 / world_zoom);
                    for compositor_layer.mut_iter().advance |layer| {
                        layer.scroll(page_delta, page_cursor, page_window);
                    }

                    recomposite = true;
                }

                NavigationWindowEvent(direction) => {
                    let direction = match direction {
                        windowing::Forward => constellation_msg::Forward,
                        windowing::Back => constellation_msg::Back,
                    };
                    match pipeline {
                        Some(ref pipeline) => pipeline.script_chan.send(NavigateMsg(direction)),
                        None => error!("Compositor: Recieved navigation event without initialized layout chan"),
                    }
                }
                
                FinishedWindowEvent => {
                    if self.opts.exit_after_load {
                        done = true;
                    }
                }
                
                QuitWindowEvent => {
                    done = true;
                }
            }
        };
        
        
        let profiler_chan = self.profiler_chan.clone();
        let write_png = self.opts.output_file.is_some();
        let exit = self.opts.exit_after_load;
        let composite = || {
            do profile(time::CompositingCategory, profiler_chan.clone()) {
                debug!("compositor: compositing");
                // Adjust the layer dimensions as necessary to correspond to the size of the window.
                scene.size = window.size();

                // Render the scene.
                rendergl::render_scene(context, &scene);
            }

            // Render to PNG. We must read from the back buffer (ie, before
            // window.present()) as OpenGL ES 2 does not have glReadBuffer().
            if write_png {
                let (width, height) = (window_size.width as uint, window_size.height as uint);
                let path = Path(*self.opts.output_file.get_ref());
                let mut pixels = gl2::read_pixels(0, 0,
                                                  width as gl2::GLsizei,
                                                  height as gl2::GLsizei,
                                                  gl2::RGB, gl2::UNSIGNED_BYTE);
                // flip image vertically (texture is upside down)
                let orig_pixels = pixels.clone();
                let stride = width * 3;
                for uint::range(0, height) |y| {
                    let dst_start = y * stride;
                    let src_start = (height - y - 1) * stride;
                    vec::bytes::copy_memory(pixels.mut_slice(dst_start, dst_start + stride),
                                            orig_pixels.slice(src_start, src_start + stride),
                                            stride);
                }
                let img = png::Image {
                    width: width as u32,
                    height: height as u32,
                    color_type: png::RGB8,
                    pixels: pixels,
                };
                let res = png::store_png(&img, &path);
                assert!(res.is_ok());

                done = true;
            }

            window.present();

            if exit { done = true; }
        };

        // Enter the main event loop.
        while !done {
            // Check for new messages coming from the rendering task.
            check_for_messages(&self.port);

            // Check for messages coming from the windowing system.
            check_for_window_messages(window.recv());

            if recomposite {
                recomposite = false;
                composite();
            }

            timer::sleep(&uv_global_loop::get(), 10);

            // If a pinch-zoom happened recently, ask for tiles at the new resolution
            if zoom_action && precise_time_s() - zoom_time > 0.3 {
                zoom_action = false;
                ask_for_tiles();
            }

        }

        self.shutdown_chan.send(())
    }
}

/// A function for spawning into the platform's main thread.
fn on_osmain(f: ~fn()) {
    // FIXME: rust#6399
    let mut main_task = task::task();
    main_task.sched_mode(task::PlatformThread);
    do main_task.spawn {
        f();
    }
}

