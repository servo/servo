/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Configuration options for a single run of the servo application. Created
//! from command line arguments.

use azure::azure_hl::{BackendType, CairoBackend, CoreGraphicsBackend};
use azure::azure_hl::{CoreGraphicsAcceleratedBackend, Direct2DBackend, SkiaBackend};
use extra::getopts::groups;

/// Global flags for Servo, currently set on the command line.
#[deriving(Clone)]
pub struct Opts {
    /// The initial URLs to load.
    urls: ~[~str],

    /// The rendering backend to use (`-r`).
    render_backend: BackendType,

    /// How many threads to use for CPU rendering (`-t`).
    ///
    /// FIXME(pcwalton): This is not currently used. All rendering is sequential.
    n_render_threads: uint,

    /// True to use CPU painting, false to use GPU painting via Skia-GL (`-c`). Note that
    /// compositing is always done on the GPU.
    cpu_painting: bool,

    /// The maximum size of each tile in pixels (`-s`).
    tile_size: uint,

    /// `None` to disable the profiler or `Some` with an interval in seconds to enable it and cause
    /// it to produce output on that interval (`-p`).
    profiler_period: Option<f64>,

    /// True to exit after the page load (`-x`).
    exit_after_load: bool,

    output_file: Option<~str>,
    headless: bool,
    hard_fail: bool,
}

fn print_usage(app: &str, opts: &[groups::OptGroup]) {
    let message = format!("Usage: {} [ options ... ] [URL]\n\twhere options include", app);
    println(groups::usage(message, opts));
}

pub fn from_cmdline_args(args: &[~str]) -> Opts {
    let app_name = args[0].to_str();
    let args = args.tail();

    let opts = ~[
        groups::optflag("c", "cpu", "CPU rendering"),
        groups::optopt("o", "output", "Output file", "output.png"),
        groups::optopt("r", "rendering", "Rendering backend", "direct2d|core-graphics|core-graphics-accelerated|cairo|skia."),
        groups::optopt("s", "size", "Size of tiles", "512"),
        groups::optopt("t", "threads", "Number of render threads", "1"),
        groups::optflagopt("p", "profile", "Profiler flag and output interval", "10"),
        groups::optflag("x", "exit", "Exit after load flag"),
        groups::optflag("z", "headless", "Headless mode"),
        groups::optflag("f", "hard-fail", "Exit on task failure instead of displaying about:failure"),
        groups::optflag("h", "help", "Print this message")
    ];

    let opt_match = match groups::getopts(args, opts) {
        Ok(m) => m,
        Err(f) => fail!(f.to_err_msg()),
    };

    if opt_match.opt_present("h") || opt_match.opt_present("help") {
        print_usage(app_name, opts);
        // TODO: how to return a null struct and let the caller know that
        // it should abort?
        fail!("")
    };

    let urls = if opt_match.free.is_empty() {
        print_usage(app_name, opts);
        fail!(~"servo asks that you provide 1 or more URLs")
    } else {
        opt_match.free.clone()
    };

    let render_backend = match opt_match.opt_str("r") {
        Some(backend_str) => {
            if backend_str == ~"direct2d" {
                Direct2DBackend
            } else if backend_str == ~"core-graphics" {
                CoreGraphicsBackend
            } else if backend_str == ~"core-graphics-accelerated" {
                CoreGraphicsAcceleratedBackend
            } else if backend_str == ~"cairo" {
                CairoBackend
            } else if backend_str == ~"skia" {
                SkiaBackend
            } else {
                fail!(~"unknown backend type")
            }
        }
        None => SkiaBackend
    };

    let tile_size: uint = match opt_match.opt_str("s") {
        Some(tile_size_str) => from_str(tile_size_str).unwrap(),
        None => 512,
    };

    let n_render_threads: uint = match opt_match.opt_str("t") {
        Some(n_render_threads_str) => from_str(n_render_threads_str).unwrap(),
        None => 1,      // FIXME: Number of cores.
    };

    // if only flag is present, default to 5 second period
    let profiler_period = opt_match.opt_default("p", "5").map(|period| {
        from_str(period).unwrap()
    });

    let cpu_painting = opt_match.opt_present("c");

    Opts {
        urls: urls,
        render_backend: render_backend,
        n_render_threads: n_render_threads,
        cpu_painting: cpu_painting,
        tile_size: tile_size,
        profiler_period: profiler_period,
        exit_after_load: opt_match.opt_present("x"),
        output_file: opt_match.opt_str("o"),
        headless: opt_match.opt_present("z"),
        hard_fail: opt_match.opt_present("f"),
    }
}
