export OSMain;
export Msg, BeginDrawing, Draw, AddKeyHandler, Exit;

import azure::*;
import azure::azure_hl::DrawTarget;
import azure::bindgen::*;
import azure::cairo;
import azure::cairo::bindgen::*;
import azure::cairo_hl::ImageSurface;
import comm::*;
import dvec::{DVec, dvec};
import azure::cairo::cairo_surface_t;
import gfx::renderer::Sink;
import dom::event::{Event, ResizeEvent};
import layers::ImageLayer;
import geom::size::Size2D;
import std::cmp::fuzzy_eq;
import task::TaskBuilder;
import vec::push;

import pipes::chan;

type OSMain = comm::Chan<Msg>;

enum Msg {
    BeginDrawing(pipes::chan<AzDrawTargetRef>),
    Draw(pipes::chan<AzDrawTargetRef>, AzDrawTargetRef),
    AddKeyHandler(pipes::chan<()>),
    AddEventListener(comm::Chan<Event>),
    Exit
}

fn OSMain() -> OSMain {
    do on_osmain::<Msg> |po| {
        do platform::runmain {
            #debug("preparing to enter main loop");
	        mainloop(po);
        }
    }
}

fn mainloop(po: Port<Msg>) {
    let key_handlers: @DVec<pipes::chan<()>> = @dvec();
    let event_listeners: @DVec<comm::Chan<Event>> = @dvec();

    glut::init();
    glut::init_display_mode(glut::DOUBLE);

    #macro[
        [#move[x],
         unsafe { let y <- *ptr::addr_of(x); y }]
    ];

    let surfaces = @SurfaceSet();

    let window = glut::create_window(~"Servo");
    glut::reshape_window(window, 800, 600);

    let context = layers::rendergl::init_render_context();

    let image = @layers::layers::Image(0, 0, layers::layers::RGB24Format, ~[]);
    let image_layer = @layers::layers::ImageLayer(image);
    image_layer.common.set_transform
        (image_layer.common.transform.scale(800.0f32, 600.0f32, 1.0f32));

    let scene = @mut layers::scene::Scene(layers::layers::ImageLayerKind(image_layer),
                                          Size2D(800.0f32, 600.0f32));

    let done = @mut false;

    let check_for_messages = fn@() {
        // Handle messages
        #debug("osmain: peeking");
        while po.peek() {
            match po.recv() {
              AddKeyHandler(key_ch) => key_handlers.push(#move(key_ch)),
              AddEventListener(event_listener) => event_listeners.push(event_listener),
              BeginDrawing(sender) => lend_surface(*surfaces, sender),
              Draw(sender, dt) => {
                #debug("osmain: received new frame");
                return_surface(*surfaces, dt);
                lend_surface(*surfaces, sender);

                let buffer = surfaces.front.cairo_surface.data();
                let image = @layers::layers::Image(800, 600, layers::layers::ARGB32Format, buffer);
                image_layer.set_image(image);
              }
              exit => {
                *done = true;
              }
            }
        }
    };

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
            layers::rendergl::render_scene(context, *scene);
        }

        glut::swap_buffers();
        glut::post_redisplay();
    }

    while !*done {
        #debug("osmain: running GLUT check loop");
        glut::check_loop();
    }
}

#[doc = "
Implementation to allow the osmain channel to be used as a graphics
sink for the renderer
"]
impl OSMain : Sink {
    fn begin_drawing(+next_dt: pipes::chan<AzDrawTargetRef>) {
        self.send(BeginDrawing(next_dt))
    }
    fn draw(+next_dt: pipes::chan<AzDrawTargetRef>, draw_me: AzDrawTargetRef) {
        self.send(Draw(next_dt, draw_me))
    }
    fn add_event_listener(listener: comm::Chan<Event>) {
        self.send(AddEventListener(listener));
    }
}

struct SurfaceSet {
    mut front: Surface;
    mut back: Surface;
}

fn lend_surface(surfaces: SurfaceSet, receiver: pipes::chan<AzDrawTargetRef>) {
    // We are in a position to lend out the surface?
    assert surfaces.front.have;
    // Ok then take it
    let draw_target = surfaces.front.draw_target.azure_draw_target;
    #debug("osmain: lending surface %?", draw_target);
    receiver.send(draw_target);
    // Now we don't have it
    surfaces.front.have = false;
    // But we (hopefully) have another!
    surfaces.front <-> surfaces.back;
    // Let's look
    assert surfaces.front.have;
}

fn return_surface(surfaces: SurfaceSet, draw_target: AzDrawTargetRef) {
    #debug("osmain: returning surface %?", draw_target);
    // We have room for a return
    assert surfaces.front.have;
    assert !surfaces.back.have;

    // FIXME: This is incompatible with page resizing.
    assert surfaces.back.draw_target.azure_draw_target == draw_target;

    // Now we have it again
    surfaces.back.have = true;
}

fn SurfaceSet() -> SurfaceSet {
    SurfaceSet { front: Surface(), back: Surface() }
}

struct Surface {
    cairo_surface: ImageSurface;
    draw_target: DrawTarget;
    mut have: bool;
}

fn Surface() -> Surface {
    let cairo_surface = ImageSurface(cairo::CAIRO_FORMAT_RGB24, 800, 600);
    let draw_target = DrawTarget(cairo_surface);
    Surface { cairo_surface: cairo_surface, draw_target: draw_target, have: true }
}

#[doc = "A function for spawning into the platform's main thread"]
fn on_osmain<T: send>(+f: fn~(comm::Port<T>)) -> comm::Chan<T> {
    task::task().sched_mode(task::PlatformThread).spawn_listener(f)
}

// #[cfg(target_os = "linux")]
mod platform {
    fn runmain(f: fn()) {
        f()
    }
}

