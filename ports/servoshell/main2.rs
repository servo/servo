/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io::Write;
use std::{env, panic, process, thread};

use getopts::Options;
use log::{error, warn};
use servo::config::opts::{self, ArgumentParsingResult};
use servo::servo_config::pref;

use crate::app::App;

pub fn main() {
    crate::crash_handler::install();

    crate::resources::init();

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
    opts.optflag("b", "no-native-titlebar", "Do not use native titlebar");
    opts.optopt("", "device-pixel-ratio", "Device pixels per px", "");
    opts.optopt(
        "u",
        "user-agent",
        "Set custom user agent string (or ios / android / desktop for platform default)",
        "NCSA Mosaic/1.0 (X11;SunOS 4.1.4 sun4m)",
    );
    opts.optmulti(
        "",
        "pref",
        "A preference to set to enable",
        "dom.bluetooth.enabled",
    );
    opts.optmulti(
        "",
        "pref",
        "A preference to set to disable",
        "dom.webgpu.enabled=false",
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

    crate::prefs::register_user_prefs(&opts_matches);

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
        let stderr = std::io::stderr();
        let mut stderr = stderr.lock();
        if let Some(location) = info.location() {
            let _ = writeln!(
                &mut stderr,
                "{} (thread {}, at {}:{})",
                msg,
                name,
                location.file(),
                location.line()
            );
        } else {
            let _ = writeln!(&mut stderr, "{} (thread {})", msg, name);
        }
        if env::var("RUST_BACKTRACE").is_ok() {
            let _ = crate::backtrace::print(&mut stderr);
        }
        drop(stderr);

        if opts::get().hard_fail && !opts::get().multiprocess {
            std::process::exit(1);
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

    let clean_shutdown = opts_matches.opt_present("clean-shutdown");
    let do_not_use_native_titlebar =
        opts_matches.opt_present("no-native-titlebar") || !(pref!(shell.native_titlebar.enabled));
    let device_pixel_ratio_override = opts_matches.opt_str("device-pixel-ratio").map(|dppx_str| {
        dppx_str.parse().unwrap_or_else(|err| {
            error!("Error parsing option: --device-pixel-ratio ({})", err);
            process::exit(1);
        })
    });

    let user_agent = opts_matches.opt_str("u");

    let url_opt = if !opts_matches.free.is_empty() {
        Some(&opts_matches.free[0][..])
    } else {
        None
    };

    App::run(
        do_not_use_native_titlebar,
        device_pixel_ratio_override,
        user_agent,
        url_opt.map(|s| s.to_string()),
    );

    crate::platform::deinit(clean_shutdown)
}

pub fn servo_version() -> String {
    format!(
        "Servo {}-{}",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_SHA")
    )
}
