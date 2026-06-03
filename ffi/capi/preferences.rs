/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{CStr, c_char, c_void};
use std::ptr;

use servo::{PrefValue, Preferences};

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
#[unsafe(no_mangle)]
pub extern "C" fn servo_preferences_create() -> *mut ServoPreferences {
    let prefs = Box::new(servo::Preferences::default());
    Box::into_raw(prefs)
}

/// Destroys `prefs` and frees its memory.
///
/// `prefs` is a handle to a `ServoPreferences` object.
/// The ownership of `prefs` is transferred to the function. The caller
/// must not use or free `prefs` again.
///
/// # Safety
///
/// The caller must ensure that `prefs` was previously returned by
/// `servo_preferences_create` and has not yet been freed nor passed to
/// another API that takes ownership of it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_free(prefs: *mut ServoPreferences) {
    assert!(!prefs.is_null(), "prefs pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `prefs` documented above.
    unsafe {
        let _ = Box::from_raw(prefs);
    }
}

/// Sets a boolean preference by name.
///
/// `prefs` is a handle to a `ServoPreferences` object.
/// The ownership of `prefs` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `prefs` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_set_bool(
    prefs: *mut ServoPreferences,
    name: *const c_char,
    value: bool,
) {
    assert!(!prefs.is_null(), "prefs pointer must not be null");
    assert!(!name.is_null(), "name pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `name` documented above.
    let name = unsafe { CStr::from_ptr(name) }
        .to_str()
        .expect(INVALID_PREFERENCE_NAME_MSG);
    assert!(Preferences::exists(name), "unknown preference: {name}");
    assert_eq!(
        Preferences::type_of(name),
        "bool",
        "preference '{name}' is not a bool"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `prefs` documented above.
    unsafe { &mut *prefs }.set_value(name, PrefValue::Bool(value));
}

/// Gets a boolean preference by name.
///
/// `prefs` is a handle to a `ServoPreferences` object.
/// The ownership of `prefs` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `prefs` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_get_bool(
    prefs: *const ServoPreferences,
    name: *const c_char,
) -> bool {
    assert!(!prefs.is_null(), "prefs pointer must not be null");
    assert!(!name.is_null(), "name pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `name` documented above.
    let name = unsafe { CStr::from_ptr(name) }
        .to_str()
        .expect(INVALID_PREFERENCE_NAME_MSG);
    assert!(Preferences::exists(name), "unknown preference: {name}");
    assert_eq!(
        Preferences::type_of(name),
        "bool",
        "preference '{name}' is not a bool"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `prefs` documented above.
    match unsafe { &*prefs }.get_value(name) {
        PrefValue::Bool(value) => value,
        _ => unreachable!(),
    }
}

/// Sets a string preference by name.
///
/// `prefs` is a handle to a `ServoPreferences` object.
/// The ownership of `prefs` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// `value` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `value` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `prefs` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` and `value` are non-null pointers to C strings that remain
///   unmodified for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_set_string(
    prefs: *mut ServoPreferences,
    name: *const c_char,
    value: *const c_char,
) {
    assert!(!prefs.is_null(), "prefs pointer must not be null");
    assert!(!name.is_null(), "name pointer must not be null");
    assert!(!value.is_null(), "value pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `name` documented above.
    let name = unsafe { CStr::from_ptr(name) }
        .to_str()
        .expect(INVALID_PREFERENCE_NAME_MSG);

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `value` documented above.
    let value = unsafe { CStr::from_ptr(value) }
        .to_str()
        .expect("invalid string value")
        .to_string();
    assert!(Preferences::exists(name), "unknown preference: {name}");
    assert_eq!(
        Preferences::type_of(name),
        "alloc::string::String",
        "preference '{name}' is not a string"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `prefs` documented above.
    unsafe { &mut *prefs }.set_value(name, PrefValue::Str(value));
}

/// Gets a string preference by name.
///
/// `prefs` is a handle to a `ServoPreferences` object.
/// The ownership of `prefs` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// Returns a newly allocated NUL terminated UTF-8 string.
/// The ownership of the returned string is transferred to the caller, who
/// must free it using the system allocator i.e., C standard library's `free()`.
/// This can be NULL if allocation fails or the string is large enough
/// to cause an overflow.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `prefs` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_get_string(
    prefs: *const ServoPreferences,
    name: *const c_char,
) -> *mut c_char {
    assert!(!prefs.is_null(), "prefs pointer must not be null");
    assert!(!name.is_null(), "name pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `name` documented above.
    let name = unsafe { CStr::from_ptr(name) }
        .to_str()
        .expect(INVALID_PREFERENCE_NAME_MSG);
    assert!(Preferences::exists(name), "unknown preference: {name}");
    assert_eq!(
        Preferences::type_of(name),
        "alloc::string::String",
        "preference '{name}' is not a string"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `prefs` documented above.
    match unsafe { &*prefs }.get_value(name) {
        PrefValue::Str(value) => create_libc_string(&value),
        _ => unreachable!(),
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
/// `prefs` is a handle to a `ServoPreferences` object.
/// The ownership of `prefs` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `prefs` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_set_i64(
    prefs: *mut ServoPreferences,
    name: *const c_char,
    value: i64,
) {
    assert!(!prefs.is_null(), "prefs pointer must not be null");
    assert!(!name.is_null(), "name pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `name` documented above.
    let name = unsafe { CStr::from_ptr(name) }
        .to_str()
        .expect(INVALID_PREFERENCE_NAME_MSG);
    assert!(Preferences::exists(name), "unknown preference: {name}");
    assert_eq!(
        Preferences::type_of(name),
        "i64",
        "preference '{name}' is not an i64"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `prefs` documented above.
    unsafe { &mut *prefs }.set_value(name, PrefValue::Int(value));
}

/// Gets an i64 preference by name.
///
/// `prefs` is a handle to a `ServoPreferences` object.
/// The ownership of `prefs` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `prefs` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_get_i64(
    prefs: *const ServoPreferences,
    name: *const c_char,
) -> i64 {
    assert!(!prefs.is_null(), "prefs pointer must not be null");
    assert!(!name.is_null(), "name pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `name` documented above.
    let name = unsafe { CStr::from_ptr(name) }
        .to_str()
        .expect(INVALID_PREFERENCE_NAME_MSG);
    assert!(Preferences::exists(name), "unknown preference: {name}");
    assert_eq!(
        Preferences::type_of(name),
        "i64",
        "preference '{name}' is not an i64"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `prefs` documented above.
    match unsafe { &*prefs }.get_value(name) {
        PrefValue::Int(value) => value,
        _ => unreachable!(),
    }
}

/// Sets a u64 preference by name.
///
/// `prefs` is a handle to a `ServoPreferences` object.
/// The ownership of `prefs` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `prefs` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_set_u64(
    prefs: *mut ServoPreferences,
    name: *const c_char,
    value: u64,
) {
    assert!(!prefs.is_null(), "prefs pointer must not be null");
    assert!(!name.is_null(), "name pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `name` documented above.
    let name = unsafe { CStr::from_ptr(name) }
        .to_str()
        .expect(INVALID_PREFERENCE_NAME_MSG);
    assert!(Preferences::exists(name), "unknown preference: {name}");
    assert_eq!(
        Preferences::type_of(name),
        "u64",
        "preference '{name}' is not a u64"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `prefs` documented above.
    unsafe { &mut *prefs }.set_value(name, PrefValue::UInt(value));
}

/// Gets a u64 preference by name.
///
/// `prefs` is a handle to a `ServoPreferences` object.
/// The ownership of `prefs` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `prefs` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_get_u64(
    prefs: *const ServoPreferences,
    name: *const c_char,
) -> u64 {
    assert!(!prefs.is_null(), "prefs pointer must not be null");
    assert!(!name.is_null(), "name pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `name` documented above.
    let name = unsafe { CStr::from_ptr(name) }
        .to_str()
        .expect(INVALID_PREFERENCE_NAME_MSG);
    assert!(Preferences::exists(name), "unknown preference: {name}");
    assert_eq!(
        Preferences::type_of(name),
        "u64",
        "preference '{name}' is not a u64"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `prefs` documented above.
    match unsafe { &*prefs }.get_value(name) {
        PrefValue::UInt(value) => value,
        _ => unreachable!(),
    }
}

/// Sets an `[f64; 4]` preference by name.
///
/// `prefs` is a handle to a `ServoPreferences` object.
/// The ownership of `prefs` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// `value` is a pointer to an array of four `f64` values.
/// The ownership of `value` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `prefs` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
/// - `value` is a non-null, properly aligned pointer to an array of at
///   least four `f64` values that remains unmodified for the duration of
///   the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_set_f64_array_4(
    prefs: *mut ServoPreferences,
    name: *const c_char,
    value: *const f64,
) {
    assert!(!prefs.is_null(), "prefs pointer must not be null");
    assert!(!name.is_null(), "name pointer must not be null");
    assert!(!value.is_null(), "value pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `name` documented above.
    let name = unsafe { CStr::from_ptr(name) }
        .to_str()
        .expect(INVALID_PREFERENCE_NAME_MSG);

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `value` documented above.
    let slice = unsafe { std::slice::from_raw_parts(value, 4) };
    let array = vec![
        PrefValue::Float(slice[0]),
        PrefValue::Float(slice[1]),
        PrefValue::Float(slice[2]),
        PrefValue::Float(slice[3]),
    ];
    assert!(Preferences::exists(name), "unknown preference: {name}");
    assert_eq!(
        Preferences::type_of(name),
        "[f64; 4]",
        "preference '{name}' is not an [f64; 4]"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `prefs` documented above.
    unsafe { &mut *prefs }.set_value(name, PrefValue::Array(array));
}

/// Gets an `[f64; 4]` preference by name.
///
/// `prefs` is a handle to a `ServoPreferences` object.
/// The ownership of `prefs` remains with the caller after the call.
///
/// `name` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `name` remains with the caller after the call.
///
/// `output` is a pointer to a buffer with space for four `f64` values.
/// On return, the four values of the preference are written into the buffer.
/// The ownership of `output` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `prefs` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `name` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
/// - `output` is a non-null, properly aligned pointer to a buffer with
///   space for at least four `f64` values that the caller has exclusive
///   write access to for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_preferences_get_f64_array_4(
    prefs: *const ServoPreferences,
    name: *const c_char,
    output: *mut f64,
) {
    assert!(!prefs.is_null(), "prefs pointer must not be null");
    assert!(!name.is_null(), "name pointer must not be null");
    assert!(!output.is_null(), "output pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `name` documented above.
    let name = unsafe { CStr::from_ptr(name) }
        .to_str()
        .expect(INVALID_PREFERENCE_NAME_MSG);
    assert!(Preferences::exists(name), "unknown preference: {name}");
    assert_eq!(
        Preferences::type_of(name),
        "[f64; 4]",
        "preference '{name}' is not an [f64; 4]"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `prefs` documented above.
    match unsafe { &*prefs }.get_value(name) {
        PrefValue::Array(array) => {
            let values: Vec<f64> = array
                .iter()
                .map(|v| match v {
                    PrefValue::Float(f) => *f,
                    _ => unreachable!(),
                })
                .collect();
            // SAFETY: The caller is assumed to uphold the safety
            // requirements for `output` documented above.
            // The source `values` buffer is a newly allocated `Vec` of four elements.
            unsafe { ptr::copy_nonoverlapping(values.as_ptr(), output, 4) };
        },
        _ => unreachable!(),
    }
}
