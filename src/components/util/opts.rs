/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Configuration options for a single run of the servo application. Created
//! from command line arguments.

use geometry::{DevicePixel, ScreenPx};

use azure::azure_hl::{BackendType, CairoBackend, CoreGraphicsBackend};
use azure::azure_hl::{CoreGraphicsAcceleratedBackend, Direct2DBackend, SkiaBackend};
use geom::scale_factor::ScaleFactor;
use getopts;
use std::cmp;
use std::io;
use std::os;
use std::rt;

/// Global flags for Servo, currently set on the command line.
#[deriving(Clone)]
pub struct Opts {
    /// The initial URLs to load.
    pub urls: Vec<String>,

    /// The rendering backend to use (`-r`).
    pub render_backend: BackendType,

    /// How many threads to use for CPU rendering (`-t`).
    ///
    /// FIXME(pcwalton): This is not currently used. All rendering is sequential.
    pub n_render_threads: uint,

    /// True to use CPU painting, false to use GPU painting via Skia-GL (`-c`). Note that
    /// compositing is always done on the GPU.
    pub cpu_painting: bool,

    /// The maximum size of each tile in pixels (`-s`).
    pub tile_size: uint,

    /// The ratio of device pixels per px at the default scale. If unspecified, will use the
    /// platform default setting.
    pub device_pixels_per_px: Option<ScaleFactor<ScreenPx, DevicePixel, f32>>,

    /// `None` to disable the profiler or `Some` with an interval in seconds to enable it and cause
    /// it to produce output on that interval (`-p`).
    pub profiler_period: Option<f64>,

    /// The number of threads to use for layout (`-y`). Defaults to 1, which results in a recursive
    /// sequential algorithm.
    pub layout_threads: uint,

    /// True to exit after the page load (`-x`).
    pub exit_after_load: bool,

    pub output_file: Option<String>,
    pub headless: bool,
    pub hard_fail: bool,

    /// True if we should bubble intrinsic widths sequentially (`-b`). If this is true, then
    /// intrinsic widths are computed as a separate pass instead of during flow construction. You
    /// may wish to turn this flag on in order to benchmark style recalculation against other
    /// browser engines.
    pub bubble_widths_separately: bool,
}

fn print_usage(app: &str, opts: &[getopts::OptGroup]) {
    let message = format!("Usage: {} [ options ... ] [URL]\n\twhere options include", app);
    println!("{}", getopts::usage(message.as_slice(), opts));
}

fn args_fail(msg: &str) {
    io::stderr().write_line(msg).unwrap();
    os::set_exit_status(1);
}

pub fn from_cmdline_args(args: &[String]) -> Option<Opts> {
    let app_name = args[0].to_str();
    let args = args.tail();

    let opts = vec!(
        getopts::optflag("c", "cpu", "CPU rendering"),
        getopts::optopt("o", "output", "Output file", "output.png"),
        getopts::optopt("r", "rendering", "Rendering backend", "direct2d|core-graphics|core-graphics-accelerated|cairo|skia."),
        getopts::optopt("s", "size", "Size of tiles", "512"),
        getopts::optopt("", "device-pixel-ratio", "Device pixels per px", ""),
        getopts::optopt("t", "threads", "Number of render threads", "1"),
        getopts::optflagopt("p", "profile", "Profiler flag and output interval", "10"),
        getopts::optflag("x", "exit", "Exit after load flag"),
        getopts::optopt("y", "layout-threads", "Number of threads to use for layout", "1"),
        getopts::optflag("z", "headless", "Headless mode"),
        getopts::optflag("f", "hard-fail", "Exit on task failure instead of displaying about:failure"),
        getopts::optflag("b", "bubble-widths", "Bubble intrinsic widths separately like other engines"),
        getopts::optflag("h", "help", "Print this message")
    );

    let opt_match = match getopts::getopts(args, opts.as_slice()) {
        Ok(m) => m,
        Err(f) => {
            args_fail(f.to_err_msg().as_slice());
            return None;
        }
    };

    if opt_match.opt_present("h") || opt_match.opt_present("help") {
        print_usage(app_name.as_slice(), opts.as_slice());
        return None;
    };

    let urls = if opt_match.free.is_empty() {
        print_usage(app_name.as_slice(), opts.as_slice());
        args_fail("servo asks that you provide 1 or more URLs");
        return None;
    } else {
        opt_match.free.clone()
    };

    let render_backend = match opt_match.opt_str("r") {
        Some(backend_str) => {
            if "direct2d" == backend_str.as_slice() {
                Direct2DBackend
            } else if "core-graphics" == backend_str.as_slice() {
                CoreGraphicsBackend
            } else if "core-graphics-accelerated" == backend_str.as_slice() {
                CoreGraphicsAcceleratedBackend
            } else if "cairo" == backend_str.as_slice() {
                CairoBackend
            } else if "skia" == backend_str.as_slice() {
                SkiaBackend
            } else {
                fail!("unknown backend type")
            }
        }
        None => SkiaBackend
    };

    let tile_size: uint = match opt_match.opt_str("s") {
        Some(tile_size_str) => from_str(tile_size_str.as_slice()).unwrap(),
        None => 512,
    };

    let device_pixels_per_px = opt_match.opt_str("device-pixel-ratio").map(|dppx_str|
        ScaleFactor(from_str(dppx_str.as_slice()).unwrap())
    );

    let n_render_threads: uint = match opt_match.opt_str("t") {
        Some(n_render_threads_str) => from_str(n_render_threads_str.as_slice()).unwrap(),
        None => 1,      // FIXME: Number of cores.
    };

    // if only flag is present, default to 5 second period
    let profiler_period = opt_match.opt_default("p", "5").map(|period| {
        from_str(period.as_slice()).unwrap()
    });

    let cpu_painting = opt_match.opt_present("c");

    let layout_threads: uint = match opt_match.opt_str("y") {
        Some(layout_threads_str) => from_str(layout_threads_str.as_slice()).unwrap(),
        None => cmp::max(rt::default_sched_threads() * 3 / 4, 1),
    };

    Some(Opts {
        urls: urls,
        render_backend: render_backend,
        n_render_threads: n_render_threads,
        cpu_painting: cpu_painting,
        tile_size: tile_size,
        device_pixels_per_px: device_pixels_per_px,
        profiler_period: profiler_period,
        layout_threads: layout_threads,
        exit_after_load: opt_match.opt_present("x"),
        output_file: opt_match.opt_str("o"),
        headless: opt_match.opt_present("z"),
        hard_fail: opt_match.opt_present("f"),
        bubble_widths_separately: opt_match.opt_present("b"),
    })
}
