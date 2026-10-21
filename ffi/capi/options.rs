/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{CStr, c_char, c_int};
use std::path::PathBuf;

use log::warn;

use crate::ServoError;

/// An opaque struct representing an Options object for configuring
/// Servo at initialization time. These configuration options cannot
/// be changed at runtime.
///
/// Handles to this object can be created using [`servo_options_create`].
///
/// # Handle validity
///
/// Every entry point that takes a `ServoOptions` handle checks it against the
/// live-handle registry (`HARDENING_PLAN.md` §B.1). A handle that was never
/// created, was already freed or consumed, or is not an options handle at all
/// returns `ServoError::InvalidArgument` instead of being dereferenced. The
/// check is a lookup on the handle's address, so it does not touch memory that
/// may no longer belong to the caller; address reuse is its one blind spot.
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
/// must free it using [`servo_options_free`]. Returns `NULL` on allocation failure.
#[unsafe(no_mangle)]
pub extern "C" fn servo_options_create() -> *mut ServoOptions {
    ServoError::catch_ffi_ptr(|| {
        let options = Box::into_raw(Box::new(servo::Opts::default()));
        // W2.5: hand the handle to the registry before the embedder can use it.
        crate::handle_registry::register(options, crate::handle_registry::HandleKind::Options);
        options
    })
}

/// Enables and configures Servo's time profiler to write profiling data in TSV
/// format to the given file path upon Servo's termination.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// `file_path` is a NUL terminated UTF-8 string.
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` or `file_path` is null.
/// Returns `ServoError::InvalidUtf8` if `file_path` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_enable_time_profiling_to_file(
    options: *mut ServoOptions,
    file_path: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }
        if file_path.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `file_path` documented above.
        let filename = match unsafe { CStr::from_ptr(file_path) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.time_profiling = Some(servo::OutputOptions::FileName(filename));

        ServoError::Ok
    })
}

/// Sets the path to dump a self-contained HTML file visualizing the
/// traces as a timeline. This only has effect when time profiler
/// is enabled.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// `trace_path` is a NUL terminated UTF-8 string.
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` or `trace_path` is null.
/// Returns `ServoError::InvalidUtf8` if `trace_path` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_time_profiler_trace_path(
    options: *mut ServoOptions,
    trace_path: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }
        if trace_path.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `trace_path` documented above.
        let trace_path = match unsafe { CStr::from_ptr(trace_path) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.time_profiler_trace_path = Some(trace_path);

        ServoError::Ok
    })
}

/// Enables and configures Servo's time profiler to write profiling data to
/// stdout at the given interval in seconds.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_enable_time_profiling_to_stdout(
    options: *mut ServoOptions,
    interval_seconds: f64,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.time_profiling =
            Some(servo::OutputOptions::Stdout(interval_seconds));

        ServoError::Ok
    })
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_hard_fail(
    options: *mut ServoOptions,
    hard_fail: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.hard_fail = hard_fail;

        ServoError::Ok
    })
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_multiprocess(
    options: *mut ServoOptions,
    multiprocess: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.multiprocess = multiprocess;

        ServoError::Ok
    })
}

/// Sets whether origin isolation is enforced. When enabled, different
/// origins within the same registrable domain (eTLD+1) run in distinct
/// event loops and content processes.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_origin_isolation(
    options: *mut ServoOptions,
    origin_isolation: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.origin_isolation = origin_isolation;

        ServoError::Ok
    })
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_force_ipc(
    options: *mut ServoOptions,
    force_ipc: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.force_ipc = force_ipc;

        ServoError::Ok
    })
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_background_hang_monitor(
    options: *mut ServoOptions,
    background_hang_monitor: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.background_hang_monitor = background_hang_monitor;

        ServoError::Ok
    })
}

/// Enables or disables automatic termination of permanently hung scripts
/// detected by the background hang monitor.
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_hang_monitor_kill_script(
    options: *mut ServoOptions,
    hang_monitor_kill_script: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.hang_monitor_kill_script = hang_monitor_kill_script;

        ServoError::Ok
    })
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_sandbox(
    options: *mut ServoOptions,
    sandbox: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.sandbox = sandbox;

        ServoError::Ok
    })
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_temporary_storage(
    options: *mut ServoOptions,
    temporary_storage: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.temporary_storage = temporary_storage;

        ServoError::Ok
    })
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_ignore_certificate_errors(
    options: *mut ServoOptions,
    ignore_certificate_errors: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.ignore_certificate_errors = ignore_certificate_errors;

        ServoError::Ok
    })
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_unminify_js(
    options: *mut ServoOptions,
    unminify_js: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.unminify_js = unminify_js;

        ServoError::Ok
    })
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_unminify_css(
    options: *mut ServoOptions,
    unminify_css: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.unminify_css = unminify_css;

        ServoError::Ok
    })
}

/// Sets the PEM-encoded SSL CA certificate store path.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// `certificate_path` is a NUL terminated UTF-8 string.
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` or `certificate_path` is null.
/// Returns `ServoError::InvalidUtf8` if `certificate_path` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_certificate_path(
    options: *mut ServoOptions,
    certificate_path: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }
        if certificate_path.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `certificate_path` documented above.
        let path = match unsafe { CStr::from_ptr(certificate_path) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.certificate_path = Some(path);

        ServoError::Ok
    })
}

/// Sets the directory path that was created with 'unminify-js'.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// `local_script_source` is a NUL terminated UTF-8 string.
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` or `local_script_source` is null.
/// Returns `ServoError::InvalidUtf8` if `local_script_source` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_local_script_source(
    options: *mut ServoOptions,
    local_script_source: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }
        if local_script_source.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `local_script_source` documented above.
        let path = match unsafe { CStr::from_ptr(local_script_source) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.local_script_source = Some(path);

        ServoError::Ok
    })
}

/// Sets the path to load WebRender shaders from disk.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// `shaders_path` is a NUL terminated UTF-8 string.
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` or `shaders_path` is null.
/// Returns `ServoError::InvalidUtf8` if `shaders_path` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_shaders_path(
    options: *mut ServoOptions,
    shaders_path: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }
        if shaders_path.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `shaders_path` documented above.
        let path = match unsafe { CStr::from_ptr(shaders_path) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.shaders_path = Some(PathBuf::from(path));

        ServoError::Ok
    })
}

/// Sets the path to the default config directory.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` remains with the caller after the call.
///
/// `config_dir` is a NUL terminated UTF-8 string.
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` or `config_dir` is null.
/// Returns `ServoError::InvalidUtf8` if `config_dir` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_config_dir(
    options: *mut ServoOptions,
    config_dir: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }
        if config_dir.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `config_dir` documented above.
        let path = match unsafe { CStr::from_ptr(config_dir) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.config_dir = Some(PathBuf::from(path));

        ServoError::Ok
    })
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_random_pipeline_closure_probability(
    options: *mut ServoOptions,
    probability: f32,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.random_pipeline_closure_probability = Some(probability);

        ServoError::Ok
    })
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_random_pipeline_closure_seed(
    options: *mut ServoOptions,
    seed: usize,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }.random_pipeline_closure_seed = Some(seed);

        ServoError::Ok
    })
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
///
/// # Errors
///
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or not an options handle at all.
/// Returns `ServoError::NullArgument` if `options` is null.
/// Returns `ServoError::InvalidState` if `option` is not a valid diagnostics option.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_set_debug_option(
    options: *mut ServoOptions,
    option: c_int,
    enabled: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: reject a handle that is not live (never created, already freed,
        // or a pointer to something that is not an options handle) instead of
        // dereferencing it.
        if !crate::handle_registry::is_live(options, crate::handle_registry::HandleKind::Options) {
            return ServoError::InvalidArgument;
        }
        let Ok(diagnostics_option) = DiagnosticsLoggingOption::try_from(option) else {
            warn!("invalid diagnostics option value: {option}");
            return ServoError::InvalidState;
        };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `options` documented above.
        unsafe { &mut *options }
            .debug
            .toggle_option(diagnostics_option.into(), enabled);

        ServoError::Ok
    })
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
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `servo_options` is null.
/// Returns `ServoError::InvalidArgument` if `servo_options` is not a live
/// handle: never created, or already freed or consumed. A second free is
/// therefore an error instead of a double free.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_options_free(servo_options: *mut ServoOptions) -> ServoError {
    ServoError::catch_ffi(|| {
        if servo_options.is_null() {
            return ServoError::NullArgument;
        }
        // W2.5: a handle that is not live was never created, or has already
        // been freed. Return instead of dropping it a second time: a double
        // free is memory corruption, not a diagnostic.
        if !crate::handle_registry::unregister(
            servo_options,
            crate::handle_registry::HandleKind::Options,
        ) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `servo_options` documented above.
        unsafe {
            let _ = Box::from_raw(servo_options);
        }

        ServoError::Ok
    })
}
