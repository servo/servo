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
        let azure_target = CreateDrawTargetForCairoSurface(cairo_surf);
        assert !azure_target.is_null();

        loop {
            let rect = {
                x: 200f as azure::Float,
                y: 200f as azure::Float,
                width: 100f as azure::Float,
                height: 100f as azure::Float
            };
            let color = {
                r: 0f as azure::Float,
                g: 1f as azure::Float,
                b: 0f as azure::Float,
                a: 1f as azure::Float
            };
            let pattern = CreateColorPattern(ptr::addr_of(color));
            DrawTargetFillRect(
                azure_target,
                ptr::addr_of(rect),
                unsafe { unsafe::reinterpret_cast(pattern) });
            ReleaseColorPattern(pattern);

            sdl::video::blit_surface(sdl_surf, ptr::null(),
                                     screen, ptr::null());
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
        ReleaseDrawTarget(azure_target);
        cairo_surface_destroy(cairo_surf);
        sdl::quit();
    }
}