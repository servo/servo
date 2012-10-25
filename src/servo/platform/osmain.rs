use ShareGlContext = sharegl::platform::Context;
use azure::azure_hl;
use azure::azure_hl::DrawTarget;
use cairo::cairo_hl::ImageSurface;
use cairo::cairo_surface_t;
use core::util::replace;
use dom::event::{Event, ResizeEvent};
use dvec::DVec;
use geom::matrix::{Matrix4, identity};
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::compositor::{Compositor, LayerBuffer, LayerBufferSet};
use layers::ImageLayer;
use pipes::Chan;
use resize_rate_limiter::ResizeRateLimiter;
use std::cell::Cell;
use std::cmp::FuzzyEq;
use task::TaskBuilder;
use vec::push;

pub type OSMain = comm::Chan<Msg>;

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

fn OSMain(dom_event_chan: pipes::SharedChan<Event>) -> OSMain {
    let dom_event_chan = Cell(move dom_event_chan);
    do on_osmain::<Msg> |po, move dom_event_chan| {
        do platform::runmain {
            #debug("preparing to enter main loop");

            // FIXME: Use the servo options.
			let mode;
			match os::getenv("SERVO_SHARE") {
				Some(_) => mode = ShareMode,
				None => mode = GlutMode
			}

	        mainloop(mode, po, dom_event_chan.take());
        }
    }
}

/// Cairo surface wrapping to work with layers
struct CairoSurfaceImageData {
    cairo_surface: ImageSurface,
    size: Size2D<uint>
}

impl CairoSurfaceImageData : layers::layers::ImageData {
    fn size() -> Size2D<uint> { self.size }
    fn stride() -> uint { self.cairo_surface.width() as uint }
    fn format() -> layers::layers::Format { layers::layers::ARGB32Format }
    fn with_data(f: layers::layers::WithDataFn) { f(self.cairo_surface.data()) }
}

fn mainloop(mode: Mode, po: comm::Port<Msg>, dom_event_chan: pipes::SharedChan<Event>) {

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

    let surfaces = @SurfaceSet();

    let context = layers::rendergl::init_render_context();

    let image_data = @layers::layers::BasicImageData::new(
        Size2D(0u, 0u), 0, layers::layers::RGB24Format, ~[]);
    let image = @layers::layers::Image::new(image_data as @layers::layers::ImageData);
    let image_layer = @layers::layers::ImageLayer(image);
    let original_layer_transform = image_layer.common.transform;
    image_layer.common.set_transform(original_layer_transform.scale(&800.0f32, &600.0f32, &1f32));

    let scene = @layers::scene::Scene(layers::layers::ImageLayerKind(image_layer),
                                      Size2D(800.0f32, 600.0f32),
                                      identity(0.0f32));

    let done = @mut false;
    let resize_rate_limiter = @ResizeRateLimiter(move dom_event_chan);
    let check_for_messages = fn@() {

        // Periodically check if content responded to our last resize event
        resize_rate_limiter.check_resize_response();

        // Handle messages
        #debug("osmain: peeking");
        while po.peek() {
            match po.recv() {
                AddKeyHandler(move key_ch) => key_handlers.push(move key_ch),
                BeginDrawing(move sender) => lend_surface(surfaces, move sender),
                Draw(move sender, move dt) => {
                    #debug("osmain: received new frame");
                    return_surface(surfaces, move dt);
                    lend_surface(surfaces, move sender);

                    let buffers = &mut surfaces.front.layer_buffer_set.buffers;
                    let width = buffers[0].rect.size.width as uint;
                    let height = buffers[0].rect.size.height as uint;

                    let image_data = @CairoSurfaceImageData {
                        cairo_surface: buffers[0].cairo_surface.clone(),
                        size: Size2D(width, height)
                    };
                    let image = @layers::layers::Image::new(
                        image_data as @layers::layers::ImageData);

                    image_layer.set_image(image);
                    image_layer.common.set_transform(original_layer_transform.scale(
                        &(width as f32), &(height as f32), &1.0f32));
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
        #debug("osmain: drawing to screen");

        do util::time::time(~"compositing") {
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
                debug!("osmain: display func");
                check_for_messages();
                composite();
            }

            while !*done {
                #debug("osmain: running GLUT check loop");
                glut::check_loop();
            }
        }
        ShareWindow(share_context) => {
            loop {
                check_for_messages();
                do util::time::time(~"compositing") {
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
        self.send(BeginDrawing(move next_dt))
    }
    fn draw(next_dt: pipes::Chan<LayerBufferSet>, draw_me: LayerBufferSet) {
        self.send(Draw(move next_dt, move draw_me))
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
    let old_layer_buffers = replace(&mut surfaces.front.layer_buffer_set.buffers, ~[]);
    let new_layer_buffers = do old_layer_buffers.map |layer_buffer| {
        let draw_target_ref = &layer_buffer.draw_target;
        let layer_buffer = LayerBuffer {
            cairo_surface: layer_buffer.cairo_surface.clone(),
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
    #debug("osmain: returning surface %?", layer_buffer_set);
    // We have room for a return
    assert surfaces.front.have;
    assert !surfaces.back.have;

    surfaces.back.layer_buffer_set = move layer_buffer_set;

    // Now we have it again
    surfaces.back.have = true;
}

fn SurfaceSet() -> SurfaceSet {
    SurfaceSet { front: Surface(), back: Surface() }
}

struct Surface {
    layer_buffer_set: LayerBufferSet,
    mut have: bool,
}

fn Surface() -> Surface {
    let cairo_surface = ImageSurface(cairo::CAIRO_FORMAT_RGB24, 800, 600);
    let draw_target = DrawTarget(&cairo_surface);
    let layer_buffer = LayerBuffer {
        cairo_surface: move cairo_surface,
        draw_target: move draw_target,
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

