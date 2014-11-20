/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use command_line::command_line_init;
use geom::size::TypedSize2D;
use glfw_app;
use libc::funcs::c95::string::strlen;
use libc::{c_int, c_void};
use native;
use servo::Browser;
use servo_util::opts;
use servo_util::opts::OpenGL;
use std::slice;
use switches::{KPROCESSTYPE, KWAITFORDEBUGGER};
use types::{cef_app_t, cef_main_args_t, cef_settings_t};

#[no_mangle]
pub extern "C" fn cef_initialize(args: *const cef_main_args_t,
                                 _settings: *mut cef_settings_t,
                                 application: *mut cef_app_t,
                                 _windows_sandbox_info: *const c_void)
                                 -> c_int {
    if args.is_null() {
        return 0;
    }
    unsafe {
        command_line_init((*args).argc, (*args).argv);
        (*application).get_browser_process_handler.map(|cb| {
                let handler = cb(application);
                if handler.is_not_null() {
                    (*handler).on_context_initialized.map(|hcb| hcb(handler));
                }
        });
    }
    return 1
}

#[no_mangle]
pub extern "C" fn cef_shutdown() {
}

#[no_mangle]
pub extern "C" fn cef_run_message_loop() {
    let mut urls = Vec::new();
    urls.push("http://s27.postimg.org/vqbtrolyr/servo.jpg".to_string());
    opts::set_opts(opts::Opts {
        urls: urls,
        n_render_threads: 1,
        gpu_painting: false,
        tile_size: 512,
        device_pixels_per_px: None,
        time_profiler_period: None,
        memory_profiler_period: None,
        enable_experimental: false,
        layout_threads: 1,
        nonincremental_layout: false,
        //layout_threads: cmp::max(rt::default_sched_threads() * 3 / 4, 1),
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
    native::start(0, 0 as *const *const u8, proc() {
        let window = glfw_app::create_window();
        let mut browser = Browser::new(Some(window.clone()));
        while browser.handle_event(window.wait_events()) {}
        browser.shutdown()
    });
}

#[no_mangle]
pub extern "C" fn cef_quit_message_loop() {
}

#[no_mangle]
pub extern "C" fn cef_execute_process(args: *const cef_main_args_t,
                                      _app: *mut cef_app_t,
                                      _windows_sandbox_info: *mut c_void)
                                      -> c_int {
    unsafe {
        if args.is_null() {
            println!("args must be passed");
            return -1;
        }
        for i in range(0u, (*args).argc as uint) {
             let u = (*args).argv.offset(i as int) as *const u8;
             slice::raw::buf_as_slice(u, strlen(u as *const i8) as uint, |s| {
                 if s.starts_with("--".as_bytes()) {
                     if s.slice_from(2) == KWAITFORDEBUGGER.as_bytes() {
                         //FIXME: this is NOT functionally equivalent to chromium!

                         //this should be a pause() call with an installed signal
                         //handler callback, something which is impossible now in rust
                     } else if s.slice_from(2) == KPROCESSTYPE.as_bytes() {
                         //TODO: run other process now
                     }
                 }
             });
        }
    }
   //process type not specified, must be browser process (NOOP)
   -1
}
