/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Configuration options for a single run of the servo application. Created
//! from command line arguments.

use azure::azure_hl::{BackendType, CairoBackend, CoreGraphicsBackend};
use azure::azure_hl::{CoreGraphicsAcceleratedBackend, Direct2DBackend, SkiaBackend};
use extra::getopts;

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
}

pub fn from_cmdline_args(args: &[~str]) -> Opts {
    let args = args.tail();

    let opts = ~[
        getopts::optflag("c"),      // CPU rendering
        getopts::optopt("o"),       // output file
        getopts::optopt("r"),       // rendering backend
        getopts::optopt("s"),       // size of tiles
        getopts::optopt("t"),       // threads to render with
        getopts::optflagopt("p"),   // profiler flag and output interval
        getopts::optflag("x"),      // exit after load flag
        getopts::optflag("z"),      // headless mode
    ];

    let opt_match = match getopts::getopts(args, opts) {
        Ok(m) => m,
        Err(f) => fail!(f.to_err_msg()),
    };

    let urls = if opt_match.free.is_empty() {
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
    let profiler_period = do opt_match.opt_default("p", "5").map |period| {
        from_str(period).unwrap()
    };

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
    }
}
