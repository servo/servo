export OSMain;
export Msg, BeginDrawing, Draw, AddKeyHandler, Exit;

import azure::*;
import azure::bindgen::*;
import azure::cairo;
import azure::cairo::bindgen::*;
import comm::*;
import dvec::{dvec, extensions};
import azure::cairo::cairo_surface_t;
import gfx::renderer::{Sink};
import dom::event::{Event, ResizeEvent};
import layers::ImageLayer;
import geom::size::Size2D;
import std::cmp::fuzzy_eq;
import task::task_builder;
import vec::push;

import pipes::chan;

type OSMain = comm::chan<Msg>;

enum Msg {
    BeginDrawing(pipes::chan<AzDrawTargetRef>),
    Draw(pipes::chan<AzDrawTargetRef>, AzDrawTargetRef),
    AddKeyHandler(pipes::chan<()>),
    AddEventListener(comm::chan<Event>),
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

fn mainloop(po: port<Msg>) {
    let key_handlers: @dvec<pipes::chan<()>> = @dvec();
    let event_listeners: @dvec<comm::chan<Event>> = @dvec();

    glut::init();
    glut::init_display_mode(glut::DOUBLE);

    #macro[
        [#move[x],
         unsafe { let y <- *ptr::addr_of(x); y }]
    ];

    let surfaces = @surface_set();

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

                let mut image_data;
                unsafe {
                    let buffer = cairo_image_surface_get_data(surfaces.s1.surf.cairo_surf);
                    image_data = vec::unsafe::from_buf(buffer, 800 * 600 * 4);
                }

                let image =
                    @layers::layers::Image(800, 600, layers::layers::ARGB32Format, image_data);
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

    destroy_surface(surfaces.s1.surf);
    destroy_surface(surfaces.s2.surf);
}

#[doc = "
Implementation to allow the osmain channel to be used as a graphics
sink for the renderer
"]
impl OSMain of Sink for OSMain {
    fn begin_drawing(+next_dt: pipes::chan<AzDrawTargetRef>) {
        self.send(BeginDrawing(next_dt))
    }
    fn draw(+next_dt: pipes::chan<AzDrawTargetRef>, draw_me: AzDrawTargetRef) {
        self.send(Draw(next_dt, draw_me))
    }
    fn add_event_listener(listener: comm::chan<Event>) {
        self.send(AddEventListener(listener));
    }
}

type surface_set = {
    mut s1: {
        surf: surface,
        have: bool
    },
    mut s2: {
        surf: surface,
        have: bool
    }
};

fn lend_surface(surfaces: surface_set, recvr: pipes::chan<AzDrawTargetRef>) {
    // We are in a position to lend out the surface?
    assert surfaces.s1.have;
    // Ok then take it
    let dt1 = surfaces.s1.surf.az_target;
    #debug("osmain: lending surface %?", dt1);
    recvr.send(dt1);
    // Now we don't have it
    surfaces.s1 = {
        have: false
        with surfaces.s1
    };
    // But we (hopefully) have another!
    surfaces.s1 <-> surfaces.s2;
    // Let's look
    assert surfaces.s1.have;
}

fn return_surface(surfaces: surface_set, dt: AzDrawTargetRef) {
    #debug("osmain: returning surface %?", dt);
    // We have room for a return
    assert surfaces.s1.have;
    assert !surfaces.s2.have;
    assert surfaces.s2.surf.az_target == dt;
    // Now we have it again
    surfaces.s2 = {
        have: true
        with surfaces.s2
    };
}

fn surface_set() -> surface_set {
    {
        mut s1: {
            surf: mk_surface(),
            have: true
        },
        mut s2: {
            surf: mk_surface(),
            have: true
        }
    }
}

type surface = {
    cairo_surf: *cairo_surface_t,
    az_target: AzDrawTargetRef
};

fn mk_surface() -> surface {
    let cairo_surf = cairo_image_surface_create(cairo::CAIRO_FORMAT_RGB24, 800, 600);

    assert !ptr::is_null(cairo_surf);

    let azure_target = AzCreateDrawTargetForCairoSurface(cairo_surf);
    assert !ptr::is_null(azure_target);

    {
        cairo_surf: cairo_surf,
        az_target: azure_target
    }
}

fn destroy_surface(+surface: surface) {
    AzReleaseDrawTarget(surface.az_target);
    cairo_surface_destroy(surface.cairo_surf);
}

#[doc = "A function for spawning into the platform's main thread"]
fn on_osmain<T: send>(+f: fn~(comm::port<T>)) -> comm::chan<T> {
    task::task().sched_mode(task::platform_thread).spawn_listener(f)
}

// #[cfg(target_os = "linux")]
mod platform {
    fn runmain(f: fn()) {
        f()
    }
}

