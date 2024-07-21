/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use embedder_traits::resources::{self, Resource};
use gen::Prefs;
use lazy_static::lazy_static;
use log::warn;
use serde_json::{self, Value};

use crate::pref_util::Preferences;
pub use crate::pref_util::{PrefError, PrefValue};

lazy_static! {
    static ref PREFS: Preferences<'static, Prefs> = {
        let def_prefs: Prefs = serde_json::from_str(&resources::read_string(Resource::Preferences))
            .expect("Failed to initialize config preferences.");
        let result = Preferences::new(def_prefs, &gen::PREF_ACCESSORS);
        for (key, value) in result.iter() {
            set_stylo_pref_ref(&key, &value);
        }
        result
    };
}

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
        std::cmp::max(num_cpus::get() * 3 / 4, 1) as i64
    }

    fn default_font_size() -> i64 {
        16
    }

    fn default_monospace_font_size() -> i64 {
        13
    }

    fn black() -> i64 {
        0x000000
    }

    fn white() -> i64 {
        0xFFFFFF
    }

    build_structs! {
        // type of the accessors
        accessor_type = crate::pref_util::Accessor::<Prefs, crate::pref_util::PrefValue>,
        // name of the constant, which will hold a HashMap of preference accessors
        gen_accessors = PREF_ACCESSORS,
        // tree of structs to generate
        gen_types = Prefs {
            browser: {
                display: {
                    #[serde(default = "white")]
                    background_color: i64,
                    #[serde(default = "black")]
                    foreground_color: i64,
                }
            },
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
            css: {
                animations: {
                    testing: {
                        #[serde(default)]
                        enabled: bool,
                    },
                },
            },
            devtools: {
                server: {
                    enabled: bool,
                    port: i64,
                },
            },
            dom: {
                webgpu: {
                    /// Enable WebGPU APIs.
                    enabled: bool,
                    /// List of comma-separated backends to be used by wgpu
                    wgpu_backend: String,
                },
                bluetooth: {
                    enabled: bool,
                    testing: {
                        enabled: bool,
                    }
                },
                canvas_capture: {
                    enabled: bool,
                },
                canvas_text: {
                    enabled: bool,
                },
                composition_event: {
                    #[serde(rename = "dom.compositionevent.enabled")]
                    enabled: bool,
                },
                custom_elements: {
                    #[serde(rename = "dom.customelements.enabled")]
                    enabled: bool,
                },
                document: {
                    dblclick_timeout: i64,
                    dblclick_dist: i64,
                },
                forcetouch: {
                    enabled: bool,
                },
                fullscreen: {
                    test: bool,
                },
                gamepad: {
                    enabled: bool,
                },
                imagebitmap: {
                    enabled: bool,
                },
                microdata: {
                    testing: {
                        enabled: bool,
                    }
                },
                mouse_event: {
                    which: {
                        #[serde(rename = "dom.mouseevent.which.enabled")]
                        enabled: bool,
                    }
                },
                mutation_observer: {
                    enabled: bool,
                },
                offscreen_canvas: {
                    enabled: bool,
                },
                permissions: {
                    enabled: bool,
                    testing: {
                        allowed_in_nonsecure_contexts: bool,
                    }
                },
                resize_observer: {
                    enabled: bool,
                },
                script: {
                    asynch: bool,
                },
                serviceworker: {
                    enabled: bool,
                    timeout_seconds: i64,
                },
                servo_helpers: {
                    enabled: bool,
                },
                servoparser: {
                    async_html_tokenizer: {
                        enabled: bool,
                    }
                },
                shadowdom: {
                    enabled: bool,
                },
                svg: {
                    enabled: bool,
                },
                testable_crash: {
                    enabled: bool,
                },
                testbinding: {
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
                    enabled: bool,
                },
                webrtc: {
                    transceiver: {
                        enabled: bool,
                    },
                    #[serde(default)]
                    enabled: bool,
                },
                webvtt: {
                    enabled: bool,
                },
                webxr: {
                    #[serde(default)]
                    enabled: bool,
                    #[serde(default)]
                    test: bool,
                    first_person_observer_view: bool,
                    glwindow: {
                        #[serde(default)]
                        enabled: bool,
                        #[serde(rename = "dom.webxr.glwindow.left-right")]
                        left_right: bool,
                        #[serde(rename = "dom.webxr.glwindow.red-cyan")]
                        red_cyan: bool,
                        spherical: bool,
                        cubemap: bool,
                    },
                    hands: {
                        #[serde(default)]
                        enabled: bool,
                    },
                    layers: {
                        enabled: bool,
                    },
                    openxr: {
                        enabled: bool,
                    },
                    sessionavailable: bool,
                    #[serde(rename = "dom.webxr.unsafe-assume-user-intent")]
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
                    timeout_ms: i64,
                },
            },
            gfx: {
                subpixel_text_antialiasing: {
                    #[serde(rename = "gfx.subpixel-text-antialiasing.enabled")]
                    enabled: bool,
                },
                texture_swizzling: {
                    #[serde(rename = "gfx.texture-swizzling.enabled")]
                    enabled: bool,
                },
            },
            js: {
                asmjs: {
                    enabled: bool,
                },
                asyncstack: {
                    enabled: bool,
                },
                baseline_interpreter: {
                    enabled: bool,
                },
                baseline_jit: {
                    enabled: bool,
                    unsafe_eager_compilation: {
                        enabled: bool,
                    },
                },
                discard_system_source: {
                    enabled: bool,
                },
                dump_stack_on_debuggee_would_run: {
                    enabled: bool,
                },
                ion: {
                    enabled: bool,
                    offthread_compilation: {
                        enabled: bool,
                    },
                    unsafe_eager_compilation: {
                        enabled: bool,
                    },
                },
                mem: {
                    gc: {
                        allocation_threshold_mb: i64,
                        allocation_threshold_factor: i64,
                        allocation_threshold_avoid_interrupt_factor: i64,
                        compacting: {
                            enabled: bool,
                        },
                        decommit_threshold_mb: i64,
                        dynamic_heap_growth: {
                            enabled: bool,
                        },
                        dynamic_mark_slice: {
                            enabled: bool,
                        },
                        empty_chunk_count_max: i64,
                        empty_chunk_count_min: i64,
                        high_frequency_heap_growth_max: i64,
                        high_frequency_heap_growth_min: i64,
                        high_frequency_high_limit_mb: i64,
                        high_frequency_low_limit_mb: i64,
                        high_frequency_time_limit_ms: i64,
                        incremental: {
                            enabled: bool,
                            slice_ms: i64,
                        },
                        low_frequency_heap_growth: i64,
                        per_zone: {
                            enabled: bool,
                        },
                        zeal: {
                            frequency: i64,
                            level: i64,
                        },
                    },
                    max: i64,
                },
                native_regex: {
                    enabled: bool,
                },
                offthread_compilation: {
                    enabled: bool,
                },
                parallel_parsing: {
                    enabled: bool,
                },
                shared_memory: {
                    enabled: bool,
                },
                strict: {
                    debug: {
                        enabled: bool,
                    },
                    enabled: bool,
                },
                throw_on_asmjs_validation_failure: {
                    enabled: bool,
                },
                throw_on_debuggee_would_run: {
                    enabled: bool,
                },
                timers: {
                    minimum_duration: i64,
                },
                wasm: {
                    baseline: {
                        enabled: bool,
                    },
                    enabled: bool,
                    ion: {
                        enabled: bool,
                    }
                },
                werror: {
                    enabled: bool,
                },
            },
            layout: {
                animations: {
                    test: {
                        enabled: bool,
                    }
                },
                columns: {
                    enabled: bool,
                },
                flexbox: {
                    enabled: bool,
                },
                legacy_layout: bool,
                #[serde(default = "default_layout_threads")]
                threads: i64,
                writing_mode: {
                    #[serde(rename = "layout.writing-mode.enabled")]
                    enabled: bool,
                }
            },
            media: {
                glvideo: {
                    /// Enable hardware acceleration for video playback.
                    enabled: bool,
                },
                testing: {
                    /// Enable a non-standard event handler for verifying behavior of media elements during tests.
                    enabled: bool,
                }
            },
            network: {
                enforce_tls: {
                    enabled: bool,
                    localhost: bool,
                    onion: bool,
                },
                http_cache: {
                    #[serde(rename = "network.http-cache.disabled")]
                    disabled: bool,
                },
                local_directory_listing: {
                    enabled: bool,
                },
                mime: {
                    sniff: bool,
                }
            },
            session_history: {
                #[serde(rename = "session-history.max-length")]
                max_length: i64,
            },
            shell: {
                background_color: {
                    /// The background color of shell's viewport. This will be used by OpenGL's `glClearColor`.
                    #[serde(rename = "shell.background-color.rgba")]
                    rgba: [f64; 4],
                },
                crash_reporter: {
                    enabled: bool,
                },
                /// URL string of the homepage.
                homepage: String,
                keep_screen_on: {
                    enabled: bool,
                },
                #[serde(rename = "shell.native-orientation")]
                native_orientation: String,
                native_titlebar: {
                    /// Enable native window's titlebar and decorations.
                    #[serde(rename = "shell.native-titlebar.enabled")]
                    enabled: bool,
                },
                /// URL string of the search engine page (for example <https://google.com> or and <https://duckduckgo.com>.
                searchpage: String,
            },
            webgl: {
                testing: {
                    context_creation_error: bool,
                }
            },
        }
    }
}
