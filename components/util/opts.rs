/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Configuration options for a single run of the servo application. Created
//! from command line arguments.

use geometry::ScreenPx;

use euclid::size::{Size2D, TypedSize2D};
use getopts::Options;
use num_cpus;
use std::cmp;
use std::default::Default;
use std::env;
use std::fs::{File, PathExt};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};
use url::{self, Url};

/// Global flags for Servo, currently set on the command line.
#[derive(Clone)]
pub struct Opts {
    /// The initial URL to load.
    pub url: Option<Url>,

    /// How many threads to use for CPU painting (`-t`).
    ///
    /// Note that painting is sequentialized when using GPU painting.
    pub paint_threads: usize,

    /// True to use GPU painting via Skia-GL, false to use CPU painting via Skia (`-g`). Note that
    /// compositing is always done on the GPU.
    pub gpu_painting: bool,

    /// The maximum size of each tile in pixels (`-s`).
    pub tile_size: usize,

    /// The ratio of device pixels per px at the default scale. If unspecified, will use the
    /// platform default setting.
    pub device_pixels_per_px: Option<f32>,

    /// `None` to disable the time profiler or `Some` with an interval in seconds to enable it and
    /// cause it to produce output on that interval (`-p`).
    pub time_profiler_period: Option<f64>,

    /// `None` to disable the memory profiler or `Some` with an interval in seconds to enable it
    /// and cause it to produce output on that interval (`-m`).
    pub mem_profiler_period: Option<f64>,

    /// Enable experimental web features (`-e`).
    pub enable_experimental: bool,

    /// The number of threads to use for layout (`-y`). Defaults to 1, which results in a recursive
    /// sequential algorithm.
    pub layout_threads: usize,

    pub nonincremental_layout: bool,

    pub nossl: bool,

    /// Where to load userscripts from, if any. An empty string will load from
    /// the resources/user-agent-js directory, and if the option isn't passed userscripts
    /// won't be loaded
    pub userscripts: Option<String>,

    pub user_stylesheets: Vec<(Vec<u8>, Url)>,

    pub output_file: Option<String>,

    /// Replace unpaires surrogates in DOM strings with U+FFFD.
    /// See https://github.com/servo/servo/issues/6564
    pub replace_surrogates: bool,

    /// Log GC passes and their durations.
    pub gc_profile: bool,

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

    /// True if we should show borders on all fragments for debugging purposes
    /// (`--show-debug-fragment-borders`).
    pub show_debug_fragment_borders: bool,

    /// True if we should paint tiles with overlays based on which thread painted them.
    pub show_debug_parallel_paint: bool,

    /// True if we should paint borders around flows based on which thread painted them.
    pub show_debug_parallel_layout: bool,

    /// True if we should paint tiles a random color whenever they're repainted. Useful for
    /// debugging invalidation.
    pub paint_flashing: bool,

    /// If set with --disable-text-aa, disable antialiasing on fonts. This is primarily useful for reftests
    /// where pixel perfect results are required when using fonts such as the Ahem
    /// font for layout tests.
    pub enable_text_antialiasing: bool,

    /// If set with --disable-canvas-aa, disable antialiasing on the HTML canvas element.
    /// Like --disable-text-aa, this is useful for reftests where pixel perfect results are required.
    pub enable_canvas_antialiasing: bool,

    /// True if each step of layout is traced to an external JSON file
    /// for debugging purposes. Settings this implies sequential layout
    /// and paint.
    pub trace_layout: bool,

    /// If true, instrument the runtime for each task created and dump
    /// that information to a JSON file that can be viewed in the task
    /// profile viewer.
    pub profile_tasks: bool,

    /// Periodically print out on which events script tasks spend their processing time.
    pub profile_script_events: bool,

    /// `None` to disable devtools or `Some` with a port number to start a server to listen to
    /// remote Firefox devtools connections.
    pub devtools_port: Option<u16>,

    /// `None` to disable WebDriver or `Some` with a port number to start a server to listen to
    /// remote WebDriver commands.
    pub webdriver_port: Option<u16>,

    /// The initial requested size of the window.
    pub initial_window_size: TypedSize2D<ScreenPx, u32>,

    /// An optional string allowing the user agent to be set for testing.
    pub user_agent: String,

    /// Whether to run in multiprocess mode.
    pub multiprocess: bool,

    /// Dumps the flow tree after a layout.
    pub dump_flow_tree: bool,

    /// Dumps the display list after a layout.
    pub dump_display_list: bool,

    /// Dumps the display list in JSON form after a layout.
    pub dump_display_list_json: bool,

    /// Dumps the display list after optimization (post layout, at painting time).
    pub dump_display_list_optimized: bool,

    /// Emits notifications when there is a relayout.
    pub relayout_event: bool,

    /// Whether to show an error when display list geometry escapes flow overflow regions.
    pub validate_display_list_geometry: bool,

    /// A specific path to find required resources (such as user-agent.css).
    pub resources_path: Option<String>,

    /// Whether MIME sniffing should be used
    pub sniff_mime_types: bool,

    /// Whether Style Sharing Cache is used
    pub disable_share_style_cache: bool,

    /// Whether to run absolute position calculation and display list construction in parallel.
    pub parallel_display_list_building: bool,

    /// True to exit after the page load (`-x`).
    pub exit_after_load: bool,
}

fn print_usage(app: &str, opts: &Options) {
    let message = format!("Usage: {} [ options ... ] [URL]\n\twhere options include", app);
    println!("{}", opts.usage(&message));
}


/// Debug options for Servo, currently set on the command line with -Z
#[derive(Default)]
pub struct DebugOptions {
    /// List all the debug options.
    pub help: bool,

    /// Bubble intrinsic widths separately like other engines.
    pub bubble_widths: bool,

    /// Disable antialiasing of rendered text.
    pub disable_text_aa: bool,

    /// Disable antialiasing of rendered text on the HTML canvas element.
    pub disable_canvas_aa: bool,

    /// Print the flow tree after each layout.
    pub dump_flow_tree: bool,

    /// Print the display list after each layout.
    pub dump_display_list: bool,

    /// Print the display list in JSON form.
    pub dump_display_list_json: bool,

    /// Print optimized display list (at paint time).
    pub dump_display_list_optimized: bool,

    /// Print notifications when there is a relayout.
    pub relayout_event: bool,

    /// Instrument each task, writing the output to a file.
    pub profile_tasks: bool,

    /// Profile which events script tasks spend their time on.
    pub profile_script_events: bool,

    /// Paint borders along layer and tile boundaries.
    pub show_compositor_borders: bool,

    /// Paint borders along fragment boundaries.
    pub show_fragment_borders: bool,

    /// Overlay tiles with colors showing which thread painted them.
    pub show_parallel_paint: bool,

    /// Mark which thread laid each flow out with colors.
    pub show_parallel_layout: bool,

    /// Overlay repainted areas with a random color.
    pub paint_flashing: bool,

    /// Write layout trace to an external file for debugging.
    pub trace_layout: bool,

    /// Display an error when display list geometry escapes overflow region.
    pub validate_display_list_geometry: bool,

    /// Disable the style sharing cache.
    pub disable_share_style_cache: bool,

    /// Build display lists in parallel.
    pub parallel_display_list_building: bool,

    /// Replace unpaires surrogates in DOM strings with U+FFFD.
    /// See https://github.com/servo/servo/issues/6564
    pub replace_surrogates: bool,

    /// Log GC passes and their durations.
    pub gc_profile: bool,
}


impl DebugOptions {
    pub fn new<'a>(debug_string: &'a str) -> Result<DebugOptions, &'a str> {
        let mut debug_options = DebugOptions::default();

        for option in debug_string.split(',') {
            match option {
                "help" => debug_options.help = true,
                "bubble-widths" => debug_options.bubble_widths = true,
                "disable-text-aa" => debug_options.disable_text_aa = true,
                "disable-canvas-aa" => debug_options.disable_text_aa = true,
                "dump-flow-tree" => debug_options.dump_flow_tree = true,
                "dump-display-list" => debug_options.dump_display_list = true,
                "dump-display-list-json" => debug_options.dump_display_list_json = true,
                "dump-display-list-optimized" => debug_options.dump_display_list_optimized = true,
                "relayout-event" => debug_options.relayout_event = true,
                "profile-tasks" => debug_options.profile_tasks = true,
                "profile-script-events" => debug_options.profile_script_events = true,
                "show-compositor-borders" => debug_options.show_compositor_borders = true,
                "show-fragment-borders" => debug_options.show_fragment_borders = true,
                "show-parallel-paint" => debug_options.show_parallel_paint = true,
                "show-parallel-layout" => debug_options.show_parallel_layout = true,
                "paint-flashing" => debug_options.paint_flashing = true,
                "trace-layout" => debug_options.trace_layout = true,
                "validate-display-list-geometry" => debug_options.validate_display_list_geometry = true,
                "disable-share-style-cache" => debug_options.disable_share_style_cache = true,
                "parallel-display-list-building" => debug_options.parallel_display_list_building = true,
                "replace-surrogates" => debug_options.replace_surrogates = true,
                "gc-profile" => debug_options.gc_profile = true,
                "" => {},
                _ => return Err(option)
            };
        };

        Ok(debug_options)
    }
}


pub fn print_debug_usage(app: &str) -> ! {
    fn print_option(name: &str, description: &str) {
        println!("\t{:<35} {}", name, description);
    }

    println!("Usage: {} debug option,[options,...]\n\twhere options include\n\nOptions:", app);

    print_option("bubble-widths", "Bubble intrinsic widths separately like other engines.");
    print_option("disable-text-aa", "Disable antialiasing of rendered text.");
    print_option("disable-canvas-aa", "Disable antialiasing on the HTML canvas element.");
    print_option("dump-flow-tree", "Print the flow tree after each layout.");
    print_option("dump-display-list", "Print the display list after each layout.");
    print_option("dump-display-list-json", "Print the display list in JSON form.");
    print_option("dump-display-list-optimized", "Print optimized display list (at paint time).");
    print_option("relayout-event", "Print notifications when there is a relayout.");
    print_option("profile-tasks", "Instrument each task, writing the output to a file.");
    print_option("show-compositor-borders", "Paint borders along layer and tile boundaries.");
    print_option("show-fragment-borders", "Paint borders along fragment boundaries.");
    print_option("show-parallel-paint", "Overlay tiles with colors showing which thread painted them.");
    print_option("show-parallel-layout", "Mark which thread laid each flow out with colors.");
    print_option("paint-flashing", "Overlay repainted areas with a random color.");
    print_option("trace-layout", "Write layout trace to an external file for debugging.");
    print_option("validate-display-list-geometry",
                 "Display an error when display list geometry escapes overflow region.");
    print_option("disable-share-style-cache",
                 "Disable the style sharing cache.");
    print_option("parallel-display-list-building", "Build display lists in parallel.");
    print_option("replace-surrogates", "Replace unpaires surrogates in DOM strings with U+FFFD. \
                                        See https://github.com/servo/servo/issues/6564");
    print_option("gc-profile", "Log GC passes and their durations.");

    println!("");

    process::exit(0)
}

fn args_fail(msg: &str) -> ! {
    let mut stderr = io::stderr();
    stderr.write_all(msg.as_bytes()).unwrap();
    stderr.write_all(b"\n").unwrap();
    process::exit(1)
}

// Always use CPU painting on android.

#[cfg(target_os="android")]
static FORCE_CPU_PAINTING: bool = true;

#[cfg(not(target_os="android"))]
static FORCE_CPU_PAINTING: bool = false;

enum UserAgent {
    Desktop,
    Android,
    Gonk,
}

fn default_user_agent_string(agent: UserAgent) -> String {
    match agent {
        UserAgent::Desktop => {
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.10; rv:37.0) Servo/1.0 Firefox/37.0"
        }
        UserAgent::Android => {
            "Mozilla/5.0 (Android; Mobile; rv:37.0) Servo/1.0 Firefox/37.0"
        }
        UserAgent::Gonk => {
            "Mozilla/5.0 (Mobile; rv:37.0) Servo/1.0 Firefox/37.0"
        }
    }.to_owned()
}

#[cfg(target_os="android")]
const DEFAULT_USER_AGENT: UserAgent = UserAgent::Android;

// FIXME: This requires https://github.com/servo/servo/issues/7138 to provide the
// correct string in Gonk builds (i.e., it will never be chosen today).
#[cfg(target_os="gonk")]
const DEFAULT_USER_AGENT: UserAgent = UserAgent::Gonk;

#[cfg(not(any(target_os="android", target_os="gonk")))]
const DEFAULT_USER_AGENT: UserAgent = UserAgent::Desktop;

pub fn default_opts() -> Opts {
    Opts {
        url: Some(Url::parse("about:blank").unwrap()),
        paint_threads: 1,
        gpu_painting: false,
        tile_size: 512,
        device_pixels_per_px: None,
        time_profiler_period: None,
        mem_profiler_period: None,
        enable_experimental: false,
        layout_threads: 1,
        nonincremental_layout: false,
        nossl: false,
        userscripts: None,
        user_stylesheets: Vec::new(),
        output_file: None,
        replace_surrogates: false,
        gc_profile: false,
        headless: true,
        hard_fail: true,
        bubble_inline_sizes_separately: false,
        show_debug_borders: false,
        show_debug_fragment_borders: false,
        show_debug_parallel_paint: false,
        show_debug_parallel_layout: false,
        paint_flashing: false,
        enable_text_antialiasing: false,
        enable_canvas_antialiasing: false,
        trace_layout: false,
        devtools_port: None,
        webdriver_port: None,
        initial_window_size: Size2D::typed(800, 600),
        user_agent: default_user_agent_string(DEFAULT_USER_AGENT),
        multiprocess: false,
        dump_flow_tree: false,
        dump_display_list: false,
        dump_display_list_json: false,
        dump_display_list_optimized: false,
        relayout_event: false,
        validate_display_list_geometry: false,
        profile_tasks: false,
        profile_script_events: false,
        resources_path: None,
        sniff_mime_types: false,
        disable_share_style_cache: false,
        parallel_display_list_building: false,
        exit_after_load: false,
    }
}

pub fn from_cmdline_args(args: &[String]) {
    let (app_name, args) = args.split_first().unwrap();

    let mut opts = Options::new();
    opts.optflag("c", "cpu", "CPU painting (default)");
    opts.optflag("g", "gpu", "GPU painting");
    opts.optopt("o", "output", "Output file", "output.png");
    opts.optopt("s", "size", "Size of tiles", "512");
    opts.optopt("", "device-pixel-ratio", "Device pixels per px", "");
    opts.optflag("e", "experimental", "Enable experimental web features");
    opts.optopt("t", "threads", "Number of paint threads", "1");
    opts.optflagopt("p", "profile", "Profiler flag and output interval", "10");
    opts.optflagopt("m", "memory-profile", "Memory profiler flag and output interval", "10");
    opts.optflag("x", "exit", "Exit after load flag");
    opts.optopt("y", "layout-threads", "Number of threads to use for layout", "1");
    opts.optflag("i", "nonincremental-layout", "Enable to turn off incremental layout.");
    opts.optflag("", "no-ssl", "Disables ssl certificate verification.");
    opts.optflagopt("", "userscripts",
                    "Uses userscripts in resources/user-agent-js, or a specified full path","");
    opts.optmulti("", "user-stylesheet",
                  "A user stylesheet to be added to every document", "file.css");
    opts.optflag("z", "headless", "Headless mode");
    opts.optflag("f", "hard-fail", "Exit on task failure instead of displaying about:failure");
    opts.optflagopt("", "devtools", "Start remote devtools server on port", "6000");
    opts.optflagopt("", "webdriver", "Start remote WebDriver server on port", "7000");
    opts.optopt("", "resolution", "Set window resolution.", "800x600");
    opts.optopt("u",
                "user-agent",
                "Set custom user agent string (or android / gonk / desktop for platform default)",
                "NCSA Mosaic/1.0 (X11;SunOS 4.1.4 sun4m)");
    opts.optflag("M", "multiprocess", "Run in multiprocess mode");
    opts.optopt("Z", "debug",
                "A comma-separated string of debug options. Pass help to show available options.", "");
    opts.optflag("h", "help", "Print this message");
    opts.optopt("", "resources-path", "Path to find static resources", "/home/servo/resources");
    opts.optflag("", "sniff-mime-types" , "Enable MIME sniffing");

    let opt_match = match opts.parse(args) {
        Ok(m) => m,
        Err(f) => args_fail(&f.to_string()),
    };

    if opt_match.opt_present("h") || opt_match.opt_present("help") {
        print_usage(app_name, &opts);
        process::exit(0);
    };

    let debug_string = match opt_match.opt_str("Z") {
        Some(string) => string,
        None => String::new()
    };

    let debug_options = match DebugOptions::new(&debug_string) {
        Ok(debug_options) => debug_options,
        Err(e) => args_fail(&format!("error: unrecognized debug option: {}", e)),
    };

    if debug_options.help {
        print_debug_usage(app_name)
    }

    let cwd = env::current_dir().unwrap();
    let url = if opt_match.free.is_empty() {
        print_usage(app_name, &opts);
        args_fail("servo asks that you provide a URL")
    } else {
        let ref url = opt_match.free[0];
        match Url::parse(url) {
            Ok(url) => url,
            Err(url::ParseError::RelativeUrlWithoutBase) => {
                if Path::new(url).exists() {
                    Url::from_file_path(&*cwd.join(url)).unwrap()
                } else {
                    args_fail(&format!("File not found: {}", url))
                }
            }
            Err(_) => panic!("URL parsing failed"),
        }
    };

    let tile_size: usize = match opt_match.opt_str("s") {
        Some(tile_size_str) => tile_size_str.parse().unwrap(),
        None => 512,
    };

    let device_pixels_per_px = opt_match.opt_str("device-pixel-ratio").map(|dppx_str|
        dppx_str.parse().unwrap()
    );

    let mut paint_threads: usize = match opt_match.opt_str("t") {
        Some(paint_threads_str) => paint_threads_str.parse().unwrap(),
        None => cmp::max(num_cpus::get() * 3 / 4, 1),
    };

    // If only the flag is present, default to a 5 second period for both profilers.
    let time_profiler_period = opt_match.opt_default("p", "5").map(|period| {
        period.parse().unwrap()
    });
    let mem_profiler_period = opt_match.opt_default("m", "5").map(|period| {
        period.parse().unwrap()
    });

    let gpu_painting = !FORCE_CPU_PAINTING && opt_match.opt_present("g");

    let mut layout_threads: usize = match opt_match.opt_str("y") {
        Some(layout_threads_str) => layout_threads_str.parse().unwrap(),
        None => cmp::max(num_cpus::get() * 3 / 4, 1),
    };

    let nonincremental_layout = opt_match.opt_present("i");
    let nossl = opt_match.opt_present("no-ssl");

    let mut bubble_inline_sizes_separately = debug_options.bubble_widths;
    if debug_options.trace_layout {
        paint_threads = 1;
        layout_threads = 1;
        bubble_inline_sizes_separately = true;
    }

    let devtools_port = opt_match.opt_default("devtools", "6000").map(|port| {
        port.parse().unwrap()
    });

    let webdriver_port = opt_match.opt_default("webdriver", "7000").map(|port| {
        port.parse().unwrap()
    });

    let initial_window_size = match opt_match.opt_str("resolution") {
        Some(res_string) => {
            let res: Vec<u32> = res_string.split('x').map(|r| r.parse().unwrap()).collect();
            Size2D::typed(res[0], res[1])
        }
        None => {
            Size2D::typed(800, 600)
        }
    };

    let user_agent = match opt_match.opt_str("u") {
        Some(ref ua) if ua == "android" => default_user_agent_string(UserAgent::Android),
        Some(ref ua) if ua == "gonk" => default_user_agent_string(UserAgent::Gonk),
        Some(ref ua) if ua == "desktop" => default_user_agent_string(UserAgent::Desktop),
        Some(ua) => ua,
        None => default_user_agent_string(DEFAULT_USER_AGENT),
    };

    let user_stylesheets = opt_match.opt_strs("user-stylesheet").iter().map(|filename| {
        let path = cwd.join(filename);
        let url = Url::from_file_path(&path).unwrap();
        let mut contents = Vec::new();
        File::open(path)
            .unwrap_or_else(|err| args_fail(&format!("Couldn’t open {}: {}", filename, err)))
            .read_to_end(&mut contents)
            .unwrap_or_else(|err| args_fail(&format!("Couldn’t read {}: {}", filename, err)));
        (contents, url)
    }).collect();

    let opts = Opts {
        url: Some(url),
        paint_threads: paint_threads,
        gpu_painting: gpu_painting,
        tile_size: tile_size,
        device_pixels_per_px: device_pixels_per_px,
        time_profiler_period: time_profiler_period,
        mem_profiler_period: mem_profiler_period,
        enable_experimental: opt_match.opt_present("e"),
        layout_threads: layout_threads,
        nonincremental_layout: nonincremental_layout,
        nossl: nossl,
        userscripts: opt_match.opt_default("userscripts", ""),
        user_stylesheets: user_stylesheets,
        output_file: opt_match.opt_str("o"),
        replace_surrogates: debug_options.replace_surrogates,
        gc_profile: debug_options.gc_profile,
        headless: opt_match.opt_present("z"),
        hard_fail: opt_match.opt_present("f"),
        bubble_inline_sizes_separately: bubble_inline_sizes_separately,
        profile_tasks: debug_options.profile_tasks,
        profile_script_events: debug_options.profile_script_events,
        trace_layout: debug_options.trace_layout,
        devtools_port: devtools_port,
        webdriver_port: webdriver_port,
        initial_window_size: initial_window_size,
        user_agent: user_agent,
        multiprocess: opt_match.opt_present("M"),
        show_debug_borders: debug_options.show_compositor_borders,
        show_debug_fragment_borders: debug_options.show_fragment_borders,
        show_debug_parallel_paint: debug_options.show_parallel_paint,
        show_debug_parallel_layout: debug_options.show_parallel_layout,
        paint_flashing: debug_options.paint_flashing,
        enable_text_antialiasing: !debug_options.disable_text_aa,
        enable_canvas_antialiasing: !debug_options.disable_canvas_aa,
        dump_flow_tree: debug_options.dump_flow_tree,
        dump_display_list: debug_options.dump_display_list,
        dump_display_list_json: debug_options.dump_display_list_json,
        dump_display_list_optimized: debug_options.dump_display_list_optimized,
        relayout_event: debug_options.relayout_event,
        validate_display_list_geometry: debug_options.validate_display_list_geometry,
        resources_path: opt_match.opt_str("resources-path"),
        sniff_mime_types: opt_match.opt_present("sniff-mime-types"),
        disable_share_style_cache: debug_options.disable_share_style_cache,
        parallel_display_list_building: debug_options.parallel_display_list_building,
        exit_after_load: opt_match.opt_present("x"),
    };

    set_defaults(opts);
}

static EXPERIMENTAL_ENABLED: AtomicBool = ATOMIC_BOOL_INIT;

/// Turn on experimental features globally. Normally this is done
/// during initialization by `set` or `from_cmdline_args`, but
/// tests that require experimental features will also set it.
pub fn set_experimental_enabled(new_value: bool) {
    EXPERIMENTAL_ENABLED.store(new_value, Ordering::SeqCst);
}

pub fn experimental_enabled() -> bool {
    EXPERIMENTAL_ENABLED.load(Ordering::SeqCst)
}

// Make Opts available globally. This saves having to clone and pass
// opts everywhere it is used, which gets particularly cumbersome
// when passing through the DOM structures.
static mut DEFAULT_OPTIONS: *mut Opts = 0 as *mut Opts;
const INVALID_OPTIONS: *mut Opts = 0x01 as *mut Opts;

lazy_static! {
    static ref OPTIONS: Opts = {
        let opts = unsafe {
            let initial = if !DEFAULT_OPTIONS.is_null() {
                let opts = Box::from_raw(DEFAULT_OPTIONS);
                *opts
            } else {
                default_opts()
            };
            DEFAULT_OPTIONS = INVALID_OPTIONS;
            initial
        };
        set_experimental_enabled(opts.enable_experimental);
        opts
    };
}

pub fn set_defaults(opts: Opts) {
    unsafe {
        assert!(DEFAULT_OPTIONS.is_null());
        assert!(DEFAULT_OPTIONS != INVALID_OPTIONS);
        let box_opts = box opts;
        DEFAULT_OPTIONS = Box::into_raw(box_opts);
    }
}

#[inline]
pub fn get() -> &'static Opts {
    &OPTIONS
}
