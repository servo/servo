import libc::c_double;
import azure::*;
import azure::bindgen::*;
import azure::cairo;
import azure::cairo::bindgen::*;

// FIXME: Busy wait hack
fn sleep() {
    iter::repeat(100000u) {||
        task::yield();
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

// Messages to the platform event handler task
enum osmain_msg {
    om_get_draw_target(comm::chan<AzDrawTargetRef>),
    om_add_key_handler(comm::chan<()>),
    om_draw(comm::chan<()>),
    om_exit
}

enum draw_msg {
    dm_draw(model),
    dm_exit(comm::chan<()>)
}

type model = {
    x1: int, y1: int, w1: int, h1: int,
    x2: int, y2: int, w2: int, h2: int
};

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
                  om_add_key_handler(key_ch) {
                    key_handlers += [key_ch];
                  }
                  om_get_draw_target(response_ch) {
                    comm::send(response_ch, azure_target);
                  }
                  om_draw(response_ch) {
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
    let draw_ch = task::spawn_listener {|po|
        let draw_target_po = comm::port();
        comm::send(osmain_ch, om_get_draw_target(comm::chan(draw_target_po)));
        let draw_target = comm::recv(draw_target_po);

        let black_color = {
            r: 0f as azure::AzFloat,
            g: 0f as azure::AzFloat,
            b: 0f as azure::AzFloat,
            a: 1f as azure::AzFloat
        };
        let black_pattern = AzCreateColorPattern(ptr::addr_of(black_color));

        let red_color = {
            r: 1f as azure::AzFloat,
            g: 0f as azure::AzFloat,
            b: 0f as azure::AzFloat,
            a: 0.5f as azure::AzFloat
        };
        let red_pattern = AzCreateColorPattern(ptr::addr_of(red_color));

        let green_color = {
            r: 0f as azure::AzFloat,
            g: 1f as azure::AzFloat,
            b: 0f as azure::AzFloat,
            a: 0.5f as azure::AzFloat
        };
        let green_pattern = AzCreateColorPattern(ptr::addr_of(green_color));

        let mut exit_confirm_ch = none;
        loop {
            alt comm::recv::<draw_msg>(po) {
              dm_draw(model) {
                let black_rect = {
                    x: 0 as azure::AzFloat,
                    y: 0 as azure::AzFloat,
                    width: 800 as azure::AzFloat,
                    height: 600 as azure::AzFloat,
                };
                AzDrawTargetFillRect(
                    draw_target,
                    ptr::addr_of(black_rect),
                    unsafe { unsafe::reinterpret_cast(black_pattern) }
                );

                let red_rect = {
                    x: model.x1 as azure::AzFloat,
                    y: model.y1 as azure::AzFloat,
                    width: model.w1 as azure::AzFloat,
                    height: model.h1 as azure::AzFloat
                };
                AzDrawTargetFillRect(
                    draw_target,
                    ptr::addr_of(red_rect),
                    unsafe { unsafe::reinterpret_cast(red_pattern) }
                );
                let green_rect = {
                    x: model.x2 as azure::AzFloat,
                    y: model.y2 as azure::AzFloat,
                    width: model.w2 as azure::AzFloat,
                    height: model.h2 as azure::AzFloat
                };
                AzDrawTargetFillRect(
                    draw_target,
                    ptr::addr_of(green_rect),
                    unsafe { unsafe::reinterpret_cast(green_pattern) }
                );
                let draw_po = comm::port();
                comm::send(osmain_ch, om_draw(comm::chan(draw_po)));
                comm::recv(draw_po);
              }
              dm_exit(response_ch) {
                exit_confirm_ch = some(response_ch);
                break;
              }
            }
        }

        AzReleaseColorPattern(red_pattern);
        AzReleaseColorPattern(green_pattern);

        assert exit_confirm_ch.is_some();
        comm::send(exit_confirm_ch.get(), ());
    };

    // The model
    let model_ch = task::spawn_listener {|po|
        let mut x1 = 100;
        let mut y1 = 100;
        let mut w1 = 200;
        let mut h1 = 200;
        let mut x2 = 200;
        let mut y2 = 200;
        let mut w2 = 300;
        let mut h2 = 300;

        while !comm::peek(po) {
            let model = {
                x1: x1, y1: y1, w1: w1, h1: h1,
                x2: x2, y2: y2, w2: w2, h2: h2
            };
            comm::send(draw_ch, dm_draw(model));

            sleep();

            x1 += 1;
            y1 += 1;
            x2 -= 1;
            y2 -= 1;
            if x1 > 800 { x1 = 0 }
            if y1 > 600 { y1 = 0 }
            if x2 < 0 { x2 = 800 }
            if y2 < 0 { y2 = 600 }
        }
    };

    // The keyboard handler
    task::spawn {||
        let key_po = comm::port();
        comm::send(osmain_ch, om_add_key_handler(comm::chan(key_po)));
        loop {
            alt comm::recv(key_po) {
              _ {
                comm::send(model_ch, ());
                let draw_exit_confirm_po = comm::port();
                comm::send(draw_ch, dm_exit(comm::chan(draw_exit_confirm_po)));
                comm::recv(draw_exit_confirm_po);
                comm::send(osmain_ch, om_exit);
                break;
              }
            }
        }
    }
}