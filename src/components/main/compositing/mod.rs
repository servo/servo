/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::resize_rate_limiter::ResizeRateLimiter;
use platform::{Application, Window};
use script::script_task::{LoadMsg, ScriptMsg, SendEventMsg};
use windowing::{ApplicationMethods, WindowMethods};
use script::dom::event::ClickEvent;

use azure::azure_hl::{DataSourceSurface, DrawTarget, SourceSurfaceMethods};
use core::cell::Cell;
use core::comm::{Chan, SharedChan, Port};
use core::num::Orderable;
use core::util;
use geom::matrix::identity;
use geom::point::Point2D;
use geom::size::Size2D;
use gfx::compositor::{Compositor, LayerBufferSet};
use layers::layers::{ARGB32Format, BasicImageData, ContainerLayer, ContainerLayerKind, Format};
use layers::layers::{Image, ImageData, ImageLayer, ImageLayerKind, RGB24Format, WithDataFn};
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

impl CompositorTask {
    /// Starts the compositor. Returns an interface that can be used to communicate with the
    /// compositor and a port which allows notification when the compositor shuts down.
    pub fn new(script_chan: SharedChan<ScriptMsg>,
               profiler_chan: ProfilerChan)
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
    /// Requests that the compositor paint the given layer buffer set for the given page size.
    Paint(LayerBufferSet, Size2D<uint>),
    /// Requests that the compositor shut down.
    Exit,
}

/// Azure surface wrapping to work with the layers infrastructure.
struct AzureDrawTargetImageData {
    draw_target: DrawTarget,
    data_source_surface: DataSourceSurface,
    size: Size2D<uint>
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
    let original_layer_transform;
    {
        let image_data = @BasicImageData::new(Size2D(0, 0), 0, RGB24Format, ~[]);
        let image = @mut Image::new(image_data as @ImageData);
        let image_layer = @mut ImageLayer(image);
        original_layer_transform = image_layer.common.transform;
        image_layer.common.set_transform(original_layer_transform.scale(800.0, 600.0, 1.0));
        root_layer.add_child(ImageLayerKind(image_layer));
    }

    let scene = @mut Scene(ContainerLayerKind(root_layer), Size2D(800.0, 600.0), identity());
    let done = @mut false;

    // FIXME: This should not be a separate offset applied after the fact but rather should be
    // applied to the layers themselves on a per-layer basis. However, this won't work until scroll
    // positions are sent to content.
    let world_offset = @mut Point2D(0f32, 0f32);
    let page_size = @mut Size2D(0f32, 0f32);
    let window_size = @mut Size2D(800, 600);

    let check_for_messages: @fn() = || {
        // Periodically check if the script task responded to our last resize event
        resize_rate_limiter.check_resize_response();

        // Handle messages
        while port.peek() {
            match port.recv() {
                Exit => *done = true,

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

                        let image_data = @AzureDrawTargetImageData {
                            draw_target: buffer.draw_target.clone(),
                            data_source_surface: buffer.draw_target.snapshot().get_data_surface(),
                            size: Size2D(width, height)
                        };
                        let image = @mut Image::new(image_data as @ImageData);

                        // Find or create an image layer.
                        let image_layer;
                        current_layer_child = match current_layer_child {
                            None => {
                                debug!("osmain: adding new image layer");
                                image_layer = @mut ImageLayer(image);
                                root_layer.add_child(ImageLayerKind(image_layer));
                                None
                            }
                            Some(ImageLayerKind(existing_image_layer)) => {
                                image_layer = existing_image_layer;
                                image_layer.set_image(image);

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
                        let transform = original_layer_transform.translate(origin.x,
                                                                           origin.y,
                                                                           0.0);
                        let transform = transform.scale(width as f32, height as f32, 1.0);
                        image_layer.common.set_transform(transform)
                    }

                    // TODO: Recycle the old buffers; send them back to the renderer to reuse if
                    // it wishes.

                    window.set_needs_display()
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

    // When the user clicks, perform hit testing
    do window.set_click_callback |layer_click_point| {
        let world_click_point = layer_click_point + *world_offset;
        debug!("osmain: clicked at %?", world_click_point);

        script_chan_clone.send(SendEventMsg(ClickEvent(world_click_point)));
    }

    // When the user scrolls, move the layer around.
    do window.set_scroll_callback |delta| {
        // FIXME (Rust #2528): Can't use `-=`.
        let world_offset_copy = *world_offset;
        *world_offset = world_offset_copy - delta;

        // Clamp the world offset to the screen size.
        let max_x = (page_size.width - window_size.width as f32).max(&0.0);
        world_offset.x = world_offset.x.clamp(&0.0, &max_x);
        let max_y = (page_size.height - window_size.height as f32).max(&0.0);
        world_offset.y = world_offset.y.clamp(&0.0, &max_y);

        debug!("compositor: scrolled to %?", *world_offset);

        root_layer.common.set_transform(identity().translate(-world_offset.x,
                                                             -world_offset.y,
                                                             0.0));

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
    fn paint(&self, layer_buffer_set: LayerBufferSet, new_size: Size2D<uint>) {
        self.chan.send(Paint(layer_buffer_set, new_size))
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

