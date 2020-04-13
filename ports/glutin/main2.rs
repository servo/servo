/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[cfg(any(target_os = "macos", target_os = "linux"))]
#[macro_use]
extern crate sig;

mod app;
mod backtrace;
mod browser;
mod context;
mod embedder;
mod events_loop;
mod headed_window;
mod headless_window;
mod keyutils;
mod resources;
mod window_trait;

use app::App;
use getopts::Options;
use servo::config::opts::{self, ArgumentParsingResult};
use servo::config::servo_version;
use servo::servo_config::pref;
use std::env;
use std::io::Write;
use std::panic;
use std::process;
use std::thread;

pub mod platform {
    #[cfg(target_os = "macos")]
    pub use crate::platform::macos::deinit;

    #[cfg(target_os = "macos")]
    pub mod macos;

    #[cfg(not(target_os = "macos"))]
    pub fn deinit(_clean_shutdown: bool) {}
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn install_crash_handler() {}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn install_crash_handler() {
    use libc::_exit;
    use sig::ffi::Sig;
    use std::thread;

    extern "C" fn handler(sig: i32) {
        use std::sync::atomic;
        static BEEN_HERE_BEFORE: atomic::AtomicBool = atomic::AtomicBool::new(false);
        if !BEEN_HERE_BEFORE.swap(true, atomic::Ordering::SeqCst) {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            let _ = write!(&mut stdout, "Stack trace");
            if let Some(name) = thread::current().name() {
                let _ = write!(&mut stdout, " for thread \"{}\"", name);
            }
            let _ = write!(&mut stdout, "\n");
            let _ = backtrace::print(&mut stdout);
        }
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
    let mut opts = Options::new();
    opts.optflag(
        "",
        "angle",
        "Use ANGLE to create a GL context (Windows-only)",
    );
    opts.optflag(
        "",
        "clean-shutdown",
        "Do not shutdown until all threads have finished (macos only)",
    );
    opts.optflag(
        "",
        "disable-vsync",
        "Disable vsync mode in the compositor to allow profiling at more than monitor refresh rate",
    );
    opts.optflag("", "msaa", "Use multisample antialiasing in WebRender.");
    opts.optflag("b", "no-native-titlebar", "Do not use native titlebar");
    opts.optopt("", "device-pixel-ratio", "Device pixels per px", "");
    opts.optopt(
        "u",
        "user-agent",
        "Set custom user agent string (or ios / android / desktop for platform default)",
        "NCSA Mosaic/1.0 (X11;SunOS 4.1.4 sun4m)",
    );

    let opts_matches;
    let content_process_token;
    match opts::from_cmdline_args(opts, &args) {
        ArgumentParsingResult::ContentProcess(matches, token) => {
            opts_matches = matches;
            content_process_token = Some(token);
            if opts::get().is_running_problem_test && env::var("RUST_LOG").is_err() {
                env::set_var("RUST_LOG", "compositing::constellation");
            }
        },
        ArgumentParsingResult::ChromeProcess(matches) => {
            opts_matches = matches;
            content_process_token = None;
        },
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
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        if let Some(location) = info.location() {
            let _ = writeln!(
                &mut stdout,
                "{} (thread {}, at {}:{})",
                msg,
                name,
                location.file(),
                location.line()
            );
        } else {
            let _ = writeln!(&mut stdout, "{} (thread {})", msg, name);
        }
        if env::var("RUST_BACKTRACE").is_ok() {
            let _ = backtrace::print(&mut stdout);
        }
        drop(stdout);

        error!("{}", msg);
    }));

    if let Some(token) = content_process_token {
        return servo::run_content_process(token);
    }

    if opts::get().is_printing_version {
        println!("{}", servo_version());
        process::exit(0);
    }

    let angle = opts_matches.opt_present("angle");
    let clean_shutdown = opts_matches.opt_present("clean-shutdown");
    let do_not_use_native_titlebar =
        opts_matches.opt_present("no-native-titlebar") || !(pref!(shell.native_titlebar.enabled));
    let enable_vsync = !opts_matches.opt_present("disable-vsync");
    let use_msaa = opts_matches.opt_present("msaa");
    let device_pixels_per_px = opts_matches.opt_str("device-pixel-ratio").map(|dppx_str| {
        dppx_str.parse().unwrap_or_else(|err| {
            error!("Error parsing option: --device-pixel-ratio ({})", err);
            process::exit(1);
        })
    });
    let user_agent = opts_matches.opt_str("u");

    App::run(
        angle,
        enable_vsync,
        use_msaa,
        do_not_use_native_titlebar,
        device_pixels_per_px,
        user_agent,
    );

    platform::deinit(clean_shutdown)
}
