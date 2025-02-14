/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

use euclid::Size2D;
use getopts::{Matches, Options};
use log::{error, warn};
use serde_json::Value;
use servo::config::opts::{DebugOptions, Opts, OutputOptions};
use servo::config::prefs::{PrefValue, Preferences};
use servo::servo_geometry::DeviceIndependentPixel;
use servo::servo_url::ServoUrl;
use url::Url;

#[cfg_attr(any(target_os = "android", target_env = "ohos"), allow(dead_code))]
pub(crate) struct ServoShellPreferences {
    /// The user agent to use for servoshell.
    pub user_agent: Option<String>,
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
    /// The initial requested size of the window.
    pub initial_window_size: Size2D<u32, DeviceIndependentPixel>,
    /// An override for the screen resolution. This is useful for testing behavior on different screen sizes,
    /// such as the screen of a mobile device.
    pub screen_size_override: Option<Size2D<u32, DeviceIndependentPixel>>,
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
            searchpage: "https://duckduckgo.com/html/?q=%s".into(),
            tracing_filter: None,
            url: None,
            user_agent: None,
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

#[cfg(any(target_os = "android", target_env = "ohos"))]
pub fn default_config_dir() -> Option<PathBuf> {
    None
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
fn get_preferences(opts_matches: &Matches, config_dir: &Option<PathBuf>) -> Preferences {
    // Do not read any preferences files from the disk when testing as we do not want it
    // to throw off test results.
    if cfg!(test) {
        return Preferences::default();
    }

    let user_prefs_path = config_dir
        .clone()
        .or_else(default_config_dir)
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
    for pref_file_path in opts_matches.opt_strs("prefs-file").iter() {
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
}

pub(crate) fn parse_command_line_arguments(args: Vec<String>) -> ArgumentParsingResult {
    let (app_name, args) = args.split_first().unwrap();

    let mut opts = Options::new();
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
    opts.optopt(
        "",
        "window-size",
        "Set the initial window size in logical (device independenrt) pixels",
        "1024x740",
    );
    opts.optopt(
        "",
        "screen-size",
        "Override the screen resolution in logical (device independent) pixels",
        "1024x768",
    );
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
    opts.optflag("", "unminify-css", "Unminify Css");

    opts.optflag(
        "",
        "clean-shutdown",
        "Do not shutdown until all threads have finished (macos only)",
    );
    opts.optflag("b", "no-native-titlebar", "Do not use native titlebar");
    opts.optopt("", "device-pixel-ratio", "Device pixels per px", "");
    opts.optopt(
        "u",
        "user-agent",
        "Set custom user agent string (or ios / android / desktop for platform default)",
        "NCSA Mosaic/1.0 (X11;SunOS 4.1.4 sun4m)",
    );
    opts.optmulti(
        "",
        "tracing-filter",
        "Define a custom filter for traces. Overrides `SERVO_TRACING` if set.",
        "FILTER",
    );

    opts.optmulti(
        "",
        "pref",
        "A preference to set to enable",
        "dom.bluetooth.enabled",
    );
    opts.optmulti(
        "",
        "pref",
        "A preference to set to disable",
        "dom_webgpu_enabled=false",
    );
    opts.optmulti(
        "",
        "prefs-file",
        "Load in additional prefs from a file.",
        "--prefs-file /path/to/prefs.json",
    );

    let opt_match = match opts.parse(args) {
        Ok(m) => m,
        Err(f) => args_fail(&f.to_string()),
    };

    if opt_match.opt_present("v") || opt_match.opt_present("version") {
        println!("{}", crate::servo_version());
        process::exit(0);
    }

    if opt_match.opt_present("h") || opt_match.opt_present("help") {
        print_usage(app_name, &opts);
        process::exit(0);
    };

    let config_dir = opt_match.opt_str("config-dir").map(Into::into);
    let mut preferences = get_preferences(&opt_match, &config_dir);

    // If this is the content process, we'll receive the real options over IPC. So just fill in
    // some dummy options for now.
    if let Some(content_process) = opt_match.opt_str("content-process") {
        return ArgumentParsingResult::ContentProcess(content_process);
    }
    // Env-Filter directives are comma seperated.
    let filters = opt_match.opt_strs("tracing-filter").join(",");
    let tracing_filter = if filters.is_empty() {
        None
    } else {
        Some(filters)
    };

    let mut debug_options = DebugOptions::default();
    for debug_string in opt_match.opt_strs("Z") {
        if let Err(e) = debug_options.extend(debug_string) {
            args_fail(&format!("error: unrecognized debug option: {}", e));
        }
    }

    if debug_options.help {
        print_debug_options_usage(app_name);
    }

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

    let layout_threads: Option<usize> = opt_match.opt_str("y").map(|layout_threads_str| {
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

    if opt_match.opt_present("devtools") {
        let port = opt_match
            .opt_str("devtools")
            .map(|port| {
                port.parse().unwrap_or_else(|err| {
                    args_fail(&format!("Error parsing option: --devtools ({})", err))
                })
            })
            .unwrap_or(preferences.devtools_server_port);
        preferences.devtools_server_enabled = true;
        preferences.devtools_server_port = port;
    }

    let webdriver_port = opt_match.opt_default("webdriver", "7000").map(|port| {
        port.parse().unwrap_or_else(|err| {
            args_fail(&format!("Error parsing option: --webdriver ({})", err))
        })
    });

    let parse_resolution_string = |string: String| {
        let components: Vec<u32> = string
            .split('x')
            .map(|component| {
                component.parse().unwrap_or_else(|error| {
                    args_fail(&format!("Error parsing resolution '{string}': {error}"));
                })
            })
            .collect();
        Size2D::new(components[0], components[1])
    };

    let screen_size_override = opt_match
        .opt_str("screen-size")
        .map(parse_resolution_string);

    // Make sure the default window size is not larger than any provided screen size.
    let default_window_size = Size2D::new(1024, 740);
    let default_window_size = screen_size_override
        .map_or(default_window_size, |screen_size_override| {
            default_window_size.min(screen_size_override)
        });
    let initial_window_size = opt_match
        .opt_str("window-size")
        .map_or(default_window_size, parse_resolution_string);

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

    // Handle all command-line preferences overrides.
    for pref in opt_match.opt_strs("pref") {
        let split: Vec<&str> = pref.splitn(2, '=').collect();
        let pref_name = split[0];
        let pref_value = PrefValue::from_booleanish_str(split.get(1).copied().unwrap_or("true"));
        preferences.set_value(pref_name, pref_value);
    }

    let legacy_layout = opt_match.opt_present("legacy-layout");
    if legacy_layout {
        preferences.layout_legacy_layout = true;
    }

    if let Some(layout_threads) = layout_threads {
        preferences.layout_threads = layout_threads as i64;
    }

    let no_native_titlebar = opt_match.opt_present("no-native-titlebar");
    let mut device_pixel_ratio_override = opt_match.opt_str("device-pixel-ratio").map(|dppx_str| {
        dppx_str.parse().unwrap_or_else(|err| {
            error!("Error parsing option: --device-pixel-ratio ({})", err);
            process::exit(1);
        })
    });

    // If an output file is specified the device pixel ratio is always 1.
    let output_file = opt_match.opt_str("o");
    if output_file.is_some() {
        device_pixel_ratio_override = Some(1.0);
    }

    let url = if !opt_match.free.is_empty() {
        Some(opt_match.free[0][..].into())
    } else {
        None
    };

    // FIXME: enable JIT compilation on 32-bit Android after the startup crash issue (#31134) is fixed.
    if cfg!(target_os = "android") && cfg!(target_pointer_width = "32") {
        preferences.js_baseline_interpreter_enabled = false;
        preferences.js_baseline_jit_enabled = false;
        preferences.js_ion_enabled = false;
    }

    let servoshell_preferences = ServoShellPreferences {
        user_agent: opt_match.opt_str("u"),
        url,
        no_native_titlebar,
        device_pixel_ratio_override,
        clean_shutdown: opt_match.opt_present("clean-shutdown"),
        headless: opt_match.opt_present("z"),
        tracing_filter,
        initial_window_size,
        screen_size_override,
        ..Default::default()
    };

    if servoshell_preferences.headless && preferences.media_glvideo_enabled {
        warn!("GL video rendering is not supported on headless windows.");
        preferences.media_glvideo_enabled = false;
    }

    let opts = Opts {
        debug: debug_options.clone(),
        legacy_layout,
        time_profiling,
        time_profiler_trace_path: opt_match.opt_str("profiler-trace-path"),
        mem_profiler_period,
        nonincremental_layout,
        userscripts: opt_match.opt_default("userscripts", ""),
        user_stylesheets,
        output_file,
        hard_fail: opt_match.opt_present("f") && !opt_match.opt_present("F"),
        webdriver_port,
        multiprocess: opt_match.opt_present("M"),
        background_hang_monitor: opt_match.opt_present("B"),
        sandbox: opt_match.opt_present("S"),
        random_pipeline_closure_probability,
        random_pipeline_closure_seed,
        exit_after_load: opt_match.opt_present("x"),
        config_dir,
        shaders_dir: opt_match.opt_str("shaders").map(Into::into),
        certificate_path: opt_match.opt_str("certificate-path"),
        ignore_certificate_errors: opt_match.opt_present("ignore-certificate-errors"),
        unminify_js: opt_match.opt_present("unminify-js"),
        local_script_source: opt_match.opt_str("local-script-source"),
        unminify_css: opt_match.opt_present("unminify-css"),
        print_pwm: opt_match.opt_present("print-pwm"),
    };

    ArgumentParsingResult::ChromeProcess(opts, preferences, servoshell_preferences)
}

fn args_fail(msg: &str) -> ! {
    eprintln!("{}", msg);
    process::exit(1)
}

fn print_usage(app: &str, opts: &Options) {
    let message = format!(
        "Usage: {} [ options ... ] [URL]\n\twhere options include",
        app
    );
    println!("{}", opts.usage(&message));
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
        "bubble-inline-sizes-separately",
        "Bubble intrinsic widths separately like other engines.",
    );
    print_option(
        "convert-mouse-to-touch",
        "Send touch events instead of mouse events",
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
        "parallel-display-list-building",
        "Build display lists in parallel.",
    );
    print_option(
        "profile-script-events",
        "Enable profiling of script-related events.",
    );
    print_option(
        "relayout-event",
        "Print notifications when there is a relayout.",
    );
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

#[cfg(test)]
fn test_parse_pref(arg: &str) -> Preferences {
    let args = vec!["servo".to_string(), "--pref".to_string(), arg.to_string()];
    match parse_command_line_arguments(args) {
        ArgumentParsingResult::ContentProcess(..) => {
            unreachable!("No preferences for content process")
        },
        ArgumentParsingResult::ChromeProcess(_, preferences, _) => preferences,
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
