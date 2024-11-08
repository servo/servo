/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::sync::LazyLock;

use embedder_traits::resources::{self, Resource};
use gen::Prefs;
use log::warn;
use serde_json::{self, Value};

use crate::pref_util::Preferences;
pub use crate::pref_util::{PrefError, PrefValue};

static PREFS: LazyLock<Preferences<'static, Prefs>> = LazyLock::new(|| {
    let def_prefs: Prefs = serde_json::from_str(&resources::read_string(Resource::Preferences))
        .unwrap_or_else(|e| {
            panic!("Failed to get Preference json file: {e}");
        });
    let result = Preferences::new(def_prefs, &gen::PREF_ACCESSORS);
    for (key, value) in result.iter() {
        set_stylo_pref_ref(&key, &value);
    }
    result
});

/// A convenience macro for accessing a preference value using its static path.
/// Passing an invalid path is a compile-time error.
#[macro_export]
macro_rules! pref {
    ($($segment: ident).+) => {{
        let values = $crate::prefs::pref_map().values();
        let lock = values.read()
            .map(|prefs| prefs $(.$segment)+.clone());
        lock.unwrap()
    }};
}

/// A convenience macro for updating a preference value using its static path.
/// Passing an invalid path is a compile-time error.
#[macro_export]
macro_rules! set_pref {
    ($($segment: ident).+, $value: expr) => {{
        let value = $value;
        $crate::prefs::set_stylo_pref(stringify!($($segment).+), value);
        let values = $crate::prefs::pref_map().values();
        let mut lock = values.write().unwrap();
        lock$ (.$segment)+ = value;
    }};
}

/// Access preferences using their `String` keys. Note that the key may be different from the
/// static path because legacy keys contain hyphens, or because a preference name has been renamed.
///
/// When retrieving a preference, the value will always be a `PrefValue`. When setting a value, it
/// may be a `PrefValue` or the type that converts into the correct underlying value; one of `bool`,
/// `i64`, `f64` or `String`.
#[inline]
pub fn pref_map() -> &'static Preferences<'static, Prefs> {
    &PREFS
}

pub fn add_user_prefs(prefs: HashMap<String, PrefValue>) {
    for (key, value) in prefs.iter() {
        set_stylo_pref_ref(key, value);
    }
    if let Err(error) = PREFS.set_all(prefs) {
        panic!("Error setting preference: {:?}", error);
    }
}

pub fn set_stylo_pref(key: &str, value: impl Into<PrefValue>) {
    set_stylo_pref_ref(key, &value.into());
}

fn set_stylo_pref_ref(key: &str, value: &PrefValue) {
    match value.try_into() {
        Ok(StyloPrefValue::Bool(value)) => style_config::set_bool(key, value),
        Ok(StyloPrefValue::Int(value)) => style_config::set_i32(key, value),
        Err(TryFromPrefValueError::IntegerOverflow(value)) => {
            // TODO: logging doesn’t actually work this early, so we should
            // split PrefValue into i32 and i64 variants.
            warn!("Pref value too big for Stylo: {} ({})", key, value);
        },
        Err(TryFromPrefValueError::UnmappedType) => {
            // Most of Servo’s prefs will hit this. When adding a new pref type
            // in Stylo, update TryFrom<&PrefValue> for StyloPrefValue as well.
        },
    }
}

enum StyloPrefValue {
    Bool(bool),
    Int(i32),
}

enum TryFromPrefValueError {
    IntegerOverflow(i64),
    UnmappedType,
}

impl TryFrom<&PrefValue> for StyloPrefValue {
    type Error = TryFromPrefValueError;

    fn try_from(value: &PrefValue) -> Result<Self, Self::Error> {
        match *value {
            PrefValue::Int(value) => {
                if let Ok(value) = value.try_into() {
                    Ok(Self::Int(value))
                } else {
                    Err(TryFromPrefValueError::IntegerOverflow(value))
                }
            },
            PrefValue::Bool(value) => Ok(Self::Bool(value)),
            _ => Err(TryFromPrefValueError::UnmappedType),
        }
    }
}

pub fn read_prefs_map(txt: &str) -> Result<HashMap<String, PrefValue>, PrefError> {
    let prefs: HashMap<String, Value> =
        serde_json::from_str(txt).map_err(PrefError::JsonParseErr)?;
    prefs
        .into_iter()
        .map(|(k, pref_value)| {
            Ok({
                let v = match &pref_value {
                    Value::Bool(b) => PrefValue::Bool(*b),
                    Value::Number(n) if n.is_i64() => PrefValue::Int(n.as_i64().unwrap()),
                    Value::Number(n) if n.is_f64() => PrefValue::Float(n.as_f64().unwrap()),
                    Value::String(s) => PrefValue::Str(s.to_owned()),
                    Value::Array(v) => {
                        let mut array = v.iter().map(PrefValue::from_json_value);
                        if array.all(|v| v.is_some()) {
                            PrefValue::Array(array.flatten().collect())
                        } else {
                            return Err(PrefError::InvalidValue(format!(
                                "Invalid value: {}",
                                pref_value
                            )));
                        }
                    },
                    _ => {
                        return Err(PrefError::InvalidValue(format!(
                            "Invalid value: {}",
                            pref_value
                        )));
                    },
                };
                (k.to_owned(), v)
            })
        })
        .collect()
}

mod gen {
    use serde::{Deserialize, Serialize};
    use servo_config_plugins::build_structs;

    // The number of layout threads is calculated if it is not present in `prefs.json`.
    fn default_layout_threads() -> i64 {
        3
    }

    fn default_font_size() -> i64 {
        16
    }

    fn default_monospace_font_size() -> i64 {
        13
    }

    fn default_true() -> bool {
        true
    }

    fn default_dblclick_timeout() -> i64 {
        300
    }

    fn default_dblclick_dist() -> i64 {
        1
    }

    fn default_serviceworker_timeout_seconds() -> i64 {
        60
    }

    fn default_worklet_timeout_ms() -> i64 {
        10
    }

    fn default_gc_allocation_threshold_avoid_interrupt_factor() -> i64 {
        100
    }

    fn default_gc_allocation_threshold_factor() -> i64 {
        100
    }

    fn default_gc_allocation_threshold_mb() -> i64 {
        30
    }

    fn default_gc_decommit_threshold_mb() -> i64 {
        32
    }

    fn default_gc_empty_chunk_count_max() -> i64 {
        30
    }

    fn default_gc_empty_chunk_count_min() -> i64 {
        1
    }

    fn default_gc_high_frequency_heap_growth_max() -> i64 {
        300
    }

    fn default_gc_high_frequency_heap_growth_min() -> i64 {
        150
    }

    fn default_gc_high_frequency_high_limit_mb() -> i64 {
        500
    }

    fn default_gc_high_frequency_low_limit_mb() -> i64 {
        100
    }

    fn default_gc_high_frequency_time_limit_ms() -> i64 {
        1000
    }

    fn default_gc_incremental_slice_ms() -> i64 {
        10
    }

    fn default_gc_low_frequency_heap_growth() -> i64 {
        150
    }

    fn default_gc_zeal_frequency() -> i64 {
        100
    }

    fn default_js_mem_max() -> i64 {
        -1
    }

    fn default_js_timers_minimum_duration() -> i64 {
        1000
    }

    fn default_session_history_max_length() -> i64 {
        20
    }

    fn default_background_color() -> [f64; 4] {
        [1.0, 1.0, 1.0, 1.0]
    }

    fn default_homepage() -> String {
        String::from("https://servo.org")
    }

    fn default_native_orientation() -> String {
        String::from("both")
    }

    fn default_searchpage() -> String {
        String::from("https://duckduckgo.com/html/?q=%s")
    }

    build_structs! {
        // type of the accessors
        accessor_type = crate::pref_util::Accessor::<Prefs, crate::pref_util::PrefValue>,
        // name of the constant, which will hold a HashMap of preference accessors
        gen_accessors = PREF_ACCESSORS,
        // tree of structs to generate
        gen_types = Prefs {
            fonts: {
                #[serde(default)]
                default: String,
                #[serde(default)]
                serif: String,
                #[serde(default)]
                #[serde(rename = "fonts.sans-serif")]
                sans_serif: String,
                #[serde(default)]
                monospace: String,
                #[serde(default = "default_font_size")]
                #[serde(rename = "fonts.default-size")]
                default_size: i64,
                #[serde(default = "default_monospace_font_size")]
                #[serde(rename = "fonts.default-monospace-size")]
                default_monospace_size: i64,
            },
            devtools: {
                server: {
                    #[serde(default)]
                    enabled: bool,
                    #[serde(default)]
                    port: i64,
                },
            },
            dom: {
                webgpu: {
                    /// Enable WebGPU APIs.
                    #[serde(default)]
                    enabled: bool,
                    /// List of comma-separated backends to be used by wgpu
                    #[serde(default)]
                    wgpu_backend: String,
                },
                bluetooth: {
                    #[serde(default)]
                    enabled: bool,
                    testing: {
                        #[serde(default)]
                        enabled: bool,
                    }
                },
                #[serde(default)]
                allow_scripts_to_close_windows: bool,
                canvas_capture: {
                    #[serde(default)]
                    enabled: bool,
                },
                canvas_text: {
                    #[serde(default)]
                    enabled: bool,
                },
                composition_event: {
                    #[serde(rename = "dom.compositionevent.enabled")]
                    #[serde(default)]
                    enabled: bool,
                },
                crypto: {
                    subtle: {
                        #[serde(default)]
                        enabled: bool,
                    }
                },
                custom_elements: {
                    #[serde(rename = "dom.customelements.enabled")]
                    #[serde(default = "default_true")]
                    enabled: bool,
                },
                document: {
                    #[serde(default = "default_dblclick_timeout")]
                    dblclick_timeout: i64,
                    #[serde(default = "default_dblclick_dist")]
                    dblclick_dist: i64,
                },
                forcetouch: {
                    #[serde(default)]
                    enabled: bool,
                },
                fullscreen: {
                    #[serde(default)]
                    test: bool,
                },
                gamepad: {
                    #[serde(default = "default_true")]
                    enabled: bool,
                },
                imagebitmap: {
                    #[serde(default)]
                    enabled: bool,
                },
                intersection_observer: {
                    #[serde(default)]
                    enabled: bool,
                },
                microdata: {
                    testing: {
                        #[serde(default)]
                        enabled: bool,
                    }
                },
                mouse_event: {
                    which: {
                        #[serde(rename = "dom.mouseevent.which.enabled")]
                        #[serde(default)]
                        enabled: bool,
                    }
                },
                mutation_observer: {
                    #[serde(default = "default_true")]
                    enabled: bool,
                },
                offscreen_canvas: {
                    #[serde(default)]
                    enabled: bool,
                },
                permissions: {
                    #[serde(default)]
                    enabled: bool,
                    testing: {
                        #[serde(default)]
                        allowed_in_nonsecure_contexts: bool,
                    }
                },
                resize_observer: {
                    #[serde(default)]
                    enabled: bool,
                },
                script: {
                    #[serde(default = "default_true")]
                    asynch: bool,
                },
                serviceworker: {
                    #[serde(default)]
                    enabled: bool,
                    #[serde(default = "default_serviceworker_timeout_seconds")]
                    timeout_seconds: i64,
                },
                servo_helpers: {
                    #[serde(default)]
                    enabled: bool,
                },
                servoparser: {
                    async_html_tokenizer: {
                        #[serde(default)]
                        enabled: bool,
                    }
                },
                shadowdom: {
                    #[serde(default)]
                    enabled: bool,
                },
                svg: {
                    #[serde(default)]
                    enabled: bool,
                },
                testable_crash: {
                    #[serde(default)]
                    enabled: bool,
                },
                testbinding: {
                    #[serde(default)]
                    enabled: bool,
                    prefcontrolled: {
                        #[serde(default)]
                        enabled: bool,
                    },
                    prefcontrolled2: {
                        #[serde(default)]
                        enabled: bool,
                    },
                    preference_value: {
                        #[serde(default)]
                        falsy: bool,
                        #[serde(default)]
                        quote_string_test: String,
                        #[serde(default)]
                        space_string_test: String,
                        #[serde(default)]
                        string_empty: String,
                        #[serde(default)]
                        string_test: String,
                        #[serde(default)]
                        truthy: bool,
                    },
                },
                testing: {
                    element: {
                        activation: {
                            #[serde(default)]
                            enabled: bool,
                        }
                    },
                    html_input_element: {
                        select_files: {
                            #[serde(rename = "dom.testing.htmlinputelement.select_files.enabled")]
                            #[serde(default)]
                            enabled: bool,
                        }
                    },
                },
                testperf: {
                    #[serde(default)]
                    enabled: bool,
                },
                webgl2: {
                    /// Enable WebGL2 APIs.
                    #[serde(default)]
                    enabled: bool,
                },
                webrtc: {
                    transceiver: {
                        #[serde(default)]
                        enabled: bool,
                    },
                    #[serde(default)]
                    enabled: bool,
                },
                webvtt: {
                    #[serde(default)]
                    enabled: bool,
                },
                webxr: {
                    #[serde(default)]
                    enabled: bool,
                    #[serde(default)]
                    test: bool,
                    #[serde(default)]
                    first_person_observer_view: bool,
                    glwindow: {
                        #[serde(default = "default_true")]
                        enabled: bool,
                        #[serde(rename = "dom.webxr.glwindow.left-right")]
                        #[serde(default)]
                        left_right: bool,
                        #[serde(rename = "dom.webxr.glwindow.red-cyan")]
                        #[serde(default)]
                        red_cyan: bool,
                        #[serde(default)]
                        spherical: bool,
                        #[serde(default)]
                        cubemap: bool,
                    },
                    hands: {
                        #[serde(default = "default_true")]
                        enabled: bool,
                    },
                    layers: {
                        #[serde(default)]
                        enabled: bool,
                    },
                    openxr: {
                        #[serde(default = "default_true")]
                        enabled: bool,
                    },
                    #[serde(default)]
                    sessionavailable: bool,
                    #[serde(rename = "dom.webxr.unsafe-assume-user-intent")]
                    #[serde(default)]
                    unsafe_assume_user_intent: bool,
                },
                worklet: {
                    blockingsleep: {
                        #[serde(default)]
                        enabled: bool,
                    },
                    #[serde(default)]
                    enabled: bool,
                    testing: {
                        #[serde(default)]
                        enabled: bool,
                    },
                    #[serde(default = "default_worklet_timeout_ms")]
                    timeout_ms: i64,
                },
            },
            gfx: {
                subpixel_text_antialiasing: {
                    #[serde(rename = "gfx.subpixel-text-antialiasing.enabled")]
                    #[serde(default = "default_true")]
                    enabled: bool,
                },
                texture_swizzling: {
                    #[serde(rename = "gfx.texture-swizzling.enabled")]
                    #[serde(default = "default_true")]
                    enabled: bool,
                },
            },
            js: {
                asmjs: {
                    #[serde(default = "default_true")]
                    enabled: bool,
                },
                asyncstack: {
                    #[serde(default)]
                    enabled: bool,
                },
                baseline_interpreter: {
                    #[serde(default = "default_true")]
                    enabled: bool,
                },
                baseline_jit: {
                    #[serde(default = "default_true")]
                    enabled: bool,
                    unsafe_eager_compilation: {
                        #[serde(default)]
                        enabled: bool,
                    },
                },
                discard_system_source: {
                    #[serde(default)]
                    enabled: bool,
                },
                dump_stack_on_debuggee_would_run: {
                    #[serde(default)]
                    enabled: bool,
                },
                ion: {
                    #[serde(default = "default_true")]
                    enabled: bool,
                    offthread_compilation: {
                        #[serde(default = "default_true")]
                        enabled: bool,
                    },
                    unsafe_eager_compilation: {
                        #[serde(default)]
                        enabled: bool,
                    },
                },
                mem: {
                    gc: {
                        #[serde(default = "default_gc_allocation_threshold_mb")]
                        allocation_threshold_mb: i64,
                        #[serde(default = "default_gc_allocation_threshold_factor")]
                        allocation_threshold_factor: i64,
                        #[serde(default = "default_gc_allocation_threshold_avoid_interrupt_factor")]
                        allocation_threshold_avoid_interrupt_factor: i64,
                        compacting: {
                            #[serde(default = "default_true")]
                            enabled: bool,
                        },
                        #[serde(default = "default_gc_decommit_threshold_mb")]
                        decommit_threshold_mb: i64,
                        dynamic_heap_growth: {
                            #[serde(default = "default_true")]
                            enabled: bool,
                        },
                        dynamic_mark_slice: {
                            #[serde(default = "default_true")]
                            enabled: bool,
                        },
                        #[serde(default = "default_gc_empty_chunk_count_max")]
                        empty_chunk_count_max: i64,
                        #[serde(default = "default_gc_empty_chunk_count_min")]
                        empty_chunk_count_min: i64,
                        #[serde(default = "default_gc_high_frequency_heap_growth_max")]
                        high_frequency_heap_growth_max: i64,
                        #[serde(default = "default_gc_high_frequency_heap_growth_min")]
                        high_frequency_heap_growth_min: i64,
                        #[serde(default = "default_gc_high_frequency_high_limit_mb")]
                        high_frequency_high_limit_mb: i64,
                        #[serde(default = "default_gc_high_frequency_low_limit_mb")]
                        high_frequency_low_limit_mb: i64,
                        #[serde(default = "default_gc_high_frequency_time_limit_ms")]
                        high_frequency_time_limit_ms: i64,
                        incremental: {
                            #[serde(default = "default_true")]
                            enabled: bool,
                            #[serde(default = "default_gc_incremental_slice_ms")]
                            slice_ms: i64,
                        },
                        #[serde(default = "default_gc_low_frequency_heap_growth")]
                        low_frequency_heap_growth: i64,
                        per_zone: {
                            #[serde(default)]
                            enabled: bool,
                        },
                        zeal: {
                            #[serde(default = "default_gc_zeal_frequency")]
                            frequency: i64,
                            #[serde(default)]
                            level: i64,
                        },
                    },
                    #[serde(default = "default_js_mem_max")]
                    max: i64,
                },
                native_regex: {
                    #[serde(default = "default_true")]
                    enabled: bool,
                },
                offthread_compilation: {
                    #[serde(default = "default_true")]
                    enabled: bool,
                },
                parallel_parsing: {
                    #[serde(default = "default_true")]
                    enabled: bool,
                },
                shared_memory: {
                    #[serde(default = "default_true")]
                    enabled: bool,
                },
                throw_on_asmjs_validation_failure: {
                    #[serde(default)]
                    enabled: bool,
                },
                throw_on_debuggee_would_run: {
                    #[serde(default)]
                    enabled: bool,
                },
                timers: {
                    #[serde(default = "default_js_timers_minimum_duration")]
                    minimum_duration: i64,
                },
                wasm: {
                    baseline: {
                        #[serde(default = "default_true")]
                        enabled: bool,
                    },
                    #[serde(default = "default_true")]
                    enabled: bool,
                    ion: {
                        #[serde(default = "default_true")]
                        enabled: bool,
                    }
                },
                werror: {
                    #[serde(default)]
                    enabled: bool,
                },
            },
            layout: {
                animations: {
                    test: {
                        #[serde(default)]
                        enabled: bool,
                    }
                },
                columns: {
                    #[serde(default)]
                    enabled: bool,
                },
                css: {
                    transition_behavior: {
                        #[serde(rename = "layout.css.transition-behavior.enabled")]
                        #[serde(default = "default_true")]
                        enabled: bool,
                    }
                },
                flexbox: {
                    #[serde(default = "default_true")]
                    enabled: bool,
                },
                #[serde(default)]
                legacy_layout: bool,
                #[serde(default = "default_layout_threads")]
                threads: i64,
                writing_mode: {
                    #[serde(rename = "layout.writing-mode.enabled")]
                    #[serde(default)]
                    enabled: bool,
                }
            },
            media: {
                glvideo: {
                    /// Enable hardware acceleration for video playback.
                    #[serde(default)]
                    enabled: bool,
                },
                testing: {
                    /// Enable a non-standard event handler for verifying behavior of media elements during tests.
                    #[serde(default)]
                    enabled: bool,
                }
            },
            network: {
                enforce_tls: {
                    #[serde(default)]
                    enabled: bool,
                    #[serde(default)]
                    localhost: bool,
                    #[serde(default)]
                    onion: bool,
                },
                http_cache: {
                    #[serde(rename = "network.http-cache.disabled")]
                    #[serde(default)]
                    disabled: bool,
                },
                local_directory_listing: {
                    #[serde(default)]
                    enabled: bool,
                },
                mime: {
                    #[serde(default)]
                    sniff: bool,
                },
                tls: {
                    /// Ignore `std::io::Error` with `ErrorKind::UnexpectedEof` received when a TLS connection
                    /// is closed without a close_notify.
                    ///
                    /// Used for tests because WPT server doesn't properly close the TLS connection.
                    // TODO: remove this when WPT server is updated to use a proper TLS implementation.
                    #[serde(default)]
                    ignore_unexpected_eof: bool,
                },
            },
            session_history: {
                #[serde(rename = "session-history.max-length")]
                #[serde(default = "default_session_history_max_length")]
                max_length: i64,
            },
            shell: {
                background_color: {
                    /// The background color of shell's viewport. This will be used by OpenGL's `glClearColor`.
                    #[serde(rename = "shell.background-color.rgba")]
                    #[serde(default = "default_background_color")]
                    rgba: [f64; 4],
                },
                crash_reporter: {
                    #[serde(default)]
                    enabled: bool,
                },
                /// URL string of the homepage.
                #[serde(default = "default_homepage")]
                homepage: String,
                #[serde(rename = "shell.native-orientation")]
                #[serde(default = "default_native_orientation")]
                native_orientation: String,
                native_titlebar: {
                    /// Enable native window's titlebar and decorations.
                    #[serde(rename = "shell.native-titlebar.enabled")]
                    #[serde(default = "default_true")]
                    enabled: bool,
                },
                /// URL string of the search engine page (for example <https://google.com> or and <https://duckduckgo.com>.
                #[serde(default = "default_searchpage")]
                searchpage: String,
            },
            webgl: {
                testing: {
                    #[serde(default)]
                    context_creation_error: bool,
                }
            },
        }
    }
}
