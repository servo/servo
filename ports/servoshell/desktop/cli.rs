/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{env, panic, process};

use getopts::Options;
use log::error;
use servo::config::opts::{self, ArgumentParsingResult};
use servo::servo_config::pref;

use crate::desktop::app::App;
use crate::panic_hook;

pub fn main() {
    crate::crash_handler::install();

    crate::resources::init();

    // Parse the command line options and store them globally
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
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
        },
        ArgumentParsingResult::ChromeProcess(matches) => {
            opts_matches = matches;
            content_process_token = None;
        },
    };

    crate::prefs::register_user_prefs(&opts_matches);

    // TODO: once log-panics is released, can this be replaced by
    // log_panics::init()?
    panic::set_hook(Box::new(panic_hook::panic_hook));

    if let Some(token) = content_process_token {
        return servo::run_content_process(token);
    }

    if opts::get().is_printing_version {
        println!("{}", crate::servo_version());
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
