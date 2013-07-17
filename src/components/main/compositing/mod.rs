/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use platform::{Application, Window};
use script::dom::event::{Event, ClickEvent, MouseDownEvent, MouseUpEvent, ResizeEvent};
use script::script_task::{LoadMsg, NavigateMsg, SendEventMsg};
use script::layout_interface::{LayoutChan, RouteScriptMsg};

use windowing::{ApplicationMethods, WindowEvent, WindowMethods};
use windowing::{IdleWindowEvent, ResizeWindowEvent, LoadUrlWindowEvent, MouseWindowEventClass};
use windowing::{ScrollWindowEvent, ZoomWindowEvent, NavigationWindowEvent, FinishedWindowEvent};
use windowing::{QuitWindowEvent, MouseWindowClickEvent, MouseWindowMouseDownEvent, MouseWindowMouseUpEvent};

use servo_msg::compositor_msg::{RenderListener, LayerBuffer, LayerBufferSet, RenderState};
use servo_msg::compositor_msg::{ReadyState, ScriptListener};
use servo_msg::constellation_msg::{CompositorAck, ConstellationChan};
use servo_msg::constellation_msg;
use gfx::render_task::{RenderChan, ReRenderMsg};
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
use layers::layers::{TextureLayerKind, TextureLayer, TextureManager};
use layers::rendergl;
use layers::scene::Scene;
use opengles::gl2;
use png;
use servo_util::{time, url};
use servo_util::time::profile;
use servo_util::time::ProfilerChan;

use extra::arc;
pub use windowing;

use extra::time::precise_time_s;
use compositing::quadtree::Quadtree;
mod quadtree;

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

    fn paint(&self, id: uint, layer_buffer_set: arc::ARC<LayerBufferSet>, new_size: Size2D<uint>) {
        self.chan.send(Paint(id, layer_buffer_set, new_size))
    }

    fn new_layer(&self, page_size: Size2D<uint>, tile_size: uint) {
        self.chan.send(NewLayer(page_size, tile_size))
    }
    fn resize_layer(&self, page_size: Size2D<uint>) {
        self.chan.send(ResizeLayer(page_size))
    }
    fn delete_layer(&self) {
        self.chan.send(DeleteLayer)
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

    // TODO: Attach layer ids and epochs to these messages
    /// Alerts the compositor that there is a new layer to be rendered.
    NewLayer(Size2D<uint>, uint),
    /// Alerts the compositor that the current layer has changed size.
    ResizeLayer(Size2D<uint>),
    /// Alerts the compositor that the current layer has been deleted.
    DeleteLayer,

    /// Requests that the compositor paint the given layer buffer set for the given page size.
    Paint(uint, arc::ARC<LayerBufferSet>, Size2D<uint>),
    /// Alerts the compositor to the current status of page loading.
    ChangeReadyState(ReadyState),
    /// Alerts the compositor to the current status of rendering.
    ChangeRenderState(RenderState),
    /// Sets the channel to the current layout and render tasks, along with their id
    SetLayoutRenderChans(LayoutChan, RenderChan , uint, ConstellationChan)
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
        let mut done = false;
        let mut recomposite = false;

        // FIXME: This should not be a separate offset applied after the fact but rather should be
        // applied to the layers themselves on a per-layer basis. However, this won't work until scroll
        // positions are sent to content.
        let mut world_offset = Point2D(0f32, 0f32);
        let mut page_size = Size2D(0f32, 0f32);
        let mut window_size = Size2D(window_size.width as int,
                                     window_size.height as int);

        // Keeps track of the current zoom factor
        let mut world_zoom = 1f32;
        // Keeps track of local zoom factor. Reset to 1 after a rerender event.
        let mut local_zoom = 1f32;
        // Channel to the current renderer.
        // FIXME: This probably shouldn't be stored like this.

        let mut render_chan: Option<RenderChan> = None;
        let mut pipeline_id: Option<uint> = None;
        let mut layout_chan: Option<LayoutChan> = None;

        // Quadtree for this layer
        // FIXME: This should be one-per-layer
        let mut quadtree: Option<Quadtree<~LayerBuffer>> = None;
        
        // Keeps track of if we have performed a zoom event and how recently.
        let mut zoom_action = false;
        let mut zoom_time = 0f;

        // Extract tiles from the given quadtree and build and display the render tree.
        let build_layer_tree: &fn(&Quadtree<~LayerBuffer>) = |quad: &Quadtree<~LayerBuffer>| {
            // Iterate over the children of the container layer.
            let mut current_layer_child = root_layer.first_child;
            
            // Delete old layer
            while current_layer_child.is_some() {
                let trash = current_layer_child.get();
                do current_layer_child.get().with_common |common| {
                    current_layer_child = common.next_sibling;
                }
                root_layer.remove_child(trash);
            }

            let all_tiles = quad.get_all_tiles();
            for all_tiles.iter().advance |buffer| {
                let width = buffer.screen_pos.size.width as uint;
                let height = buffer.screen_pos.size.height as uint;
                debug!("osmain: compositing buffer rect %?", &buffer.rect);

                // Find or create a texture layer.
                let texture_layer;
                current_layer_child = match current_layer_child {
                    None => {
                        debug!("osmain: adding new texture layer");
                        texture_layer = @mut TextureLayer::new(@buffer.draw_target.clone() as @TextureManager,
                                                               buffer.screen_pos.size);
                        root_layer.add_child(TextureLayerKind(texture_layer));
                        None
                    }
                    Some(TextureLayerKind(existing_texture_layer)) => {
                        texture_layer = existing_texture_layer;
                        texture_layer.manager = @buffer.draw_target.clone() as @TextureManager;
                        
                        // Move on to the next sibling.
                        do current_layer_child.get().with_common |common| {
                            common.next_sibling
                        }
                    }
                    Some(_) => fail!(~"found unexpected layer kind"),
                };

                let origin = buffer.rect.origin;
                let origin = Point2D(origin.x as f32, origin.y as f32);
                
                // Set the layer's transform.
                let transform = identity().translate(origin.x * world_zoom, origin.y * world_zoom, 0.0);
                let transform = transform.scale(width as f32 * world_zoom / buffer.resolution, height as f32 * world_zoom / buffer.resolution, 1.0);
                texture_layer.common.set_transform(transform);

            }
            
            // Reset zoom
            local_zoom = 1f32;
            root_layer.common.set_transform(identity().translate(-world_offset.x,
                                                                 -world_offset.y,
                                                                 0.0));
            recomposite = true;
        };

        let ask_for_tiles = || {
            match quadtree {
                Some(ref mut quad) => {
                    let (tile_request, redisplay) = quad.get_tile_rects(Rect(Point2D(world_offset.x as int,
                                                                                     world_offset.y as int),
                                                                             window_size), world_zoom);

                    if !tile_request.is_empty() {
                        match render_chan {
                            Some(ref chan) => {
                                chan.send(ReRenderMsg(tile_request, world_zoom));
                            }
                            _ => {
                                println("Warning: Compositor: Cannot send tile request, no render chan initialized");
                            }
                        }
                    } else if redisplay {
                        build_layer_tree(quad);
                    }
                }
                _ => {
                    fail!("Compositor: Tried to ask for tiles without an initialized quadtree");
                }
            }
        };
        
        let check_for_messages: &fn(&Port<Msg>) = |port: &Port<Msg>| {
            // Handle messages
            while port.peek() {
                match port.recv() {
                    Exit => done = true,

                    ChangeReadyState(ready_state) => window.set_ready_state(ready_state),
                    ChangeRenderState(render_state) => window.set_render_state(render_state),

                    SetLayoutRenderChans(new_layout_chan,
                                         new_render_chan,
                                         new_pipeline_id,
                                         response_chan) => {
                        layout_chan = Some(new_layout_chan);
                        render_chan = Some(new_render_chan);
                        pipeline_id = Some(new_pipeline_id);
                        response_chan.send(CompositorAck(new_pipeline_id));
                    }

                    GetSize(chan) => {
                        let size = window.size();
                        chan.send(Size2D(size.width as int, size.height as int));
                    }

                    GetGLContext(chan) => chan.send(current_gl_context()),

                    NewLayer(new_size, tile_size) => {
                        page_size = Size2D(new_size.width as f32, new_size.height as f32);
                        quadtree = Some(Quadtree::new(new_size.width.max(&(window_size.width as uint)),
                                                       new_size.height.max(&(window_size.height as uint)),
                                                       tile_size, Some(10000000u)));
                        ask_for_tiles();
                        
                    }
                    ResizeLayer(new_size) => {
                        page_size = Size2D(new_size.width as f32, new_size.height as f32);
                        // TODO: update quadtree, ask for tiles
                    }
                    DeleteLayer => {
                        // TODO: create secondary layer tree, keep displaying until new tiles come in
                    }

                    Paint(id, new_layer_buffer_set, new_size) => {
                        match pipeline_id {
                            Some(pipeline_id) => if id != pipeline_id { loop; },
                            None => { loop; },
                        }
                        
                        debug!("osmain: received new frame"); 

                        let quad;
                        match quadtree {
                            Some(ref mut q) => quad = q,
                            None => fail!("Compositor: given paint command with no quadtree initialized"),
                        }
                        
                        let new_layer_buffer_set = new_layer_buffer_set.get();
                        for new_layer_buffer_set.buffers.iter().advance |buffer| {
                            // FIXME: Don't copy the buffers here
                            quad.add_tile(buffer.screen_pos.origin.x, buffer.screen_pos.origin.y,
                                          buffer.resolution, ~buffer.clone());
                        }
                        
                        page_size = Size2D(new_size.width as f32, new_size.height as f32);
                        
                        build_layer_tree(quad);
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
                        match layout_chan {
                            Some(ref chan) => chan.send(RouteScriptMsg(SendEventMsg(ResizeEvent(width, height)))),
                            None => error!("Compositor: Recieved resize event without initialized layout chan"),
                        }
                    } else {
                        debug!("osmain: dropping window resize since size is still %ux%u", width, height);
                    }
                }
                
                LoadUrlWindowEvent(url_string) => {
                    debug!("osmain: loading URL `%s`", url_string);
                    match layout_chan {
                        Some(ref chan) => chan.send(RouteScriptMsg(LoadMsg(url::make_url(url_string.to_str(), None)))),
                        None => error!("Compositor: Recieved loadurl event without initialized layout chan"),
                    }
                }
                
                MouseWindowEventClass(mouse_window_event) => {
                    let event: Event;
                    let world_mouse_point = |layer_mouse_point: Point2D<f32>| {
                        layer_mouse_point + world_offset
                    };
                    match mouse_window_event {
                        MouseWindowClickEvent(button, layer_mouse_point) => {
                            event = ClickEvent(button, world_mouse_point(layer_mouse_point));
                        }
                        MouseWindowMouseDownEvent(button, layer_mouse_point) => {
                            event = MouseDownEvent(button, world_mouse_point(layer_mouse_point));
                        }
                        MouseWindowMouseUpEvent(button, layer_mouse_point) => {
                            event = MouseUpEvent(button, world_mouse_point(layer_mouse_point));
                        }
                    }
                    match layout_chan {
                        Some(ref chan) => chan.send(RouteScriptMsg(SendEventMsg(event))),
                        None => error!("Compositor: Recieved mouse event without initialized layout chan"),
                    }
                }
                
                ScrollWindowEvent(delta) => {
                    // FIXME (Rust #2528): Can't use `-=`.
                    let world_offset_copy = world_offset;
                    world_offset = world_offset_copy - delta;
                    
                    // Clamp the world offset to the screen size.
                    let max_x = (page_size.width * world_zoom - window_size.width as f32).max(&0.0);
                    world_offset.x = world_offset.x.clamp(&0.0, &max_x).round();
                    let max_y = (page_size.height * world_zoom - window_size.height as f32).max(&0.0);
                    world_offset.y = world_offset.y.clamp(&0.0, &max_y).round();
                    
                    debug!("compositor: scrolled to %?", world_offset);
                    
                    
                    let mut scroll_transform = identity();
                    
                    scroll_transform = scroll_transform.translate(window_size.width as f32 / 2f32 * local_zoom - world_offset.x,
                                                                  window_size.height as f32 / 2f32 * local_zoom - world_offset.y,
                                                                  0.0);
                    scroll_transform = scroll_transform.scale(local_zoom, local_zoom, 1f32);
                    scroll_transform = scroll_transform.translate(window_size.width as f32 / -2f32,
                                                                  window_size.height as f32 / -2f32,
                                                                  0.0);
                    
                    root_layer.common.set_transform(scroll_transform);
                    
                    ask_for_tiles();
                    
                    recomposite = true;
                }
                
                ZoomWindowEvent(magnification) => {
                    zoom_action = true;
                    zoom_time = precise_time_s();
                    let old_world_zoom = world_zoom;

                    // Determine zoom amount
                    world_zoom = (world_zoom * magnification).max(&1.0);            
                    local_zoom = local_zoom * world_zoom/old_world_zoom;
                    
                    // Update world offset
                    let corner_to_center_x = world_offset.x + window_size.width as f32 / 2f32;
                    let new_corner_to_center_x = corner_to_center_x * world_zoom / old_world_zoom;
                    world_offset.x = world_offset.x + new_corner_to_center_x - corner_to_center_x;
                    
                    let corner_to_center_y = world_offset.y + window_size.height as f32 / 2f32;
                    let new_corner_to_center_y = corner_to_center_y * world_zoom / old_world_zoom;
                    world_offset.y = world_offset.y + new_corner_to_center_y - corner_to_center_y;        
                    
                    // Clamp to page bounds when zooming out
                    let max_x = (page_size.width * world_zoom - window_size.width as f32).max(&0.0);
                    world_offset.x = world_offset.x.clamp(&0.0, &max_x).round();
                    let max_y = (page_size.height * world_zoom - window_size.height as f32).max(&0.0);
                    world_offset.y = world_offset.y.clamp(&0.0, &max_y).round();
                    
                    // Apply transformations
                    let mut zoom_transform = identity();
                    zoom_transform = zoom_transform.translate(window_size.width as f32 / 2f32 * local_zoom - world_offset.x,
                                                              window_size.height as f32 / 2f32 * local_zoom - world_offset.y,
                                                              0.0);
                    zoom_transform = zoom_transform.scale(local_zoom, local_zoom, 1f32);
                    zoom_transform = zoom_transform.translate(window_size.width as f32 / -2f32,
                                                              window_size.height as f32 / -2f32,
                                                              0.0);
                    root_layer.common.set_transform(zoom_transform);
                    
                    recomposite = true;
                }

                NavigationWindowEvent(direction) => {
                    let direction = match direction {
                        windowing::Forward => constellation_msg::Forward,
                        windowing::Back => constellation_msg::Back,
                    };
                    match layout_chan {
                        Some(ref chan) => chan.send(RouteScriptMsg(NavigateMsg(direction))),
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

