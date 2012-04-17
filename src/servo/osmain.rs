enum msg {
    get_draw_target(comm::chan<AzDrawTargetRef>),
    add_key_handler(comm::chan<()>),
    draw(comm::chan<()>),
    exit
}

fn osmain() -> comm::chan<msg> {
    on_osmain::<msg> {|po|
        let mut key_handlers = [];

        sdl::init([
            sdl::init_video
        ]);
        let screen = sdl::video::set_video_mode(
            800, 600, 32,
            [sdl::video::swsurface],
            [sdl::video::doublebuf]);
        assert !screen.is_null();
        let sdl_surf = sdl::video::create_rgb_surface(
            [sdl::video::swsurface],
            800, 600, 32,
            0x00FF0000u32,
            0x0000FF00u32,
            0x000000FFu32,
            0x00000000u32
            );
        assert !sdl_surf.is_null();
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
        assert !cairo_surf.is_null();
        let azure_target = AzCreateDrawTargetForCairoSurface(cairo_surf);
        assert !azure_target.is_null();

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
