export OSMain;
export Msg, BeginDrawing, Draw, AddKeyHandler, Exit;

import azure::*;
import azure::bindgen::*;
import azure::cairo;
import azure::cairo::bindgen::*;
import comm::*;
import azure::cairo::cairo_surface_t;
import gfx::renderer::{Sink};

type OSMain = chan<Msg>;

enum Msg {
    BeginDrawing(chan<AzDrawTargetRef>),
    Draw(chan<AzDrawTargetRef>, AzDrawTargetRef),
    AddKeyHandler(chan<()>),
    Exit
}

fn OSMain() -> OSMain {
    on_osmain::<Msg>(|po| {
        platform::runmain(|| {
            #debug("preparing to enter main loop");
	        mainloop(po);
        })
    })
}

fn mainloop(po: port<Msg>) {

    let mut key_handlers: [chan<()>] = [];

    sdl::init([
        sdl::init_video
    ]);

    let screen = sdl::video::set_video_mode(
        800, 600, 32,
        [sdl::video::swsurface],
        [sdl::video::doublebuf]);
    assert !ptr::is_null(screen);

    let surfaces = surface_set();

    loop {
        sdl::event::poll_event(|event| {

            alt event {
              sdl::event::keydown_event(_) {
                key_handlers.iter(|key_ch| key_ch.send(()))
              }
              _ { }
            }
        });

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

                #debug("osmain: drawing to screen");
                assert surfaces.s1.surf.az_target == dt;
                let sdl_surf = surfaces.s1.surf.sdl_surf;

                cairo_surface_flush(surfaces.s1.surf.cairo_surf);
                sdl::video::unlock_surface(sdl_surf);
                sdl::video::blit_surface(sdl_surf, ptr::null(),
                                         screen, ptr::null());
                sdl::video::lock_surface(sdl_surf);
                sdl::video::flip(screen);
              }
              exit { break; }
            }
        }
    }
    destroy_surface(surfaces.s1.surf);
    destroy_surface(surfaces.s2.surf);
    sdl::quit();
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
    sdl_surf: *sdl::video::surface,
    cairo_surf: *cairo_surface_t,
    az_target: AzDrawTargetRef
};

fn mk_surface() -> surface {
    let sdl_surf = sdl::video::create_rgb_surface(
        [sdl::video::swsurface],
        800, 600, 32,
        0x00FF0000u32,
        0x0000FF00u32,
        0x000000FFu32,
        0x00000000u32
        );
    assert !ptr::is_null(sdl_surf);
    sdl::video::lock_surface(sdl_surf);
    let cairo_surf = unsafe {
        cairo_image_surface_create_for_data(
            unsafe::reinterpret_cast((*sdl_surf).pixels),
            cairo::CAIRO_FORMAT_RGB24,
            (*sdl_surf).w,
            (*sdl_surf).h,
            (*sdl_surf).pitch as libc::c_int
        )
    };
    assert !ptr::is_null(cairo_surf);

    let azure_target = AzCreateDrawTargetForCairoSurface(cairo_surf);
    assert !ptr::is_null(azure_target);

    {
        sdl_surf: sdl_surf,
        cairo_surf: cairo_surf,
        az_target: azure_target
    }
}

fn destroy_surface(surface: surface) {
    AzReleaseDrawTarget(surface.az_target);
    cairo_surface_destroy(surface.cairo_surf);
    sdl::video::unlock_surface(surface.sdl_surf);
    sdl::video::free_surface(surface.sdl_surf);
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

#[cfg(target_os = "linux")]
mod platform {
    fn runmain(f: fn()) {
        f()
    }
}

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
