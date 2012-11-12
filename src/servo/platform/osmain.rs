use ShareGlContext = sharegl::platform::Context;
use dom::event::{Event, ResizeEvent};
use resize_rate_limiter::ResizeRateLimiter;

use azure::azure_hl::{BackendType, B8G8R8A8, DataSourceSurface, DrawTarget, SourceSurfaceMethods};
use core::dvec::DVec;
use core::pipes::Chan;
use core::task::TaskBuilder;
use core::util;
use geom::matrix::{Matrix4, identity};
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::compositor::{Compositor, LayerBuffer, LayerBufferSet};
use gfx::opts::Opts;
use gfx::util::time;
use layers::ImageLayer;
use std::cell::Cell;
use std::cmp::FuzzyEq;

pub struct OSMain {
    chan: comm::Chan<Msg>
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
    BeginDrawing(pipes::Chan<LayerBufferSet>),
    Draw(pipes::Chan<LayerBufferSet>, LayerBufferSet),
    AddKeyHandler(pipes::Chan<()>),
    Exit
}

fn OSMain(dom_event_chan: pipes::SharedChan<Event>, opts: Opts) -> OSMain {
    let dom_event_chan = Cell(move dom_event_chan);
    OSMain {
        chan: do on_osmain::<Msg> |po, move dom_event_chan, move opts| {
            do platform::runmain {
                #debug("preparing to enter main loop");

                // FIXME: Use the servo options.
                let mode;
                match os::getenv("SERVO_SHARE") {
                    Some(_) => mode = ShareMode,
                    None => mode = GlutMode
                }

                mainloop(mode, po, dom_event_chan.take(), &opts);
            }
        }
    }
}

/// Azure surface wrapping to work with the layers infrastructure.
struct AzureDrawTargetImageData {
    draw_target: DrawTarget,
    data_source_surface: DataSourceSurface,
    size: Size2D<uint>
}

impl AzureDrawTargetImageData : layers::layers::ImageData {
    fn size() -> Size2D<uint> { self.size }
    fn stride() -> uint { self.data_source_surface.get_size().width as uint }
    fn format() -> layers::layers::Format {
        // FIXME: This is not always correct. We should query the Azure draw target for the format.
        layers::layers::ARGB32Format
    }
    fn with_data(f: layers::layers::WithDataFn) { 
        do self.data_source_surface.with_data |data| {
            f(data);
        }
    }
}

fn mainloop(mode: Mode,
            po: comm::Port<Msg>,
            dom_event_chan: pipes::SharedChan<Event>,
            opts: &Opts) {
    let key_handlers: @DVec<pipes::Chan<()>> = @DVec();

	let window;
	match mode {
		GlutMode => {
			glut::init();
			glut::init_display_mode(glut::DOUBLE);
			let glut_window = glut::create_window(~"Servo");
			glut::reshape_window(glut_window, 800, 600);
			window = GlutWindow(move glut_window);
		}
		ShareMode => {
			let share_context: ShareGlContext = sharegl::base::new(Size2D(800, 600));
			io::println(fmt!("Sharing ID is %d", share_context.id()));
			window = ShareWindow(move share_context);
		}
	}

    let surfaces = @SurfaceSet(opts.render_backend);

    let context = layers::rendergl::init_render_context();

    let root_layer = @layers::layers::ContainerLayer();
    let original_layer_transform;
    {
        let image_data = @layers::layers::BasicImageData::new(
            Size2D(0u, 0u), 0, layers::layers::RGB24Format, ~[]);
        let image = @layers::layers::Image::new(image_data as @layers::layers::ImageData);
        let image_layer = @layers::layers::ImageLayer(image);
        original_layer_transform = image_layer.common.transform;
        image_layer.common.set_transform(original_layer_transform.scale(&800.0f32, &600.0f32,
                                                                        &1f32));
        root_layer.add_child(layers::layers::ImageLayerKind(image_layer));
    }


    let scene = @layers::scene::Scene(layers::layers::ContainerLayerKind(root_layer),
                                      Size2D(800.0f32, 600.0f32),
                                      identity(0.0f32));

    let done = @mut false;
    let resize_rate_limiter = @ResizeRateLimiter(move dom_event_chan);
    let check_for_messages = fn@() {

        // Periodically check if content responded to our last resize event
        resize_rate_limiter.check_resize_response();

        // Handle messages
        //#debug("osmain: peeking");
        while po.peek() {
            match po.recv() {
                AddKeyHandler(move key_ch) => key_handlers.push(move key_ch),
                BeginDrawing(move sender) => lend_surface(surfaces, move sender),
                Draw(move sender, move draw_target) => {
                    #debug("osmain: received new frame");
                    return_surface(surfaces, move draw_target);
                    lend_surface(surfaces, move sender);

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
                        let image = @layers::layers::Image::new(
                            image_data as @layers::layers::ImageData);

                        // Find or create an image layer.
                        let image_layer;
                        current_layer_child = match current_layer_child {
                            None => {
                                debug!("osmain: adding new image layer");
                                image_layer = @layers::layers::ImageLayer(image);
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
                                fail ~"found unexpected layer kind"
                            }
                        };

                        // Set the layer's transform.
                        let x = buffer.rect.origin.x as f32;
                        let y = buffer.rect.origin.y as f32;
                        image_layer.common.set_transform(
                            original_layer_transform.translate(&x, &y, &0.0f32)
                                .scale(&(width as f32), &(height as f32), &1.0f32));
                    }
                    surfaces.front.layer_buffer_set.buffers = move buffers;
                }
                Exit => {
                    *done = true;
                }
            }
        }
    };

    let adjust_for_window_resizing: fn@() = || {
        let window_width = glut::get(glut::WindowWidth) as uint;
        let window_height = glut::get(glut::WindowHeight) as uint;

        // FIXME: Cross-crate struct mutability is broken.
        let size: &mut Size2D<f32>;
        unsafe { size = cast::transmute(&scene.size); }
        *size = Size2D(window_width as f32, window_height as f32);
    };

    let composite: fn@() = || {
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
                #debug("osmain: window resized to %d,%d", width as int, height as int);
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
impl OSMain : Compositor {
    fn begin_drawing(next_dt: pipes::Chan<LayerBufferSet>) {
        self.chan.send(BeginDrawing(move next_dt))
    }
    fn draw(next_dt: pipes::Chan<LayerBufferSet>, draw_me: LayerBufferSet) {
        self.chan.send(Draw(move next_dt, move draw_me))
    }
}

struct SurfaceSet {
    mut front: Surface,
    mut back: Surface,
}

fn lend_surface(surfaces: &SurfaceSet, receiver: pipes::Chan<LayerBufferSet>) {
    // We are in a position to lend out the surface?
    assert surfaces.front.have;
    // Ok then take it
    let old_layer_buffers = util::replace(&mut surfaces.front.layer_buffer_set.buffers, ~[]);
    let new_layer_buffers = do old_layer_buffers.map |layer_buffer| {
        let draw_target_ref = &layer_buffer.draw_target;
        let layer_buffer = LayerBuffer {
            draw_target: draw_target_ref.clone(),
            rect: copy layer_buffer.rect,
            stride: layer_buffer.stride
        };
        #debug("osmain: lending surface %?", layer_buffer);
        move layer_buffer
    };
    surfaces.front.layer_buffer_set.buffers = move old_layer_buffers;

    let new_layer_buffer_set = LayerBufferSet { buffers: move new_layer_buffers };
    receiver.send(move new_layer_buffer_set);
    // Now we don't have it
    surfaces.front.have = false;
    // But we (hopefully) have another!
    surfaces.front <-> surfaces.back;
    // Let's look
    assert surfaces.front.have;
}

fn return_surface(surfaces: &SurfaceSet, layer_buffer_set: LayerBufferSet) {
    //#debug("osmain: returning surface %?", layer_buffer_set);
    // We have room for a return
    assert surfaces.front.have;
    assert !surfaces.back.have;

    surfaces.back.layer_buffer_set = move layer_buffer_set;

    // Now we have it again
    surfaces.back.have = true;
}

fn SurfaceSet(backend: BackendType) -> SurfaceSet {
    SurfaceSet { front: Surface(backend), back: Surface(backend) }
}

struct Surface {
    layer_buffer_set: LayerBufferSet,
    mut have: bool,
}

fn Surface(backend: BackendType) -> Surface {
    let layer_buffer = LayerBuffer {
        draw_target: DrawTarget::new(backend, Size2D(800i32, 600i32), B8G8R8A8),
        rect: Rect(Point2D(0u, 0u), Size2D(800u, 600u)),
        stride: 800
    };
    let layer_buffer_set = LayerBufferSet { buffers: ~[ move layer_buffer ] };
    Surface { layer_buffer_set: move layer_buffer_set, have: true }
}

/// A function for spawning into the platform's main thread
fn on_osmain<T: Send>(f: fn~(po: comm::Port<T>)) -> comm::Chan<T> {
    task::task().sched_mode(task::PlatformThread).spawn_listener(move f)
}

// #[cfg(target_os = "linux")]
mod platform {
    pub fn runmain(f: fn()) {
        f()
    }
}

