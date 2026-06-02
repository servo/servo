/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Configuration options for a single run of the servo application. Created
//! from command line arguments.

use std::default::Default;
use std::path::PathBuf;
use std::process;
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
    pub debug: DiagnosticsLogging,

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
pub struct DiagnosticsLogging {
    /// List all the debug options.
    pub help: bool,

    /// Print the DOM after each restyle.
    pub style_tree: bool,

    /// Log the rule tree.
    pub rule_tree: bool,

    /// Log the fragment tree after each layout.
    pub flow_tree: bool,

    /// Log the stacking context tree after each layout.
    pub stacking_context_tree: bool,

    /// Log the scroll tree after each layout.
    ///
    /// Displays the hierarchy of scrollable areas and their properties.
    pub scroll_tree: bool,

    /// Log the display list after each layout.
    pub display_list: bool,

    /// Log notifications when a relayout occurs.
    pub relayout_event: bool,

    /// Periodically log on which events script threads spend their processing time.
    pub profile_script_events: bool,

    /// Log style sharing cache statistics to after each restyle.
    ///
    /// Shows hit/miss statistics for the style sharing cache
    pub style_statistics: bool,

    /// Log garbage collection passes and their durations.
    pub gc_profile: bool,
}

impl DiagnosticsLogging {
    /// Create a new DiagnosticsLogging configuration.
    ///
    /// In non-production builds, this will automatically read and parse the
    /// SERVO_DIAGNOSTICS environment variable if it is set.
    pub fn new() -> Self {
        let mut config: DiagnosticsLogging = Default::default();

        // Disabled for production builds
        #[cfg(debug_assertions)]
        {
            if let Ok(diagnostics_var) = std::env::var("SERVO_DIAGNOSTICS") {
                if let Err(error) = config.extend_from_string(&diagnostics_var) {
                    eprintln!("Could not parse debug logging option: {error}");
                }
            }
        }

        config
    }

    /// Print available diagnostic logging options and their descriptions.
    fn print_debug_options_usage(app: &str) {
        fn print_option(name: &str, description: &str) {
            println!("\t{:<35} {}", name, description);
        }

        println!(
            "Usage: {} debug option,[options,...]\n\twhere options include\n\nOptions:",
            app
        );
        print_option("help", "Show this help message");
        print_option("style-tree", "Log the style tree after each restyle");
        print_option("rule-tree", "Log the rule tree");
        print_option("flow-tree", "Log the fragment tree after each layout");
        print_option(
            "stacking-context-tree",
            "Log the stacking context tree after each layout",
        );
        print_option("scroll-tree", "Log the scroll tree after each layout");
        print_option("display-list", "Log the display list after each layout");
        print_option("style-stats", "Log style sharing cache statistics");
        print_option("relayout-event", "Log when relayout occurs");
        print_option("profile-script-events", "Log script event processing time");
        print_option("gc-profile", "Log garbage collection statistics");
        println!();

        process::exit(0);
    }

    /// Extend the current configuration with additional options.
    ///
    /// Parses the string and merges any enabled options into the current configuration.
    pub fn extend_from_string(&mut self, option_string: &str) -> Result<(), String> {
        for option in option_string.split(',') {
            let option = option.trim();
            match option {
                "help" => Self::print_debug_options_usage("servo"),
                "display-list" => self.display_list = true,
                "stacking-context-tree" => self.stacking_context_tree = true,
                "flow-tree" => self.flow_tree = true,
                "rule-tree" => self.rule_tree = true,
                "style-tree" => self.style_tree = true,
                "style-stats" => self.style_statistics = true,
                "scroll-tree" => self.scroll_tree = true,
                "gc-profile" => self.gc_profile = true,
                "profile-script-events" => self.profile_script_events = true,
                "relayout-event" => self.relayout_event = true,
                "" => {},
                _ => return Err(format!("Unknown diagnostic option: {option}")),
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
