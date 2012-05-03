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
            let klass = str::as_c_str("NSApplication") { |s|
                objc::objc_getClass(s)
            };

            let sel = str::as_c_str("sharedApplication") { |s|
                objc::sel_registerName(s)
            };

            let nsapp = objc::objc_msgSend(klass, sel);
            #debug("nsapp: %d", (nsapp as int));

	    ret nsapp;
        }
    }

    mod NSAutoreleasePool {
        fn alloc() -> id {
            let klass = str::as_c_str("NSAutoreleasePool") { |s|
                objc::objc_getClass(s)
            };
            let sel = str::as_c_str("alloc") { |s|
                objc::sel_registerName(s)
            };
            let pool = objc::objc_msgSend(klass, sel);
            #debug("autorelease pool: %?", pool);
            ret pool;
        }
        fn init(pool: id) {
            let sel = str::as_c_str("init") { |s|
                objc::sel_registerName(s)
            };
            objc::objc_msgSend(pool, sel);
        }
        fn release(pool: id) {
            let sel = str::as_c_str("release") { |s|
                objc::sel_registerName(s)
            };
            objc::objc_msgSend(pool, sel);
        }
    }

    mod NSApp {
         fn setDelegate(nsapp: id, main: id) {
	     #debug("NSApp::setDelegate");
	     let sel = str::as_c_str("setDelegate:") { |s|
	         objc::sel_registerName(s)
	     };
	     cocoa::msgSend1Id(nsapp, sel, main);
         }

         fn run(nsapp: id) {
	    #debug("NSApp::run");
            let sel = str::as_c_str("run") { |s|
                objc::sel_registerName(s)
            };
            objc::objc_msgSend(nsapp, sel);
         }
    }

    mod MainObj {
         crust fn applicationDidFinishLaunching(this: id, _sel: SEL) {
	     #debug("applicationDidFinishLaunching");

	     let fptr: *fn() = ptr::null();
	     str::as_c_str("fptr") { |name|
	         let outValue = unsafe { unsafe::reinterpret_cast(ptr::addr_of(fptr)) };
                 #debug("*fptr %?", outValue);
                 objc::object_getInstanceVariable(this, name, outValue)
             };

	     #debug("getting osmain fptr: %?", fptr);

	     unsafe {
	         // FIXME: We probably don't want to run the main routine in a crust function
                 (*fptr)();
             }
	 }

    	 fn create(f: fn()) -> id {
             let NSObject = str::as_c_str("NSObject") { |s|
	         objc::objc_getClass(s)
             };
	     let MainObj = str::as_c_str("MainObj") { |s|
	         objc::objc_allocateClassPair(NSObject, s, 0 as libc::size_t)
	     };

             // Add a field to our class to contain a pointer to a rust closure
	     let res = str::as_c_str("fptr") { |name|
                 str::as_c_str("^i") { |types|
                     objc::class_addIvar(MainObj, name,
                                         sys::size_of::<libc::uintptr_t>() as libc::size_t,
                                         16u8, types)
                 }
             };
 	     assert res == true;

	     let launchfn = str::as_c_str("applicationDidFinishLaunching:") { |s|
	         objc::sel_registerName(s)
	     };
	     let _ = str::as_c_str("@@:") { |types|
	         objc::class_addMethod(MainObj, launchfn, applicationDidFinishLaunching, types)
	     };

	     objc::objc_registerClassPair(MainObj);

             let sel = str::as_c_str("alloc") { |s|
                 objc::sel_registerName(s)
             };
             let mainobj = objc::objc_msgSend(MainObj, sel);

             let sel = str::as_c_str("init") { |s|
                 objc::sel_registerName(s)
             };
             objc::objc_msgSend(mainobj, sel);

	     let fptr = ptr::addr_of(f);
	     str::as_c_str("fptr") { |name|
	         #debug("setting osmain fptr: %?", fptr);
		 let value = unsafe { unsafe::reinterpret_cast(fptr) };
                 #debug("*fptr: %?", value);
                 objc::object_setInstanceVariable(mainobj, name, value)
             };

	     ret mainobj;
	 }
	 fn release(mainobj: id) {
             let sel = str::as_c_str("release") { |s|
                 objc::sel_registerName(s)
             };
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

enum msg {
    get_draw_target(comm::chan<AzDrawTargetRef>),
    add_key_handler(comm::chan<()>),
    draw(comm::chan<()>),
    exit
}

fn osmain() -> comm::chan<msg> {
    on_osmain::<msg> {|po|
        platform::runmain {||
            #debug("preparing to enter main loop");
	    mainloop(po);
        }
    }
}

// A function for spawning into the platform's main thread
fn on_osmain<T: send>(f: fn~(comm::port<T>)) -> comm::chan<T> {
    let builder = task::builder();
    let opts = {
        sched: some({
            mode: task::osmain,
            native_stack_size: none
        })
        with task::get_opts(builder)
    };
    task::set_opts(builder, opts);
    ret task::run_listener(builder, f);
}

fn mainloop(po: comm::port<msg>) {

    let mut key_handlers = [];

    sdl::init([
        sdl::init_video
    ]);
    let screen = sdl::video::set_video_mode(
        800, 600, 32,
        [sdl::video::swsurface],
        [sdl::video::doublebuf]);
    assert !ptr::is_null(screen);
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

    loop {
        sdl::event::poll_event {|event|
            alt event {
              sdl::event::keydown_event(_) {
                key_handlers.iter {|key_ch|
                    comm::send(key_ch, ())
                }
              }
              _ { }
            }
        }

        // Handle messages
        if comm::peek(po) {
            alt check comm::recv(po) {
              add_key_handler(key_ch) {
                key_handlers += [key_ch];
              }
              get_draw_target(response_ch) {
                comm::send(response_ch, azure_target);
              }
              draw(response_ch) {
                sdl::video::unlock_surface(sdl_surf);
                sdl::video::blit_surface(sdl_surf, ptr::null(),
                                         screen, ptr::null());
                sdl::video::lock_surface(sdl_surf);
                sdl::video::flip(screen);
                comm::send(response_ch, ());
              }
              exit { break; }
            }
        }
    }
    AzReleaseDrawTarget(azure_target);
    cairo_surface_destroy(cairo_surf);
    sdl::video::unlock_surface(sdl_surf);
    sdl::quit();
}