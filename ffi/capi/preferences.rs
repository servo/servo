/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{CStr, c_char};
use std::ptr;

use servo::{PrefValue, Preferences};

use crate::ServoError;

/// An opaque struct representing a Servo preferences object.
/// Refer to the documentation of the corresponding
/// [servo::Preferences] struct in Rust API for more information.
/// [servo::Preferences]: https://doc.servo.org/servo/struct.Preferences.html
///
/// Handles to the object can be created using [`servo_preferences_create`].
///
/// # Thread safety
///
/// `ServoPreferences` has no internal synchronization, so the embedder is
/// responsible for serializing access if the handle is shared between threads.
// cbindgen:no-export
pub type ServoPreferences = servo::Preferences;

const INVALID_PREFERENCE_NAME_MSG: &str = "Preference name must be a valid UTF8 string";

/// Creates a handle to a new [`ServoPreferences`] object populated with
/// the default values.
///
/// Returns a newly allocated `ServoPreferences` handle. Returns `NULL` on allocation failure.
#[unsafe(no_mangle)]
pub extern "C" fn servo_preferences_create() -> *mut ServoPreferences {
    ServoError::catch_ffi_ptr(|| Box::into_raw(Box::new(servo::Preferences::default())))
}

/// Destroys `preferences` and frees its memory.
///
/// `preferences` is a handle to a `ServoPreferences` object.
/// The ownership of `preferences` is transferred to the function.
/// The caller must not use or free `preferences` again.
///
/// # Safety
///
/// The caller must ensure that `preferences` was previously returned by
/// `servo_preferences_create` and has not yet been freed nor passed to
/// another API that takes ownership of it.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `preferences` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_free(preferences: *mut ServoPreferences) -> ServoError {
    ServoError::catch_ffi(|| {
        if preferences.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `preferences` documented above.
        unsafe {
            let _ = Box::from_raw(preferences);
        }

        ServoError::Ok
    })
}

/// Sets a boolean preference by name.
///
/// `preferences` is a handle to a `ServoPreferences` object.
/// The ownership of `preferences` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `preferences` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `preferences` or `name` is null.
/// Returns `ServoError::InvalidUtf8` if `name` is not valid UTF-8.
/// Returns `ServoError::InvalidState` if the preference does not exist or is not a bool.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_set_bool(
    preferences: *mut ServoPreferences,
    name: *const c_char,
    value: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if preferences.is_null() {
            return ServoError::NullArgument;
        }
        if name.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `name` documented above.
        let name = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8,
        };
        if !Preferences::exists(name) {
            return ServoError::InvalidState;
        }
        if Preferences::type_of(name) != "bool" {
            return ServoError::InvalidState;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `preferences` documented above.
        unsafe { &mut *preferences }.set_value(name, PrefValue::Bool(value));

        ServoError::Ok
    })
}

/// Gets a boolean preference by name.
///
/// `preferences` is a handle to a `ServoPreferences` object.
/// The ownership of `preferences` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// Returns `false` on error (null pointer, invalid UTF-8, unknown preference,
/// or wrong type).
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `preferences` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_get_bool(
    preferences: *const ServoPreferences,
    name: *const c_char,
) -> bool {
    if preferences.is_null() {
        return false;
    }
    if name.is_null() {
        return false;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `name` documented above.
    let name = match unsafe { CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    if !Preferences::exists(name) {
        return false;
    }
    if Preferences::type_of(name) != "bool" {
        return false;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `preferences` documented above.
    match unsafe { &*preferences }.get_value(name) {
        PrefValue::Bool(value) => value,
        _ => false,
    }
}

/// Sets a string preference by name.
///
/// `preferences` is a handle to a `ServoPreferences` object.
/// The ownership of `preferences` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// `value` is a NUL terminated UTF-8 string.
/// The ownership of `value` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `preferences` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` and `value` are non-null pointers to C strings that remain
///   unmodified for the duration of the call.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `preferences`, `name`, or `value` is null.
/// Returns `ServoError::InvalidUtf8` if `name` or `value` is not valid UTF-8.
/// Returns `ServoError::InvalidState` if the preference does not exist or is not a string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_set_string(
    preferences: *mut ServoPreferences,
    name: *const c_char,
    value: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if preferences.is_null() {
            return ServoError::NullArgument;
        }
        if name.is_null() {
            return ServoError::NullArgument;
        }
        if value.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `name` documented above.
        let name = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8,
        };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `value` documented above.
        let value = match unsafe { CStr::from_ptr(value) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };
        if !Preferences::exists(name) {
            return ServoError::InvalidState;
        }
        if Preferences::type_of(name) != "alloc::string::String" {
            return ServoError::InvalidState;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `preferences` documented above.
        unsafe { &mut *preferences }.set_value(name, PrefValue::Str(value));

        ServoError::Ok
    })
}

/// Gets a string preference by name.
///
/// `preferences` is a handle to a `ServoPreferences` object.
/// The ownership of `preferences` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// Returns a newly allocated NUL terminated UTF-8 string.
/// The ownership of the returned string is transferred to the caller, who
/// must free it using the system allocator i.e., C standard library's `free()`.
/// Returns `NULL` on error (null pointer, invalid UTF-8, unknown preference,
/// wrong type, or allocation failure).
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `preferences` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_get_string(
    preferences: *const ServoPreferences,
    name: *const c_char,
) -> *mut c_char {
    if preferences.is_null() {
        return std::ptr::null_mut();
    }
    if name.is_null() {
        return std::ptr::null_mut();
    }

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `name` documented above.
    let name = match unsafe { CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    if !Preferences::exists(name) {
        return std::ptr::null_mut();
    }
    if Preferences::type_of(name) != "alloc::string::String" {
        return std::ptr::null_mut();
    }

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `preferences` documented above.
    match unsafe { &*preferences }.get_value(name) {
        PrefValue::Str(value) => create_libc_string(&value),
        _ => std::ptr::null_mut(),
    }
}

/// Allocates a new NUL-terminated copy of `string` using the system allocator
/// (`libc::malloc`).  Returns a NULL pointer if the allocation fails or
/// if `string` is too large.
fn create_libc_string(string: &str) -> *mut c_char {
    let Some(buffer_size) = string.len().checked_add(1) else {
        return ptr::null_mut();
    };

    // SAFETY: calling `malloc` is safe.
    let dest_buffer = unsafe { libc::malloc(buffer_size) as *mut u8 };

    if dest_buffer.is_null() {
        return ptr::null_mut();
    }

    let source_buffer = string.as_ptr();

    // SAFETY: `source_buffer` is derived from a valid Rust `&str` of length
    // `buffer_size - 1` and does not include a NUL terminator.
    // `dest_buffer` is a freshly allocated, non-null memory block of
    // size `buffer_size` and includes space for the NUL terminator.
    // `buffer_size - 1` is valid because we used `checked_add` above,
    //  so `buffer_size >= 1`.
    unsafe {
        ptr::copy_nonoverlapping(source_buffer, dest_buffer, buffer_size - 1);
        dest_buffer.add(buffer_size - 1).write(0);
    }

    dest_buffer as *mut c_char
}

/// Sets an i64 preference by name.
///
/// `preferences` is a handle to a `ServoPreferences` object.
/// The ownership of `preferences` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `preferences` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `preferences` or `name` is null.
/// Returns `ServoError::InvalidUtf8` if `name` is not valid UTF-8.
/// Returns `ServoError::InvalidState` if the preference does not exist or is not an i64.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_set_i64(
    preferences: *mut ServoPreferences,
    name: *const c_char,
    value: i64,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if preferences.is_null() {
            return ServoError::NullArgument;
        }
        if name.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `name` documented above.
        let name = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8,
        };
        if !Preferences::exists(name) {
            return ServoError::InvalidState;
        }
        if Preferences::type_of(name) != "i64" {
            return ServoError::InvalidState;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `preferences` documented above.
        unsafe { &mut *preferences }.set_value(name, PrefValue::Int(value));

        ServoError::Ok
    })
}

/// Gets an i64 preference by name.
///
/// `preferences` is a handle to a `ServoPreferences` object.
/// The ownership of `preferences` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// Returns 0 on error (null pointer, invalid UTF-8, unknown preference, or wrong type).
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `preferences` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_get_i64(
    preferences: *const ServoPreferences,
    name: *const c_char,
) -> i64 {
    if preferences.is_null() {
        return 0;
    }
    if name.is_null() {
        return 0;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `name` documented above.
    let name = match unsafe { CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    if !Preferences::exists(name) {
        return 0;
    }
    if Preferences::type_of(name) != "i64" {
        return 0;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `preferences` documented above.
    match unsafe { &*preferences }.get_value(name) {
        PrefValue::Int(value) => value,
        _ => 0,
    }
}

/// Sets a u64 preference by name.
///
/// `preferences` is a handle to a `ServoPreferences` object.
/// The ownership of `preferences` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `preferences` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `preferences` or `name` is null.
/// Returns `ServoError::InvalidUtf8` if `name` is not valid UTF-8.
/// Returns `ServoError::InvalidState` if the preference does not exist or is not a u64.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_set_u64(
    preferences: *mut ServoPreferences,
    name: *const c_char,
    value: u64,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if preferences.is_null() {
            return ServoError::NullArgument;
        }
        if name.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `name` documented above.
        let name = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8,
        };
        if !Preferences::exists(name) {
            return ServoError::InvalidState;
        }
        if Preferences::type_of(name) != "u64" {
            return ServoError::InvalidState;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `preferences` documented above.
        unsafe { &mut *preferences }.set_value(name, PrefValue::UInt(value));

        ServoError::Ok
    })
}

/// Gets a u64 preference by name.
///
/// `preferences` is a handle to a `ServoPreferences` object.
/// The ownership of `preferences` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// Returns 0 on error (null pointer, invalid UTF-8, unknown preference, or wrong type).
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `preferences` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_get_u64(
    preferences: *const ServoPreferences,
    name: *const c_char,
) -> u64 {
    if preferences.is_null() {
        return 0;
    }
    if name.is_null() {
        return 0;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `name` documented above.
    let name = match unsafe { CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    if !Preferences::exists(name) {
        return 0;
    }
    if Preferences::type_of(name) != "u64" {
        return 0;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `preferences` documented above.
    match unsafe { &*preferences }.get_value(name) {
        PrefValue::UInt(value) => value,
        _ => 0,
    }
}

/// Configures the Chrome DevTools remote debugging server.
///
/// This provides parity with CEF's `CefSettings.remote_debugging_port`.
/// When enabled, the DevTools server listens on the specified `port` on
/// 127.0.0.1 and speaks the legacy Firefox Remote Debugging Protocol
/// (length-prefixed JSON packets over TCP, not HTTP and not the Chrome
/// DevTools Protocol). The server is started by `servo_builder_build` and its
/// start is reported through `ServoDelegate::notify_devtools_server_started`.
///
/// `preferences` is a handle to a `ServoPreferences` object.
/// `enabled` turns the DevTools server on or off.
/// `port` specifies the TCP port to listen on (e.g. 7000 or 9222). If 0 and enabled, default 7000 is used.
///
/// Returns `ServoError::Ok` on success, or `ServoError::NullArgument` if preferences is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_set_devtools_server(
    preferences: *mut ServoPreferences,
    enabled: bool,
    port: u16,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if preferences.is_null() {
            return ServoError::NullArgument;
        }

        let prefs = unsafe { &mut *preferences };
        prefs.devtools_server_enabled = enabled;
        let effective_port = if port == 0 { 7000 } else { port };
        prefs.devtools_server_listen_address = format!("127.0.0.1:{}", effective_port);

        ServoError::Ok
    })
}

/// Checks whether the DevTools server is enabled in `preferences`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_get_devtools_server_enabled(
    preferences: *const ServoPreferences,
) -> bool {
    if preferences.is_null() {
        return false;
    }
    unsafe { (*preferences).devtools_server_enabled }
}

/// Returns the configured port for the DevTools server in `preferences`, or 0 if disabled/invalid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_get_devtools_server_port(
    preferences: *const ServoPreferences,
) -> u16 {
    if preferences.is_null() {
        return 0;
    }
    let prefs = unsafe { &*preferences };
    if !prefs.devtools_server_enabled {
        return 0;
    }
    prefs
        .devtools_server_listen_address
        .rsplit_once(':')
        .and_then(|(_, p)| p.parse::<u16>().ok())
        .unwrap_or(7000)
}
