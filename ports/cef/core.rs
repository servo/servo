/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use command_line::command_line_init;
use interfaces::cef_app_t;
use types::{cef_main_args_t, cef_settings_t};
use window::init_window;

use browser;
use libc::{c_char, c_int, c_void};
use servo_config::opts;
use std::ffi;
use std::str;

//static HOME_URL: &'static str = "http://s27.postimg.org/vqbtrolyr/servo.jpg";

static CEF_API_HASH_UNIVERSAL: &'static [u8] = b"8efd129f4afc344bd04b2feb7f73a149b6c4e27f\0";
#[cfg(target_os="windows")]
static CEF_API_HASH_PLATFORM: &'static [u8] = b"5c7f3e50ff5265985d11dc1a466513e25748bedd\0";
#[cfg(target_os="macos")]
static CEF_API_HASH_PLATFORM: &'static [u8] = b"6813214accbf2ebfb6bdcf8d00654650b251bf3d\0";
#[cfg(target_os="linux")]
static CEF_API_HASH_PLATFORM: &'static [u8] = b"2bc564c3871965ef3a2531b528bda3e17fa17a6d\0";

pub static mut CEF_APP: *mut cef_app_t = 0 as *mut cef_app_t;


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
        if !CEF_APP.is_null() {
            panic!("Attempting to call cef_initialize() multiple times!");
        }
    }

    unsafe {
        command_line_init((*args).argc, (*args).argv);

        if !application.is_null() {
            (*application).get_browser_process_handler.map(|cb| {
                    let handler = cb(application);
                    if !handler.is_null() {
                        (*handler).on_context_initialized.map(|hcb| hcb(handler));
                    }
            });
            CEF_APP = application;
        }
    }

    let mut temp_opts = opts::default_opts();
    temp_opts.headless = false;
    temp_opts.hard_fail = false;
    temp_opts.enable_text_antialiasing = true;
    temp_opts.enable_canvas_antialiasing = true;
    temp_opts.url = None;
    opts::set_defaults(temp_opts);

    if unsafe { (*settings).windowless_rendering_enabled != 0 } {
        init_window();
    }

    return 1
}

#[no_mangle]
pub extern "C" fn cef_shutdown() {
}

#[no_mangle]
pub extern "C" fn cef_run_message_loop() {
    // GWTODO: Support blocking message loop
    // again. Although, will it ever actually
    // be used or will everything use the
    // cef_do_message_loop_work function below
    // as our current miniservo apps do?
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn cef_do_message_loop_work() {
    browser::update();
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
        let slice = ffi::CStr::from_ptr(message);
        println!("{}", str::from_utf8(slice.to_bytes()).unwrap())
    }
}

#[no_mangle]
pub extern "C" fn cef_get_min_log_level() -> c_int {
    0
}
