/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use platform::{Application, Window};
use script::dom::event::{Event, ClickEvent, MouseDownEvent, MouseUpEvent, ResizeEvent};
use script::script_task::{LoadMsg, SendEventMsg};
use script::layout_interface::{LayoutChan, RouteScriptMsg};
use script::compositor_interface::{ReadyState, ScriptListener};
use windowing::{ApplicationMethods, WindowMethods, WindowMouseEvent, WindowClickEvent};
use windowing::{WindowMouseDownEvent, WindowMouseUpEvent};
use azure::azure_hl::{DataSourceSurface, DrawTarget, SourceSurfaceMethods, current_gl_context};
use azure::azure::AzGLContext;
use core::cell::Cell;
use core::comm::{Chan, SharedChan, Port};
use core::num::Orderable;
use core::util;
use geom::matrix::identity;
use geom::point::Point2D;
use geom::size::Size2D;
use gfx::compositor::{RenderListener, LayerBufferSet, RenderState};
use layers::layers::{ARGB32Format, ContainerLayer, ContainerLayerKind, Format};
use layers::layers::{ImageData, WithDataFn};
use layers::layers::{TextureLayerKind, TextureLayer, TextureManager};
use layers::rendergl;
use layers::scene::Scene;
use servo_util::{time, url};
use servo_util::time::profile;
use servo_util::time::ProfilerChan;

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
    fn paint(&self, layer_buffer_set: LayerBufferSet, new_size: Size2D<uint>) {
        self.chan.send(Paint(layer_buffer_set, new_size))
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
    /// Sets the channel to the current layout task
    SetLayoutChan(LayoutChan),
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
    port: Port<Msg>,
    profiler_chan: ProfilerChan,
    shutdown_chan: SharedChan<()>,
}

impl CompositorTask {
    pub fn new(port: Port<Msg>,
               profiler_chan: ProfilerChan,
               shutdown_chan: Chan<()>)
               -> CompositorTask {
        CompositorTask {
            port: port,
            profiler_chan: profiler_chan,
            shutdown_chan: SharedChan::new(shutdown_chan),
        }
    }

    /// Starts the compositor, which listens for messages on the specified port. 
    pub fn create(port: Port<Msg>,
                                  profiler_chan: ProfilerChan,
                                  shutdown_chan: Chan<()>) {
        let port = Cell(port);
        let shutdown_chan = Cell(shutdown_chan);
        do on_osmain {
            let compositor_task = CompositorTask::new(port.take(),
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

        let update_layout_callbacks: @fn(LayoutChan) = |layout_chan: LayoutChan| {
            let layout_chan_clone = layout_chan.clone();
            // Hook the windowing system's resize callback up to the resize rate limiter.
            do window.set_resize_callback |width, height| {
                debug!("osmain: window resized to %ux%u", width, height);
                *window_size = Size2D(width, height);
                layout_chan_clone.chan.send(RouteScriptMsg(SendEventMsg(ResizeEvent(width, height))));
            }

            let layout_chan_clone = layout_chan.clone();

            // When the user enters a new URL, load it.
            do window.set_load_url_callback |url_string| {
                debug!("osmain: loading URL `%s`", url_string);
                layout_chan_clone.chan.send(RouteScriptMsg(LoadMsg(url::make_url(url_string.to_str(), None))));
            }

            let layout_chan_clone = layout_chan.clone();

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
                layout_chan_clone.chan.send(RouteScriptMsg(SendEventMsg(event)));
            }
        };

        let check_for_messages: @fn(&Port<Msg>) = |port: &Port<Msg>| {
            // Handle messages
            while port.peek() {
                match port.recv() {
                    Exit => *done = true,

                    ChangeReadyState(ready_state) => window.set_ready_state(ready_state),
                    ChangeRenderState(render_state) => window.set_render_state(render_state),

                    SetLayoutChan(layout_chan) => {
                        update_layout_callbacks(layout_chan);
                    }

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

        let profiler_chan = self.profiler_chan.clone();
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
            check_for_messages(&self.port);

            // Check for messages coming from the windowing system.
            window.check_loop();
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
