/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::basedir::default_config_dir;
use crate::opts;
use embedder_traits::resources::{self, Resource};
use serde_json::{self, Value};
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::fs::File;
use std::io::{stderr, Read, Write};
use std::path::PathBuf;

use crate::pref_util::Preferences;
pub use crate::pref_util::{PrefError, PrefValue};
use gen::Prefs;

lazy_static! {
    static ref PREFS: Preferences<'static, Prefs> = {
        let def_prefs: Prefs = serde_json::from_str(&resources::read_string(Resource::Preferences))
            .expect("Failed to initialize config preferences.");
        Preferences::new(def_prefs, &gen::PREF_ACCESSORS)
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
        let values = $crate::prefs::pref_map().values();
        let mut lock = values.write().unwrap();
        lock$ (.$segment)+ = $value;
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

pub(crate) fn add_user_prefs() {
    if let Some(path) = user_prefs_path() {
        init_user_prefs(path);
    }
}

fn user_prefs_path() -> Option<PathBuf> {
    opts::get()
        .config_dir
        .clone()
        .or_else(|| default_config_dir())
        .map(|path| path.join("prefs.json"))
        .filter(|path| path.exists())
}

fn init_user_prefs(path: PathBuf) {
    if let Ok(mut file) = File::open(&path) {
        let mut txt = String::new();
        file.read_to_string(&mut txt)
            .expect("Can't read user prefs");
        match read_prefs_map(&txt) {
            Ok(prefs) => {
                if let Err(error) = PREFS.set_all(prefs.into_iter()) {
                    writeln!(&mut stderr(), "Error setting preference: {:?}", error)
                } else {
                    Ok(())
                }
            },
            Err(error) => writeln!(&mut stderr(), "Error parsing prefs.json: {:?}", error),
        }
    } else {
        writeln!(&mut stderr(), "Error opening user prefs from {:?}", path)
    }
    .expect("failed printing to stderr");
}

pub fn read_prefs_map(txt: &str) -> Result<HashMap<String, PrefValue>, PrefError> {
    let prefs: HashMap<String, Value> =
        serde_json::from_str(txt).map_err(|e| PrefError::JsonParseErr(e))?;
    prefs
        .into_iter()
        .map(|(k, pref_value)| {
            Ok({
                let v = match &pref_value {
                    Value::Bool(b) => PrefValue::Bool(*b),
                    Value::Number(n) if n.is_i64() => PrefValue::Int(n.as_i64().unwrap()),
                    Value::Number(n) if n.is_f64() => PrefValue::Float(n.as_f64().unwrap()),
                    Value::String(s) => PrefValue::Str(s.to_owned()),
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
    use servo_config_plugins::build_structs;

    // The number of layout threads is calculated if it is not present in `prefs.json`.
    fn default_layout_threads() -> i64 {
        std::cmp::max(num_cpus::get() * 3 / 4, 1) as i64
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
            css: {
                animations: {
                    testing: {
                        #[serde(default)]
                        enabled: bool,
                    },
                },
            },
            dom: {
                webgpu: {
                    enabled: bool,
                },
                bluetooth: {
                    enabled: bool,
                    testing: {
                        enabled: bool,
                    }
                },
                canvas_text: {
                    #[serde(rename = "dom.canvas-text.enabled")]
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
                serviceworker: {
                    enabled: bool,
                    timeout_seconds: i64,
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
                webgl: {
                    dom_to_texture: {
                        enabled: bool,
                    }
                },
                webgl2: {
                    enabled: bool,
                },
                webrtc: {
                    #[serde(default)]
                    enabled: bool,
                },
                webvr: {
                    enabled: bool,
                    event_polling_interval: i64,
                    test: bool,
                },
                webxr: {
                    #[serde(default)]
                    enabled: bool,
                    #[serde(default)]
                    test: bool,
                    #[serde(default)]
                    glwindow: bool,
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
                }
            },
            js: {
                asmjs: {
                    enabled: bool,
                },
                asyncstack: {
                    enabled: bool,
                },
                baseline: {
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
                #[serde(default = "default_layout_threads")]
                threads: i64,
                viewport: {
                    enabled: bool,
                },
                writing_mode: {
                    #[serde(rename = "layout.writing-mode.enabled")]
                    enabled: bool,
                }
            },
            media: {
                glvideo: {
                    enabled: bool,
                },
                testing: {
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
                mime: {
                    sniff: bool,
                }
            },
            session_history: {
                #[serde(rename = "session-history.max-length")]
                max_length: i64,
            },
            shell: {
                homepage: String,
                keep_screen_on: {
                    enabled: bool,
                },
                #[serde(rename = "shell.native-orientation")]
                native_orientation: String,
                native_titlebar: {
                    #[serde(rename = "shell.native-titlebar.enabled")]
                    enabled: bool,
                },
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
