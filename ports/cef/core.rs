/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use command_line::command_line_init;
use interfaces::cef_app_t;
use types::{cef_main_args_t, cef_settings_t};
use window;

use compositing::windowing::{IdleWindowEvent, WindowEvent};
use geom::size::TypedSize2D;
use glfw_app;
use libc::{c_char, c_int, c_void};
use native;
use rustrt::local::Local;
use servo::Browser;
use servo_util::opts;
use servo_util::opts::OpenGL;
use std::c_str::CString;
use std::cell::RefCell;
use std::rc::Rc;
use std::rt;

const MAX_RENDERING_THREADS: uint = 128;

// TODO(pcwalton): Get the home page via the CEF API.
static HOME_URL: &'static str = "http://s27.postimg.org/vqbtrolyr/servo.jpg";

// TODO(pcwalton): Support multiple windows.
pub enum ServoCefGlobals {
    OnScreenGlobals(RefCell<Rc<glfw_app::window::Window>>,
                    RefCell<Browser<glfw_app::window::Window>>),
    OffScreenGlobals(RefCell<Rc<window::Window>>, RefCell<Browser<window::Window>>),
}

local_data_key!(pub globals: ServoCefGlobals)

local_data_key!(pub message_queue: RefCell<Vec<WindowEvent>>)

// Copied from `libnative/lib.rs`.
static OS_DEFAULT_STACK_ESTIMATE: uint = 2 * (1 << 20);

static CEF_API_HASH_UNIVERSAL: &'static [u8] = b"8efd129f4afc344bd04b2feb7f73a149b6c4e27f\0";
#[cfg(target_os="windows")]
static CEF_API_HASH_PLATFORM: &'static [u8] = b"5c7f3e50ff5265985d11dc1a466513e25748bedd\0";
#[cfg(target_os="macos")]
static CEF_API_HASH_PLATFORM: &'static [u8] = b"6813214accbf2ebfb6bdcf8d00654650b251bf3d\0";
#[cfg(target_os="linux")]
static CEF_API_HASH_PLATFORM: &'static [u8] = b"2bc564c3871965ef3a2531b528bda3e17fa17a6d\0";

#[no_mangle]
pub extern "C" fn cef_initialize(args: *const cef_main_args_t,
                                 settings: *mut cef_settings_t,
                                 application: *mut cef_app_t,
                                 _windows_sandbox_info: *const c_void)
                                 -> c_int {
    if args.is_null() {
        return 0;
    }

    unsafe {
        rt::init((*args).argc as int, (*args).argv);
        command_line_init((*args).argc, (*args).argv);

        if !application.is_null() {
            (*application).get_browser_process_handler.map(|cb| {
                    let handler = cb(application);
                    if handler.is_not_null() {
                        (*handler).on_context_initialized.map(|hcb| hcb(handler));
                    }
            });
        }
    }

    create_rust_task();

    message_queue.replace(Some(RefCell::new(Vec::new())));

    let urls = vec![HOME_URL.to_string()];
    opts::set_opts(opts::Opts {
        urls: urls,
        n_paint_threads: 1,
        gpu_painting: false,
        tile_size: 512,
        device_pixels_per_px: None,
        time_profiler_period: None,
        memory_profiler_period: None,
        enable_experimental: false,
        nonincremental_layout: false,
        layout_threads: unsafe {
            if ((*settings).rendering_threads as uint) < 1 {
                1
            } else if (*settings).rendering_threads as uint > MAX_RENDERING_THREADS {
                MAX_RENDERING_THREADS
            } else {
                (*settings).rendering_threads as uint
            }
        },
        output_file: None,
        headless: false,
        hard_fail: false,
        bubble_inline_sizes_separately: false,
        show_debug_borders: false,
        show_debug_fragment_borders: false,
        enable_text_antialiasing: true,
        trace_layout: false,
        devtools_port: None,
        initial_window_size: TypedSize2D(800, 600),
        profile_tasks: false,
        user_agent: None,
        dump_flow_tree: false,
        validate_display_list_geometry: false,
        render_api: OpenGL,
    });

    return 1
}

// Copied from `libnative/lib.rs`.
fn create_rust_task() {
    let something_around_the_top_of_the_stack = 1;
    let addr = &something_around_the_top_of_the_stack as *const int;
    let my_stack_top = addr as uint;

    // FIXME #11359 we just assume that this thread has a stack of a
    // certain size, and estimate that there's at most 20KB of stack
    // frames above our current position.

    let my_stack_bottom = my_stack_top + 20000 - OS_DEFAULT_STACK_ESTIMATE;

    let task = native::task::new((my_stack_bottom, my_stack_top), rt::thread::main_guard_page());
    Local::put(task);
}

#[no_mangle]
pub extern "C" fn cef_shutdown() {
}

#[no_mangle]
pub extern "C" fn cef_run_message_loop() {
    let mut the_globals = globals.get();
    let the_globals = the_globals.as_mut().unwrap();
    match **the_globals {
        OnScreenGlobals(ref window, ref browser) => {
            while browser.borrow_mut().handle_event(window.borrow_mut().wait_events()) {}
        }
        OffScreenGlobals(ref window, ref browser) => {
            while browser.borrow_mut().handle_event(window.borrow_mut().wait_events()) {}
        }
    }
}

#[no_mangle]
pub extern "C" fn cef_do_message_loop_work() {
    send_window_event(IdleWindowEvent)
}

#[no_mangle]
pub extern "C" fn cef_quit_message_loop() {
}

#[no_mangle]
pub extern "C" fn cef_execute_process(_args: *const cef_main_args_t,
                                      _app: *mut cef_app_t,
                                      _windows_sandbox_info: *mut c_void)
                                      -> c_int {
   -1
}

pub fn send_window_event(event: WindowEvent) {
    message_queue.get().as_mut().unwrap().borrow_mut().push(event);

    let mut the_globals = globals.get();
    let the_globals = match the_globals.as_mut() {
        None => return,
        Some(the_globals) => the_globals,
    };
    loop {
        match **the_globals {
            OnScreenGlobals(_, ref browser) => {
                match browser.try_borrow_mut() {
                    None => {
                        // We're trying to send an event while processing another one. This will
                        // cause general badness, so queue up that event instead of immediately
                        // processing it.
                        break
                    }
                    Some(ref mut browser) => {
                        let event = match message_queue.get()
                                                       .as_mut()
                                                       .unwrap()
                                                       .borrow_mut()
                                                       .pop() {
                            None => return,
                            Some(event) => event,
                        };
                        browser.handle_event(event);
                    }
                }
            }
            OffScreenGlobals(_, ref browser) => {
                match browser.try_borrow_mut() {
                    None => {
                        // We're trying to send an event while processing another one. This will
                        // cause general badness, so queue up that event instead of immediately
                        // processing it.
                        break
                    }
                    Some(ref mut browser) => {
                        let event = match message_queue.get()
                                                       .as_mut()
                                                       .unwrap()
                                                       .borrow_mut()
                                                       .pop() {
                            None => return,
                            Some(event) => event,
                        };
                        browser.handle_event(event);
                    }
                }
            }
        }
    }
}

macro_rules! browser_method_delegate(
    ( $( fn $method:ident ( ) -> $return_type:ty ; )* ) => (
        $(
            pub fn $method() -> $return_type {
                let mut the_globals = globals.get();
                let the_globals = match the_globals.as_mut() {
                    None => panic!("{}: no globals created", stringify!($method)),
                    Some(the_globals) => the_globals,
                };
                match **the_globals {
                    OnScreenGlobals(_, ref browser) => browser.borrow_mut().$method(),
                    OffScreenGlobals(_, ref browser) => browser.borrow_mut().$method(),
                }
            }
        )*
    )
)

browser_method_delegate! {
    fn repaint_synchronously() -> ();
    fn pinch_zoom_level() -> f32;
    fn get_title_for_main_frame() -> ();
}

#[no_mangle]
pub extern "C" fn cef_api_hash(entry: c_int) -> *const c_char {
    if entry == 0 {
        &CEF_API_HASH_PLATFORM[0] as *const u8 as *const c_char
    } else {
        &CEF_API_HASH_UNIVERSAL[0] as *const u8 as *const c_char
    }
}

#[no_mangle]
pub extern "C" fn cef_log(_file: *const c_char,
                          _line: c_int,
                          _severity: c_int,
                          message: *const c_char) {
    unsafe {
        println!("{}", CString::new(message, false))
    }
}

#[no_mangle]
pub extern "C" fn cef_get_min_log_level() -> c_int {
    0
}

