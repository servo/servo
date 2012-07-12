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

type OSMain = chan<Msg>;

enum Msg {
    BeginDrawing(chan<AzDrawTargetRef>),
    Draw(chan<AzDrawTargetRef>, AzDrawTargetRef),
    AddKeyHandler(chan<()>),
    AddEventListener(chan<Event>),
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
    let mut key_handlers: [chan<()>] = [];
    let event_listeners: @dvec<chan<Event>> = @dvec();

    glut::init();
    glut::init_display_mode(glut::DOUBLE);

    let surfaces = surface_set();

    let window = glut::create_window("Servo");
    glut::reshape_window(window, 800, 600);

    let context = layers::rendergl::init_render_context();

    let image = @layers::layers::Image(0, 0, layers::layers::RGB24Format, ~[]);
    let image_layer = @layers::layers::ImageLayer(image);
    image_layer.common.set_transform
        (image_layer.common.transform.scale(800.0f32, 600.0f32, 1.0f32));

    let scene = @mut layers::scene::Scene(layers::layers::ImageLayerKind(image_layer),
                                          Size2D(800.0f32, 600.0f32));

    do glut::reshape_func(window) |width, height| {
        #debug("osmain: window resized to %d,%d", width as int, height as int);
        for event_listeners.each |event_listener| {
            event_listener.send(ResizeEvent(width as int, height as int));
        }
    }

    loop {
        do glut::display_func() {
            #debug("osmain: drawing to screen");

            layers::rendergl::render_scene(context, *scene);
            glut::swap_buffers();
            glut::post_redisplay();
        }

        // Handle messages
        if po.peek() {
            alt check po.recv() {
              AddKeyHandler(key_ch) {
                key_handlers += [key_ch];
              }
              AddEventListener(event_listener) {
                event_listeners.push(event_listener);
              }
              BeginDrawing(sender) {
                lend_surface(surfaces, sender);
              }
              Draw(sender, dt) {
                return_surface(surfaces, dt);
                lend_surface(surfaces, sender);

                let mut image_data;
                unsafe {
                    let buffer = cairo_image_surface_get_data(surfaces.s1.surf.cairo_surf);
                    image_data = vec::unsafe::from_buf(buffer, 800 * 600 * 4);
                }

                let image =
                    @layers::layers::Image(800, 600, layers::layers::RGB24Format,
                                           layers::util::convert_rgb32_to_rgb24(image_data));
                image_layer.set_image(image);

                glut::post_redisplay();
              }
              exit { break; }
            }
        }

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
    fn begin_drawing(next_dt: chan<AzDrawTargetRef>) {
        self.send(BeginDrawing(next_dt))
    }
    fn draw(next_dt: chan<AzDrawTargetRef>, draw_me: AzDrawTargetRef) {
        self.send(Draw(next_dt, draw_me))
    }
    fn add_event_listener(listener: chan<Event>) {
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

fn lend_surface(surfaces: surface_set, recvr: chan<AzDrawTargetRef>) {
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

fn destroy_surface(surface: surface) {
    AzReleaseDrawTarget(surface.az_target);
    cairo_surface_destroy(surface.cairo_surf);
}

#[doc = "A function for spawning into the platform's main thread"]
fn on_osmain<T: send>(+f: fn~(comm::port<T>)) -> comm::chan<T> {
    let builder = task::builder();
    let opts = {
        sched: some({
            mode: task::osmain,
            foreign_stack_size: none
        })
        with task::get_opts(builder)
    };
    task::set_opts(builder, opts);
    ret task::run_listener(builder, f);
}

// #[cfg(target_os = "linux")]
mod platform {
    fn runmain(f: fn()) {
        f()
    }
}

