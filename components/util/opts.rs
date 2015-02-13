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
use std::collections::HashSet;
use std::cmp;
use std::env;
use std::old_io as io;
use std::mem;
use std::ptr;
use std::rt;

/// Global flags for Servo, currently set on the command line.
#[derive(Clone)]
pub struct Opts {
    /// The initial URLs to load.
    pub urls: Vec<String>,

    /// How many threads to use for CPU painting (`-t`).
    ///
    /// FIXME(pcwalton): This is not currently used. All painting is sequential.
    pub n_paint_threads: uint,

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
    /// and paint.
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

    /// A specific path to find required resources (such as user-agent.css).
    pub resources_path: Option<String>,
}

fn print_usage(app: &str, opts: &[getopts::OptGroup]) {
    let message = format!("Usage: {} [ options ... ] [URL]\n\twhere options include", app);
    println!("{}", getopts::usage(message.as_slice(), opts));
}

pub fn print_debug_usage(app: &str)  {
    fn print_option(name: &str, description: &str) {
        println!("\t{:<35} {}", name, description);
    }

    println!("Usage: {} debug option,[options,...]\n\twhere options include\n\nOptions:", app);

    print_option("bubble-widths", "Bubble intrinsic widths separately like other engines.");
    print_option("disable-text-aa", "Disable antialiasing of rendered text.");
    print_option("dump-flow-tree", "Print the flow tree after each layout.");
    print_option("profile-tasks", "Instrument each task, writing the output to a file.");
    print_option("show-compositor-borders", "Paint borders along layer and tile boundaries.");
    print_option("show-fragment-borders", "Paint borders along fragment boundaries.");
    print_option("trace-layout", "Write layout trace to an external file for debugging.");
    print_option("validate-display-list-geometry",
                 "Display an error when display list geometry escapes overflow region.");

    println!("");
}

fn args_fail(msg: &str) {
    io::stderr().write_line(msg).unwrap();
    env::set_exit_status(1);
}

// Always use CPU painting on android.

#[cfg(target_os="android")]
static FORCE_CPU_PAINTING: bool = true;

#[cfg(not(target_os="android"))]
static FORCE_CPU_PAINTING: bool = false;

pub fn default_opts() -> Opts {
    Opts {
        urls: vec!(),
        n_paint_threads: 1,
        gpu_painting: false,
        tile_size: 512,
        device_pixels_per_px: None,
        time_profiler_period: None,
        memory_profiler_period: None,
        enable_experimental: false,
        layout_threads: 1,
        nonincremental_layout: false,
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
        resources_path: None,
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
        getopts::optopt("t", "threads", "Number of paint threads", "1"),
        getopts::optflagopt("p", "profile", "Profiler flag and output interval", "10"),
        getopts::optflagopt("m", "memory-profile", "Memory profiler flag and output interval", "10"),
        getopts::optflag("x", "exit", "Exit after load flag"),
        getopts::optopt("y", "layout-threads", "Number of threads to use for layout", "1"),
        getopts::optflag("i", "nonincremental-layout", "Enable to turn off incremental layout."),
        getopts::optflag("z", "headless", "Headless mode"),
        getopts::optflag("f", "hard-fail", "Exit on task failure instead of displaying about:failure"),
        getopts::optflagopt("", "devtools", "Start remote devtools server on port", "6000"),
        getopts::optopt("", "resolution", "Set window resolution.", "800x600"),
        getopts::optopt("u", "user-agent", "Set custom user agent string", "NCSA Mosaic/1.0 (X11;SunOS 4.1.4 sun4m)"),
        getopts::optopt("Z", "debug", "A comma-separated string of debug options. Pass help to show available options.", ""),
        getopts::optflag("h", "help", "Print this message"),
        getopts::optopt("r", "render-api", "Set the rendering API to use", "gl|mesa"),
        getopts::optopt("", "resources-path", "Path to find static resources", "/home/servo/resources"),
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

    let debug_string = match opt_match.opt_str("Z") {
        Some(string) => string,
        None => String::new()
    };
    let mut debug_options = HashSet::new();
    for split in debug_string.as_slice().split(',') {
        debug_options.insert(split.clone());
    }
    if debug_options.contains(&"help") {
        print_debug_usage(app_name.as_slice());
        return false;
    }

    let urls = if opt_match.free.is_empty() {
        print_usage(app_name.as_slice(), opts.as_slice());
        args_fail("servo asks that you provide 1 or more URLs");
        return false;
    } else {
        opt_match.free.clone()
    };

    let tile_size: uint = match opt_match.opt_str("s") {
        Some(tile_size_str) => tile_size_str.parse().unwrap(),
        None => 512,
    };

    let device_pixels_per_px = opt_match.opt_str("device-pixel-ratio").map(|dppx_str|
        ScaleFactor(dppx_str.parse().unwrap())
    );

    let mut n_paint_threads: uint = match opt_match.opt_str("t") {
        Some(n_paint_threads_str) => n_paint_threads_str.parse().unwrap(),
        None => 1,      // FIXME: Number of cores.
    };

    // If only the flag is present, default to a 5 second period for both profilers.
    let time_profiler_period = opt_match.opt_default("p", "5").map(|period| {
        period.parse().unwrap()
    });
    let memory_profiler_period = opt_match.opt_default("m", "5").map(|period| {
        period.parse().unwrap()
    });

    let gpu_painting = !FORCE_CPU_PAINTING && opt_match.opt_present("g");

    let mut layout_threads: uint = match opt_match.opt_str("y") {
        Some(layout_threads_str) => layout_threads_str.parse().unwrap(),
        None => cmp::max(rt::default_sched_threads() * 3 / 4, 1),
    };

    let nonincremental_layout = opt_match.opt_present("i");

    let mut bubble_inline_sizes_separately = debug_options.contains(&"bubble-widths");
    let trace_layout = debug_options.contains(&"trace-layout");
    if trace_layout {
        n_paint_threads = 1;
        layout_threads = 1;
        bubble_inline_sizes_separately = true;
    }

    let devtools_port = opt_match.opt_default("devtools", "6000").map(|port| {
        port.parse().unwrap()
    });

    let initial_window_size = match opt_match.opt_str("resolution") {
        Some(res_string) => {
            let res: Vec<uint> = res_string.split('x').map(|r| r.parse().unwrap()).collect();
            TypedSize2D(res[0], res[1])
        }
        None => {
            TypedSize2D(800, 600)
        }
    };

    let opts = Opts {
        urls: urls,
        n_paint_threads: n_paint_threads,
        gpu_painting: gpu_painting,
        tile_size: tile_size,
        device_pixels_per_px: device_pixels_per_px,
        time_profiler_period: time_profiler_period,
        memory_profiler_period: memory_profiler_period,
        enable_experimental: opt_match.opt_present("e"),
        layout_threads: layout_threads,
        nonincremental_layout: nonincremental_layout,
        output_file: opt_match.opt_str("o"),
        headless: opt_match.opt_present("z"),
        hard_fail: opt_match.opt_present("f"),
        bubble_inline_sizes_separately: bubble_inline_sizes_separately,
        profile_tasks: debug_options.contains(&"profile-tasks"),
        trace_layout: trace_layout,
        devtools_port: devtools_port,
        initial_window_size: initial_window_size,
        user_agent: opt_match.opt_str("u"),
        show_debug_borders: debug_options.contains(&"show-compositor-borders"),
        show_debug_fragment_borders: debug_options.contains(&"show-fragment-borders"),
        enable_text_antialiasing: !debug_options.contains(&"disable-text-aa"),
        dump_flow_tree: debug_options.contains(&"dump-flow-tree"),
        validate_display_list_geometry: debug_options.contains(&"validate-display-list-geometry"),
        resources_path: opt_match.opt_str("resources-path"),
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
