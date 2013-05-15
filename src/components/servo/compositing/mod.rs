/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::resize_rate_limiter::ResizeRateLimiter;
use dom::event::Event;
use platform::{Application, Window};
use windowing::{ApplicationMethods, WindowMethods};

use azure::azure_hl::{BackendType, B8G8R8A8, DataSourceSurface, DrawTarget, SourceSurfaceMethods};
use core::cell::Cell;
use core::comm::{Chan, SharedChan, Port};
use core::util;
use geom::matrix::identity;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::compositor::{Compositor, LayerBuffer, LayerBufferSet};
use gfx::opts::Opts;
use layers;
use servo_util::time;

mod resize_rate_limiter;

/// The implementation of the layers-based compositor.
#[deriving(Clone)]
pub struct CompositorImpl {
    chan: SharedChan<Msg>
}

impl CompositorImpl {
    /// Creates a new compositor instance.
    pub fn new(dom_event_chan: SharedChan<Event>, opts: Opts) -> CompositorImpl {
        let dom_event_chan = Cell(dom_event_chan);
        let chan: Chan<Msg> = do on_osmain |port| {
            debug!("preparing to enter main loop");
            mainloop(port, dom_event_chan.take(), &opts);
        };

        CompositorImpl {
            chan: SharedChan::new(chan)
        }
    }
}

/// Messages to the compositor.
pub enum Msg {
    BeginDrawing(Chan<LayerBufferSet>),
    Draw(Chan<LayerBufferSet>, LayerBufferSet),
    AddKeyHandler(Chan<()>),
    Exit
}

/// Azure surface wrapping to work with the layers infrastructure.
struct AzureDrawTargetImageData {
    draw_target: DrawTarget,
    data_source_surface: DataSourceSurface,
    size: Size2D<uint>
}

impl layers::layers::ImageData for AzureDrawTargetImageData {
    fn size(&self) -> Size2D<uint> {
        self.size
    }
    fn stride(&self) -> uint {
        self.data_source_surface.stride() as uint
    }
    fn format(&self) -> layers::layers::Format {
        // FIXME: This is not always correct. We should query the Azure draw target for the format.
        layers::layers::ARGB32Format
    }
    fn with_data(&self, f: layers::layers::WithDataFn) { 
        do self.data_source_surface.with_data |data| {
            f(data);
        }
    }
}

fn mainloop(po: Port<Msg>, dom_event_chan: SharedChan<Event>, opts: &Opts) {
    let key_handlers: @mut ~[Chan<()>] = @mut ~[];

    let app: Application = ApplicationMethods::new();
    let window: @mut Window = WindowMethods::new(&app);

    let surfaces = @mut SurfaceSet(opts.render_backend);

    let context = layers::rendergl::init_render_context();

    // Create an initial layer tree.
    //
    // TODO: There should be no initial layer tree until the renderer creates one from the display
    // list. This is only here because we don't have that logic in the renderer yet.
    let root_layer = @mut layers::layers::ContainerLayer();
    let original_layer_transform;
    {
        let image_data = @layers::layers::BasicImageData::new(Size2D(0u, 0u),
                                                              0,
                                                              layers::layers::RGB24Format,
                                                              ~[]);
        let image = @mut layers::layers::Image::new(image_data as @layers::layers::ImageData);
        let image_layer = @mut layers::layers::ImageLayer(image);
        original_layer_transform = image_layer.common.transform;
        image_layer.common.set_transform(original_layer_transform.scale(800.0, 600.0, 1.0));
        root_layer.add_child(layers::layers::ImageLayerKind(image_layer));
    }


    let scene = @mut layers::scene::Scene(layers::layers::ContainerLayerKind(root_layer),
                                          Size2D(800.0, 600.0),
                                          identity());

    let done = @mut false;
    let resize_rate_limiter = @mut ResizeRateLimiter(dom_event_chan);
    let check_for_messages: @fn() = || {
        // Periodically check if content responded to our last resize event
        resize_rate_limiter.check_resize_response();

        // Handle messages
        while po.peek() {
            match po.recv() {
                AddKeyHandler(key_ch) => key_handlers.push(key_ch),
                BeginDrawing(sender) => lend_surface(surfaces, sender),
                Draw(sender, draw_target) => {
                    debug!("osmain: received new frame");
                    return_surface(surfaces, draw_target);
                    lend_surface(surfaces, sender);

                    // Iterate over the children of the container layer.
                    let mut current_layer_child = root_layer.first_child;

                    // Replace the image layer data with the buffer data.
                    let buffers = util::replace(&mut surfaces.front.layer_buffer_set.buffers, ~[]);
                    for buffers.each |buffer| {
                        let width = buffer.rect.size.width as uint;
                        let height = buffer.rect.size.height as uint;

                        debug!("osmain: compositing buffer rect %?", &buffer.rect);

                        let image_data = @AzureDrawTargetImageData {
                            draw_target: buffer.draw_target.clone(),
                            data_source_surface: buffer.draw_target.snapshot().get_data_surface(),
                            size: Size2D(width, height)
                        };
                        let image = @mut layers::layers::Image::new(image_data as @layers::layers::ImageData);

                        // Find or create an image layer.
                        let image_layer;
                        current_layer_child = match current_layer_child {
                            None => {
                                debug!("osmain: adding new image layer");
                                image_layer = @mut layers::layers::ImageLayer(image);
                                root_layer.add_child(layers::layers::ImageLayerKind(image_layer));
                                None
                            }
                            Some(layers::layers::ImageLayerKind(existing_image_layer)) => {
                                image_layer = existing_image_layer;
                                image_layer.set_image(image);

                                // Move on to the next sibling.
                                do current_layer_child.get().with_common |common| {
                                    common.next_sibling
                                }
                            }
                            Some(_) => fail!(~"found unexpected layer kind"),
                        };

                        // Set the layer's transform.
                        let x = buffer.rect.origin.x as f32;
                        let y = buffer.rect.origin.y as f32;
                        image_layer.common.set_transform(
                            original_layer_transform.translate(x, y, 0.0)
                                .scale(width as f32, height as f32, 1.0));
                    }
                    surfaces.front.layer_buffer_set.buffers = buffers;
                }
                Exit => {
                    *done = true;
                }
            }
        }
    };

    do window.set_composite_callback {
        do time::time(~"compositing") {
            // Adjust the layer dimensions as necessary to correspond to the size of the window.
            scene.size = window.size();

            // Render the scene.
            layers::rendergl::render_scene(context, scene);
        }

        window.present();
    }

    do window.set_resize_callback |width, height| {
        debug!("osmain: window resized to %ux%u", width, height);
        resize_rate_limiter.window_resized(width, height);
    }

    // Enter the main event loop.
    while !*done {
        // Check for new messages coming from the rendering task.
        check_for_messages();

        // Check for messages coming from the windowing system.
        window.check_loop();
    }
}

/// Implementation of the abstract `Compositor` interface.
impl Compositor for CompositorImpl {
    fn begin_drawing(&self, next_dt: Chan<LayerBufferSet>) {
        self.chan.send(BeginDrawing(next_dt))
    }
    fn draw(&self, next_dt: Chan<LayerBufferSet>, draw_me: LayerBufferSet) {
        self.chan.send(Draw(next_dt, draw_me))
    }
}

struct SurfaceSet {
    front: Surface,
    back: Surface,
}

fn lend_surface(surfaces: &mut SurfaceSet, receiver: Chan<LayerBufferSet>) {
    // We are in a position to lend out the surface?
    assert!(surfaces.front.have);
    // Ok then take it
    let old_layer_buffers = util::replace(&mut surfaces.front.layer_buffer_set.buffers, ~[]);
    let new_layer_buffers = do old_layer_buffers.map |layer_buffer| {
        let draw_target_ref = &layer_buffer.draw_target;
        let layer_buffer = LayerBuffer {
            draw_target: draw_target_ref.clone(),
            rect: copy layer_buffer.rect,
            stride: layer_buffer.stride
        };
        debug!("osmain: lending surface %?", layer_buffer);
        layer_buffer
    };
    surfaces.front.layer_buffer_set.buffers = old_layer_buffers;

    let new_layer_buffer_set = LayerBufferSet { buffers: new_layer_buffers };
    receiver.send(new_layer_buffer_set);
    // Now we don't have it
    surfaces.front.have = false;
    // But we (hopefully) have another!
    util::swap(&mut surfaces.front, &mut surfaces.back);
    // Let's look
    assert!(surfaces.front.have);
}

fn return_surface(surfaces: &mut SurfaceSet, layer_buffer_set: LayerBufferSet) {
    //#debug("osmain: returning surface %?", layer_buffer_set);
    // We have room for a return
    assert!(surfaces.front.have);
    assert!(!surfaces.back.have);

    surfaces.back.layer_buffer_set = layer_buffer_set;

    // Now we have it again
    surfaces.back.have = true;
}

fn SurfaceSet(backend: BackendType) -> SurfaceSet {
    SurfaceSet { front: Surface(backend), back: Surface(backend) }
}

struct Surface {
    layer_buffer_set: LayerBufferSet,
    have: bool,
}

fn Surface(backend: BackendType) -> Surface {
    let layer_buffer = LayerBuffer {
        draw_target: DrawTarget::new(backend, Size2D(800i32, 600i32), B8G8R8A8),
        rect: Rect(Point2D(0u, 0u), Size2D(800u, 600u)),
        stride: 800 * 4
    };
    let layer_buffer_set = LayerBufferSet {
        buffers: ~[ layer_buffer ]
    };
    Surface {
        layer_buffer_set: layer_buffer_set,
        have: true
    }
}

/// A function for spawning into the platform's main thread.
fn on_osmain<T: Owned>(f: ~fn(po: Port<T>)) -> Chan<T> {
    let (setup_po, setup_ch) = comm::stream();
    // FIXME: rust#6399
    let mut main_task = task::task();
    main_task.sched_mode(task::PlatformThread);
    do main_task.spawn {
        let (po, ch) = comm::stream();
        setup_ch.send(ch);
        f(po);
    }
    setup_po.recv()
}

