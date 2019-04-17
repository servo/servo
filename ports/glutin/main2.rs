/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[cfg(all(feature = "unstable", any(target_os = "macos", target_os = "linux")))]
#[macro_use] extern crate sig;

mod browser;
mod keyutils;
mod window;
mod resources;
mod skia_symbols;

use backtrace::Backtrace;
use servo::{Servo, BrowserId};
use servo::compositing::windowing::WindowEvent;
use servo::config::opts::{self, ArgumentParsingResult, parse_url_or_filename};
use servo::config::servo_version;
use servo::servo_config::pref;
use servo::servo_url::ServoUrl;
use std::env;
use std::panic;
use std::process;
use std::thread;

pub mod platform {
    #[cfg(target_os = "macos")]
    pub use crate::platform::macos::deinit;

    #[cfg(target_os = "macos")]
    pub mod macos;

    #[cfg(not(target_os = "macos"))]
    pub fn deinit() {}
}

#[cfg(any(not(feature = "unstable"), not(any(target_os = "macos", target_os = "linux"))))]
fn install_crash_handler() {}

#[cfg(all(feature = "unstable", any(target_os = "macos", target_os = "linux")))]
fn install_crash_handler() {
    use backtrace::Backtrace;
    use libc::_exit;
    use sig::ffi::Sig;
    use std::thread;

    extern "C" fn handler(sig: i32) {
        let name = thread::current()
            .name()
            .map(|n| format!(" for thread \"{}\"", n))
            .unwrap_or("".to_owned());
        println!("Stack trace{}\n{:?}", name, Backtrace::new());
        unsafe {
            _exit(sig);
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

    let opts = opts::get();

    let content_process_token = if let ArgumentParsingResult::ContentProcess(token) = opts_result {
        Some(token)
    } else {
        if opts.is_running_problem_test && env::var("RUST_LOG").is_err() {
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

    if opts.is_printing_version {
        println!("{}", servo_version());
        process::exit(0);
    }

    let foreground = opts.output_file.is_none() && !opts.headless;
    let window = window::Window::new(foreground, opts.initial_window_size);

    let mut browser = browser::Browser::new(window.clone());

    // If the url is not provided, we fallback to the homepage in prefs,
    // or a blank page in case the homepage is not set either.
    let cwd = env::current_dir().unwrap();
    let cmdline_url = opts.url.clone();
    let pref_url = {
        let homepage_url = pref!(shell.homepage);
        parse_url_or_filename(&cwd, &homepage_url).ok()
    };
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
