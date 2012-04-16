import azure::cairo;

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
        let surface = sdl::video::create_rgb_surface(
            [sdl::video::swsurface],
            320, 200, 32,
            0x00FF0000u32,
            0x0000FF00u32,
            0x000000FFu32,
            0x00000000u32
            );
        assert ptr::is_not_null(surface);
        loop {
            sdl::video::blit_surface(surface, ptr::null(),
                                     screen, ptr::null());
            sdl::video::flip(screen);
            sdl::event::poll_event {|_event|
            }
        }
        sdl::quit();
    }
}