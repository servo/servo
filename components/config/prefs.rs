/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{RwLock, RwLockReadGuard};

use serde::{Deserialize, Serialize};
use servo_config_macro::ServoPreferences;

pub use crate::pref_util::PrefValue;

static PREFERENCES: RwLock<Preferences> = RwLock::new(Preferences::const_default());

#[inline]
/// Get the current set of global preferences for Servo.
pub fn get() -> RwLockReadGuard<'static, Preferences> {
    PREFERENCES.read().unwrap()
}

pub fn set(preferences: Preferences) {
    // Map between Stylo preference names and Servo preference names as the This should be
    // kept in sync with components/script/dom/bindings/codegen/run.py which generates the
    // DOM CSS style accessors.
    stylo_config::set_bool("layout.unimplemented", preferences.layout_unimplemented);
    stylo_config::set_i32("layout.threads", preferences.layout_threads as i32);
    stylo_config::set_bool("layout.flexbox.enabled", preferences.layout_flexbox_enabled);
    stylo_config::set_bool("layout.columns.enabled", preferences.layout_columns_enabled);
    stylo_config::set_bool("layout.grid.enabled", preferences.layout_grid_enabled);
    stylo_config::set_bool(
        "layout.css.transition-behavior.enabled",
        preferences.layout_css_transition_behavior_enabled,
    );
    stylo_config::set_bool(
        "layout.writing-mode.enabled",
        preferences.layout_writing_mode_enabled,
    );
    stylo_config::set_bool(
        "layout.container-queries.enabled",
        preferences.layout_container_queries_enabled,
    );

    *PREFERENCES.write().unwrap() = preferences;
}

/// A convenience macro for accessing a preference value using its static path.
/// Passing an invalid path is a compile-time error.
#[macro_export]
macro_rules! pref {
    ($name: ident) => {
        $crate::prefs::get().$name.clone()
    };
}

#[derive(Clone, Deserialize, Serialize, ServoPreferences)]
pub struct Preferences {
    pub fonts_default: String,
    pub fonts_serif: String,
    pub fonts_sans_serif: String,
    pub fonts_monospace: String,
    pub fonts_default_size: i64,
    pub fonts_default_monospace_size: i64,
    pub css_animations_testing_enabled: bool,
    /// Start the devtools server at startup
    pub devtools_server_enabled: bool,
    /// Port number to start a server to listen to remote Firefox devtools connections.
    /// 0 for random port.
    pub devtools_server_port: i64,
    pub dom_webgpu_enabled: bool,
    /// List of comma-separated backends to be used by wgpu.
    pub dom_webgpu_wgpu_backend: String,
    pub dom_abort_controller_enabled: bool,
    pub dom_async_clipboard_enabled: bool,
    pub dom_bluetooth_enabled: bool,
    pub dom_bluetooth_testing_enabled: bool,
    pub dom_allow_scripts_to_close_windows: bool,
    pub dom_canvas_capture_enabled: bool,
    pub dom_canvas_text_enabled: bool,
    pub dom_clipboardevent_enabled: bool,
    pub dom_composition_event_enabled: bool,
    pub dom_crypto_subtle_enabled: bool,
    pub dom_customelements_enabled: bool,
    pub dom_document_dblclick_timeout: i64,
    pub dom_document_dblclick_dist: i64,
    pub dom_fontface_enabled: bool,
    pub dom_fullscreen_test: bool,
    pub dom_gamepad_enabled: bool,
    pub dom_imagebitmap_enabled: bool,
    pub dom_indexeddb_enabled: bool,
    pub dom_intersection_observer_enabled: bool,
    pub dom_microdata_testing_enabled: bool,
    pub dom_mouse_event_which_enabled: bool,
    pub dom_mutation_observer_enabled: bool,
    pub dom_notification_enabled: bool,
    pub dom_offscreen_canvas_enabled: bool,
    pub dom_permissions_enabled: bool,
    pub dom_permissions_testing_allowed_in_nonsecure_contexts: bool,
    pub dom_resize_observer_enabled: bool,
    pub dom_script_asynch: bool,
    pub dom_serviceworker_enabled: bool,
    pub dom_serviceworker_timeout_seconds: i64,
    pub dom_servo_helpers_enabled: bool,
    pub dom_servoparser_async_html_tokenizer_enabled: bool,
    pub dom_svg_enabled: bool,
    pub dom_testable_crash_enabled: bool,
    pub dom_testbinding_enabled: bool,
    pub dom_testbinding_prefcontrolled_enabled: bool,
    pub dom_testbinding_prefcontrolled2_enabled: bool,
    pub dom_testbinding_preference_value_falsy: bool,
    pub dom_testbinding_preference_value_quote_string_test: String,
    pub dom_testbinding_preference_value_space_string_test: String,
    pub dom_testbinding_preference_value_string_empty: String,
    pub dom_testbinding_preference_value_string_test: String,
    pub dom_testbinding_preference_value_truthy: bool,
    pub dom_testing_element_activation_enabled: bool,
    pub dom_testing_html_input_element_select_files_enabled: bool,
    pub dom_testperf_enabled: bool,
    // https://testutils.spec.whatwg.org#availability
    pub dom_testutils_enabled: bool,
    pub dom_trusted_types_enabled: bool,
    pub dom_xpath_enabled: bool,
    /// Enable WebGL2 APIs.
    pub dom_webgl2_enabled: bool,
    pub dom_webrtc_enabled: bool,
    pub dom_webrtc_transceiver_enabled: bool,
    pub dom_webvtt_enabled: bool,
    pub dom_webxr_enabled: bool,
    pub dom_webxr_test: bool,
    pub dom_webxr_first_person_observer_view: bool,
    pub dom_webxr_glwindow_enabled: bool,
    pub dom_webxr_glwindow_left_right: bool,
    pub dom_webxr_glwindow_red_cyan: bool,
    pub dom_webxr_glwindow_spherical: bool,
    pub dom_webxr_glwindow_cubemap: bool,
    pub dom_webxr_hands_enabled: bool,
    pub dom_webxr_layers_enabled: bool,
    pub dom_webxr_openxr_enabled: bool,
    pub dom_webxr_sessionavailable: bool,
    pub dom_webxr_unsafe_assume_user_intent: bool,
    pub dom_worklet_enabled: bool,
    pub dom_worklet_blockingsleep: bool,
    pub dom_worklet_testing_enabled: bool,
    pub dom_worklet_timeout_ms: i64,
    /// True to compile all WebRender shaders when Servo initializes. This is mostly
    /// useful when modifying the shaders, to ensure they all compile after each change is
    /// made.
    pub gfx_precache_shaders: bool,
    /// Whether or not antialiasing is enabled for text rendering.
    pub gfx_text_antialiasing_enabled: bool,
    /// Whether or not subpixel antialiasing is enabled for text rendering.
    pub gfx_subpixel_text_antialiasing_enabled: bool,
    pub gfx_texture_swizzling_enabled: bool,
    /// The amount of image keys we request per batch for the image cache.
    pub image_key_batch_size: i64,
    /// Whether or not the DOM inspector should show shadow roots of user-agent shadow trees
    pub inspector_show_servo_internal_shadow_roots: bool,
    pub js_asmjs_enabled: bool,
    pub js_asyncstack: bool,
    pub js_baseline_interpreter_enabled: bool,
    /// Whether to disable the jit within SpiderMonkey
    pub js_disable_jit: bool,
    pub js_baseline_jit_enabled: bool,
    pub js_baseline_jit_unsafe_eager_compilation_enabled: bool,
    pub js_discard_system_source: bool,
    pub js_dump_stack_on_debuggee_would_run: bool,
    pub js_ion_enabled: bool,
    pub js_ion_offthread_compilation_enabled: bool,
    pub js_ion_unsafe_eager_compilation_enabled: bool,
    pub js_mem_gc_allocation_threshold_mb: i64,
    pub js_mem_gc_allocation_threshold_factor: i64,
    pub js_mem_gc_allocation_threshold_avoid_interrupt_factor: i64,
    pub js_mem_gc_compacting_enabled: bool,
    pub js_mem_gc_decommit_threshold_mb: i64,
    pub js_mem_gc_dynamic_heap_growth_enabled: bool,
    pub js_mem_gc_dynamic_mark_slice_enabled: bool,
    pub js_mem_gc_empty_chunk_count_max: i64,
    pub js_mem_gc_empty_chunk_count_min: i64,
    pub js_mem_gc_high_frequency_heap_growth_max: i64,
    pub js_mem_gc_high_frequency_heap_growth_min: i64,
    pub js_mem_gc_high_frequency_high_limit_mb: i64,
    pub js_mem_gc_high_frequency_low_limit_mb: i64,
    pub js_mem_gc_high_frequency_time_limit_ms: i64,
    pub js_mem_gc_incremental_enabled: bool,
    pub js_mem_gc_incremental_slice_ms: i64,
    pub js_mem_gc_low_frequency_heap_growth: i64,
    pub js_mem_gc_per_zone_enabled: bool,
    pub js_mem_gc_zeal_frequency: i64,
    pub js_mem_gc_zeal_level: i64,
    pub js_mem_max: i64,
    pub js_native_regex_enabled: bool,
    pub js_offthread_compilation_enabled: bool,
    pub js_parallel_parsing_enabled: bool,
    pub js_shared_memory: bool,
    pub js_throw_on_asmjs_validation_failure: bool,
    pub js_throw_on_debuggee_would_run: bool,
    pub js_timers_minimum_duration: i64,
    pub js_wasm_baseline_enabled: bool,
    pub js_wasm_enabled: bool,
    pub js_wasm_ion_enabled: bool,
    pub js_werror_enabled: bool,
    pub layout_animations_test_enabled: bool,
    pub layout_columns_enabled: bool,
    pub layout_grid_enabled: bool,
    pub layout_container_queries_enabled: bool,
    pub layout_css_transition_behavior_enabled: bool,
    pub layout_flexbox_enabled: bool,
    pub layout_threads: i64,
    pub layout_unimplemented: bool,
    pub layout_writing_mode_enabled: bool,
    /// Enable hardware acceleration for video playback.
    pub media_glvideo_enabled: bool,
    /// Enable a non-standard event handler for verifying behavior of media elements during tests.
    pub media_testing_enabled: bool,
    pub network_enforce_tls_enabled: bool,
    pub network_enforce_tls_localhost: bool,
    pub network_enforce_tls_onion: bool,
    pub network_http_cache_disabled: bool,
    pub network_local_directory_listing_enabled: bool,
    pub network_mime_sniff: bool,
    pub session_history_max_length: i64,
    /// The background color of shell's viewport. This will be used by OpenGL's `glClearColor`.
    pub shell_background_color_rgba: [f64; 4],
    pub webgl_testing_context_creation_error: bool,
    /// Number of workers per threadpool, if we fail to detect how much
    /// parallelism is available at runtime.
    pub threadpools_fallback_worker_num: i64,
    /// Maximum number of workers for the Image Cache thread pool
    pub threadpools_image_cache_workers_max: i64,
    /// Maximum number of workers for the IndexedDB thread pool
    pub threadpools_indexeddb_workers_max: i64,
    /// Maximum number of workers for the Networking async runtime thread pool
    pub threadpools_async_runtime_workers_max: i64,
    /// Maximum number of workers for the Core Resource Manager
    pub threadpools_resource_workers_max: i64,
    /// Maximum number of workers for webrender
    pub threadpools_webrender_workers_max: i64,
    /// The user-agent to use for Servo. This can also be set via [`UserAgentPlatform`] in
    /// order to set the value to the default value for the given platform.
    pub user_agent: String,

    pub log_filter: String,
}

impl Preferences {
    const fn const_default() -> Self {
        Self {
            css_animations_testing_enabled: false,
            devtools_server_enabled: false,
            devtools_server_port: 0,
            dom_abort_controller_enabled: false,
            dom_allow_scripts_to_close_windows: false,
            dom_async_clipboard_enabled: false,
            dom_bluetooth_enabled: false,
            dom_bluetooth_testing_enabled: false,
            dom_canvas_capture_enabled: false,
            dom_canvas_text_enabled: true,
            dom_clipboardevent_enabled: true,
            dom_composition_event_enabled: false,
            dom_crypto_subtle_enabled: true,
            dom_customelements_enabled: true,
            dom_document_dblclick_dist: 1,
            dom_document_dblclick_timeout: 300,
            dom_fontface_enabled: false,
            dom_fullscreen_test: false,
            dom_gamepad_enabled: true,
            dom_imagebitmap_enabled: false,
            dom_indexeddb_enabled: false,
            dom_intersection_observer_enabled: false,
            dom_microdata_testing_enabled: false,
            dom_mouse_event_which_enabled: false,
            dom_mutation_observer_enabled: true,
            dom_notification_enabled: false,
            dom_offscreen_canvas_enabled: false,
            dom_permissions_enabled: false,
            dom_permissions_testing_allowed_in_nonsecure_contexts: false,
            dom_resize_observer_enabled: false,
            dom_script_asynch: true,
            dom_serviceworker_enabled: false,
            dom_serviceworker_timeout_seconds: 60,
            dom_servo_helpers_enabled: false,
            dom_servoparser_async_html_tokenizer_enabled: false,
            dom_svg_enabled: false,
            dom_testable_crash_enabled: false,
            dom_testbinding_enabled: false,
            dom_testbinding_prefcontrolled2_enabled: false,
            dom_testbinding_prefcontrolled_enabled: false,
            dom_testbinding_preference_value_falsy: false,
            dom_testbinding_preference_value_quote_string_test: String::new(),
            dom_testbinding_preference_value_space_string_test: String::new(),
            dom_testbinding_preference_value_string_empty: String::new(),
            dom_testbinding_preference_value_string_test: String::new(),
            dom_testbinding_preference_value_truthy: false,
            dom_testing_element_activation_enabled: false,
            dom_testing_html_input_element_select_files_enabled: false,
            dom_testperf_enabled: false,
            dom_testutils_enabled: false,
            dom_trusted_types_enabled: false,
            dom_webgl2_enabled: false,
            dom_webgpu_enabled: false,
            dom_webgpu_wgpu_backend: String::new(),
            dom_webrtc_enabled: false,
            dom_webrtc_transceiver_enabled: false,
            dom_webvtt_enabled: false,
            dom_webxr_enabled: true,
            dom_webxr_first_person_observer_view: false,
            dom_webxr_glwindow_cubemap: false,
            dom_webxr_glwindow_enabled: true,
            dom_webxr_glwindow_left_right: false,
            dom_webxr_glwindow_red_cyan: false,
            dom_webxr_glwindow_spherical: false,
            dom_webxr_hands_enabled: true,
            dom_webxr_layers_enabled: false,
            dom_webxr_openxr_enabled: true,
            dom_webxr_sessionavailable: false,
            dom_webxr_test: false,
            dom_webxr_unsafe_assume_user_intent: false,
            dom_worklet_blockingsleep: false,
            dom_worklet_enabled: false,
            dom_worklet_testing_enabled: false,
            dom_worklet_timeout_ms: 10,
            dom_xpath_enabled: false,
            fonts_default: String::new(),
            fonts_default_monospace_size: 13,
            fonts_default_size: 16,
            fonts_monospace: String::new(),
            fonts_sans_serif: String::new(),
            fonts_serif: String::new(),
            gfx_precache_shaders: false,
            gfx_text_antialiasing_enabled: true,
            gfx_subpixel_text_antialiasing_enabled: true,
            gfx_texture_swizzling_enabled: true,
            image_key_batch_size: 10,
            inspector_show_servo_internal_shadow_roots: false,
            js_asmjs_enabled: true,
            js_asyncstack: false,
            js_baseline_interpreter_enabled: true,
            js_baseline_jit_enabled: true,
            js_baseline_jit_unsafe_eager_compilation_enabled: false,
            js_disable_jit: false,
            js_discard_system_source: false,
            js_dump_stack_on_debuggee_would_run: false,
            js_ion_enabled: true,
            js_ion_offthread_compilation_enabled: true,
            js_ion_unsafe_eager_compilation_enabled: false,
            js_mem_gc_allocation_threshold_avoid_interrupt_factor: 100,
            js_mem_gc_allocation_threshold_factor: 100,
            js_mem_gc_allocation_threshold_mb: 30,
            js_mem_gc_compacting_enabled: true,
            js_mem_gc_decommit_threshold_mb: 32,
            js_mem_gc_dynamic_heap_growth_enabled: true,
            js_mem_gc_dynamic_mark_slice_enabled: true,
            js_mem_gc_empty_chunk_count_max: 30,
            js_mem_gc_empty_chunk_count_min: 1,
            js_mem_gc_high_frequency_heap_growth_max: 300,
            js_mem_gc_high_frequency_heap_growth_min: 150,
            js_mem_gc_high_frequency_high_limit_mb: 500,
            js_mem_gc_high_frequency_low_limit_mb: 100,
            js_mem_gc_high_frequency_time_limit_ms: 1000,
            js_mem_gc_incremental_enabled: true,
            js_mem_gc_incremental_slice_ms: 10,
            js_mem_gc_low_frequency_heap_growth: 150,
            js_mem_gc_per_zone_enabled: false,
            js_mem_gc_zeal_frequency: 100,
            js_mem_gc_zeal_level: 0,
            js_mem_max: -1,
            js_native_regex_enabled: true,
            js_offthread_compilation_enabled: true,
            js_parallel_parsing_enabled: true,
            js_shared_memory: true,
            js_throw_on_asmjs_validation_failure: false,
            js_throw_on_debuggee_would_run: false,
            js_timers_minimum_duration: 1000,
            js_wasm_baseline_enabled: true,
            js_wasm_enabled: true,
            js_wasm_ion_enabled: true,
            js_werror_enabled: false,
            layout_animations_test_enabled: false,
            layout_columns_enabled: false,
            layout_container_queries_enabled: false,
            layout_css_transition_behavior_enabled: true,
            layout_flexbox_enabled: true,
            layout_grid_enabled: false,
            // TODO(mrobinson): This should likely be based on the number of processors.
            layout_threads: 3,
            layout_unimplemented: false,
            layout_writing_mode_enabled: false,
            media_glvideo_enabled: false,
            media_testing_enabled: false,
            network_enforce_tls_enabled: false,
            network_enforce_tls_localhost: false,
            network_enforce_tls_onion: false,
            network_http_cache_disabled: false,
            network_local_directory_listing_enabled: true,
            network_mime_sniff: false,
            session_history_max_length: 20,
            shell_background_color_rgba: [1.0, 1.0, 1.0, 1.0],
            threadpools_async_runtime_workers_max: 6,
            threadpools_fallback_worker_num: 3,
            threadpools_image_cache_workers_max: 4,
            threadpools_indexeddb_workers_max: 4,
            threadpools_resource_workers_max: 4,
            threadpools_webrender_workers_max: 4,
            webgl_testing_context_creation_error: false,
            user_agent: String::new(),
            log_filter: String::new(),
        }
    }
}

impl Default for Preferences {
    fn default() -> Self {
        let mut preferences = Self::const_default();
        preferences.user_agent = UserAgentPlatform::default().to_user_agent_string();
        preferences
    }
}

pub enum UserAgentPlatform {
    Desktop,
    Android,
    OpenHarmony,
    Ios,
}

impl UserAgentPlatform {
    /// Return the default `UserAgentPlatform` for this platform. This is
    /// not an implementation of `Default` so that it can be `const`.
    const fn default() -> Self {
        if cfg!(target_os = "android") {
            Self::Android
        } else if cfg!(target_env = "ohos") {
            Self::OpenHarmony
        } else if cfg!(target_os = "ios") {
            Self::Ios
        } else {
            Self::Desktop
        }
    }
}

impl UserAgentPlatform {
    /// Convert this [`UserAgentPlatform`] into its corresponding `String` value, ie the
    /// default user-agent to use for this platform.
    pub fn to_user_agent_string(&self) -> String {
        const SERVO_VERSION: &str = env!("CARGO_PKG_VERSION");
        match self {
            UserAgentPlatform::Desktop
                if cfg!(all(target_os = "windows", target_arch = "x86_64")) =>
            {
                #[cfg(target_arch = "x86_64")]
                const ARCHITECTURE: &str = "x86; ";
                #[cfg(not(target_arch = "x86_64"))]
                const ARCHITECTURE: &str = "";

                format!(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; {ARCHITECTURE}rv:128.0) Servo/{SERVO_VERSION} Firefox/128.0"
                )
            },
            UserAgentPlatform::Desktop if cfg!(target_os = "macos") => {
                format!(
                    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:128.0) Servo/{SERVO_VERSION} Firefox/128.0"
                )
            },
            UserAgentPlatform::Desktop => {
                #[cfg(target_arch = "x86_64")]
                const ARCHITECTURE: &str = "x86_64";
                // TODO: This is clearly wrong for other platforms.
                #[cfg(not(target_arch = "x86_64"))]
                const ARCHITECTURE: &str = "i686";

                format!(
                    "Mozilla/5.0 (X11; Linux {ARCHITECTURE}; rv:128.0) Servo/{SERVO_VERSION} Firefox/128.0"
                )
            },
            UserAgentPlatform::Android => {
                format!(
                    "Mozilla/5.0 (Android 10; Mobile; rv:128.0) Servo/{SERVO_VERSION} Firefox/128.0"
                )
            },
            UserAgentPlatform::OpenHarmony => format!(
                "Mozilla/5.0 (OpenHarmony; Mobile; rv:128.0) Servo/{SERVO_VERSION} Firefox/128.0"
            ),
            UserAgentPlatform::Ios => format!(
                "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X; rv:128.0) Servo/{SERVO_VERSION} Firefox/128.0"
            ),
        }
    }
}
