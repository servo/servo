/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ShareGlContext = sharegl::platform::Context;
use dom::event::Event;
use platform::resize_rate_limiter::ResizeRateLimiter;

use azure::azure_hl::{BackendType, B8G8R8A8, DataSourceSurface, DrawTarget, SourceSurfaceMethods};
use core::comm::{Chan, SharedChan, Port};
use core::util;
use geom::matrix::identity;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::compositor::{Compositor, LayerBuffer, LayerBufferSet};
use gfx::opts::Opts;
use servo_util::time;
use core::cell::Cell;
use glut::glut;
use layers;
use sharegl;
use sharegl::ShareGlContext;
use sharegl::base::ShareContext;

pub struct OSMain {
    chan: SharedChan<Msg>
}

impl Clone for OSMain {
    fn clone(&self) -> OSMain {
        OSMain {
            chan: self.chan.clone()
        }
    }
}

// FIXME: Move me over to opts.rs.
enum Mode {
    GlutMode,
    ShareMode
}

enum Window {
    GlutWindow(glut::Window),
    ShareWindow(ShareGlContext)
}

pub enum Msg {
    BeginDrawing(comm::Chan<LayerBufferSet>),
    Draw(comm::Chan<LayerBufferSet>, LayerBufferSet),
    AddKeyHandler(comm::Chan<()>),
    Exit
}

pub fn OSMain(dom_event_chan: comm::SharedChan<Event>, opts: Opts) -> OSMain {
    let dom_event_chan = Cell(dom_event_chan);
    OSMain {
        chan: SharedChan::new(on_osmain::<Msg>(|po| {
            let po = Cell(po);
            do platform::runmain {
                debug!("preparing to enter main loop");

                // FIXME: Use the servo options.
                let mode;
                match os::getenv("SERVO_SHARE") {
                    Some(_) => mode = ShareMode,
                    None => mode = GlutMode
                }

                mainloop(mode, po.take(), dom_event_chan.take(), &opts);
            }
        }))
    }
}

/// Azure surface wrapping to work with the layers infrastructure.
struct AzureDrawTargetImageData {
    draw_target: DrawTarget,
    data_source_surface: DataSourceSurface,
    size: Size2D<uint>
}

impl layers::layers::ImageData for AzureDrawTargetImageData {
    fn size(&self) -> Size2D<uint> { self.size }
    fn stride(&self) -> uint { self.data_source_surface.stride() as uint }
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

fn mainloop(mode: Mode,
            po: Port<Msg>,
            dom_event_chan: SharedChan<Event>,
            opts: &Opts) {
    let key_handlers: @mut ~[Chan<()>] = @mut ~[];

    let window;
    match mode {
        GlutMode => {
            glut::init();
            glut::init_display_mode(glut::DOUBLE);
            let glut_window = glut::create_window(~"Servo");
            glut::reshape_window(glut_window, 800, 600);
            window = GlutWindow(glut_window);
        }
        ShareMode => {
            let size = Size2D(800, 600);
            let share_context: ShareGlContext = sharegl::base::ShareContext::new(size);
            io::println(fmt!("Sharing ID is %d", share_context.id()));
            window = ShareWindow(share_context);
        }
    }

    let surfaces = @mut SurfaceSet(opts.render_backend);

    let context = layers::rendergl::init_render_context();

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


    let scene = @layers::scene::Scene(layers::layers::ContainerLayerKind(root_layer),
                                      Size2D(800.0, 600.0),
                                      identity());

    let done = @mut false;
    let resize_rate_limiter = @mut ResizeRateLimiter(dom_event_chan);
    let check_for_messages: @fn() = || {

        // Periodically check if content responded to our last resize event
        resize_rate_limiter.check_resize_response();

        // Handle messages
        //#debug("osmain: peeking");
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
                            Some(_) => {
                                fail!(~"found unexpected layer kind")
                            }
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

    let adjust_for_window_resizing: @fn() = || {
        let window_width = glut::get(glut::WindowWidth) as uint;
        let window_height = glut::get(glut::WindowHeight) as uint;

        // FIXME: Cross-crate struct mutability is broken.
        let size: &mut Size2D<f32>;
        unsafe { size = cast::transmute(&scene.size); }
        *size = Size2D(window_width as f32, window_height as f32);
    };

    let composite: @fn() = || {
        //#debug("osmain: drawing to screen");

        do time::time(~"compositing") {
            adjust_for_window_resizing();
            layers::rendergl::render_scene(context, scene);
        }

        glut::swap_buffers();
        glut::post_redisplay();
    };

    match window {
        GlutWindow(window) => {
            do glut::reshape_func(window) |width, height| {
                debug!("osmain: window resized to %d,%d", width as int, height as int);
                check_for_messages();
                resize_rate_limiter.window_resized(width as uint, height as uint);
                //composite();
            }

            do glut::display_func() {
                //debug!("osmain: display func");
                check_for_messages();
                composite();
            }

            while !*done {
                //#debug("osmain: running GLUT check loop");
                glut::check_loop();
            }
        }
        ShareWindow(share_context) => {
            loop {
                check_for_messages();
                do time::time(~"compositing") {
                    layers::rendergl::render_scene(context, scene);
                }

                share_context.flush();
            }
        }
    }
}

/**
Implementation to allow the osmain channel to be used as a graphics
compositor for the renderer
*/
impl Compositor for OSMain {
    fn begin_drawing(&self, next_dt: comm::Chan<LayerBufferSet>) {
        self.chan.send(BeginDrawing(next_dt))
    }
    fn draw(&self, next_dt: comm::Chan<LayerBufferSet>, draw_me: LayerBufferSet) {
        self.chan.send(Draw(next_dt, draw_me))
    }
}

struct SurfaceSet {
    front: Surface,
    back: Surface,
}

fn lend_surface(surfaces: &mut SurfaceSet, receiver: comm::Chan<LayerBufferSet>) {
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
    surfaces.front <-> surfaces.back;
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
    let layer_buffer_set = LayerBufferSet { buffers: ~[ layer_buffer ] };
    Surface { layer_buffer_set: layer_buffer_set, have: true }
}

/// A function for spawning into the platform's main thread
fn on_osmain<T: Owned>(f: ~fn(po: Port<T>)) -> Chan<T> {
    let (setup_po, setup_ch) = comm::stream();
    do task::task().sched_mode(task::PlatformThread).spawn {
        let (po, ch) = comm::stream();
        setup_ch.send(ch);
        f(po);
    }
    setup_po.recv()
}

// #[cfg(target_os = "linux")]
mod platform {
    pub fn runmain(f: &fn()) {
        f()
    }
}

