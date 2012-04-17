import libc::c_double;
import azure::bindgen::*;
import azure::cairo;
import azure::cairo::bindgen::*;

fn on_osmain(f: fn~()) {
    let builder = task::builder();
    let opts = {
        sched: some({
            mode: task::osmain,
            native_stack_size: none
        })
        with task::get_opts(builder)
    };
    task::set_opts(builder, opts);
    task::run(builder, f);
}

fn main() {
    on_osmain {||
        sdl::init([
            sdl::init_video
        ]);
        let screen = sdl::video::set_video_mode(
            320, 200, 32,
            [sdl::video::swsurface],
            [sdl::video::doublebuf]);
        assert !screen.is_null();
        let sdl_surf = sdl::video::create_rgb_surface(
            [sdl::video::swsurface],
            320, 200, 32,
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
            let color = {
                r: 1f as azure::AzFloat,
                g: 1f as azure::AzFloat,
                b: 1f as azure::AzFloat,
                a: 0.5f as azure::AzFloat
            };
            let pattern = AzCreateColorPattern(ptr::addr_of(color));
            let rect = {
                x: 100f as azure::AzFloat,
                y: 100f as azure::AzFloat,
                width: 100f as azure::AzFloat,
                height: 100f as azure::AzFloat
            };
            AzDrawTargetFillRect(
                azure_target,
                ptr::addr_of(rect),
                unsafe { unsafe::reinterpret_cast(pattern) });
            AzReleaseColorPattern(pattern);

            sdl::video::unlock_surface(sdl_surf);
            sdl::video::blit_surface(sdl_surf, ptr::null(),
                                     screen, ptr::null());
            sdl::video::lock_surface(sdl_surf);
            sdl::video::flip(screen);
            let mut mustbreak = false;
            sdl::event::poll_event {|event|
                alt event {
                  sdl::event::keyup_event(_) { mustbreak = true; }
                  _ { }
                }
            }
            if mustbreak { break }
        }
        AzReleaseDrawTarget(azure_target);
        cairo_surface_destroy(cairo_surf);
        sdl::video::unlock_surface(sdl_surf);
        sdl::quit();
    }
}