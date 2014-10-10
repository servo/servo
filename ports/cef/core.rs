/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use azure;
use command_line::command_line_init;
use eutil::fptr_is_null;
use geom::size::TypedSize2D;
use glfw_app;
use libc::{c_int, c_void};
use native;
use servo;
use servo_util::opts;
use std::mem;
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
        let cb = (*application).get_browser_process_handler;
        if !fptr_is_null(mem::transmute(cb)) {
            let handler = cb(application);
            if handler.is_not_null() {
                let hcb = (*handler).on_context_initialized;
                if !fptr_is_null(mem::transmute(hcb)) {
                    hcb(handler);
                }
            }
        }
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
    let opts = opts::Opts {
        urls: urls,
        render_backend: azure::azure_hl::SkiaBackend,
        n_render_threads: 1,
        cpu_painting: false,
        tile_size: 512,
        device_pixels_per_px: None,
        time_profiler_period: None,
        memory_profiler_period: None,
        enable_experimental: false,
        layout_threads: 1,
        //layout_threads: cmp::max(rt::default_sched_threads() * 3 / 4, 1),
        exit_after_load: false,
        output_file: None,
        headless: false,
        hard_fail: false,
        bubble_inline_sizes_separately: false,
        show_debug_borders: false,
        enable_text_antialiasing: true,
        trace_layout: false,
        devtools_port: None,
        initial_window_size: TypedSize2D(800, 600),
        user_agent: None,
        dump_flow_tree: false,
    };
    native::start(0, 0 as *const *const u8, proc() {
       let window = Some(glfw_app::create_window(&opts));
       servo::run(opts, window);
    });
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
