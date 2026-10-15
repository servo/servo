/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::c_char;

use servo_api::Servo;

use crate::{ServoError, value::write_json_string, write_c_string};

/// Opaque handle for the NetworkManager.
///
/// Obtained via `servo_get_network_manager(servo)`. The handle is tied to
/// the `Servo` instance lifetime and must not be used after `servo_free` is
/// called.
///
/// # Thread safety
///
/// The handle must be used only from the thread that created the `Servo`
/// instance.
// cbindgen:opaque
pub struct ServoNetworkManager {
    servo: Servo,
}

/// Gets the NetworkManager handle from a `Servo` instance.
///
/// `servo` is a handle to a `Servo` object. The ownership of `servo` remains
/// with the caller after the call.
///
/// Returns a newly allocated `ServoNetworkManager` handle, or `NULL` on
/// failure. The caller owns the returned handle and must free it with
/// [`servo_network_manager_free`].
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `servo` is a non-null pointer to a `Servo` instance previously
///   returned by `servo_builder_build` and has not yet been freed.
/// - The call is made from the same thread that created the `Servo` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_get_network_manager(
    servo: *mut Servo,
) -> *mut ServoNetworkManager {
    ServoError::catch_ffi_ptr(|| {
        if servo.is_null() {
            return std::ptr::null_mut();
        }

        // SAFETY: The caller is assumed to uphold the safety requirements above.
        let servo = unsafe { &*servo }.clone();

        Box::into_raw(Box::new(ServoNetworkManager { servo }))
    })
}

/// Frees the `ServoNetworkManager` handle.
///
/// `manager` is a handle to a `ServoNetworkManager` object. The ownership of
/// `manager` is transferred to the function; the caller must not use or free
/// `manager` again.
///
/// # Safety
///
/// `manager` must have been previously returned by
/// [`servo_get_network_manager`] and not yet been freed.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `manager` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_network_manager_free(
    manager: *mut ServoNetworkManager,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if manager.is_null() {
            return ServoError::NullArgument;
        }
        // SAFETY: The caller is assumed to uphold the safety requirements above.
        unsafe {
            let _ = Box::from_raw(manager);
        }
        ServoError::Ok
    })
}

/// Gets the cache entry keys (URLs) as a JSON array string using the
/// buffer-query pattern.
///
/// Writes a JSON array of strings, e.g. `["https://example.com/","https://servo.org/"]`,
/// into `buffer`. Returns the number of bytes required (including the NUL
/// terminator). If `buffer` is `NULL` or `buffer_len` is too small, nothing is
/// written and the required length is returned.
///
/// `manager` is a handle to a `ServoNetworkManager` object. The ownership
/// remains with the caller.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `manager` is a non-null pointer previously returned by
///   [`servo_get_network_manager`] and not yet freed.
/// - If `buffer` is non-`NULL`, it points to at least `buffer_len` writable
///   bytes.
/// - The call is made from the same thread that created the `Servo` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_network_manager_get_cache_entries(
    manager: *const ServoNetworkManager,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    if manager.is_null() {
        return 0;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements above.
    let manager = unsafe { &*manager };

    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let entries = manager.servo.network_manager().cache_entries();

        let mut output = String::new();
        output.push('[');
        for (index, entry) in entries.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            write_json_string(entry.key(), &mut output);
        }
        output.push(']');

        // SAFETY: The caller is assumed to uphold the safety requirements above.
        unsafe { write_c_string(&output, buffer, buffer_len) }
    }))
    .unwrap_or(0)
}

/// Clears the network (HTTP) cache.
///
/// Removes all cached network responses for both public and private browsing
/// contexts.
///
/// `manager` is a handle to a `ServoNetworkManager` object. The ownership
/// remains with the caller.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `manager` is a non-null pointer previously returned by
///   [`servo_get_network_manager`] and not yet freed.
/// - The call is made from the same thread that created the `Servo` instance.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `manager` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_network_manager_clear_cache(
    manager: *mut ServoNetworkManager,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if manager.is_null() {
            return ServoError::NullArgument;
        }
        // SAFETY: The caller is assumed to uphold the safety requirements above.
        let manager = unsafe { &*manager };
        manager.servo.network_manager().clear_cache();
        ServoError::Ok
    })
}
