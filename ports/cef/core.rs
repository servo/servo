/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use browser::{GLOBAL_BROWSERS, browser_callback_after_created};
use command_line::command_line_init;
use glfw_app;
use libc::funcs::c95::string::strlen;
use libc::{c_int, c_void};
use native;
use servo::Browser;
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
    native::start(0, 0 as *const *const u8, proc() {
        GLOBAL_BROWSERS.get().map(|refcellbrowsers| {
            unsafe {
                let browsers = refcellbrowsers.borrow();
                let mut num = browsers.len();
                for active_browser in browsers.iter() {
                    (**active_browser).window = glfw_app::create_window();
                    (**active_browser).servo_browser = Some(Browser::new(Some((**active_browser).window.clone())));
                    if !(**active_browser).callback_executed { browser_callback_after_created(*active_browser); }
                }
                while num > 0 {
                    for active_browser in browsers.iter().filter(|&active_browser| (**active_browser).servo_browser.is_some()) {
                        let ref mut browser = **active_browser;
                        let mut servobrowser = browser.servo_browser.take().unwrap();
                        if !servobrowser.handle_event(browser.window.wait_events()) {
                            servobrowser.shutdown();
                            num -= 1;
                        }
                    }
                }
            }
        });
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
