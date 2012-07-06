export OSMain;
export Msg, BeginDrawing, Draw, AddKeyHandler, Exit;

import azure::*;
import azure::bindgen::*;
import azure::cairo;
import azure::cairo::bindgen::*;
import comm::*;
import azure::cairo::cairo_surface_t;
import gfx::renderer::{Sink};
import layers::ImageLayer;
import geom::size::Size2D;
import std::cmp::fuzzy_eq;

type OSMain = chan<Msg>;

enum Msg {
    BeginDrawing(chan<AzDrawTargetRef>),
    Draw(chan<AzDrawTargetRef>, AzDrawTargetRef),
    AddKeyHandler(chan<()>),
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
              BeginDrawing(sender) {
                lend_surface(surfaces, sender);
              }
              Draw(sender, dt) {
                return_surface(surfaces, dt);
                lend_surface(surfaces, sender);
                //assert surfaces.s1.surf.az_target == dt;

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

/*
#[cfg(target_os = "macos")]
mod platform {
    use cocoa;
    import cocoa::base::*;

    mod NSApplication {
        fn sharedApplication() -> id {
            let klass = str::as_c_str("NSApplication", |s| objc::objc_getClass(s));

            let sel = str::as_c_str("sharedApplication", |s| objc::sel_registerName(s));

            let nsapp = objc::objc_msgSend(klass, sel);
            #debug("nsapp: %d", (nsapp as int));

	    ret nsapp;
        }
    }

    mod NSAutoreleasePool {
        fn alloc() -> id {
            let klass = str::as_c_str("NSAutoreleasePool", |s| objc::objc_getClass(s));
            let sel = str::as_c_str("alloc", |s| objc::sel_registerName(s));
            let pool = objc::objc_msgSend(klass, sel);
            #debug("autorelease pool: %?", pool);
            ret pool;
        }
        fn init(pool: id) {
            let sel = str::as_c_str("init", |s| objc::sel_registerName(s));
            objc::objc_msgSend(pool, sel);
        }
        fn release(pool: id) {
            let sel = str::as_c_str("release", |s| objc::sel_registerName(s));
            objc::objc_msgSend(pool, sel);
        }
    }

    mod NSApp {
         fn setDelegate(nsapp: id, main: id) {
	         #debug("NSApp::setDelegate");
	         let sel = str::as_c_str("setDelegate:", |s| objc::sel_registerName(s));
	         cocoa::msgSend1Id(nsapp, sel, main);
         }

        fn run(nsapp: id) {
	        #debug("NSApp::run");
            let sel = str::as_c_str("run", |s| objc::sel_registerName(s));
            objc::objc_msgSend(nsapp, sel);
         }
    }

    mod MainObj {
         extern fn applicationDidFinishLaunching(this: id, _sel: SEL) {
	         #debug("applicationDidFinishLaunching");

	         let fptr: *fn() = ptr::null();
	         str::as_c_str("fptr", |name| {
	             let outValue = unsafe { unsafe::reinterpret_cast(ptr::addr_of(fptr)) };
                 #debug("*fptr %?", outValue);
                 objc::object_getInstanceVariable(this, name, outValue)
             });

	         #debug("getting osmain fptr: %?", fptr);

	         unsafe {
	             // FIXME: We probably don't want to run the main routine in a foreign function
                 (*fptr)();
             }
	     }

    	fn create(f: fn()) -> id {
            let NSObject = str::as_c_str("NSObject", |s| objc::objc_getClass(s));
	        let MainObj = str::as_c_str("MainObj", |s| {
	            objc::objc_allocateClassPair(NSObject, s, 0 as libc::size_t)
	        });

             // Add a field to our class to contain a pointer to a rust closure
	        let res = str::as_c_str("fptr", |name| {
                str::as_c_str("^i", |types| {
                     objc::class_addIvar(MainObj, name,
                                         sys::size_of::<libc::uintptr_t>() as libc::size_t,
                                         16u8, types)
                })
            });
 	        assert res == true;

	        let launchfn = str::as_c_str("applicationDidFinishLaunching:", |s| {
	            objc::sel_registerName(s)
	        });
	        let _ = str::as_c_str("@@:", |types| {
	            objc::class_addMethod(MainObj, launchfn, applicationDidFinishLaunching, types)
	        });

	        objc::objc_registerClassPair(MainObj);

            let sel = str::as_c_str("alloc", |s| objc::sel_registerName(s));
            let mainobj = objc::objc_msgSend(MainObj, sel);

            let sel = str::as_c_str("init", |s| objc::sel_registerName(s));
            objc::objc_msgSend(mainobj, sel);

	        let fptr = ptr::addr_of(f);
	        str::as_c_str("fptr", |name| {
	            #debug("setting osmain fptr: %?", fptr);
		        let value = unsafe { unsafe::reinterpret_cast(fptr) };
                #debug("*fptr: %?", value);
                objc::object_setInstanceVariable(mainobj, name, value)
            });

	        ret mainobj;
	    }
	    fn release(mainobj: id) {
            let sel = str::as_c_str("release", |s| objc::sel_registerName(s));
            objc::objc_msgSend(mainobj, sel);
	    }
    }

    fn runmain(f: fn()) {
	let pool = NSAutoreleasePool::alloc();
	NSAutoreleasePool::init(pool);
        let NSApp = NSApplication::sharedApplication();

        let mainobj = MainObj::create(f);
	NSApp::setDelegate(NSApp, mainobj);
	NSApp::run(NSApp);
	
	MainObj::release(mainobj);	
	NSAutoreleasePool::release(pool);
    }
}
*/

