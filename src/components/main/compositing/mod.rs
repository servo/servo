/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::resize_rate_limiter::ResizeRateLimiter;
use platform::{Application, Window};
use script::script_task::{LoadMsg, ScriptMsg, SendEventMsg};
use windowing::{ApplicationMethods, WindowMethods, WindowMouseEvent, WindowClickEvent};
use windowing::{WindowMouseDownEvent, WindowMouseUpEvent};

use gfx::compositor::RenderState;
use script::dom::event::{Event, ClickEvent, MouseDownEvent, MouseUpEvent};
use script::compositor_interface::{ReadyState, CompositorInterface};
use script::compositor_interface;

use azure::azure_hl::{DataSourceSurface, DrawTarget, SourceSurfaceMethods, current_gl_context};
use azure::azure::AzGLContext;
use core::cell::Cell;
use core::comm::{Chan, SharedChan, Port};
use core::num::Orderable;
use core::util;
use geom::matrix::identity;
use geom::point::Point2D;
use geom::size::Size2D;
use gfx::compositor::{Compositor, LayerBufferSet, RenderState};
use layers::layers::{ARGB32Format, ContainerLayer, ContainerLayerKind, Format};
use layers::layers::{ImageData, WithDataFn};
use layers::layers::{TextureLayerKind, TextureLayer, TextureManager};
use layers::rendergl;
use layers::scene::Scene;
use servo_util::{time, url};
use servo_util::time::profile;
use servo_util::time::ProfilerChan;

mod resize_rate_limiter;

/// The implementation of the layers-based compositor.
#[deriving(Clone)]
pub struct CompositorTask {
    /// A channel on which messages can be sent to the compositor.
    chan: SharedChan<Msg>,
}

impl CompositorInterface for CompositorTask {
    fn set_ready_state(&self, ready_state: ReadyState) {
        let msg = ChangeReadyState(ready_state);
        self.chan.send(msg);
    }
}

impl CompositorTask {
    /// Starts the compositor. Returns an interface that can be used to communicate with the
    /// compositor and a port which allows notification when the compositor shuts down.
    pub fn new(script_chan: SharedChan<ScriptMsg>, profiler_chan: ProfilerChan)
               -> (CompositorTask, Port<()>) {
        let script_chan = Cell(script_chan);
        let (shutdown_port, shutdown_chan) = stream();
        let shutdown_chan = Cell(shutdown_chan);

        let chan: Chan<Msg> = do on_osmain |port| {
            debug!("preparing to enter main loop");
            run_main_loop(port,
                          script_chan.take(),
                          shutdown_chan.take(),
                          profiler_chan.clone());
        };

        let task = CompositorTask {
            chan: SharedChan::new(chan),
        };
        (task, shutdown_port)
    }
}

/// Messages to the compositor.
pub enum Msg {
    /// Requests that the compositor shut down.
    Exit,
    /// Requests the compositors GL context.
    GetGLContext(Chan<AzGLContext>),
    /// Requests that the compositor paint the given layer buffer set for the given page size.
    Paint(LayerBufferSet, Size2D<uint>),
    /// Alerts the compositor to the current status of page loading.
    ChangeReadyState(ReadyState),
    /// Alerts the compositor to the current status of rendering.
    ChangeRenderState(RenderState),
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

fn run_main_loop(port: Port<Msg>,
                 script_chan: SharedChan<ScriptMsg>,
                 shutdown_chan: Chan<()>,
                 profiler_chan: ProfilerChan) {
    let app: Application = ApplicationMethods::new();
    let window: @mut Window = WindowMethods::new(&app);
    let resize_rate_limiter = @mut ResizeRateLimiter(script_chan.clone());

    // Create an initial layer tree.
    //
    // TODO: There should be no initial layer tree until the renderer creates one from the display
    // list. This is only here because we don't have that logic in the renderer yet.
    let context = rendergl::init_render_context();
    let root_layer = @mut ContainerLayer();
    let scene = @mut Scene(ContainerLayerKind(root_layer), Size2D(800.0, 600.0), identity());
    let done = @mut false;

    // FIXME: This should not be a separate offset applied after the fact but rather should be
    // applied to the layers themselves on a per-layer basis. However, this won't work until scroll
    // positions are sent to content.
    let world_offset = @mut Point2D(0f32, 0f32);
    let page_size = @mut Size2D(0f32, 0f32);
    let window_size = @mut Size2D(800, 600);

    // Keeps track of the current zoom factor
    let world_zoom = @mut 1f32;

    let check_for_messages: @fn() = || {
        // Periodically check if the script task responded to our last resize event
        resize_rate_limiter.check_resize_response();
        // Handle messages
        while port.peek() {
            match port.recv() {
                Exit => *done = true,

                ChangeReadyState(ready_state) => window.set_ready_state(ready_state),
                ChangeRenderState(render_state) => window.set_render_state(render_state),

                GetGLContext(chan) => chan.send(current_gl_context()),

                Paint(new_layer_buffer_set, new_size) => {
                    debug!("osmain: received new frame");

                    *page_size = Size2D(new_size.width as f32, new_size.height as f32);

                    let mut new_layer_buffer_set = new_layer_buffer_set;

                    // Iterate over the children of the container layer.
                    let mut current_layer_child = root_layer.first_child;

                    // Replace the image layer data with the buffer data. Also compute the page
                    // size here.
                    let buffers = util::replace(&mut new_layer_buffer_set.buffers, ~[]);

                    for buffers.each |buffer| {
                        let width = buffer.rect.size.width as uint;
                        let height = buffer.rect.size.height as uint;

                        debug!("osmain: compositing buffer rect %?", &buffer.rect);

                        // Find or create a texture layer.
                        let texture_layer;
                        current_layer_child = match current_layer_child {
                            None => {
                                debug!("osmain: adding new texture layer");
                                texture_layer = @mut TextureLayer::new(@buffer.draw_target.clone() as @TextureManager,
                                                                       buffer.rect.size);
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

                        let origin = buffer.screen_pos.origin;
                        let origin = Point2D(origin.x as f32, origin.y as f32);

                        // Set the layer's transform.
                        let transform = identity().translate(origin.x, origin.y, 0.0);
                        let transform = transform.scale(width as f32, height as f32, 1.0);
                        texture_layer.common.set_transform(transform);
                    }

                    // TODO: Recycle the old buffers; send them back to the renderer to reuse if
                    // it wishes.

                    window.set_needs_display();
                }
            }
        }
    };

    do window.set_composite_callback {
        do profile(time::CompositingCategory, profiler_chan.clone()) {
            debug!("compositor: compositing");
            // Adjust the layer dimensions as necessary to correspond to the size of the window.
            scene.size = window.size();

            // Render the scene.
            rendergl::render_scene(context, scene);
        }

        window.present();
    }

    // Hook the windowing system's resize callback up to the resize rate limiter.
    do window.set_resize_callback |width, height| {
        debug!("osmain: window resized to %ux%u", width, height);
        *window_size = Size2D(width, height);
        resize_rate_limiter.window_resized(width, height)
    }

    let script_chan_clone = script_chan.clone();

    // When the user enters a new URL, load it.
    do window.set_load_url_callback |url_string| {
        debug!("osmain: loading URL `%s`", url_string);
        script_chan_clone.send(LoadMsg(url::make_url(url_string.to_str(), None)))
    }

    let script_chan_clone = script_chan.clone();

    // When the user triggers a mouse event, perform appropriate hit testing
    do window.set_mouse_callback |window_mouse_event: WindowMouseEvent| {
        let event: Event;
        let world_mouse_point = |layer_mouse_point: Point2D<f32>| {
            layer_mouse_point + *world_offset
        };
        match window_mouse_event {
            WindowClickEvent(button, layer_mouse_point) => {
                event = ClickEvent(button, world_mouse_point(layer_mouse_point));
            }
            WindowMouseDownEvent(button, layer_mouse_point) => {
                event = MouseDownEvent(button, world_mouse_point(layer_mouse_point));
            }
            WindowMouseUpEvent(button, layer_mouse_point) => {
                event = MouseUpEvent(button, world_mouse_point(layer_mouse_point));
            }
        }
        script_chan_clone.send(SendEventMsg(event));
    }

    // When the user scrolls, move the layer around.
    do window.set_scroll_callback |delta| {
        // FIXME (Rust #2528): Can't use `-=`.
        let world_offset_copy = *world_offset;
        *world_offset = world_offset_copy - delta;

        // Clamp the world offset to the screen size.
        let max_x = (page_size.width * *world_zoom - window_size.width as f32).max(&0.0);
        world_offset.x = world_offset.x.clamp(&0.0, &max_x);
        let max_y = (page_size.height * *world_zoom - window_size.height as f32).max(&0.0);
        world_offset.y = world_offset.y.clamp(&0.0, &max_y);

        debug!("compositor: scrolled to %?", *world_offset);

        let mut scroll_transform = identity();

        scroll_transform = scroll_transform.translate(window_size.width as f32 / 2f32 * *world_zoom - world_offset.x,
                                                  window_size.height as f32 / 2f32 * *world_zoom - world_offset.y,
                                                  0.0);
        scroll_transform = scroll_transform.scale(*world_zoom, *world_zoom, 1f32);
        scroll_transform = scroll_transform.translate(window_size.width as f32 / -2f32,
                                                  window_size.height as f32 / -2f32,
                                                  0.0);

        root_layer.common.set_transform(scroll_transform);

        window.set_needs_display()
    }



    // When the user pinch-zooms, scale the layer
    do window.set_zoom_callback |magnification| {
        let old_world_zoom = *world_zoom;

        // Determine zoom amount
        *world_zoom = (*world_zoom * magnification).max(&1.0);            

        // Update world offset
        let corner_to_center_x = world_offset.x + window_size.width as f32 / 2f32;
        let new_corner_to_center_x = corner_to_center_x * *world_zoom / old_world_zoom;
        world_offset.x = world_offset.x + new_corner_to_center_x - corner_to_center_x;

        let corner_to_center_y = world_offset.y + window_size.height as f32 / 2f32;
        let new_corner_to_center_y = corner_to_center_y * *world_zoom / old_world_zoom;
        world_offset.y = world_offset.y + new_corner_to_center_y - corner_to_center_y;        

        // Clamp to page bounds when zooming out
        let max_x = (page_size.width * *world_zoom - window_size.width as f32).max(&0.0);
        world_offset.x = world_offset.x.clamp(&0.0, &max_x);
        let max_y = (page_size.height * *world_zoom - window_size.height as f32).max(&0.0);
        world_offset.y = world_offset.y.clamp(&0.0, &max_y);


        // Apply transformations
        let mut zoom_transform = identity();
        zoom_transform = zoom_transform.translate(window_size.width as f32 / 2f32 * *world_zoom - world_offset.x,
                                                  window_size.height as f32 / 2f32 * *world_zoom - world_offset.y,
                                                  0.0);
        zoom_transform = zoom_transform.scale(*world_zoom, *world_zoom, 1f32);
        zoom_transform = zoom_transform.translate(window_size.width as f32 / -2f32,
                                                  window_size.height as f32 / -2f32,
                                                  0.0);
        root_layer.common.set_transform(zoom_transform);


        window.set_needs_display()
    }

    // Enter the main event loop.
    while !*done {
        // Check for new messages coming from the rendering task.
        check_for_messages();

        // Check for messages coming from the windowing system.
        window.check_loop();
    }

    shutdown_chan.send(())
}

/// Implementation of the abstract `Compositor` interface.
impl Compositor for CompositorTask {
    fn get_gl_context(&self) -> AzGLContext {
        let (port, chan) = comm::stream();
        self.chan.send(GetGLContext(chan));
        port.recv()
    }

    fn paint(&self, layer_buffer_set: LayerBufferSet, new_size: Size2D<uint>) {
        self.chan.send(Paint(layer_buffer_set, new_size))
    }
    fn set_render_state(&self, render_state: RenderState) {
        self.chan.send(ChangeRenderState(render_state))
    }
}

/// A function for spawning into the platform's main thread.
fn on_osmain<T: Owned>(f: ~fn(port: Port<T>)) -> Chan<T> {
    let (setup_port, setup_chan) = comm::stream();
    // FIXME: rust#6399
    let mut main_task = task::task();
    main_task.sched_mode(task::PlatformThread);
    do main_task.spawn {
        let (port, chan) = comm::stream();
        setup_chan.send(chan);
        f(port);
    }
    setup_port.recv()
}

