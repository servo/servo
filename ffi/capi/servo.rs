/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CStr;
use std::os::raw::c_char;

use servo_api::{PrefValue, Servo};

use crate::ServoError;

/// Destroys the `Servo` instance returned by `servo_builder_build` and
/// frees its memory.
///
/// `servo` is a handle to a `Servo` object.
/// The ownership of `servo` is transferred to the function. The caller
/// must not use or free `servo` again.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `servo` was previously returned by `servo_builder_build` and has
///   not yet been freed nor passed to another API that takes ownership
///   of it.
/// - The call is made from the same thread that originally created the
///   `Servo` instance via `servo_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_free(servo: *mut Servo) -> ServoError {
    ServoError::catch_ffi(|| {
        if servo.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `servo` documented above.
        unsafe {
            let _: Box<Servo> = Box::from_raw(servo);
        }

        ServoError::Ok
    })
}

/// Spin the Servo event loop once. The embedder should call this
/// periodically to process incoming messages and perform rendering
/// updates.
///
/// `servo` is a handle to a `Servo` object.
/// The ownership of `servo` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `servo` is a non-null pointer to a `Servo` instance previously
///   returned by `servo_builder_build` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - The call is made from the same thread that originally created the
///   `Servo` instance via `servo_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_spin_event_loop(servo: *mut Servo) -> ServoError {
    ServoError::catch_ffi(|| {
        if servo.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `servo` documented above.
        let servo = unsafe { &*servo };

        servo.spin_event_loop();

        ServoError::Ok
    })
}

/// Initialize logging for the Servo instance.
///
/// `servo` is a handle to a `Servo` object.
/// The ownership of `servo` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `servo` is a non-null pointer to a `Servo` instance previously
///   returned by `servo_builder_build` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - The call is made from the same thread that originally created the
///   `Servo` instance via `servo_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_setup_logging(servo: *mut Servo) -> ServoError {
    ServoError::catch_ffi(|| {
        if servo.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `servo` documented above.
        let servo = unsafe { &*servo };

        servo.setup_logging();

        ServoError::Ok
    })
}

/// Sets a boolean preference at runtime.
///
/// `servo` is a handle to a `Servo` object.
/// The ownership of `servo` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string naming a known preference.
/// The ownership of `name` remains with the caller after the call.
/// If the preference does not exist or has a different type, Servo logs
/// an error and leaves the preference unchanged.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `servo` is a non-null pointer to a `Servo` instance previously returned
///   by `servo_builder_build` and not yet freed nor passed to another API
///   that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified for
///   the duration of the call.
/// - The call is made from the same thread that created the `Servo` instance.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `servo` or `name` is null.
/// Returns `ServoError::InvalidUtf8` if `name` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_set_preference_bool(
    servo: *mut Servo,
    name: *const c_char,
    value: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if servo.is_null() {
            return ServoError::NullArgument;
        }
        if name.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `name` documented above.
        let name = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8,
        };

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `servo` documented above.
        unsafe { &*servo }.set_preference(name, PrefValue::Bool(value));

        ServoError::Ok
    })
}

/// Sets a signed integer preference at runtime.
///
/// See [`servo_set_preference_bool`] for the ownership, threading, and error
/// requirements, which apply identically here.
///
/// # Safety
///
/// See [`servo_set_preference_bool`] for the safety requirements, which apply
/// identically here.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `servo` or `name` is null.
/// Returns `ServoError::InvalidUtf8` if `name` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_set_preference_i64(
    servo: *mut Servo,
    name: *const c_char,
    value: i64,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if servo.is_null() {
            return ServoError::NullArgument;
        }
        if name.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `name` documented above.
        let name = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8,
        };

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `servo` documented above.
        unsafe { &*servo }.set_preference(name, PrefValue::Int(value));

        ServoError::Ok
    })
}

/// Sets an unsigned integer preference at runtime.
///
/// See [`servo_set_preference_bool`] for the ownership, threading, and error
/// requirements, which apply identically here.
///
/// # Safety
///
/// See [`servo_set_preference_bool`] for the safety requirements, which apply
/// identically here.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `servo` or `name` is null.
/// Returns `ServoError::InvalidUtf8` if `name` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_set_preference_u64(
    servo: *mut Servo,
    name: *const c_char,
    value: u64,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if servo.is_null() {
            return ServoError::NullArgument;
        }
        if name.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `name` documented above.
        let name = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8,
        };

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `servo` documented above.
        unsafe { &*servo }.set_preference(name, PrefValue::UInt(value));

        ServoError::Ok
    })
}

/// Sets a string preference at runtime.
///
/// See [`servo_set_preference_bool`] for the ownership, threading, and error
/// requirements, which apply identically here.
///
/// # Safety
///
/// See [`servo_set_preference_bool`] for the safety requirements, which apply
/// identically here.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `servo`, `name`, or `value` is null.
/// Returns `ServoError::InvalidUtf8` if `name` or `value` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_set_preference_string(
    servo: *mut Servo,
    name: *const c_char,
    value: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if servo.is_null() {
            return ServoError::NullArgument;
        }
        if name.is_null() {
            return ServoError::NullArgument;
        }
        if value.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `name` documented above.
        let name = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `value` documented above.
        let value = match unsafe { CStr::from_ptr(value) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `servo` documented above.
        unsafe { &*servo }.set_preference(&name, PrefValue::Str(value));

        ServoError::Ok
    })
}
