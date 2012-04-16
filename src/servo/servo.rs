import azure::cairo;
import azure::cairo::bindgen::*;

fn on_main(f: fn~()) {
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
    on_main {||
        sdl::init([
            sdl::init_video
        ]);
        let screen = sdl::video::set_video_mode(
            320, 200, 32,
            [sdl::video::swsurface],
            [sdl::video::doublebuf]);
        assert ptr::is_not_null(screen);
        let sdl_surf = sdl::video::create_rgb_surface(
            [sdl::video::swsurface],
            320, 200, 32,
            0x00FF0000u32,
            0x0000FF00u32,
            0x000000FFu32,
            0x00000000u32
            );
        assert ptr::is_not_null(sdl_surf);
        let cairo_surf = unsafe {
            cairo_image_surface_create_for_data(
                unsafe::reinterpret_cast((*sdl_surf).pixels),
                cairo::CAIRO_FORMAT_RGB24,
                (*sdl_surf).w,
                (*sdl_surf).h,
                (*sdl_surf).pitch as libc::c_int
            )
        };
        loop {
            sdl::video::blit_surface(sdl_surf, ptr::null(),
                                     screen, ptr::null());
            sdl::video::flip(screen);
            sdl::event::poll_event {|_event|
            }
        }
        cairo_surface_destroy(cairo_surf);
        sdl::quit();
    }
}