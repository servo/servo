/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `servo` test application.
//!
//! Creates a `Browser` instance with a simple implementation of
//! the compositor's `WindowMethods` to create a working web browser.
//!
//! This browser's implementation of `WindowMethods` is built on top
//! of [glutin], the cross-platform OpenGL utility and windowing
//! library.
//!
//! For the engine itself look next door in lib.rs.
//!
//! [glutin]: https://github.com/tomaka/glutin

#![feature(start)]

// The Servo engine
extern crate servo;

// The window backed by glutin
extern crate glutin_app as app;
extern crate time;
extern crate env_logger;

#[cfg(target_os = "android")]
#[macro_use]
extern crate android_glue;

use servo::Browser;
use servo::compositing::windowing::WindowEvent;
use servo::net_traits::hosts;
use servo::util::opts;
use std::rc::Rc;

#[cfg(target_os = "android")]
use std::borrow::ToOwned;

fn main() {
    env_logger::init().unwrap();

    // Parse the command line options and store them globally
    opts::from_cmdline_args(&*args());

    setup_logging();

    // Possibly interpret the `HOST_FILE` environment variable
    hosts::global_init();

    let window = if opts::get().headless {
        None
    } else {
        Some(app::create_window(std::ptr::null_mut()))
    };

    // Our wrapper around `Browser` that also implements some
    // callbacks required by the glutin window implementation.
    let mut browser = BrowserWrapper {
        browser: Browser::new(window.clone()),
    };

    maybe_register_glutin_resize_handler(&window, &mut browser);

    browser.browser.handle_events(vec![WindowEvent::InitializeCompositing]);

    // Feed events from the window to the browser until the browser
    // says to stop.
    loop {
        let should_continue = match window {
            None => browser.browser.handle_events(Vec::new()),
            Some(ref window) => browser.browser.handle_events(window.wait_events()),
        };
        if !should_continue {
            break
        }
    };

    maybe_unregister_glutin_resize_handler(&window);

    let BrowserWrapper {
        browser
    } = browser;
    browser.shutdown();
}

fn maybe_register_glutin_resize_handler(window: &Option<Rc<app::window::Window>>,
                                        browser: &mut BrowserWrapper) {
    match *window {
        None => {}
        Some(ref window) => {
            unsafe {
                window.set_nested_event_loop_listener(browser);
            }
        }
    }
}

fn maybe_unregister_glutin_resize_handler(window: &Option<Rc<app::window::Window>>) {
    match *window {
        None => {}
        Some(ref window) => {
            unsafe {
                window.remove_nested_event_loop_listener();
            }
        }
    }
}

struct BrowserWrapper {
    browser: Browser,
}

impl app::NestedEventLoopListener for BrowserWrapper {
    fn handle_event_from_nested_event_loop(&mut self, event: WindowEvent) -> bool {
        let is_resize = match event {
            WindowEvent::Resize(..) => true,
            _ => false,
        };
        if !self.browser.handle_events(vec![event]) {
            return false
        }
        if is_resize {
            self.browser.repaint_synchronously()
        }
        true
    }
}

#[cfg(target_os = "android")]
fn setup_logging() {
    android::setup_logging();
}

#[cfg(not(target_os = "android"))]
fn setup_logging() {
}

#[cfg(target_os = "android")]
fn args() -> Vec<String> {
    vec![
        "servo".to_owned(),
        "http://en.wikipedia.org/wiki/Rust".to_owned()
    ]
}

#[cfg(not(target_os = "android"))]
fn args() -> Vec<String> {
    use std::env;
    env::args().collect()
}

// This macro must be used at toplevel because it defines a nested
// module, but macros can only accept identifiers - not paths -
// preventing the expansion of this macro within the android module
// without use of an additionl stub method or other hackery.
#[cfg(target_os = "android")]
android_start!(main);

#[cfg(target_os = "android")]
mod android {
    extern crate libc;
    extern crate android_glue;

    use self::libc::c_int;
    use std::borrow::ToOwned;

    pub fn setup_logging() {
        use self::libc::consts::os::posix88::{STDERR_FILENO, STDOUT_FILENO};
        //use std::env;

        //env::set_var("RUST_LOG", "servo,gfx,msg,util,layers,js,std,rt,extra");
        redirect_output(STDERR_FILENO);
        redirect_output(STDOUT_FILENO);
    }

    struct FilePtr(*mut self::libc::types::common::c95::FILE);

    unsafe impl Send for FilePtr {}

    fn redirect_output(file_no: c_int) {
        use self::libc::funcs::c95::stdio::fgets;
        use self::libc::funcs::posix88::stdio::fdopen;
        use self::libc::funcs::posix88::unistd::{pipe, dup2};
        use servo::util::task::spawn_named;
        use std::ffi::CStr;
        use std::ffi::CString;
        use std::str::from_utf8;

        unsafe {
            let mut pipes: [c_int; 2] = [ 0, 0 ];
            pipe(pipes.as_mut_ptr());
            dup2(pipes[1], file_no);
            let mode = CString::new("r").unwrap();
            let input_file = FilePtr(fdopen(pipes[0], mode.as_ptr()));
            spawn_named("android-logger".to_owned(), move || {
                static READ_SIZE: usize = 1024;
                let mut read_buffer = vec![0; READ_SIZE];
                let FilePtr(input_file) = input_file;
                loop {
                    fgets(read_buffer.as_mut_ptr(), (read_buffer.len() as i32)-1, input_file);
                    let c_str = CStr::from_ptr(read_buffer.as_ptr());
                    let slice = from_utf8(c_str.to_bytes()).unwrap();
                    android_glue::write_log(slice);
                }
            });
        }
    }

}
