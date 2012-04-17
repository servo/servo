import libc::c_double;
import azure::*;
import azure::bindgen::*;
import azure::cairo;
import azure::cairo::bindgen::*;

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

// Messages to the platform event handler task
enum osmain_msg {
    get_draw_target(comm::chan<AzDrawTargetRef>),
    add_key_handler(comm::chan<()>),
    draw(comm::chan<()>),
    exit
}

fn main() {
    // The platform event handler thread
    let osmain_ch = on_osmain::<osmain_msg> {|po|
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
    };

    // The drawing task
    let draw_exit_ch = task::spawn_listener {|exit_po|
        let draw_target_po = comm::port();
        comm::send(osmain_ch, get_draw_target(comm::chan(draw_target_po)));
        let draw_target = comm::recv(draw_target_po);

        let red_color = {
            r: 1f as azure::AzFloat,
            g: 0f as azure::AzFloat,
            b: 0f as azure::AzFloat,
            a: 0.8f as azure::AzFloat
        };
        let red_pattern = AzCreateColorPattern(ptr::addr_of(red_color));

        let green_color = {
            r: 0f as azure::AzFloat,
            g: 1f as azure::AzFloat,
            b: 0f as azure::AzFloat,
            a: 0.8f as azure::AzFloat
        };
        let green_pattern = AzCreateColorPattern(ptr::addr_of(green_color));

        while !comm::peek(exit_po) {
            let red_rect = {
                x: 100f as azure::AzFloat,
                y: 100f as azure::AzFloat,
                width: 200f as azure::AzFloat,
                height: 200f as azure::AzFloat
            };
            AzDrawTargetFillRect(
                draw_target,
                ptr::addr_of(red_rect),
                unsafe { unsafe::reinterpret_cast(red_pattern) }
            );
            let green_rect = {
                x: 200f as azure::AzFloat,
                y: 200f as azure::AzFloat,
                width: 200f as azure::AzFloat,
                height: 200f as azure::AzFloat
            };
            AzDrawTargetFillRect(
                draw_target,
                ptr::addr_of(green_rect),
                unsafe { unsafe::reinterpret_cast(green_pattern) }
            );
            let draw_po = comm::port();
            comm::send(osmain_ch, draw(comm::chan(draw_po)));
            comm::recv(draw_po);
        }

        AzReleaseColorPattern(red_pattern);
        AzReleaseColorPattern(green_pattern);

        let exit_confirm_ch = comm::recv(exit_po);
        comm::send(exit_confirm_ch, ());
    };

    // The keyboard handler
    task::spawn {||
        let key_po = comm::port();
        comm::send(osmain_ch, add_key_handler(comm::chan(key_po)));
        loop {
            alt comm::recv(key_po) {
              _ {
                let draw_exit_confirm_po = comm::port();
                comm::send(draw_exit_ch, comm::chan(draw_exit_confirm_po));
                comm::recv(draw_exit_confirm_po);
                comm::send(osmain_ch, exit);
                break;
              }
            }
        }
    }

}