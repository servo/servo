use mod azure::azure_hl;
use azure::azure_hl::DrawTarget;
use azure::cairo;
use azure::cairo_hl::ImageSurface;
use dvec::DVec;
use azure::cairo::cairo_surface_t;
use gfx::compositor::Compositor;
use gfx::render_task::LayerBuffer;
use dom::event::{Event, ResizeEvent};
use layers::ImageLayer;
use geom::size::Size2D;
use ShareGlContext = sharegl::platform::Context;
use std::cmp::FuzzyEq;
use task::TaskBuilder;
use vec::push;
use pipes::Chan;

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
    BeginDrawing(pipes::Chan<LayerBuffer>),
    Draw(pipes::Chan<LayerBuffer>, LayerBuffer),
    AddKeyHandler(pipes::Chan<()>),
    AddEventListener(comm::Chan<Event>),
    Exit
}

fn OSMain() -> OSMain {
    do on_osmain::<Msg> |po| {
        do platform::runmain {
            #debug("preparing to enter main loop");

            // FIXME: Use the servo options.
			let mode;
			match os::getenv("SERVO_SHARE") {
				Some(_) => mode = ShareMode,
				None => mode = GlutMode
			}

	        mainloop(mode, po);
        }
    }
}

fn mainloop(mode: Mode, po: comm::Port<Msg>) {
    let key_handlers: @DVec<pipes::Chan<()>> = @DVec();
    let event_listeners: @DVec<comm::Chan<Event>> = @DVec();

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

    let image = @layers::layers::Image(0, 0, layers::layers::RGB24Format, ~[]);
    let image_layer = @layers::layers::ImageLayer(image);
    image_layer.common.set_transform
        (image_layer.common.transform.scale(&800.0f32, &600.0f32, &1.0f32));

    let scene = @layers::scene::Scene(layers::layers::ImageLayerKind(image_layer),
                                          Size2D(800.0f32, 600.0f32));

    let done = @mut false;

    #macro[
        [#moov[x],
         unsafe { let y <- *ptr::addr_of(x); y }]
    ];

    let check_for_messages = fn@() {
        // Handle messages
        #debug("osmain: peeking");
        while po.peek() {
            match po.recv() {
              AddKeyHandler(move key_ch) => key_handlers.push(move key_ch),
              AddEventListener(event_listener) => event_listeners.push(event_listener),
              BeginDrawing(move sender) => lend_surface(surfaces, sender),
              Draw(move sender, move dt) => {
                #debug("osmain: received new frame");
                return_surface(surfaces, dt);
                lend_surface(surfaces, sender);

                let buffer = surfaces.front.layer_buffer.cairo_surface.data();
                let image = @layers::layers::Image(800, 600, layers::layers::ARGB32Format,
												   buffer);
                image_layer.set_image(image);
              }
              Exit => {
                *done = true;
              }
            }
        }
    };

    match window {
        GlutWindow(window) => {
            do glut::reshape_func(window) |width, height| {
                check_for_messages();

                #debug("osmain: window resized to %d,%d", width as int, height as int);
                for event_listeners.each |event_listener| {
                    event_listener.send(ResizeEvent(width as int, height as int));
                }
            }

            do glut::display_func() {
                check_for_messages();

                #debug("osmain: drawing to screen");

                do util::time::time(~"compositing") {
                    layers::rendergl::render_scene(context, scene);
                }

                glut::swap_buffers();
                glut::post_redisplay();
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
    fn begin_drawing(next_dt: pipes::Chan<LayerBuffer>) {
        self.send(BeginDrawing(next_dt))
    }
    fn draw(next_dt: pipes::Chan<LayerBuffer>, draw_me: LayerBuffer) {
        self.send(Draw(next_dt, draw_me))
    }
    fn add_event_listener(listener: comm::Chan<Event>) {
        self.send(AddEventListener(listener));
    }
}

struct SurfaceSet {
    mut front: Surface,
    mut back: Surface,
}

fn lend_surface(surfaces: &SurfaceSet, receiver: pipes::Chan<LayerBuffer>) {
    // We are in a position to lend out the surface?
    assert surfaces.front.have;
    // Ok then take it
    let draw_target_ref = &mut surfaces.front.layer_buffer.draw_target;
    let layer_buffer = LayerBuffer {
        cairo_surface: surfaces.front.layer_buffer.cairo_surface.clone(),
        draw_target: azure_hl::clone_mutable_draw_target(draw_target_ref),
        size: copy surfaces.front.layer_buffer.size
    };
    #debug("osmain: lending surface %?", layer_buffer);
    receiver.send(layer_buffer);
    // Now we don't have it
    surfaces.front.have = false;
    // But we (hopefully) have another!
    surfaces.front <-> surfaces.back;
    // Let's look
    assert surfaces.front.have;
}

fn return_surface(surfaces: &SurfaceSet, layer_buffer: LayerBuffer) {
    #debug("osmain: returning surface %?", layer_buffer);
    // We have room for a return
    assert surfaces.front.have;
    assert !surfaces.back.have;

    // FIXME: This is incompatible with page resizing.
    assert surfaces.back.layer_buffer.draw_target.azure_draw_target ==
        layer_buffer.draw_target.azure_draw_target;

    // Now we have it again
    surfaces.back.have = true;
}

fn SurfaceSet() -> SurfaceSet {
    SurfaceSet { front: Surface(), back: Surface() }
}

struct Surface {
    layer_buffer: LayerBuffer,
    mut have: bool,
}

fn Surface() -> Surface {
    let cairo_surface = ImageSurface(cairo::CAIRO_FORMAT_RGB24, 800, 600);
    let draw_target = DrawTarget(&cairo_surface);
    let layer_buffer = LayerBuffer {
        cairo_surface: move cairo_surface,
        draw_target: move draw_target,
        size: Size2D(800u, 600u)
    };
    Surface { layer_buffer: move layer_buffer, have: true }
}

/// A function for spawning into the platform's main thread
fn on_osmain<T: Send>(f: fn~(po: comm::Port<T>)) -> comm::Chan<T> {
    task::task().sched_mode(task::PlatformThread).spawn_listener(f)
}

// #[cfg(target_os = "linux")]
mod platform {
    pub fn runmain(f: fn()) {
        f()
    }
}

