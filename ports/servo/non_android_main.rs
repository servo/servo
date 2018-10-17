/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate backtrace;
extern crate euclid;
#[cfg(target_os = "windows")]
extern crate gdi32;
extern crate gleam;
extern crate glutin;
extern crate keyboard_types;
#[macro_use]
extern crate lazy_static;
#[cfg(any(target_os = "linux", target_os = "macos"))]
extern crate osmesa_sys;
extern crate servo;
#[cfg(feature = "unstable")]
#[macro_use]
extern crate sig;
#[cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows"
))]
extern crate tinyfiledialogs;
#[cfg(target_os = "windows")]
extern crate user32;
#[cfg(target_os = "windows")]
extern crate winapi;
extern crate winit;

// The window backed by glutin
mod glutin_app;

mod resources;

mod browser;

use backtrace::Backtrace;
use servo::{Servo, BrowserId};
use servo::compositing::windowing::WindowEvent;
use servo::config::opts::{self, ArgumentParsingResult, parse_url_or_filename};
use servo::config::servo_version;
use servo::servo_config::prefs::PREFS;
use servo::servo_url::ServoUrl;
use std::env;
use std::panic;
use std::process;
use std::thread;

pub mod platform {
    #[cfg(target_os = "macos")]
    pub use platform::macos::deinit;

    #[cfg(target_os = "macos")]
    pub mod macos;

    #[cfg(not(target_os = "macos"))]
    pub fn deinit() {}
}

#[cfg(not(feature = "unstable"))]
fn install_crash_handler() {}

#[cfg(feature = "unstable")]
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

pub fn main() {
    install_crash_handler();

    resources::init();

    // Parse the command line options and store them globally
    let args: Vec<String> = env::args().collect();
    let opts_result = opts::from_cmdline_args(&args);

    let content_process_token = if let ArgumentParsingResult::ContentProcess(token) = opts_result {
        Some(token)
    } else {
        if opts::get().is_running_problem_test && env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG", "compositing::constellation");
        }

        None
    };

    // TODO: once log-panics is released, can this be replaced by
    // log_panics::init()?
    panic::set_hook(Box::new(|info| {
        warn!("Panic hook called.");
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &**s,
                None => "Box<Any>",
            },
        };
        let current_thread = thread::current();
        let name = current_thread.name().unwrap_or("<unnamed>");
        if let Some(location) = info.location() {
            println!(
                "{} (thread {}, at {}:{})",
                msg,
                name,
                location.file(),
                location.line()
            );
        } else {
            println!("{} (thread {})", msg, name);
        }
        if env::var("RUST_BACKTRACE").is_ok() {
            println!("{:?}", Backtrace::new());
        }

        error!("{}", msg);
    }));

    if let Some(token) = content_process_token {
        return servo::run_content_process(token);
    }

    if opts::get().is_printing_version {
        println!("{}", servo_version());
        process::exit(0);
    }

    let window = glutin_app::create_window();

    let mut browser = browser::Browser::new(window.clone());

    // If the url is not provided, we fallback to the homepage in PREFS,
    // or a blank page in case the homepage is not set either.
    let cwd = env::current_dir().unwrap();
    let cmdline_url = opts::get().url.clone();
    let pref_url = PREFS
        .get("shell.homepage")
        .as_string()
        .and_then(|str| parse_url_or_filename(&cwd, str).ok());
    let blank_url = ServoUrl::parse("about:blank").ok();

    let target_url = cmdline_url.or(pref_url).or(blank_url).unwrap();

    let mut servo = Servo::new(window.clone());
    let browser_id = BrowserId::new();
    servo.handle_events(vec![WindowEvent::NewBrowser(target_url, browser_id)]);

    servo.setup_logging();

    window.run(|| {
        let win_events = window.get_events();

        // FIXME: this could be handled by Servo. We don't need
        // a repaint_synchronously function exposed.
        let need_resize = win_events.iter().any(|e| match *e {
            WindowEvent::Resize => true,
            _ => false,
        });

        browser.handle_window_events(win_events);

        let mut servo_events = servo.get_events();
        loop {
            browser.handle_servo_events(servo_events);
            servo.handle_events(browser.get_events());
            if browser.shutdown_requested() {
                return true;
            }
            servo_events = servo.get_events();
            if servo_events.is_empty() {
                break;
            }
        }

        if need_resize {
            servo.repaint_synchronously();
        }
        false
    });

    servo.deinit();

    platform::deinit()
}

// These functions aren't actually called. They are here as a link
// hack because Skia references them.

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glBindVertexArrayOES(_array: usize) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glDeleteVertexArraysOES(_n: isize, _arrays: *const ()) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glGenVertexArraysOES(_n: isize, _arrays: *const ()) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glRenderbufferStorageMultisampleIMG(
    _: isize,
    _: isize,
    _: isize,
    _: isize,
    _: isize,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glFramebufferTexture2DMultisampleIMG(
    _: isize,
    _: isize,
    _: isize,
    _: isize,
    _: isize,
    _: isize,
) {
    unimplemented!()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glDiscardFramebufferEXT(_: isize, _: isize, _: *const ()) {
    unimplemented!()
}
