/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Configuration options for a single run of the servo application. Created
//! from command line arguments.

use azure::azure_hl::{BackendType, CairoBackend, CoreGraphicsBackend};
use azure::azure_hl::{CoreGraphicsAcceleratedBackend, Direct2DBackend, SkiaBackend};
use extra::getopts::groups;
use std::num;
use std::rt;
use std::io;
use std::os;

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

    /// The number of threads to use for layout (`-y`). Defaults to 1, which results in a recursive
    /// sequential algorithm.
    layout_threads: uint,

    /// True to exit after the page load (`-x`).
    exit_after_load: bool,

    output_file: Option<~str>,
    headless: bool,
    hard_fail: bool,

    /// True if we should bubble intrinsic widths sequentially (`-b`). If this is true, then
    /// intrinsic widths are computed as a separate pass instead of during flow construction. You
    /// may wish to turn this flag on in order to benchmark style recalculation against other
    /// browser engines.
    bubble_widths_separately: bool,
}

fn print_usage(app: &str, opts: &[groups::OptGroup]) {
    let message = format!("Usage: {} [ options ... ] [URL]\n\twhere options include", app);
    println(groups::usage(message, opts));
}

fn args_fail(msg: &str) {
    io::stderr().write_line(msg);
    os::set_exit_status(1);
}

pub fn from_cmdline_args(args: &[~str]) -> Option<Opts> {
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
        groups::optopt("y", "layout-threads", "Number of threads to use for layout", "1"),
        groups::optflag("z", "headless", "Headless mode"),
        groups::optflag("f", "hard-fail", "Exit on task failure instead of displaying about:failure"),
        groups::optflag("b", "bubble-widths", "Bubble intrinsic widths separately like other engines"),
        groups::optflag("h", "help", "Print this message")
    ];

    let opt_match = match groups::getopts(args, opts) {
        Ok(m) => m,
        Err(f) => {
            args_fail(f.to_err_msg());
            return None;
        }
    };

    if opt_match.opt_present("h") || opt_match.opt_present("help") {
        print_usage(app_name, opts);
        return None;
    };

    let urls = if opt_match.free.is_empty() {
        print_usage(app_name, opts);
        args_fail("servo asks that you provide 1 or more URLs");
        return None;
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

    let layout_threads: uint = match opt_match.opt_str("y") {
        Some(layout_threads_str) => from_str(layout_threads_str).unwrap(),
        None => num::max(rt::default_sched_threads() * 3 / 4, 1),
    };

    Some(Opts {
        urls: urls,
        render_backend: render_backend,
        n_render_threads: n_render_threads,
        cpu_painting: cpu_painting,
        tile_size: tile_size,
        profiler_period: profiler_period,
        layout_threads: layout_threads,
        exit_after_load: opt_match.opt_present("x"),
        output_file: opt_match.opt_str("o"),
        headless: opt_match.opt_present("z"),
        hard_fail: opt_match.opt_present("f"),
        bubble_widths_separately: opt_match.opt_present("b"),
    })
}
