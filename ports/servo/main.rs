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
//! For the engine itself look next door in `components/servo/lib.rs`.
//!
//! [glutin]: https://github.com/tomaka/glutin

#![feature(start, core_intrinsics)]

#[cfg(target_os = "android")]
extern crate android_injected_glue;
extern crate backtrace;
// The window backed by glutin
extern crate glutin_app as app;
#[macro_use]
extern crate log;
// The Servo engine
extern crate servo;
#[cfg(not(target_os = "android"))]
#[macro_use]
extern crate sig;

use backtrace::Backtrace;
use servo::Browser;
use servo::compositing::windowing::WindowEvent;
#[cfg(target_os = "android")]
use servo::config;
use servo::config::opts::{self, ArgumentParsingResult};
use servo::config::servo_version;
use std::env;
use std::panic;
use std::process;
use std::rc::Rc;
use std::thread;

pub mod platform {
    #[cfg(target_os = "macos")]
    pub use platform::macos::deinit;

    #[cfg(target_os = "macos")]
    pub mod macos;

    #[cfg(not(target_os = "macos"))]
    pub fn deinit() {}
}

#[cfg(not(target_os = "android"))]
fn install_crash_handler() {
    use backtrace::Backtrace;
    use sig::ffi::Sig;
    use std::intrinsics::abort;
    use std::thread;

    fn handler(_sig: i32) {
        let name = thread::current()
            .name()
            .map(|n| format!(" for thread \"{}\"", n))
            .unwrap_or("".to_owned());
        println!("Stack trace{}\n{:?}", name, Backtrace::new());
        unsafe {
            // N.B. Using process::abort() here causes the crash handler to be
            //      triggered recursively.
            abort();
        }
    }

    signal!(Sig::SEGV, handler); // handle segfaults
    signal!(Sig::ILL, handler); // handle stack overflow and unsupported CPUs
    signal!(Sig::IOT, handler); // handle double panics
    signal!(Sig::BUS, handler); // handle invalid memory access
}

#[cfg(target_os = "android")]
fn install_crash_handler() {}

fn main() {
    install_crash_handler();

    // Parse the command line options and store them globally
    let opts_result = opts::from_cmdline_args(&*args());

    let content_process_token = if let ArgumentParsingResult::ContentProcess(token) = opts_result {
        Some(token)
    } else {
        if opts::get().is_running_problem_test && ::std::env::var("RUST_LOG").is_err() {
            ::std::env::set_var("RUST_LOG", "compositing::constellation");
        }

        None
    };

    // TODO: once log-panics is released, can this be replaced by
    // log_panics::init()?
    panic::set_hook(Box::new(|info| {
        warn!("Panic hook called.");
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => {
                match info.payload().downcast_ref::<String>() {
                    Some(s) => &**s,
                    None => "Box<Any>",
                }
            },
        };
        let current_thread = thread::current();
        let name = current_thread.name().unwrap_or("<unnamed>");
        if let Some(location) = info.location() {
            println!("{} (thread {}, at {}:{})",
                     msg,
                     name,
                     location.file(),
                     location.line());
        } else {
            println!("{} (thread {})", msg, name);
        }
        if env::var("RUST_BACKTRACE").is_ok() {
            println!("{:?}", Backtrace::new());
        }

        error!("{}", msg);
    }));

    setup_logging();

    if let Some(token) = content_process_token {
        return servo::run_content_process(token);
    }

    if opts::get().is_printing_version {
        println!("{}", servo_version());
        process::exit(0);
    }

    let window = app::create_window(None);

    // Our wrapper around `Browser` that also implements some
    // callbacks required by the glutin window implementation.
    let mut browser = BrowserWrapper {
        browser: Browser::new(window.clone()),
    };

    browser.browser.setup_logging();

    register_glutin_resize_handler(&window, &mut browser);

    browser.browser.handle_events(vec![WindowEvent::InitializeCompositing]);

    // Feed events from the window to the browser until the browser
    // says to stop.
    loop {
        let should_continue = browser.browser.handle_events(window.wait_events());
        if !should_continue {
            break;
        }
    }

    unregister_glutin_resize_handler(&window);

    platform::deinit()
}

fn register_glutin_resize_handler(window: &Rc<app::window::Window>, browser: &mut BrowserWrapper) {
    unsafe {
        window.set_nested_event_loop_listener(browser);
    }
}

fn unregister_glutin_resize_handler(window: &Rc<app::window::Window>) {
    unsafe {
        window.remove_nested_event_loop_listener();
    }
}

struct BrowserWrapper {
    browser: Browser<app::window::Window>,
}

impl app::NestedEventLoopListener for BrowserWrapper {
    fn handle_event_from_nested_event_loop(&mut self, event: WindowEvent) -> bool {
        let is_resize = match event {
            WindowEvent::Resize(..) => true,
            _ => false,
        };
        if !self.browser.handle_events(vec![event]) {
            return false;
        }
        if is_resize {
            self.browser.repaint_synchronously()
        }
        true
    }
}

#[cfg(target_os = "android")]
fn setup_logging() {
    // Piping logs from stdout/stderr to logcat happens in android_injected_glue.
    ::std::env::set_var("RUST_LOG", "error");

    unsafe { android_injected_glue::ffi::app_dummy() };
}

#[cfg(not(target_os = "android"))]
fn setup_logging() {}

#[cfg(target_os = "android")]
/// Attempt to read parameters from a file since they are not passed to us in Android environments.
/// The first line should be the "servo" argument and the last should be the URL to load.
/// Blank lines and those beginning with a '#' are ignored.
/// Each line should be a separate parameter as would be parsed by the shell.
/// For example, "servo -p 10 http://en.wikipedia.org/wiki/Rust" would take 4 lines.
fn args() -> Vec<String> {
    use std::error::Error;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let mut params_file = config::basedir::default_config_dir().unwrap();
    params_file.push("android_params");
    match File::open(params_file.to_str().unwrap()) {
        Ok(f) => {
            let mut vec = Vec::new();
            let file = BufReader::new(&f);
            for line in file.lines() {
                let l = line.unwrap().trim().to_owned();
                // ignore blank lines and those that start with a '#'
                match l.is_empty() || l.as_bytes()[0] == b'#' {
                    true => (),
                    false => vec.push(l),
                }
            }
            vec
        },
        Err(e) => {
            debug!("Failed to open params file '{}': {}",
                   params_file.to_str().unwrap(),
                   Error::description(&e));
            vec!["servo".to_owned(), "http://en.wikipedia.org/wiki/Rust".to_owned()]
        },
    }
}

#[cfg(not(target_os = "android"))]
fn args() -> Vec<String> {
    use std::env;
    env::args().collect()
}


#[cfg(target_os = "android")]
#[no_mangle]
#[inline(never)]
#[allow(non_snake_case)]
pub extern "C" fn android_main(app: *mut ()) {
    android_injected_glue::android_main2(app as *mut _, move |_, _| main());
}
