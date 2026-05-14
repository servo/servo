/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{CStr, c_char, c_int};
use std::path::PathBuf;

use log::warn;

/// An opaque struct representing an Options object for configuring
/// Servo at initialization time. These configuration options cannot
/// be changed at runtime.
///
/// Handles to this object can be created using [`servo_options_create`].
///
/// # Thread safety
///
/// `ServoOptions` has no internal synchronization, so the embedder is
/// responsible for serializing access if the handle is shared between threads.
// cbindgen:no-export
pub type ServoOptions = servo::Opts;

pub use servo::DiagnosticsLoggingOption;

/// Creates a handle to new [`ServoOptions`] object populated with
/// the default values.
///
/// Returns a newly allocated `ServoOptions` handle.
/// The ownership of the returned handle is transferred to the caller, who
/// must free it using [`servo_options_free`].
#[unsafe(no_mangle)]
pub extern "C" fn servo_options_create() -> *mut ServoOptions {
    let opts = Box::new(servo::Opts::default());
    Box::into_raw(opts)
}

/// Enables and configures Servo's time profiler to write profiling data in TSV
/// format to the given file path upon Servo's termination.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// `file_path` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `file_path` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `options` is a non-null pointer to a `ServoOptions` previously
///   returned by `servo_options_create` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - `file_path` is a non-null pointer to a C string that remains
///   unmodified for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_enable_time_profiling_to_file(
    options: *mut ServoOptions,
    file_path: *const c_char,
) {
    assert!(!options.is_null(), "options pointer must not be null");
    assert!(!file_path.is_null(), "file_path pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `file_path` documented above.
    let filename = unsafe { CStr::from_ptr(file_path) }
        .to_str()
        .expect("invalid time profiler file path")
        .to_string();

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.time_profiling = Some(servo::OutputOptions::FileName(filename));
}

/// Sets the path to dump a self-contained HTML file visualizing the
/// traces as a timeline. This only has effect when time profiler
/// is enabled.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// `trace_path` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `trace_path` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `options` is a non-null pointer to a `ServoOptions` previously
///   returned by `servo_options_create` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - `trace_path` is a non-null pointer to a C string that remains
///   unmodified for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_time_profiler_trace_path(
    options: *mut ServoOptions,
    trace_path: *const c_char,
) {
    assert!(!options.is_null(), "options pointer must not be null");
    assert!(!trace_path.is_null(), "trace_path pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `trace_path` documented above.
    let trace_path = unsafe { CStr::from_ptr(trace_path) }
        .to_str()
        .expect("invalid time profiler trace path")
        .to_string();

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.time_profiler_trace_path = Some(trace_path);
}

/// Enables and configures Servo's time profiler to write profiling data to
/// stdout at the given interval in seconds.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that `options` is a non-null pointer to a
/// `ServoOptions` previously returned by `servo_options_create` and has
/// not yet been freed nor passed to another API that takes ownership of
/// it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_enable_time_profiling_to_stdout(
    options: *mut ServoOptions,
    interval_seconds: f64,
) {
    assert!(!options.is_null(), "options pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.time_profiling = Some(servo::OutputOptions::Stdout(interval_seconds));
}

/// Enables or diables the hard fail option. When enabled Servo exits
/// on thread failure instead of displaying about:failure
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that `options` is a non-null pointer to a
/// `ServoOptions` previously returned by `servo_options_create` and has
/// not yet been freed nor passed to another API that takes ownership of
/// it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_hard_fail(options: *mut ServoOptions, hard_fail: bool) {
    assert!(!options.is_null(), "options pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.hard_fail = hard_fail;
}

/// Enabled or disables the multiprocess mode.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that `options` is a non-null pointer to a
/// `ServoOptions` previously returned by `servo_options_create` and has
/// not yet been freed nor passed to another API that takes ownership of
/// it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_multiprocess(
    options: *mut ServoOptions,
    multiprocess: bool,
) {
    assert!(!options.is_null(), "options pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.multiprocess = multiprocess;
}

/// Enables or disables the force IPC option. When enabled
/// Servo uses `ipc_channel` instead of `crossbeam_channel` event in
/// singleprocess mode. Does nothing in multiprocess mode.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that `options` is a non-null pointer to a
/// `ServoOptions` previously returned by `servo_options_create` and has
/// not yet been freed nor passed to another API that takes ownership of
/// it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_force_ipc(options: *mut ServoOptions, force_ipc: bool) {
    assert!(!options.is_null(), "options pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.force_ipc = force_ipc;
}

/// Enables or disables the background hang monitor.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that `options` is a non-null pointer to a
/// `ServoOptions` previously returned by `servo_options_create` and has
/// not yet been freed nor passed to another API that takes ownership of
/// it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_background_hang_monitor(
    options: *mut ServoOptions,
    background_hang_monitor: bool,
) {
    assert!(!options.is_null(), "options pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.background_hang_monitor = background_hang_monitor;
}

/// Enables or disables Servo's sandbox.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that `options` is a non-null pointer to a
/// `ServoOptions` previously returned by `servo_options_create` and has
/// not yet been freed nor passed to another API that takes ownership of
/// it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_sandbox(options: *mut ServoOptions, sandbox: bool) {
    assert!(!options.is_null(), "options pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.sandbox = sandbox;
}

/// Enables or disables Servo's use of temporary storage. When enabled
/// data on disk will not persist across restarts.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that `options` is a non-null pointer to a
/// `ServoOptions` previously returned by `servo_options_create` and has
/// not yet been freed nor passed to another API that takes ownership of
/// it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_temporary_storage(
    options: *mut ServoOptions,
    temporary_storage: bool,
) {
    assert!(!options.is_null(), "options pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.temporary_storage = temporary_storage;
}

/// Enables or disables the option to ignore SSL certificate
/// validation errors.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that `options` is a non-null pointer to a
/// `ServoOptions` previously returned by `servo_options_create` and has
/// not yet been freed nor passed to another API that takes ownership of
/// it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_ignore_certificate_errors(
    options: *mut ServoOptions,
    ignore_certificate_errors: bool,
) {
    assert!(!options.is_null(), "options pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.ignore_certificate_errors = ignore_certificate_errors;
}

/// Enables or disables the unminification of JavaScript for debugging.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that `options` is a non-null pointer to a
/// `ServoOptions` previously returned by `servo_options_create` and has
/// not yet been freed nor passed to another API that takes ownership of
/// it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_unminify_js(
    options: *mut ServoOptions,
    unminify_js: bool,
) {
    assert!(!options.is_null(), "options pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.unminify_js = unminify_js;
}

/// Enables or disables the unminification of stylesheets for debugging.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that `options` is a non-null pointer to a
/// `ServoOptions` previously returned by `servo_options_create` and has
/// not yet been freed nor passed to another API that takes ownership of
/// it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_unminify_css(
    options: *mut ServoOptions,
    unminify_css: bool,
) {
    assert!(!options.is_null(), "options pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.unminify_css = unminify_css;
}

/// Sets the PEM-encoded SSL CA certificate store path.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// `certificate_path` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `certificate_path` remains with the caller after the
/// call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `options` is a non-null pointer to a `ServoOptions` previously
///   returned by `servo_options_create` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - `certificate_path` is a non-null pointer to a C string that remains
///   unmodified for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_certificate_path(
    options: *mut ServoOptions,
    certificate_path: *const c_char,
) {
    assert!(!options.is_null(), "options pointer must not be null");
    assert!(
        !certificate_path.is_null(),
        "certificate_path pointer must not be null"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `certificate_path` documented above.
    let path = unsafe { CStr::from_ptr(certificate_path) }
        .to_str()
        .expect("invalid certificate path")
        .to_string();

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.certificate_path = Some(path);
}

/// Sets the directory path that was created with 'unminify-js'.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// `local_script_source` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `local_script_source` remains with the caller after
/// the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `options` is a non-null pointer to a `ServoOptions` previously
///   returned by `servo_options_create` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - `local_script_source` is a non-null pointer to a C string that
///   remains unmodified for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_local_script_source(
    options: *mut ServoOptions,
    local_script_source: *const c_char,
) {
    assert!(!options.is_null(), "options pointer must not be null");
    assert!(
        !local_script_source.is_null(),
        "local_script_source pointer must not be null"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `local_script_source` documented above.
    let path = unsafe { CStr::from_ptr(local_script_source) }
        .to_str()
        .expect("invalid local script source path")
        .to_string();

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.local_script_source = Some(path);
}

/// Sets the path to load WebRender shaders from disk.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// `shaders_path` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `shaders_path` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `options` is a non-null pointer to a `ServoOptions` previously
///   returned by `servo_options_create` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - `shaders_path` is a non-null pointer to a C string that remains
///   unmodified for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_shaders_path(
    options: *mut ServoOptions,
    shaders_path: *const c_char,
) {
    assert!(!options.is_null(), "options pointer must not be null");
    assert!(
        !shaders_path.is_null(),
        "shaders_path pointer must not be null"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `shaders_path` documented above.
    let path = unsafe { CStr::from_ptr(shaders_path) }
        .to_str()
        .expect("invalid shaders path")
        .to_string();

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.shaders_path = Some(PathBuf::from(path));
}

/// Sets the path to the default config directory.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// `config_dir` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `config_dir` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `options` is a non-null pointer to a `ServoOptions` previously
///   returned by `servo_options_create` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - `config_dir` is a non-null pointer to a C string that remains
///   unmodified for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_config_dir(
    options: *mut ServoOptions,
    config_dir: *const c_char,
) {
    assert!(!options.is_null(), "options pointer must not be null");
    assert!(!config_dir.is_null(), "config_dir pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `config_dir` documented above.
    let path = unsafe { CStr::from_ptr(config_dir) }
        .to_str()
        .expect("invalid config directory path")
        .to_string();

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.config_dir = Some(PathBuf::from(path));
}

/// Sets the probability of randomly closing a pipeline. This is
/// used for testing the hardening of the constellation.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that `options` is a non-null pointer to a
/// `ServoOptions` previously returned by `servo_options_create` and has
/// not yet been freed nor passed to another API that takes ownership of
/// it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_random_pipeline_closure_probability(
    options: *mut ServoOptions,
    probability: f32,
) {
    assert!(!options.is_null(), "options pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.random_pipeline_closure_probability = Some(probability);
}

/// Sets the seed for the random number generator used to randomly close
/// pipelines. This only has effect when random pipline closure is enabled
/// using `servo_options_set_random_pipeline_closure_probability`.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that `options` is a non-null pointer to a
/// `ServoOptions` previously returned by `servo_options_create` and has
/// not yet been freed nor passed to another API that takes ownership of
/// it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_random_pipeline_closure_seed(
    options: *mut ServoOptions,
    seed: usize,
) {
    assert!(!options.is_null(), "options pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }.random_pipeline_closure_seed = Some(seed);
}

/// Enables or disables a specific diagnostics option.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// `option` must be one of the `SERVO_DIAGNOSTICS_LOGGIN_OPTION_*` constants.
///
/// # Safety
///
/// The caller must ensure that `options` is a non-null pointer to a
/// `ServoOptions` previously returned by `servo_options_create` and has
/// not yet been freed nor passed to another API that takes ownership of
/// it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_debug_option(
    options: *mut ServoOptions,
    option: c_int,
    enabled: bool,
) {
    assert!(!options.is_null(), "options pointer must not be null");
    let Ok(diagnostics_option) = DiagnosticsLoggingOption::try_from(option) else {
        warn!("invalid diagnostics option value: {option}");
        return;
    };

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `options` documented above.
    unsafe { &mut *options }
        .debug
        .toggle_option(diagnostics_option.into(), enabled);
}

/// Destroys `servo_options` and frees its memory.
///
/// `servo_options` is a handle to a `ServoOptions` object.
/// The ownership of `servo_options` is transferred to the function. The
/// caller must not use or free `servo_options` again.
///
/// # Safety
///
/// The caller must ensure that `servo_options` was previously returned by
/// `servo_options_create` and has not yet been freed nor passed to
/// another API that takes ownership of it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_free(servo_options: *mut ServoOptions) {
    assert!(
        !servo_options.is_null(),
        "servo_options pointer must not be null"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `servo_options` documented above.
    unsafe {
        let _ = Box::from_raw(servo_options);
    }
}
