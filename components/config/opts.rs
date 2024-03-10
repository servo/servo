/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Configuration options for a single run of the servo application. Created
//! from command line arguments.

use std::default::Default;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{RwLock, RwLockReadGuard};
use std::{env, process};

use euclid::Size2D;
use getopts::{Matches, Options};
use lazy_static::lazy_static;
use log::error;
use serde::{Deserialize, Serialize};
use servo_geometry::DeviceIndependentPixel;
use servo_url::ServoUrl;
use url::{self, Url};

use crate::{pref, set_pref};

/// Global flags for Servo, currently set on the command line.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Opts {
    /// Whether or not the legacy layout system is enabled.
    pub legacy_layout: bool,

    /// The maximum size of each tile in pixels (`-s`).
    pub tile_size: usize,

    /// `None` to disable the time profiler or `Some` to enable it with:
    ///
    ///  - an interval in seconds to cause it to produce output on that interval.
    ///    (`i.e. -p 5`).
    ///  - a file path to write profiling info to a TSV file upon Servo's termination.
    ///    (`i.e. -p out.tsv`).
    pub time_profiling: Option<OutputOptions>,

    /// When the profiler is enabled, this is an optional path to dump a self-contained HTML file
    /// visualizing the traces as a timeline.
    pub time_profiler_trace_path: Option<String>,

    /// `None` to disable the memory profiler or `Some` with an interval in seconds to enable it
    /// and cause it to produce output on that interval (`-m`).
    pub mem_profiler_period: Option<f64>,

    /// True to turn off incremental layout.
    pub nonincremental_layout: bool,

    /// Where to load userscripts from, if any. An empty string will load from
    /// the resources/user-agent-js directory, and if the option isn't passed userscripts
    /// won't be loaded
    pub userscripts: Option<String>,

    pub user_stylesheets: Vec<(Vec<u8>, ServoUrl)>,

    pub output_file: Option<String>,

    pub headless: bool,

    /// True to exit on thread failure instead of displaying about:failure.
    pub hard_fail: bool,

    /// Debug options that are used by developers to control Servo
    /// behavior for debugging purposes.
    pub debug: DebugOptions,

    /// Port number to start a server to listen to remote Firefox devtools connections.
    /// 0 for random port.
    pub devtools_port: u16,

    /// Start the devtools server at startup
    pub devtools_server_enabled: bool,

    /// `None` to disable WebDriver or `Some` with a port number to start a server to listen to
    /// remote WebDriver commands.
    pub webdriver_port: Option<u16>,

    /// The initial requested size of the window.
    pub initial_window_size: Size2D<u32, DeviceIndependentPixel>,

    /// Whether we're running in multiprocess mode.
    pub multiprocess: bool,

    /// Whether we want background hang monitor enabled or not
    pub background_hang_monitor: bool,

    /// Whether we're running inside the sandbox.
    pub sandbox: bool,

    /// Probability of randomly closing a pipeline,
    /// used for testing the hardening of the constellation.
    pub random_pipeline_closure_probability: Option<f32>,

    /// The seed for the RNG used to randomly close pipelines,
    /// used for testing the hardening of the constellation.
    pub random_pipeline_closure_seed: Option<usize>,

    /// True to exit after the page load (`-x`).
    pub exit_after_load: bool,

    /// Load shaders from disk.
    pub shaders_dir: Option<PathBuf>,

    /// Directory for a default config directory
    pub config_dir: Option<PathBuf>,

    /// Print the version and exit.
    pub is_printing_version: bool,

    /// Path to PEM encoded SSL CA certificate store.
    pub certificate_path: Option<String>,

    /// Whether or not to completely ignore SSL certificate validation errors.
    /// TODO: We should see if we can eliminate the need for this by fixing
    /// <https://github.com/servo/servo/issues/30080>.
    pub ignore_certificate_errors: bool,

    /// Unminify Javascript.
    pub unminify_js: bool,

    /// Directory path that was created with "unminify-js"
    pub local_script_source: Option<String>,

    /// Print Progressive Web Metrics to console.
    pub print_pwm: bool,

    /// True to enable minibrowser
    pub minibrowser: bool,
}

fn print_usage(app: &str, opts: &Options) {
    let message = format!(
        "Usage: {} [ options ... ] [URL]\n\twhere options include",
        app
    );
    println!("{}", opts.usage(&message));
}

/// Debug options for Servo, currently set on the command line with -Z
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DebugOptions {
    /// List all the debug options.
    pub help: bool,

    /// True if we should bubble intrinsic widths sequentially. If this is true,
    /// then intrinsic widths are computed as a separate pass instead of during
    /// flow construction. You may wish to turn this flag on in order to
    /// benchmark style recalculation against other browser engines.
    pub bubble_inline_sizes_separately: bool,

    /// If set with `disable-text-aa`, disable antialiasing on fonts. This is
    /// primarily useful for reftests where pixel perfect results are required
    /// when using fonts such as the Ahem font for layout tests.
    pub disable_text_antialiasing: bool,

    /// Disable subpixel antialiasing of rendered text.
    pub disable_subpixel_text_antialiasing: bool,

    /// Disable antialiasing of rendered text on the HTML canvas element.
    /// If set with `disable-canvas-aa`, disable antialiasing on the HTML canvas
    /// element.  Like `disable-text-aa`, this is useful for reftests where
    /// pixel perfect results are required.
    pub disable_canvas_antialiasing: bool,

    /// Print the DOM after each restyle.
    pub dump_style_tree: bool,

    /// Dumps the rule tree.
    pub dump_rule_tree: bool,

    /// Print the flow tree (Layout 2013) or fragment tree (Layout 2020) after each layout.
    pub dump_flow_tree: bool,

    /// Print the stacking context tree after each layout.
    pub dump_stacking_context_tree: bool,

    /// Print the display list after each layout.
    pub dump_display_list: bool,

    /// Print the display list in JSON form.
    pub dump_display_list_json: bool,

    /// Print notifications when there is a relayout.
    pub relayout_event: bool,

    /// Periodically print out on which events script threads spend their processing time.
    pub profile_script_events: bool,

    /// Paint borders along fragment boundaries.
    pub show_fragment_borders: bool,

    /// Mark which thread laid each flow out with colors.
    pub show_parallel_layout: bool,

    /// True if each step of layout is traced to an external JSON file
    /// for debugging purposes. Setting this implies sequential layout
    /// and paint.
    pub trace_layout: bool,

    /// Disable the style sharing cache.
    pub disable_share_style_cache: bool,

    /// Whether to show in stdout style sharing cache stats after a restyle.
    pub dump_style_statistics: bool,

    /// Translate mouse input into touch events.
    pub convert_mouse_to_touch: bool,

    /// Replace unpaires surrogates in DOM strings with U+FFFD.
    /// See <https://github.com/servo/servo/issues/6564>
    pub replace_surrogates: bool,

    /// Log GC passes and their durations.
    pub gc_profile: bool,

    /// Load web fonts synchronously to avoid non-deterministic network-driven reflows.
    pub load_webfonts_synchronously: bool,

    /// Show webrender profiling stats on screen.
    pub webrender_stats: bool,

    /// True to compile all webrender shaders at init time. This is mostly
    /// useful when modifying the shaders, to ensure they all compile
    /// after each change is made.
    pub precache_shaders: bool,

    /// True to use OS native signposting facilities. This makes profiling events (script activity,
    /// reflow, compositing, etc.) appear in Instruments.app on macOS.
    pub signpost: bool,
}

impl DebugOptions {
    pub fn extend(&mut self, debug_string: String) -> Result<(), String> {
        for option in debug_string.split(',') {
            match option {
                "help" => self.help = true,
                "bubble-inline-sizes-separately" => self.bubble_inline_sizes_separately = true,
                "convert-mouse-to-touch" => self.convert_mouse_to_touch = true,
                "disable-canvas-aa" => self.disable_canvas_antialiasing = true,
                "disable-share-style-cache" => self.disable_share_style_cache = true,
                "disable-subpixel-aa" => self.disable_subpixel_text_antialiasing = true,
                "disable-text-aa" => self.disable_text_antialiasing = true,
                "dump-display-list" => self.dump_display_list = true,
                "dump-display-list-json" => self.dump_display_list_json = true,
                "dump-stacking-context-tree" => self.dump_stacking_context_tree = true,
                "dump-flow-tree" => self.dump_flow_tree = true,
                "dump-rule-tree" => self.dump_rule_tree = true,
                "dump-style-tree" => self.dump_style_tree = true,
                "gc-profile" => self.gc_profile = true,
                "load-webfonts-synchronously" => self.load_webfonts_synchronously = true,
                "precache-shaders" => self.precache_shaders = true,
                "profile-script-events" => self.profile_script_events = true,
                "relayout-event" => self.relayout_event = true,
                "replace-surrogates" => self.replace_surrogates = true,
                "show-fragment-borders" => self.show_fragment_borders = true,
                "show-parallel-layout" => self.show_parallel_layout = true,
                "signpost" => self.signpost = true,
                "dump-style-stats" => self.dump_style_statistics = true,
                "trace-layout" => self.trace_layout = true,
                "wr-stats" => self.webrender_stats = true,
                "" => {},
                _ => return Err(String::from(option)),
            };
        }

        if self.trace_layout {
            self.bubble_inline_sizes_separately = true;
        }

        Ok(())
    }

    fn print_usage(app: &str) {
        fn print_option(name: &str, description: &str) {
            println!("\t{:<35} {}", name, description);
        }

        println!(
            "Usage: {} debug option,[options,...]\n\twhere options include\n\nOptions:",
            app
        );

        print_option(
            "bubble-inline-sizes-separately",
            "Bubble intrinsic widths separately like other engines.",
        );
        print_option(
            "convert-mouse-to-touch",
            "Send touch events instead of mouse events",
        );
        print_option(
            "disable-canvas-aa",
            "Disable antialiasing on the HTML canvas element.",
        );
        print_option(
            "disable-share-style-cache",
            "Disable the style sharing cache.",
        );
        print_option(
            "disable-subpixel-aa",
            "Disable subpixel text antialiasing overriding preference.",
        );
        print_option("disable-text-aa", "Disable antialiasing of rendered text.");
        print_option(
            "dump-stacking-context-tree",
            "Print the stacking context tree after each layout.",
        );
        print_option(
            "dump-display-list",
            "Print the display list after each layout.",
        );
        print_option(
            "dump-display-list-json",
            "Print the display list in JSON form.",
        );
        print_option(
            "dump-flow-tree",
            "Print the flow tree (Layout 2013) or fragment tree (Layout 2020) after each layout.",
        );
        print_option(
            "dump-rule-tree",
            "Print the style rule tree after each layout.",
        );
        print_option(
            "dump-style-tree",
            "Print the DOM with computed styles after each restyle.",
        );
        print_option("dump-style-stats", "Print style statistics each restyle.");
        print_option("gc-profile", "Log GC passes and their durations.");
        print_option(
            "load-webfonts-synchronously",
            "Load web fonts synchronously to avoid non-deterministic network-driven reflows",
        );
        print_option(
            "parallel-display-list-building",
            "Build display lists in parallel.",
        );
        print_option("precache-shaders", "Compile all shaders during init.");
        print_option(
            "profile-script-events",
            "Enable profiling of script-related events.",
        );
        print_option(
            "relayout-event",
            "Print notifications when there is a relayout.",
        );
        print_option("replace-surrogates", "Replace unpaires surrogates in DOM strings with U+FFFD. See https://github.com/servo/servo/issues/6564");
        print_option(
            "show-fragment-borders",
            "Paint borders along fragment boundaries.",
        );
        print_option(
            "show-parallel-layout",
            "Mark which thread laid each flow out with colors.",
        );
        print_option(
            "signpost",
            "Emit native OS signposts for profile events (currently macOS only)",
        );
        print_option(
            "trace-layout",
            "Write layout trace to an external file for debugging.",
        );
        print_option("wr-stats", "Show WebRender profiler on screen.");

        println!();

        process::exit(0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OutputOptions {
    /// Database connection config (hostname, name, user, pass)
    FileName(String),
    Stdout(f64),
}

fn args_fail(msg: &str) -> ! {
    eprintln!("{}", msg);
    process::exit(1)
}

static MULTIPROCESS: AtomicBool = AtomicBool::new(false);

#[inline]
pub fn multiprocess() -> bool {
    MULTIPROCESS.load(Ordering::Relaxed)
}

pub fn default_opts() -> Opts {
    Opts {
        legacy_layout: false,
        tile_size: 512,
        time_profiling: None,
        time_profiler_trace_path: None,
        mem_profiler_period: None,
        nonincremental_layout: false,
        userscripts: None,
        user_stylesheets: Vec::new(),
        output_file: None,
        headless: false,
        hard_fail: true,
        devtools_port: 0,
        devtools_server_enabled: false,
        webdriver_port: None,
        initial_window_size: Size2D::new(1024, 740),
        multiprocess: false,
        background_hang_monitor: false,
        random_pipeline_closure_probability: None,
        random_pipeline_closure_seed: None,
        sandbox: false,
        debug: Default::default(),
        exit_after_load: false,
        config_dir: None,
        is_printing_version: false,
        shaders_dir: None,
        certificate_path: None,
        ignore_certificate_errors: false,
        unminify_js: false,
        local_script_source: None,
        print_pwm: false,
        minibrowser: true,
    }
}

pub fn from_cmdline_args(mut opts: Options, args: &[String]) -> ArgumentParsingResult {
    let (app_name, args) = args.split_first().unwrap();

    opts.optflag("", "legacy-layout", "Use the legacy layout engine");
    opts.optopt("o", "output", "Output file", "output.png");
    opts.optopt("s", "size", "Size of tiles", "512");
    opts.optflagopt(
        "p",
        "profile",
        "Time profiler flag and either a TSV output filename \
         OR an interval for output to Stdout (blank for Stdout with interval of 5s)",
        "10 \
         OR time.tsv",
    );
    opts.optflagopt(
        "",
        "profiler-trace-path",
        "Path to dump a self-contained HTML timeline of profiler traces",
        "",
    );
    opts.optflagopt(
        "m",
        "memory-profile",
        "Memory profiler flag and output interval",
        "10",
    );
    opts.optflag("x", "exit", "Exit after load flag");
    opts.optopt(
        "y",
        "layout-threads",
        "Number of threads to use for layout",
        "1",
    );
    opts.optflag(
        "i",
        "nonincremental-layout",
        "Enable to turn off incremental layout.",
    );
    opts.optflagopt(
        "",
        "userscripts",
        "Uses userscripts in resources/user-agent-js, or a specified full path",
        "",
    );
    opts.optmulti(
        "",
        "user-stylesheet",
        "A user stylesheet to be added to every document",
        "file.css",
    );
    opts.optopt(
        "",
        "shaders",
        "Shaders will be loaded from the specified directory instead of using the builtin ones.",
        "",
    );
    opts.optflag("z", "headless", "Headless mode");
    opts.optflag(
        "f",
        "hard-fail",
        "Exit on thread failure instead of displaying about:failure",
    );
    opts.optflag(
        "F",
        "soft-fail",
        "Display about:failure on thread failure instead of exiting",
    );
    opts.optflagopt("", "devtools", "Start remote devtools server on port", "0");
    opts.optflagopt(
        "",
        "webdriver",
        "Start remote WebDriver server on port",
        "7000",
    );
    opts.optopt("", "resolution", "Set window resolution.", "1024x740");
    opts.optflag("M", "multiprocess", "Run in multiprocess mode");
    opts.optflag("B", "bhm", "Background Hang Monitor enabled");
    opts.optflag("S", "sandbox", "Run in a sandbox if multiprocess");
    opts.optopt(
        "",
        "random-pipeline-closure-probability",
        "Probability of randomly closing a pipeline (for testing constellation hardening).",
        "0.0",
    );
    opts.optopt(
        "",
        "random-pipeline-closure-seed",
        "A fixed seed for repeatbility of random pipeline closure.",
        "",
    );
    opts.optmulti(
        "Z",
        "debug",
        "A comma-separated string of debug options. Pass help to show available options.",
        "",
    );
    opts.optflag("h", "help", "Print this message");
    opts.optopt(
        "",
        "resources-path",
        "Path to find static resources",
        "/home/servo/resources",
    );
    opts.optopt(
        "",
        "certificate-path",
        "Path to find SSL certificates",
        "/home/servo/resources/certs",
    );
    opts.optflag(
        "",
        "ignore-certificate-errors",
        "Whether or not to completely ignore certificate errors",
    );
    opts.optopt(
        "",
        "content-process",
        "Run as a content process and connect to the given pipe",
        "servo-ipc-channel.abcdefg",
    );
    opts.optopt(
        "",
        "config-dir",
        "config directory following xdg spec on linux platform",
        "",
    );
    opts.optflag("v", "version", "Display servo version information");
    opts.optflag("", "unminify-js", "Unminify Javascript");
    opts.optflag("", "print-pwm", "Print Progressive Web Metrics");
    opts.optopt(
        "",
        "local-script-source",
        "Directory root with unminified scripts",
        "",
    );
    opts.optflag("", "no-minibrowser", "Open minibrowser");

    let opt_match = match opts.parse(args) {
        Ok(m) => m,
        Err(f) => args_fail(&f.to_string()),
    };

    if opt_match.opt_present("h") || opt_match.opt_present("help") {
        print_usage(app_name, &opts);
        process::exit(0);
    };

    // If this is the content process, we'll receive the real options over IPC. So just fill in
    // some dummy options for now.
    if let Some(content_process) = opt_match.opt_str("content-process") {
        MULTIPROCESS.store(true, Ordering::SeqCst);
        return ArgumentParsingResult::ContentProcess(opt_match, content_process);
    }

    let mut debug_options = DebugOptions::default();
    for debug_string in opt_match.opt_strs("Z") {
        if let Err(e) = debug_options.extend(debug_string) {
            args_fail(&format!("error: unrecognized debug option: {}", e));
        }
    }

    if debug_options.help {
        DebugOptions::print_usage(app_name)
    }

    let tile_size: usize = match opt_match.opt_str("s") {
        Some(tile_size_str) => tile_size_str
            .parse()
            .unwrap_or_else(|err| args_fail(&format!("Error parsing option: -s ({})", err))),
        None => 512,
    };

    // If only the flag is present, default to a 5 second period for both profilers
    let time_profiling = if opt_match.opt_present("p") {
        match opt_match.opt_str("p") {
            Some(argument) => match argument.parse::<f64>() {
                Ok(interval) => Some(OutputOptions::Stdout(interval)),
                Err(_) => match ServoUrl::parse(&argument) {
                    Ok(_) => panic!("influxDB isn't supported anymore"),
                    Err(_) => Some(OutputOptions::FileName(argument)),
                },
            },
            None => Some(OutputOptions::Stdout(5.0)),
        }
    } else {
        // if the p option doesn't exist:
        None
    };

    if let Some(ref time_profiler_trace_path) = opt_match.opt_str("profiler-trace-path") {
        let mut path = PathBuf::from(time_profiler_trace_path);
        path.pop();
        if let Err(why) = fs::create_dir_all(&path) {
            error!(
                "Couldn't create/open {:?}: {:?}",
                Path::new(time_profiler_trace_path).to_string_lossy(),
                why
            );
        }
    }

    let mem_profiler_period = opt_match.opt_default("m", "5").map(|period| {
        period
            .parse()
            .unwrap_or_else(|err| args_fail(&format!("Error parsing option: -m ({})", err)))
    });

    let mut layout_threads: Option<usize> = opt_match.opt_str("y").map(|layout_threads_str| {
        layout_threads_str
            .parse()
            .unwrap_or_else(|err| args_fail(&format!("Error parsing option: -y ({})", err)))
    });

    let nonincremental_layout = opt_match.opt_present("i");

    let random_pipeline_closure_probability = opt_match
        .opt_str("random-pipeline-closure-probability")
        .map(|prob| {
            prob.parse().unwrap_or_else(|err| {
                args_fail(&format!(
                    "Error parsing option: --random-pipeline-closure-probability ({})",
                    err
                ))
            })
        });

    let random_pipeline_closure_seed =
        opt_match
            .opt_str("random-pipeline-closure-seed")
            .map(|seed| {
                seed.parse().unwrap_or_else(|err| {
                    args_fail(&format!(
                        "Error parsing option: --random-pipeline-closure-seed ({})",
                        err
                    ))
                })
            });

    if debug_options.trace_layout {
        layout_threads = Some(1);
    }

    let (devtools_server_enabled, devtools_port) = if opt_match.opt_present("devtools") {
        let port = opt_match
            .opt_str("devtools")
            .map(|port| {
                port.parse().unwrap_or_else(|err| {
                    args_fail(&format!("Error parsing option: --devtools ({})", err))
                })
            })
            .unwrap_or(pref!(devtools.server.port));
        (true, port as u16)
    } else {
        (
            pref!(devtools.server.enabled),
            pref!(devtools.server.port) as u16,
        )
    };

    let webdriver_port = opt_match.opt_default("webdriver", "7000").map(|port| {
        port.parse().unwrap_or_else(|err| {
            args_fail(&format!("Error parsing option: --webdriver ({})", err))
        })
    });

    let initial_window_size = match opt_match.opt_str("resolution") {
        Some(res_string) => {
            let res: Vec<u32> = res_string
                .split('x')
                .map(|r| {
                    r.parse().unwrap_or_else(|err| {
                        args_fail(&format!("Error parsing option: --resolution ({})", err))
                    })
                })
                .collect();
            Size2D::new(res[0], res[1])
        },
        None => Size2D::new(1024, 740),
    };

    if opt_match.opt_present("M") {
        MULTIPROCESS.store(true, Ordering::SeqCst)
    }

    let user_stylesheets = opt_match
        .opt_strs("user-stylesheet")
        .iter()
        .map(|filename| {
            let cwd = env::current_dir().unwrap();
            let path = cwd.join(filename);
            let url = ServoUrl::from_url(Url::from_file_path(&path).unwrap());
            let mut contents = Vec::new();
            File::open(path)
                .unwrap_or_else(|err| args_fail(&format!("Couldn't open {}: {}", filename, err)))
                .read_to_end(&mut contents)
                .unwrap_or_else(|err| args_fail(&format!("Couldn't read {}: {}", filename, err)));
            (contents, url)
        })
        .collect();

    let is_printing_version = opt_match.opt_present("v") || opt_match.opt_present("version");

    let legacy_layout = opt_match.opt_present("legacy-layout");
    if legacy_layout {
        set_pref!(layout.legacy_layout, true);
        set_pref!(layout.flexbox.enabled, true);
    }

    let opts = Opts {
        debug: debug_options.clone(),
        legacy_layout,
        tile_size,
        time_profiling,
        time_profiler_trace_path: opt_match.opt_str("profiler-trace-path"),
        mem_profiler_period,
        nonincremental_layout,
        userscripts: opt_match.opt_default("userscripts", ""),
        user_stylesheets,
        output_file: opt_match.opt_str("o"),
        headless: opt_match.opt_present("z"),
        hard_fail: opt_match.opt_present("f") && !opt_match.opt_present("F"),
        devtools_port,
        devtools_server_enabled,
        webdriver_port,
        initial_window_size,
        multiprocess: opt_match.opt_present("M"),
        background_hang_monitor: opt_match.opt_present("B"),
        sandbox: opt_match.opt_present("S"),
        random_pipeline_closure_probability,
        random_pipeline_closure_seed,
        exit_after_load: opt_match.opt_present("x"),
        config_dir: opt_match.opt_str("config-dir").map(Into::into),
        is_printing_version,
        shaders_dir: opt_match.opt_str("shaders").map(Into::into),
        certificate_path: opt_match.opt_str("certificate-path"),
        ignore_certificate_errors: opt_match.opt_present("ignore-certificate-errors"),
        unminify_js: opt_match.opt_present("unminify-js"),
        local_script_source: opt_match.opt_str("local-script-source"),
        print_pwm: opt_match.opt_present("print-pwm"),
        minibrowser: !opt_match.opt_present("no-minibrowser"),
    };

    set_options(opts);

    if let Some(layout_threads) = layout_threads {
        set_pref!(layout.threads, layout_threads as i64);
    }

    ArgumentParsingResult::ChromeProcess(opt_match)
}

pub enum ArgumentParsingResult {
    ChromeProcess(Matches),
    ContentProcess(Matches, String),
}

// Make Opts available globally. This saves having to clone and pass
// opts everywhere it is used, which gets particularly cumbersome
// when passing through the DOM structures.
lazy_static! {
    static ref OPTIONS: RwLock<Opts> = RwLock::new(default_opts());
}

pub fn set_options(opts: Opts) {
    MULTIPROCESS.store(opts.multiprocess, Ordering::SeqCst);
    *OPTIONS.write().unwrap() = opts;
}

#[inline]
pub fn get() -> RwLockReadGuard<'static, Opts> {
    OPTIONS.read().unwrap()
}
