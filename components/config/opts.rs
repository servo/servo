/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Options are global configuration options that are initialized once and cannot be changed at
//! runtime.

use core::str::FromStr;
use std::default::Default;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use strum::{
    AsRefStr, Display as EnumDisplay, EnumCount, EnumIter, EnumMessage, EnumString,
    IntoEnumIterator,
};

/// The set of global options supported by Servo. The values for these can be configured during
/// initialization of Servo and cannot be changed later at runtime.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Opts {
    /// `None` to disable the time profiler or `Some` to enable it with either:
    ///
    ///  - an interval in seconds to cause it to produce output on that interval.
    ///  - a file path to write profiling info to a TSV file upon Servo's termination.
    pub time_profiling: Option<OutputOptions>,

    /// When the profiler is enabled, this is an optional path to dump a self-contained HTML file
    /// visualizing the traces as a timeline.
    pub time_profiler_trace_path: Option<String>,

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

    /// Use temporary storage (data on disk will not persist across restarts).
    pub temporary_storage: bool,

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
}

/// The set of diagnostic options that can be enabled in Servo.
#[derive(
    AsRefStr,
    Copy,
    Clone,
    Debug,
    EnumCount,
    EnumDisplay,
    EnumIter,
    EnumMessage,
    EnumString,
    PartialEq,
)]
pub enum DiagnosticsLoggingOption {
    /// Log the DOM after each restyle
    #[strum(to_string = "style-tree")]
    StyleTree,

    /// Log the rule tree
    #[strum(to_string = "rule-tree")]
    RuleTree,

    /// Log the fragment tree after each layout
    #[strum(to_string = "flow-tree")]
    FlowTree,

    /// Log the stacking context tree after each layout
    #[strum(to_string = "stacking-context-tree")]
    StackingContextTree,

    /// Log the scroll tree (the hierarchy of scrollable areas) after each layout
    #[strum(to_string = "scroll-tree")]
    ScrollTree,

    /// Log the display list after each layout
    #[strum(to_string = "display-list")]
    DisplayList,

    /// Log notifications when a relayout occurs
    #[strum(to_string = "relayout-event")]
    RelayoutEvent,

    /// Periodically log on which events script threads spend their processing time
    #[strum(to_string = "profile-script-events")]
    ProfileScriptEvents,

    /// Log the the hit/miss statistics for the style sharing cache after each restyle
    #[strum(to_string = "style-stats")]
    StyleStatistics,

    /// Log garbage collection passes and their durations
    #[strum(to_string = "gc-profile")]
    GcProfile,

    /// Log Progressive Web Metrics
    #[strum(to_string = "progressive-web-metrics")]
    ProgressiveWebMetrics,
}

impl DiagnosticsLoggingOption {
    /// Returns a string representation of this variant that is compatible with
    /// [`FromStr::from_str`] and [`DiagnosticsLogging::extend_from_string`].
    /// This value can be used as a command-line argument for an application.
    pub fn help_option(&self) -> &str {
        self.as_ref()
    }

    /// Returns a string with a short description of the diagnostic option.
    /// This value can be used as a command-line argument description for an application.
    pub fn help_message(&self) -> &str {
        self.get_documentation()
            .expect("all variants of `DiagnosticsLoggingOption` should have a help message")
    }

    /// Returns an `Iterator` that can be used to iterate over all the diagnostic options
    /// supported by Servo. This is useful when constructing a help message that enumerates
    /// all possible diagnostic flags and their respective help messages.
    pub fn iter() -> impl Iterator<Item = Self> {
        <Self as IntoEnumIterator>::iter()
    }
}

/// The current configuration of the diagnostic options for Servo.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DiagnosticsLogging {
    options: [bool; DiagnosticsLoggingOption::COUNT],
}

impl DiagnosticsLogging {
    /// Create a new DiagnosticsLogging configuration.
    ///
    /// In builds with debug assertions enabled, this will automatically read and parse the
    /// SERVO_DIAGNOSTICS environment variable if it is set.
    pub fn new() -> Self {
        #[cfg(not(debug_assertions))]
        return DiagnosticsLogging::default();

        // Disabled for production and release builds
        #[cfg(debug_assertions)]
        {
            let mut config: DiagnosticsLogging = Default::default();
            if let Ok(diagnostics_var) = std::env::var("SERVO_DIAGNOSTICS") &&
                let Err(error) = config.extend_from_string(&diagnostics_var)
            {
                eprintln!("Could not parse debug logging option: {error}");
            };
            config
        }
    }

    /// Enables or disables the diagnostics represented by the given [`DiagnosticsLoggingOption`]
    /// variant.
    pub fn toggle_option(&mut self, option: DiagnosticsLoggingOption, enabled: bool) {
        self.options[option as usize] = enabled;
    }

    /// Returns true if the given diagnostic option is enabled.
    pub fn is_enabled(&self, option: DiagnosticsLoggingOption) -> bool {
        self.options[option as usize]
    }

    /// Extend the current configuration with additional options.
    ///
    /// Parses the string and merges any enabled options into the current configuration.
    pub fn extend_from_string(&mut self, option_string: &str) -> Result<(), String> {
        for option in option_string.split(',') {
            let option = option.trim();
            match DiagnosticsLoggingOption::from_str(option) {
                Ok(flag) => self.toggle_option(flag, true),
                Err(_) => return Err(format!("Unknown diagnostic option: {option}")),
            };
        }

        Ok(())
    }
}

/// The destination for the time profiler reports.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum OutputOptions {
    FileName(String),
    Stdout(f64),
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            time_profiling: None,
            time_profiler_trace_path: None,
            hard_fail: true,
            multiprocess: false,
            force_ipc: false,
            background_hang_monitor: false,
            random_pipeline_closure_probability: None,
            random_pipeline_closure_seed: None,
            sandbox: false,
            debug: Default::default(),
            config_dir: None,
            temporary_storage: false,
            shaders_path: None,
            certificate_path: None,
            ignore_certificate_errors: false,
            unminify_js: false,
            local_script_source: None,
            unminify_css: false,
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
/// Must be called before the first call to [`get`].
pub fn initialize_options(opts: Opts) {
    OPTIONS.set(opts).expect("Already initialized");
}

/// Get the servo options
///
/// If the servo options have not been initialized by calling [`initialize_options`], then the
/// options will be initialized to default values. Outside of tests the options should be
/// explicitly initialized.
#[inline]
pub fn get() -> &'static Opts {
    // In unit-tests using default options reduces boilerplate.
    // We can't use `cfg(test)` since that only is enabled when this crate
    // is compiled in test mode.
    // We rely on the `expect` in `initialize_options` to inform us if refactoring
    // causes a `get` call to move before `initialize_options`.
    OPTIONS.get_or_init(Default::default)
}

#[test]
fn test_parsing_of_diagnostics_logging_options() {
    assert!(DiagnosticsLoggingOption::iter().collect::<Vec<_>>().len() > 0);

    let mut diagnostics = DiagnosticsLogging::new();
    for option in DiagnosticsLoggingOption::iter() {
        assert_eq!(diagnostics.is_enabled(option), false);
    }

    assert!(
        diagnostics
            .extend_from_string("profile-script-events,style-stats")
            .is_ok()
    );
    assert!(diagnostics.is_enabled(DiagnosticsLoggingOption::ProfileScriptEvents));
    assert!(diagnostics.is_enabled(DiagnosticsLoggingOption::StyleStatistics));
    assert!(!diagnostics.is_enabled(DiagnosticsLoggingOption::ProgressiveWebMetrics));

    assert!(
        diagnostics
            .extend_from_string("profile-script-events,syle-stats")
            .is_err()
    );

    let mut diagnostics = DiagnosticsLogging::new();
    for option in DiagnosticsLoggingOption::iter() {
        assert_eq!(
            DiagnosticsLoggingOption::from_str(option.help_option()),
            Ok(option)
        );

        assert!(!diagnostics.is_enabled(option));
        assert!(diagnostics.extend_from_string(option.help_option()).is_ok());
        assert!(diagnostics.is_enabled(option),);
    }
}
