/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::panic;
use std::collections::HashMap;
use std::fs::{self, File, read_to_string};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
#[cfg(any(target_os = "android", target_env = "ohos"))]
use std::sync::OnceLock;
use std::{env, fmt, process};

use bpaf::*;
use euclid::Size2D;
use log::warn;
use serde_json::Value;
use servo::config::opts::{DebugOptions, Opts, OutputOptions};
use servo::config::prefs::{PrefValue, Preferences};
use servo::servo_geometry::DeviceIndependentPixel;
use servo::servo_url::ServoUrl;
use url::Url;

use crate::VERSION;

pub(crate) static EXPERIMENTAL_PREFS: &[&str] = &[
    "dom_async_clipboard_enabled",
    "dom_fontface_enabled",
    "dom_intersection_observer_enabled",
    "dom_mouse_event_which_enabled",
    "dom_navigator_sendbeacon_enabled",
    "dom_notification_enabled",
    "dom_offscreen_canvas_enabled",
    "dom_permissions_enabled",
    "dom_resize_observer_enabled",
    "dom_webgl2_enabled",
    "dom_webgpu_enabled",
    "dom_xpath_enabled",
    "layout_columns_enabled",
    "layout_container_queries_enabled",
    "layout_grid_enabled",
];

#[cfg_attr(any(target_os = "android", target_env = "ohos"), allow(dead_code))]
#[derive(Clone)]
pub(crate) struct ServoShellPreferences {
    /// A URL to load when starting servoshell.
    pub url: Option<String>,
    /// An override value for the device pixel ratio.
    pub device_pixel_ratio_override: Option<f32>,
    /// Whether or not to attempt clean shutdown.
    pub clean_shutdown: bool,
    /// Enable native window's titlebar and decorations.
    pub no_native_titlebar: bool,
    /// URL string of the homepage.
    pub homepage: String,
    /// URL string of the search engine page with '%s' standing in for the search term.
    /// For example <https://duckduckgo.com/html/?q=%s>.
    pub searchpage: String,
    /// Whether or not to run servoshell in headless mode. While running in headless
    /// mode, image output is supported.
    pub headless: bool,
    /// Filter directives for our tracing implementation.
    ///
    /// Overrides directives specified via `SERVO_TRACING` if set.
    /// See: <https://docs.rs/tracing-subscriber/0.3.19/tracing_subscriber/filter/struct.EnvFilter.html#directives>
    pub tracing_filter: Option<String>,
    /// The initial requested inner size of the window.
    pub initial_window_size: Size2D<u32, DeviceIndependentPixel>,
    /// An override for the screen resolution. This is useful for testing behavior on different screen sizes,
    /// such as the screen of a mobile device.
    pub screen_size_override: Option<Size2D<u32, DeviceIndependentPixel>>,
    /// Whether or not to simulate touch events using mouse events.
    pub simulate_touch_events: bool,
    /// If not-None, the path to a file to output the default WebView's rendered output
    /// after waiting for a stable image, this implies `Self::exit_after_load`.
    pub output_image_path: Option<String>,
    /// Whether or not to exit after Servo detects a stable output image in all WebViews.
    pub exit_after_stable_image: bool,
    /// Where to load userscripts from, if any.
    /// and if the option isn't passed userscripts won't be loaded.
    pub userscripts_directory: Option<PathBuf>,
    /// `None` to disable WebDriver or `Some` with a port number to start a server to listen to
    /// remote WebDriver commands.
    pub webdriver_port: Option<u16>,
    /// Whether the CLI option to enable experimental prefs was present at startup.
    pub experimental_prefs_enabled: bool,
    /// Log filter given in the `log_filter` spec as a String, if any.
    /// If a filter is passed, the logger should adjust accordingly.
    #[cfg(target_env = "ohos")]
    pub log_filter: Option<String>,
    /// Log also to a file
    #[cfg(target_env = "ohos")]
    pub log_to_file: bool,
}

impl Default for ServoShellPreferences {
    fn default() -> Self {
        Self {
            clean_shutdown: false,
            device_pixel_ratio_override: None,
            headless: false,
            homepage: "https://servo.org".into(),
            initial_window_size: Size2D::new(1024, 740),
            no_native_titlebar: true,
            screen_size_override: None,
            simulate_touch_events: false,
            searchpage: "https://duckduckgo.com/html/?q=%s".into(),
            tracing_filter: None,
            url: None,
            output_image_path: None,
            exit_after_stable_image: false,
            userscripts_directory: None,
            webdriver_port: None,
            #[cfg(target_env = "ohos")]
            log_filter: None,
            #[cfg(target_env = "ohos")]
            log_to_file: false,
            experimental_prefs_enabled: false,
        }
    }
}

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "android"),
    not(target_env = "ohos")
))]
pub fn default_config_dir() -> Option<PathBuf> {
    let mut config_dir = ::dirs::config_dir().unwrap();
    config_dir.push("servo");
    config_dir.push("default");
    Some(config_dir)
}

/// Overrides the default preference dir
#[cfg(any(target_os = "android", target_env = "ohos"))]
pub(crate) static DEFAULT_CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();
#[cfg(any(target_os = "android", target_env = "ohos"))]
pub fn default_config_dir() -> Option<PathBuf> {
    DEFAULT_CONFIG_DIR.get().cloned()
}

#[cfg(target_os = "macos")]
pub fn default_config_dir() -> Option<PathBuf> {
    // FIXME: use `config_dir()` ($HOME/Library/Preferences)
    // instead of `data_dir()` ($HOME/Library/Application Support) ?
    let mut config_dir = ::dirs::data_dir().unwrap();
    config_dir.push("Servo");
    Some(config_dir)
}

#[cfg(target_os = "windows")]
pub fn default_config_dir() -> Option<PathBuf> {
    let mut config_dir = ::dirs::config_dir().unwrap();
    config_dir.push("Servo");
    Some(config_dir)
}

/// Get a Servo [`Preferences`] to use when initializing Servo by first reading the user
/// preferences file and then overriding these preferences with the ones from the `--prefs-file`
/// command-line argument, if given.
fn get_preferences(prefs_files: &[PathBuf], config_dir: &Option<PathBuf>) -> Preferences {
    // Do not read any preferences files from the disk when testing as we do not want it
    // to throw off test results.
    if cfg!(test) {
        return Preferences::default();
    }

    let user_prefs_path = config_dir
        .clone()
        .map(|path| path.join("prefs.json"))
        .filter(|path| path.exists());
    let user_prefs_hash = user_prefs_path.map(read_prefs_file).unwrap_or_default();

    let apply_preferences =
        |preferences: &mut Preferences, preferences_hash: HashMap<String, PrefValue>| {
            for (key, value) in preferences_hash.iter() {
                preferences.set_value(key, value.clone());
            }
        };

    let mut preferences = Preferences::default();
    apply_preferences(&mut preferences, user_prefs_hash);
    for pref_file_path in prefs_files.iter() {
        apply_preferences(&mut preferences, read_prefs_file(pref_file_path))
    }

    preferences
}

fn read_prefs_file<P: AsRef<Path>>(path: P) -> HashMap<String, PrefValue> {
    read_prefs_map(&read_to_string(path).expect("Error opening user prefs"))
}

pub fn read_prefs_map(txt: &str) -> HashMap<String, PrefValue> {
    let prefs: HashMap<String, Value> = serde_json::from_str(txt)
        .map_err(|_| panic!("Could not parse preferences JSON"))
        .unwrap();
    prefs
        .into_iter()
        .map(|(key, value)| {
            let value = (&value)
                .try_into()
                .map_err(|error| panic!("{error}"))
                .unwrap();
            (key, value)
        })
        .collect()
}

#[allow(clippy::large_enum_variant)]
#[cfg_attr(any(target_os = "android", target_env = "ohos"), allow(dead_code))]
pub(crate) enum ArgumentParsingResult {
    ChromeProcess(Opts, Preferences, ServoShellPreferences),
    ContentProcess(String),
    Exit,
    ErrorParsing,
}

enum ParseResolutionError {
    InvalidFormat,
    ZeroDimension,
    ParseError(std::num::ParseIntError),
}

impl fmt::Display for ParseResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseResolutionError::InvalidFormat => write!(f, "invalid resolution format"),
            ParseResolutionError::ZeroDimension => {
                write!(f, "width and height must be greater than 0")
            },
            ParseResolutionError::ParseError(e) => write!(f, "{e}"),
        }
    }
}

/// Parse a resolution string into a Size2D.
fn parse_resolution_string(
    string: String,
) -> Result<Option<Size2D<u32, DeviceIndependentPixel>>, ParseResolutionError> {
    if string.is_empty() {
        Ok(None)
    } else {
        let (width, height) = string
            .split_once(['x', 'X'])
            .ok_or(ParseResolutionError::InvalidFormat)?;

        let width = width.trim();
        let height = height.trim();
        if width.is_empty() || height.is_empty() {
            return Err(ParseResolutionError::InvalidFormat);
        }

        let width = width.parse().map_err(ParseResolutionError::ParseError)?;
        let height = height.parse().map_err(ParseResolutionError::ParseError)?;
        if width == 0 || height == 0 {
            return Err(ParseResolutionError::ZeroDimension);
        }

        Ok(Some(Size2D::new(width, height)))
    }
}

/// Parse stylesheets into the byte stream.
fn parse_user_stylesheets(string: String) -> Result<Vec<(Vec<u8>, ServoUrl)>, std::io::Error> {
    Ok(string
        .split_whitespace()
        .map(|filename| {
            let cwd = env::current_dir().unwrap();
            let path = cwd.join(filename);
            let url = ServoUrl::from_url(Url::from_file_path(&path).unwrap());
            let mut contents = Vec::new();
            File::open(path)
                .unwrap()
                .read_to_end(&mut contents)
                .unwrap();
            (contents, url)
        })
        .collect())
}

/// This is a helper function that fulfills the following parsing task
/// check for long/short cmd. If there is the flag with this
/// If the flag is not there, parse `None``
/// If the flag is there but no argument, parse `Some(default)`
/// If the flag is there and an argument parse the arugment
fn flag_with_default_parser<S, T>(
    short_cmd: Option<char>,
    long_cmd: &'static str,
    argument_help: &'static str,
    help: &'static str,
    default: T,
    transform: fn(S) -> T,
) -> impl Parser<Option<T>>
where
    S: FromStr + 'static,
    <S as FromStr>::Err: fmt::Display,
    T: Clone + 'static,
{
    let just_flag = if let Some(c) = short_cmd {
        short(c).long(long_cmd)
    } else {
        long(long_cmd)
    }
    .req_flag(default)
    .hide();

    let arg = if let Some(c) = short_cmd {
        short(c).long(long_cmd)
    } else {
        long(long_cmd)
    }
    .argument::<S>(argument_help)
    .help(help)
    .map(transform);

    construct!([arg, just_flag]).optional()
}

fn profile() -> impl Parser<Option<OutputOptions>> {
    flag_with_default_parser(
        Some('p'),
        "profile",
        "",
        "uses 5.0 output as standard if no argument supplied",
        OutputOptions::Stdout(5.0),
        |val: String| {
            if let Ok(float) = val.parse::<f64>() {
                OutputOptions::Stdout(float)
            } else {
                OutputOptions::FileName(val)
            }
        },
    )
}

fn userscripts() -> impl Parser<Option<PathBuf>> {
    flag_with_default_parser(
        None,
        "userscripts_directory",
        "your/directory",
        "Uses userscripts in resources/user-agent-js, or a specified full path",
        PathBuf::from("resources/user-agent-js"),
        |val: String| PathBuf::from(val),
    )
}

fn webdriver_port() -> impl Parser<Option<u16>> {
    flag_with_default_parser(
        None,
        "webdriver",
        "7000",
        "Start remote WebDriver server on port",
        7000,
        |val| val,
    )
}

fn map_debug_options(arg: String) -> Vec<String> {
    arg.split(',').map(|s| s.to_owned()).collect()
}

#[derive(Bpaf, Clone, Debug)]
#[bpaf(options, version(VERSION))]
struct CmdArgs {
    /// Background Hang Monitor enabled.
    #[bpaf(short('B'), long("bhm"))]
    background_hang_monitor: bool,

    /// Path to find SSL certificates.
    #[bpaf(argument("/home/servo/resources/certs"))]
    certificate_path: Option<PathBuf>,

    /// Do not shutdown until all threads have finished (macos only).
    #[bpaf(long)]
    clean_shutdown: bool,

    /// Config directory following xdg spec on linux platform.
    #[bpaf(argument("~/.config/servo"))]
    config_dir: Option<PathBuf>,

    /// Run as a content process and connect to the given pipe.
    #[bpaf(argument("servo-ipc-channel.abcdefg"))]
    content_process: Option<String>,

    /// A comma-separated string of debug options. Pass help to show available options.
    #[bpaf(
        short('Z'),
        argument("layout_grid_enabled=true,dom_async_clipboard_enabled"),
        long,
        map(map_debug_options),
        fallback(vec![])
    )]
    debug: Vec<String>,

    /// Device pixels per px.
    #[bpaf(argument("1.0"))]
    device_pixel_ratio: Option<f32>,

    /// Start remote devtools server on port.
    #[bpaf(argument("0"))]
    devtools: Option<u16>,

    /// Whether or not to enable experimental web platform features.
    #[bpaf(long)]
    enable_experimental_web_platform_features: bool,

    // Exit after Servo has loaded the page and detected a stable output image.
    #[bpaf(short('x'), long)]
    exit: bool,

    // Use ipc_channel in singleprocess mode.
    #[bpaf(short('I'), long("force-ipc"))]
    force_ipc: bool,

    /// Exit on thread failure instead of displaying about:failure.
    #[bpaf(short('f'), long)]
    hard_fail: bool,

    /// Headless mode.
    #[bpaf(short('z'), long)]
    headless: bool,

    /// Whether or not to completely ignore certificate errors.
    #[bpaf(long)]
    ignore_certificate_errors: bool,

    /// Number of threads to use for layout.
    #[bpaf(short('y'), long, argument("1"))]
    layout_threads: Option<i64>,

    /// Directory root with unminified scripts.
    #[bpaf(argument("~/.local/share/servo"))]
    local_script_source: Option<PathBuf>,

    #[cfg(target_env = "ohos")]
    /// Define a custom filter for logging.
    #[bpaf(argument("FILTER"))]
    log_filter: Option<String>,

    #[cfg(target_env = "ohos")]
    /// Also log to a file (/data/app/el2/100/base/org.servo.servo/cache/servo.log).
    #[bpaf(long)]
    log_to_file: bool,

    /// Run in multiprocess mode.
    #[bpaf(short('M'), long)]
    multiprocess: bool,

    /// Do not use native titlebar.
    #[bpaf(short('b'), long)]
    no_native_titlebar: bool,

    /// Enable to turn off incremental layout.
    #[bpaf(short('i'), long, flag(false, true))]
    nonincremental_layout: bool,

    /// Path to an output image. The format of the image is determined by the extension.
    /// Supports all formats that `rust-image` does.
    #[bpaf(short('o'), argument("test.png"), long)]
    output: Option<PathBuf>,

    /// Time profiler flag and either a TSV output filename
    /// OR an interval for output to Stdout (blank for Stdout with interval of 5s).
    #[bpaf(external)]
    profile: Option<OutputOptions>,

    /// Path to dump a self-contained HTML timeline of profiler traces.
    #[bpaf(argument("trace.html"), long)]
    profiler_trace_path: Option<PathBuf>,

    /// A preference to set.
    #[bpaf(argument("dom_bluetooth_enabled"), many)]
    pref: Vec<String>,

    /// Load in additional prefs from a file.
    #[bpaf(long, argument("/path/to/prefs.json"), many)]
    prefs_file: Vec<PathBuf>,

    /// Print Progressive Web Metrics.
    #[bpaf(long)]
    print_pwm: bool,

    /// Probability of randomly closing a pipeline (for testing constellation hardening).
    #[bpaf(argument("0.25"))]
    random_pipeline_closure_probability: Option<f32>,

    /// A fixed seed for repeatbility of random pipeline closure.
    random_pipeline_closure_seed: Option<usize>,

    /// Run in a sandbox if multiprocess.
    #[bpaf(short('S'), long)]
    sandbox: bool,

    /// Shaders will be loaded from the specified directory instead of using the builtin ones.
    shaders: Option<PathBuf>,

    /// Override the screen resolution in logical (device independent) pixels.
    #[bpaf(long("screen-size"), argument::<String>("1024x768"),
        parse(parse_resolution_string), fallback(None))]
    screen_size_override: Option<Size2D<u32, DeviceIndependentPixel>>,

    /// Use mouse events to simulate touch events. Left button presses will be converted to touch
    /// and mouse movements while the left button is pressed will be converted to touch movements.
    #[bpaf(long("simulate-touch-events"))]
    simulate_touch_events: bool,

    /// Define a custom filter for traces. Overrides `SERVO_TRACING` if set.
    #[bpaf(long("tracing-filter"), argument("FILTER"))]
    tracing_filter: Option<String>,

    /// Unminify Javascript.
    #[bpaf(long)]
    unminify_js: bool,

    /// Unminify Css.
    #[bpaf(long)]
    unminify_css: bool,

    /// Set custom user agent string (or ios / android / desktop for platform default).
    #[bpaf(short('u'),long,argument::<String>("NCSA mosaic/1.0 (X11;SunOS 4.1.4 sun4m"))]
    user_agent: Option<String>,

    /// Uses userscripts in resources/user-agent-js, or a specified full path.
    #[bpaf(external)]
    userscripts: Option<PathBuf>,

    /// A user stylesheet to be added to every document.
    #[bpaf(argument::<String>("file.css"), parse(parse_user_stylesheets), fallback(vec![]))]
    user_stylesheet: Vec<(Vec<u8>, ServoUrl)>,

    /// Start remote WebDriver server on port.
    #[bpaf(external)]
    webdriver_port: Option<u16>,

    /// Set the initial window size in logical (device independent) pixels.
    #[bpaf(argument::<String>("1024x740"), parse(parse_resolution_string), fallback(None))]
    window_size: Option<Size2D<u32, DeviceIndependentPixel>>,

    /// The url we should load.
    #[bpaf(positional("URL"), fallback(String::from("https://www.servo.org")))]
    url: String,
}

fn update_preferences_from_command_line_arguemnts(
    preferences: &mut Preferences,
    cmd_args: &CmdArgs,
) {
    if let Some(port) = cmd_args.devtools {
        preferences.devtools_server_enabled = true;
        preferences.devtools_server_port = port as i64;
    }

    if cmd_args.enable_experimental_web_platform_features {
        for pref in EXPERIMENTAL_PREFS {
            preferences.set_value(pref, PrefValue::Bool(true));
        }
    }

    for pref in &cmd_args.pref {
        let split: Vec<&str> = pref.splitn(2, '=').collect();
        let pref_name = split[0];
        let pref_value = PrefValue::from_booleanish_str(split.get(1).copied().unwrap_or("true"));
        preferences.set_value(pref_name, pref_value);
    }

    if let Some(layout_threads) = cmd_args.layout_threads {
        preferences.layout_threads = layout_threads;
    }

    if cmd_args.headless && preferences.media_glvideo_enabled {
        warn!("GL video rendering is not supported on headless windows.");
        preferences.media_glvideo_enabled = false;
    }

    if let Some(user_agent) = cmd_args.user_agent.clone() {
        preferences.user_agent = user_agent;
    }

    if cmd_args.webdriver_port.is_some() {
        preferences.dom_testing_html_input_element_select_files_enabled = true;
    }
}

pub(crate) fn parse_command_line_arguments(args: Vec<String>) -> ArgumentParsingResult {
    // Remove the binary name from the list of arguments.
    let args_without_binary = args
        .split_first()
        .expect("Expectd executable name and and arguments")
        .1;
    let cmd_args = cmd_args().run_inner(Args::from(args_without_binary));
    if let Err(error) = cmd_args {
        error.print_message(80);
        return if error.exit_code() == 0 {
            ArgumentParsingResult::Exit
        } else {
            ArgumentParsingResult::ErrorParsing
        };
    }

    let cmd_args = cmd_args.unwrap();

    if cmd_args
        .debug
        .iter()
        .any(|debug_option| debug_option.contains("help"))
    {
        print_debug_options_usage("servo");
        return ArgumentParsingResult::Exit;
    }

    // If this is the content process, we'll receive the real options over IPC. So fill in some dummy options for now.
    if let Some(content_process) = cmd_args.content_process {
        return ArgumentParsingResult::ContentProcess(content_process);
    }

    let config_dir = cmd_args
        .config_dir
        .clone()
        .or_else(default_config_dir)
        .inspect(|config_dir| {
            if !config_dir.exists() {
                fs::create_dir_all(config_dir).expect("Could not create config_dir");
            }
        });
    if let Some(ref time_profiler_trace_path) = cmd_args.profiler_trace_path {
        let mut path = PathBuf::from(time_profiler_trace_path);
        path.pop();
        fs::create_dir_all(&path).expect("Error in creating profiler trace path");
    }

    let mut preferences = get_preferences(&cmd_args.prefs_file, &config_dir);

    update_preferences_from_command_line_arguemnts(&mut preferences, &cmd_args);

    // FIXME: enable JIT compilation on 32-bit Android after the startup crash issue (#31134) is fixed.
    if cfg!(target_os = "android") && cfg!(target_pointer_width = "32") {
        preferences.js_baseline_interpreter_enabled = false;
        preferences.js_baseline_jit_enabled = false;
        preferences.js_ion_enabled = false;
    }

    // Make sure the default window size is not larger than any provided screen size.
    let default_window_size = Size2D::new(1024, 740);
    let default_window_size = cmd_args
        .screen_size_override
        .map_or(default_window_size, |screen_size_override| {
            default_window_size.min(screen_size_override)
        });

    let servoshell_preferences = ServoShellPreferences {
        url: Some(cmd_args.url),
        no_native_titlebar: cmd_args.no_native_titlebar,
        device_pixel_ratio_override: cmd_args.device_pixel_ratio,
        clean_shutdown: cmd_args.clean_shutdown,
        headless: cmd_args.headless,
        tracing_filter: cmd_args.tracing_filter,
        initial_window_size: cmd_args.window_size.unwrap_or(default_window_size),
        screen_size_override: cmd_args.screen_size_override,
        simulate_touch_events: cmd_args.simulate_touch_events,
        webdriver_port: cmd_args.webdriver_port,
        output_image_path: cmd_args.output.map(|p| p.to_string_lossy().into_owned()),
        exit_after_stable_image: cmd_args.exit,
        userscripts_directory: cmd_args.userscripts,
        experimental_prefs_enabled: cmd_args.enable_experimental_web_platform_features,
        #[cfg(target_env = "ohos")]
        log_filter: cmd_args.log_filter.or_else(|| {
            (!preferences.log_filter.is_empty()).then(|| preferences.log_filter.clone())
        }),
        #[cfg(target_env = "ohos")]
        log_to_file: cmd_args.log_to_file,
        ..Default::default()
    };

    let mut debug_options = DebugOptions::default();
    for debug_string in cmd_args.debug {
        let result = debug_options.extend(debug_string);
        if let Err(error) = result {
            println!("error: urnecognized debug option: {}", error);
            return ArgumentParsingResult::ErrorParsing;
        }
    }

    let opts = Opts {
        debug: debug_options,
        time_profiling: cmd_args.profile,
        time_profiler_trace_path: cmd_args
            .profiler_trace_path
            .map(|p| p.to_string_lossy().into_owned()),
        nonincremental_layout: cmd_args.nonincremental_layout,
        user_stylesheets: cmd_args.user_stylesheet,
        hard_fail: cmd_args.hard_fail,
        multiprocess: cmd_args.multiprocess,
        background_hang_monitor: cmd_args.background_hang_monitor,
        sandbox: cmd_args.sandbox,
        random_pipeline_closure_probability: cmd_args.random_pipeline_closure_probability,
        random_pipeline_closure_seed: cmd_args.random_pipeline_closure_seed,
        config_dir: config_dir.clone(),
        shaders_dir: cmd_args.shaders,
        certificate_path: cmd_args
            .certificate_path
            .map(|p| p.to_string_lossy().into_owned()),
        ignore_certificate_errors: cmd_args.ignore_certificate_errors,
        unminify_js: cmd_args.unminify_js,
        local_script_source: cmd_args
            .local_script_source
            .map(|p| p.to_string_lossy().into_owned()),
        unminify_css: cmd_args.unminify_css,
        print_pwm: cmd_args.print_pwm,
        force_ipc: cmd_args.force_ipc,
    };

    ArgumentParsingResult::ChromeProcess(opts, preferences, servoshell_preferences)
}

fn print_debug_options_usage(app: &str) {
    fn print_option(name: &str, description: &str) {
        println!("\t{:<35} {}", name, description);
    }

    println!(
        "Usage: {} debug option,[options,...]\n\twhere options include\n\nOptions:",
        app
    );
    print_option(
        "disable-share-style-cache",
        "Disable the style sharing cache.",
    );
    print_option(
        "dump-stacking-context-tree",
        "Print the stacking context tree after each layout.",
    );
    print_option(
        "dump-display-list",
        "Print the display list after each layout.",
    );
    print_option(
        "dump-flow-tree",
        "Print the fragment tree after each layout.",
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
    print_option("dump-scroll-tree", "Print scroll tree after each layout.");
    print_option("gc-profile", "Log GC passes and their durations.");
    print_option(
        "profile-script-events",
        "Enable profiling of script-related events.",
    );
    print_option(
        "relayout-event",
        "Print notifications when there is a relayout.",
    );

    println!();

    process::exit(0)
}

#[cfg(test)]
fn test_parse_pref(arg: &str) -> Preferences {
    let args = vec!["servo".to_string(), "--pref".to_string(), arg.to_string()];
    match parse_command_line_arguments(args) {
        ArgumentParsingResult::ContentProcess(..) => {
            unreachable!("No preferences for content process")
        },
        ArgumentParsingResult::ChromeProcess(_, preferences, _) => preferences,
        ArgumentParsingResult::Exit => {
            panic!("we supplied a --pref argument above which should be parsed")
        },
        ArgumentParsingResult::ErrorParsing => {
            unreachable!("we supplied a --pref argument above which should be parsed")
        },
    }
}

#[test]
fn test_parse_pref_from_command_line() {
    // Test with boolean values.
    let preferences = test_parse_pref("dom_bluetooth_enabled=true");
    assert!(preferences.dom_bluetooth_enabled);

    let preferences = test_parse_pref("dom_bluetooth_enabled=false");
    assert!(!preferences.dom_bluetooth_enabled);

    // Test with numbers
    let preferences = test_parse_pref("layout_threads=42");
    assert_eq!(preferences.layout_threads, 42);

    // Test string.
    let preferences = test_parse_pref("fonts_default=Lucida");
    assert_eq!(preferences.fonts_default, "Lucida");

    // Test with no value (defaults to true).
    let preferences = test_parse_pref("dom_bluetooth_enabled");
    assert!(preferences.dom_bluetooth_enabled);
}

#[test]
fn test_invalid_prefs_from_command_line_panics() {
    let err_msg = std::panic::catch_unwind(|| {
        test_parse_pref("doesntexist=true");
    })
    .err()
    .and_then(|a| a.downcast_ref::<String>().cloned())
    .expect("Should panic");
    assert_eq!(
        err_msg, "Unknown preference: \"doesntexist\"",
        "Message should describe the problem"
    )
}

#[test]
fn test_create_prefs_map() {
    let json_str = "{
        \"layout.writing-mode.enabled\": true,
        \"network.mime.sniff\": false,
        \"shell.homepage\": \"https://servo.org\"
    }";
    assert_eq!(read_prefs_map(json_str).len(), 3);
}

#[cfg(test)]
fn test_parse(arg: &str) -> (Opts, Preferences, ServoShellPreferences) {
    let mut args = vec!["servo".to_string()];
    // bpaf requires the arguments that are separated by whitespace to be different elements of the vector.
    let mut args_split = arg.split_whitespace().map(|s| s.to_owned()).collect();
    args.append(&mut args_split);
    match parse_command_line_arguments(args) {
        ArgumentParsingResult::ContentProcess(..) => {
            unreachable!("No preferences for content process")
        },
        ArgumentParsingResult::ChromeProcess(opts, preferences, servoshell_preferences) => {
            (opts, preferences, servoshell_preferences)
        },
        ArgumentParsingResult::Exit | ArgumentParsingResult::ErrorParsing => {
            unreachable!("We always have valid preference in our test cases")
        },
    }
}

#[test]
fn test_profiling_args() {
    assert_eq!(
        test_parse("-p").0.time_profiling.unwrap(),
        OutputOptions::Stdout(5_f64)
    );

    assert_eq!(
        test_parse("-p 10").0.time_profiling.unwrap(),
        OutputOptions::Stdout(10_f64)
    );

    assert_eq!(
        test_parse("-p 10.0").0.time_profiling.unwrap(),
        OutputOptions::Stdout(10_f64)
    );

    assert_eq!(
        test_parse("-p foo.txt").0.time_profiling.unwrap(),
        OutputOptions::FileName(String::from("foo.txt"))
    );
}

#[test]
fn test_servoshell_cmd() {
    assert_eq!(
        test_parse("--screen-size=1000x1000")
            .2
            .screen_size_override
            .unwrap(),
        Size2D::new(1000, 1000)
    );

    assert_eq!(
        test_parse("--certificate-path=/tmp/test")
            .0
            .certificate_path
            .unwrap(),
        String::from("/tmp/test")
    );
}
