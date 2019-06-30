/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[cfg(all(feature = "unstable", any(target_os = "macos", target_os = "linux")))]
#[macro_use]
extern crate sig;

mod app;
mod browser;
mod context;
mod embedder;
mod events_loop;
mod headed_window;
mod headless_window;
mod keyutils;
mod resources;
mod skia_symbols;
mod window_trait;

use app::App;
use backtrace::Backtrace;
use getopts::Options;
use servo::config::opts::{self, ArgumentParsingResult, args_fail, set_multiprocess, set_vsync};
use servo::config::servo_version;
use std::env;
use std::panic;
use std::process;
use std::sync::atomic::Ordering;
use std::thread;

pub mod platform {
    #[cfg(target_os = "macos")]
    pub use crate::platform::macos::deinit;

    #[cfg(target_os = "macos")]
    pub mod macos;

    #[cfg(not(target_os = "macos"))]
    pub fn deinit() {}
}

#[cfg(any(
    not(feature = "unstable"),
    not(any(target_os = "macos", target_os = "linux"))
))]
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

fn create_gluten_opts() -> Options {
    let mut opts = Options::new();
    opts.optflag("M", "multiprocess", "Run in multiprocess mode");
    opts.optflag("",
                 "disable-vsync",
                 "Disable vsync mode in the compositor to allow profiling at more than monitor refresh rate");
    //    opts.optflag("z", "headless", "Headless mode");

    opts
}

pub fn main() {
    install_crash_handler();

    resources::init();

    // Parse the command line options and store them globally
    let args: Vec<String> = env::args().collect();
    // Get Glutin only config options to pass to global.
    let opts = create_gluten_opts();
    // Get our Opts result to config remaining, Glutin specific, cli opts passed.
    let opt_result = opts::from_cmdline_args(opts ,&args);

    let opt_match = match opt_result {
        Ok(o) => o,
        Err(f) => args_fail(&f.to_string()),
    };

    if let Some(token) = opt_match.opt_str("content-process") {
        return servo::run_content_process(token);
    } else {
        if opts::get().is_running_problem_test && env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG", "compositing::constellation");
        }

        ()
    };

    if opt_match.opt_present("M") {
        set_multiprocess(true, Ordering::SeqCst);
    }

    if opt_match.opt_present("disable-vsync") {
        set_vsync(false, Ordering::SeqCst);
    }

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

//    if let Some(token) = content_process_token {
//        return servo::run_content_process(token);
//    }

    if opts::get().is_printing_version {
        println!("{}", servo_version());
        process::exit(0);
    }

    App::run();

    platform::deinit()
}
