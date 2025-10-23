/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Configuration options for a single run of the servo application. Created
//! from command line arguments.

use std::default::Default;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;

/// Global flags for Servo, currently set on the command line.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Opts {
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

    /// True to turn off incremental layout.
    pub nonincremental_layout: bool,

    pub user_stylesheets: Vec<(Vec<u8>, ServoUrl)>,

    /// True to exit on thread failure instead of displaying about:failure.
    pub hard_fail: bool,

    /// Debug options that are used by developers to control Servo
    /// behavior for debugging purposes.
    pub debug: DebugOptions,

    /// Whether we're running in multiprocess mode.
    pub multiprocess: bool,

    /// Whether to force using ipc_channel instead of crossbeam_channel in singleprocess mode. Does
    /// nothing in multiprocess mode.
    pub force_ipc: bool,

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

    /// Load shaders from disk.
    pub shaders_path: Option<PathBuf>,

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

    /// Print the fragment tree after each layout.
    pub dump_flow_tree: bool,

    /// Print the stacking context tree after each layout.
    pub dump_stacking_context_tree: bool,

    /// Print the scroll tree after each layout.
    pub dump_scroll_tree: bool,

    /// Print the display list after each layout.
    pub dump_display_list: bool,

    /// Print notifications when there is a relayout.
    pub relayout_event: bool,

    /// Periodically print out on which events script threads spend their processing time.
    pub profile_script_events: bool,

    /// Disable the style sharing cache.
    pub disable_share_style_cache: bool,

    /// Whether to show in stdout style sharing cache stats after a restyle.
    pub dump_style_statistics: bool,

    /// Log GC passes and their durations.
    pub gc_profile: bool,
}

impl DebugOptions {
    pub fn extend(&mut self, debug_string: String) -> Result<(), String> {
        for option in debug_string.split(',') {
            match option {
                "help" => self.help = true,
                "disable-share-style-cache" => self.disable_share_style_cache = true,
                "dump-display-list" => self.dump_display_list = true,
                "dump-stacking-context-tree" => self.dump_stacking_context_tree = true,
                "dump-flow-tree" => self.dump_flow_tree = true,
                "dump-rule-tree" => self.dump_rule_tree = true,
                "dump-style-tree" => self.dump_style_tree = true,
                "dump-style-stats" => self.dump_style_statistics = true,
                "dump-scroll-tree" => self.dump_scroll_tree = true,
                "gc-profile" => self.gc_profile = true,
                "profile-script-events" => self.profile_script_events = true,
                "relayout-event" => self.relayout_event = true,
                "" => {},
                _ => return Err(String::from(option)),
            };
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum OutputOptions {
    /// Database connection config (hostname, name, user, pass)
    FileName(String),
    Stdout(f64),
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            time_profiling: None,
            time_profiler_trace_path: None,
            nonincremental_layout: false,
            user_stylesheets: Vec::new(),
            hard_fail: true,
            multiprocess: false,
            force_ipc: false,
            background_hang_monitor: false,
            random_pipeline_closure_probability: None,
            random_pipeline_closure_seed: None,
            sandbox: false,
            debug: Default::default(),
            config_dir: None,
            shaders_path: None,
            certificate_path: None,
            ignore_certificate_errors: false,
            unminify_js: false,
            local_script_source: None,
            unminify_css: false,
            print_pwm: false,
        }
    }
}

// Make Opts available globally. This saves having to clone and pass
// opts everywhere it is used, which gets particularly cumbersome
// when passing through the DOM structures.
static OPTIONS: OnceLock<Opts> = OnceLock::new();

/// Initialize options.
///
/// Should only be called once at process startup.
/// Must be called before the first call to [get].
pub fn initialize_options(opts: Opts) {
    OPTIONS.set(opts).expect("Already initialized");
}

/// Get the servo options
///
/// If the servo options have not been initialized by calling [initialize_options], then the
/// options will be initialized to default values. Outside of tests the options should
/// be explicitly initialized.
#[inline]
pub fn get() -> &'static Opts {
    // In unit-tests using default options reduces boilerplate.
    // We can't use `cfg(test)` since that only is enabled when this crate
    // is compiled in test mode.
    // We rely on the `expect` in `initialize_options` to inform us if refactoring
    // causes a `get` call to move before `initialize_options`.
    OPTIONS.get_or_init(Default::default)
}
