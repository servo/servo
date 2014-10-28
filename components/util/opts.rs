/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Configuration options for a single run of the servo application. Created
//! from command line arguments.

use geometry::ScreenPx;

use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use layers::geometry::DevicePixel;
use getopts;
use std::cmp;
use std::io;
use std::mem;
use std::os;
use std::ptr;
use std::rt;

/// Global flags for Servo, currently set on the command line.
#[deriving(Clone)]
pub struct Opts {
    /// The initial URLs to load.
    pub urls: Vec<String>,

    /// How many threads to use for CPU rendering (`-t`).
    ///
    /// FIXME(pcwalton): This is not currently used. All rendering is sequential.
    pub n_render_threads: uint,

    /// True to use GPU painting via Skia-GL, false to use CPU painting via Skia (`-g`). Note that
    /// compositing is always done on the GPU.
    pub gpu_painting: bool,

    /// The maximum size of each tile in pixels (`-s`).
    pub tile_size: uint,

    /// The ratio of device pixels per px at the default scale. If unspecified, will use the
    /// platform default setting.
    pub device_pixels_per_px: Option<ScaleFactor<ScreenPx, DevicePixel, f32>>,

    /// `None` to disable the time profiler or `Some` with an interval in seconds to enable it and
    /// cause it to produce output on that interval (`-p`).
    pub time_profiler_period: Option<f64>,

    /// `None` to disable the memory profiler or `Some` with an interval in seconds to enable it
    /// and cause it to produce output on that interval (`-m`).
    pub memory_profiler_period: Option<f64>,

    /// Enable experimental web features (`-e`).
    pub enable_experimental: bool,

    /// The number of threads to use for layout (`-y`). Defaults to 1, which results in a recursive
    /// sequential algorithm.
    pub layout_threads: uint,

    pub nonincremental_layout: bool,

    /// True to exit after the page load (`-x`).
    pub exit_after_load: bool,

    pub output_file: Option<String>,
    pub headless: bool,
    pub hard_fail: bool,

    /// True if we should bubble intrinsic widths sequentially (`-b`). If this is true, then
    /// intrinsic widths are computed as a separate pass instead of during flow construction. You
    /// may wish to turn this flag on in order to benchmark style recalculation against other
    /// browser engines.
    pub bubble_inline_sizes_separately: bool,

    /// True if we should show borders on all layers and tiles for
    /// debugging purposes (`--show-debug-borders`).
    pub show_debug_borders: bool,

    /// True if we should show borders on all fragments for debugging purposes (`--show-debug-fragment-borders`).
    pub show_debug_fragment_borders: bool,

    /// If set with --disable-text-aa, disable antialiasing on fonts. This is primarily useful for reftests
    /// where pixel perfect results are required when using fonts such as the Ahem
    /// font for layout tests.
    pub enable_text_antialiasing: bool,

    /// True if each step of layout is traced to an external JSON file
    /// for debugging purposes. Settings this implies sequential layout
    /// and render.
    pub trace_layout: bool,

    /// If true, instrument the runtime for each task created and dump
    /// that information to a JSON file that can be viewed in the task
    /// profile viewer.
    pub profile_tasks: bool,

    /// `None` to disable devtools or `Some` with a port number to start a server to listen to
    /// remote Firefox devtools connections.
    pub devtools_port: Option<u16>,

    /// The initial requested size of the window.
    pub initial_window_size: TypedSize2D<ScreenPx, uint>,

    /// An optional string allowing the user agent to be set for testing.
    pub user_agent: Option<String>,

    /// Dumps the flow tree after a layout.
    pub dump_flow_tree: bool,

    /// Whether to show an error when display list geometry escapes flow overflow regions.
    pub validate_display_list_geometry: bool,
}

fn print_usage(app: &str, opts: &[getopts::OptGroup]) {
    let message = format!("Usage: {} [ options ... ] [URL]\n\twhere options include", app);
    println!("{}", getopts::usage(message.as_slice(), opts));
}

fn args_fail(msg: &str) {
    io::stderr().write_line(msg).unwrap();
    os::set_exit_status(1);
}

// Always use CPU rendering on android.

#[cfg(target_os="android")]
static FORCE_CPU_PAINTING: bool = true;

#[cfg(not(target_os="android"))]
static FORCE_CPU_PAINTING: bool = false;

fn default_opts() -> Opts {
    Opts {
        urls: vec!(),
        n_render_threads: 1,
        gpu_painting: false,
        tile_size: 512,
        device_pixels_per_px: None,
        time_profiler_period: None,
        memory_profiler_period: None,
        enable_experimental: false,
        layout_threads: 1,
        nonincremental_layout: false,
        exit_after_load: false,
        output_file: None,
        headless: true,
        hard_fail: true,
        bubble_inline_sizes_separately: false,
        show_debug_borders: false,
        show_debug_fragment_borders: false,
        enable_text_antialiasing: false,
        trace_layout: false,
        devtools_port: None,
        initial_window_size: TypedSize2D(800, 600),
        user_agent: None,
        dump_flow_tree: false,
        validate_display_list_geometry: false,
        profile_tasks: false,
    }
}

pub fn from_cmdline_args(args: &[String]) -> bool {
    let app_name = args[0].to_string();
    let args = args.tail();

    let opts = vec!(
        getopts::optflag("c", "cpu", "CPU painting (default)"),
        getopts::optflag("g", "gpu", "GPU painting"),
        getopts::optopt("o", "output", "Output file", "output.png"),
        getopts::optopt("s", "size", "Size of tiles", "512"),
        getopts::optopt("", "device-pixel-ratio", "Device pixels per px", ""),
        getopts::optflag("e", "experimental", "Enable experimental web features"),
        getopts::optopt("t", "threads", "Number of render threads", "1"),
        getopts::optflagopt("p", "profile", "Profiler flag and output interval", "10"),
        getopts::optflagopt("m", "memory-profile", "Memory profiler flag and output interval", "10"),
        getopts::optflag("x", "exit", "Exit after load flag"),
        getopts::optopt("y", "layout-threads", "Number of threads to use for layout", "1"),
        getopts::optflag("i", "nonincremental-layout", "Enable to turn off incremental layout."),
        getopts::optflag("z", "headless", "Headless mode"),
        getopts::optflag("f", "hard-fail", "Exit on task failure instead of displaying about:failure"),
        getopts::optflag("b", "bubble-widths", "Bubble intrinsic widths separately like other engines"),
        getopts::optflag("", "show-debug-borders", "Show debugging borders on layers and tiles."),
        getopts::optflag("", "show-debug-fragment-borders", "Show debugging borders on fragments."),
        getopts::optflag("", "profile-tasks", "Instrument each task, writing the output to a file."),
        getopts::optflag("", "disable-text-aa", "Disable antialiasing for text rendering."),
        getopts::optflag("", "trace-layout", "Write layout trace to external file for debugging."),
        getopts::optflagopt("", "devtools", "Start remote devtools server on port", "6000"),
        getopts::optopt("", "resolution", "Set window resolution.", "800x600"),
        getopts::optopt("u", "user-agent", "Set custom user agent string", "NCSA Mosaic/1.0 (X11;SunOS 4.1.4 sun4m)"),
        getopts::optflag("", "dump-flow-tree", "Dump the flow (render) tree during each layout."),
        getopts::optflag("", "validate-display-list-geometry", "Display an error when display list geometry escapes overflow region."),
        getopts::optflag("h", "help", "Print this message")
    );

    let opt_match = match getopts::getopts(args, opts.as_slice()) {
        Ok(m) => m,
        Err(f) => {
            args_fail(format!("{}", f).as_slice());
            return false;
        }
    };

    if opt_match.opt_present("h") || opt_match.opt_present("help") {
        print_usage(app_name.as_slice(), opts.as_slice());
        return false;
    };

    let urls = if opt_match.free.is_empty() {
        print_usage(app_name.as_slice(), opts.as_slice());
        args_fail("servo asks that you provide 1 or more URLs");
        return false;
    } else {
        opt_match.free.clone()
    };

    let tile_size: uint = match opt_match.opt_str("s") {
        Some(tile_size_str) => from_str(tile_size_str.as_slice()).unwrap(),
        None => 512,
    };

    let device_pixels_per_px = opt_match.opt_str("device-pixel-ratio").map(|dppx_str|
        ScaleFactor(from_str(dppx_str.as_slice()).unwrap())
    );

    let mut n_render_threads: uint = match opt_match.opt_str("t") {
        Some(n_render_threads_str) => from_str(n_render_threads_str.as_slice()).unwrap(),
        None => 1,      // FIXME: Number of cores.
    };

    // If only the flag is present, default to a 5 second period for both profilers.
    let time_profiler_period = opt_match.opt_default("p", "5").map(|period| {
        from_str(period.as_slice()).unwrap()
    });
    let memory_profiler_period = opt_match.opt_default("m", "5").map(|period| {
        from_str(period.as_slice()).unwrap()
    });

    let gpu_painting = !FORCE_CPU_PAINTING && opt_match.opt_present("g");

    let mut layout_threads: uint = match opt_match.opt_str("y") {
        Some(layout_threads_str) => from_str(layout_threads_str.as_slice()).unwrap(),
        None => cmp::max(rt::default_sched_threads() * 3 / 4, 1),
    };

    let nonincremental_layout = opt_match.opt_present("i");

    let mut bubble_inline_sizes_separately = opt_match.opt_present("b");

    let trace_layout = opt_match.opt_present("trace-layout");
    if trace_layout {
        n_render_threads = 1;
        layout_threads = 1;
        bubble_inline_sizes_separately = true;
    }

    let devtools_port = opt_match.opt_default("devtools", "6000").map(|port| {
        from_str(port.as_slice()).unwrap()
    });

    let initial_window_size = match opt_match.opt_str("resolution") {
        Some(res_string) => {
            let res: Vec<uint> = res_string.as_slice().split('x').map(|r| from_str(r).unwrap()).collect();
            TypedSize2D(res[0], res[1])
        }
        None => {
            TypedSize2D(800, 600)
        }
    };

    let opts = Opts {
        urls: urls,
        n_render_threads: n_render_threads,
        gpu_painting: gpu_painting,
        tile_size: tile_size,
        device_pixels_per_px: device_pixels_per_px,
        time_profiler_period: time_profiler_period,
        memory_profiler_period: memory_profiler_period,
        enable_experimental: opt_match.opt_present("e"),
        layout_threads: layout_threads,
        nonincremental_layout: nonincremental_layout,
        exit_after_load: opt_match.opt_present("x"),
        output_file: opt_match.opt_str("o"),
        headless: opt_match.opt_present("z"),
        hard_fail: opt_match.opt_present("f"),
        bubble_inline_sizes_separately: bubble_inline_sizes_separately,
        show_debug_borders: opt_match.opt_present("show-debug-borders"),
        show_debug_fragment_borders: opt_match.opt_present("show-debug-fragment-borders"),
        enable_text_antialiasing: !opt_match.opt_present("disable-text-aa"),
        profile_tasks: opt_match.opt_present("profile-tasks"),
        trace_layout: trace_layout,
        devtools_port: devtools_port,
        initial_window_size: initial_window_size,
        user_agent: opt_match.opt_str("u"),
        dump_flow_tree: opt_match.opt_present("dump-flow-tree"),
        validate_display_list_geometry: opt_match.opt_present("validate-display-list-geometry"),
    };

    set_opts(opts);
    true
}

static mut EXPERIMENTAL_ENABLED: bool = false;

pub fn set_experimental_enabled(new_value: bool) {
    unsafe {
        EXPERIMENTAL_ENABLED = new_value;
    }
}

pub fn experimental_enabled() -> bool {
    unsafe {
        EXPERIMENTAL_ENABLED
    }
}

// Make Opts available globally. This saves having to clone and pass
// opts everywhere it is used, which gets particularly cumbersome
// when passing through the DOM structures.
static mut OPTIONS: *mut Opts = 0 as *mut Opts;

pub fn set_opts(opts: Opts) {
    unsafe {
        let box_opts = box opts;
        OPTIONS = mem::transmute(box_opts);
    }
}

#[inline]
pub fn get<'a>() -> &'a Opts {
    unsafe {
        // If code attempts to retrieve the options and they haven't
        // been set by the platform init code, just return a default
        // set of options. This is mostly useful for unit tests that
        // run through a code path which queries the cmd line options.
        if OPTIONS == ptr::null_mut() {
            set_opts(default_opts());
        }
        mem::transmute(OPTIONS)
    }
}
