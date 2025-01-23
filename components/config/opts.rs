/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Configuration options for a single run of the servo application. Created
//! from command line arguments.

use std::default::Default;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock, RwLockReadGuard};

use euclid::Size2D;
use serde::{Deserialize, Serialize};
use servo_geometry::DeviceIndependentPixel;
use servo_url::ServoUrl;

/// Global flags for Servo, currently set on the command line.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Opts {
    /// Whether or not the legacy layout system is enabled.
    pub legacy_layout: bool,

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

    /// `None` to disable WebDriver or `Some` with a port number to start a server to listen to
    /// remote WebDriver commands.
    pub webdriver_port: Option<u16>,

    /// The initial requested size of the window.
    pub initial_window_size: Size2D<u32, DeviceIndependentPixel>,

    /// An override for the screen resolution. This is useful for testing behavior on different screen sizes,
    /// such as the screen of a mobile device.
    pub screen_size_override: Option<Size2D<u32, DeviceIndependentPixel>>,

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

    /// Unminify Css.
    pub unminify_css: bool,

    /// Print Progressive Web Metrics to console.
    pub print_pwm: bool,
}

/// Debug options for Servo, currently set on the command line with -Z
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DebugOptions {
    /// List all the debug options.
    pub help: bool,

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

    /// Print notifications when there is a relayout.
    pub relayout_event: bool,

    /// Periodically print out on which events script threads spend their processing time.
    pub profile_script_events: bool,

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

    /// Show webrender profiling stats on screen.
    pub webrender_stats: bool,

    /// True to use OS native signposting facilities. This makes profiling events (script activity,
    /// reflow, compositing, etc.) appear in Instruments.app on macOS.
    pub signpost: bool,
}

impl DebugOptions {
    pub fn extend(&mut self, debug_string: String) -> Result<(), String> {
        for option in debug_string.split(',') {
            match option {
                "help" => self.help = true,
                "convert-mouse-to-touch" => self.convert_mouse_to_touch = true,
                "disable-share-style-cache" => self.disable_share_style_cache = true,
                "dump-display-list" => self.dump_display_list = true,
                "dump-stacking-context-tree" => self.dump_stacking_context_tree = true,
                "dump-flow-tree" => self.dump_flow_tree = true,
                "dump-rule-tree" => self.dump_rule_tree = true,
                "dump-style-tree" => self.dump_style_tree = true,
                "gc-profile" => self.gc_profile = true,
                "profile-script-events" => self.profile_script_events = true,
                "relayout-event" => self.relayout_event = true,
                "replace-surrogates" => self.replace_surrogates = true,
                "signpost" => self.signpost = true,
                "dump-style-stats" => self.dump_style_statistics = true,
                "trace-layout" => self.trace_layout = true,
                "wr-stats" => self.webrender_stats = true,
                "" => {},
                _ => return Err(String::from(option)),
            };
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OutputOptions {
    /// Database connection config (hostname, name, user, pass)
    FileName(String),
    Stdout(f64),
}

pub fn default_opts() -> Opts {
    Opts {
        legacy_layout: false,
        time_profiling: None,
        time_profiler_trace_path: None,
        mem_profiler_period: None,
        nonincremental_layout: false,
        userscripts: None,
        user_stylesheets: Vec::new(),
        output_file: None,
        headless: false,
        hard_fail: true,
        webdriver_port: None,
        initial_window_size: Size2D::new(1024, 740),
        screen_size_override: None,
        multiprocess: false,
        background_hang_monitor: false,
        random_pipeline_closure_probability: None,
        random_pipeline_closure_seed: None,
        sandbox: false,
        debug: Default::default(),
        exit_after_load: false,
        config_dir: None,
        shaders_dir: None,
        certificate_path: None,
        ignore_certificate_errors: false,
        unminify_js: false,
        local_script_source: None,
        unminify_css: false,
        print_pwm: false,
    }
}

// Make Opts available globally. This saves having to clone and pass
// opts everywhere it is used, which gets particularly cumbersome
// when passing through the DOM structures.
static OPTIONS: LazyLock<RwLock<Opts>> = LazyLock::new(|| RwLock::new(default_opts()));

pub fn set_options(opts: Opts) {
    *OPTIONS.write().unwrap() = opts;
}

#[inline]
pub fn get() -> RwLockReadGuard<'static, Opts> {
    OPTIONS.read().unwrap()
}
