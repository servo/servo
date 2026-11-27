/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::os::raw::c_char;

use servo_api::{JSValue, JavaScriptEvaluationError};

/// An opaque struct representing a JavaScript value produced by
/// evaluating a script with `servo_webview_evaluate_javascript`.
/// Refer to the documentation of the corresponding
/// [servo::JSValue] enum in Rust API for more information.
/// [servo::JSValue]: https://doc.servo.org/servo/enum.JSValue.html
///
/// Handles to values are only passed to the callback of
/// `servo_webview_evaluate_javascript` and are valid only for the
/// duration of that callback, as are all values and strings reached
/// from them.
// cbindgen:no-export
pub type ServoJSValue = JSValue;

/// The kind of a `ServoJSValue`.
#[repr(i32)]
pub enum ServoJSValueKind {
    Undefined,
    Null,
    Boolean,
    Number,
    String,
    Element,
    ShadowRoot,
    Frame,
    Window,
    Array,
    Object,
}

/// An error that happens when trying to evaluate JavaScript on a `WebView`.
#[repr(i32)]
pub enum ServoJavaScriptEvaluationError {
    /// The `Document` of frame that the script was going to execute in no longer exists.
    DocumentNotFound,
    /// The script could not be compiled.
    CompilationFailure,
    /// The script could not be evaluated.
    EvaluationFailure,
    /// An internal Servo error prevented the JavaSript evaluation from completing properly.
    /// This indicates a bug in Servo.
    InternalError,
    /// The `WebView` on which this evaluation request was triggered is not ready. This might
    /// happen if the `WebView`'s `Document` is changing due to ongoing load events, for instance.
    WebViewNotReady,
    /// The script executed successfully, but Servo could not serialize the JavaScript return
    /// value into a [`JSValue`].
    SerializationError,
}

impl From<&JavaScriptEvaluationError> for ServoJavaScriptEvaluationError {
    fn from(error: &JavaScriptEvaluationError) -> Self {
        match error {
            JavaScriptEvaluationError::DocumentNotFound => Self::DocumentNotFound,
            JavaScriptEvaluationError::CompilationFailure => Self::CompilationFailure,
            JavaScriptEvaluationError::EvaluationFailure(_) => Self::EvaluationFailure,
            JavaScriptEvaluationError::InternalError => Self::InternalError,
            JavaScriptEvaluationError::WebViewNotReady => Self::WebViewNotReady,
            JavaScriptEvaluationError::SerializationError(_) => Self::SerializationError,
        }
    }
}

/// Gets the kind of `value`.
///
/// # Safety
///
/// The caller must ensure that `value` is a non-null pointer to a
/// `ServoJSValue` that is still valid, as documented on `ServoJSValue`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_js_value_kind(value: *const ServoJSValue) -> ServoJSValueKind {
    assert!(!value.is_null(), "value pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `value` documented above.
    match unsafe { &*value } {
        JSValue::Undefined => ServoJSValueKind::Undefined,
        JSValue::Null => ServoJSValueKind::Null,
        JSValue::Boolean(_) => ServoJSValueKind::Boolean,
        JSValue::Number(_) => ServoJSValueKind::Number,
        JSValue::String(_) => ServoJSValueKind::String,
        JSValue::Element(_) => ServoJSValueKind::Element,
        JSValue::ShadowRoot(_) => ServoJSValueKind::ShadowRoot,
        JSValue::Frame(_) => ServoJSValueKind::Frame,
        JSValue::Window(_) => ServoJSValueKind::Window,
        JSValue::Array(_) => ServoJSValueKind::Array,
        JSValue::Object(_) => ServoJSValueKind::Object,
    }
}

/// Gets the value of a boolean `value`.
///
/// The function panics if `value` is not a boolean.
///
/// # Safety
///
/// The caller must ensure that `value` is a non-null pointer to a
/// `ServoJSValue` that is still valid, as documented on `ServoJSValue`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_js_value_boolean(value: *const ServoJSValue) -> bool {
    assert!(!value.is_null(), "value pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `value` documented above.
    match unsafe { &*value } {
        JSValue::Boolean(value) => *value,
        _ => panic!("value is not a boolean"),
    }
}

/// Gets the value of a number `value`.
///
/// The function panics if `value` is not a number.
///
/// # Safety
///
/// The caller must ensure that `value` is a non-null pointer to a
/// `ServoJSValue` that is still valid, as documented on `ServoJSValue`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_js_value_number(value: *const ServoJSValue) -> f64 {
    assert!(!value.is_null(), "value pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `value` documented above.
    match unsafe { &*value } {
        JSValue::Number(value) => *value,
        _ => panic!("value is not a number"),
    }
}

/// Gets the string of a string `value`, or the identifier of an element,
/// shadow root, frame or window `value`.
///
/// Returns a pointer to UTF-8 bytes that are not NUL terminated and
/// writes their length in bytes to `length`. The bytes are owned by
/// `value` and valid for as long as it is.
///
/// The function panics if `value` is not one of those kinds.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `value` is a non-null pointer to a `ServoJSValue` that is still
///   valid, as documented on `ServoJSValue`.
/// - `length` is a non-null pointer to writable memory for the length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_js_value_string(
    value: *const ServoJSValue,
    length: *mut usize,
) -> *const c_char {
    assert!(!value.is_null(), "value pointer must not be null");
    assert!(!length.is_null(), "length pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `value` documented above.
    let string = match unsafe { &*value } {
        JSValue::String(string) |
        JSValue::Element(string) |
        JSValue::ShadowRoot(string) |
        JSValue::Frame(string) |
        JSValue::Window(string) => string,
        _ => panic!("value is not a string, element, shadow root, frame or window"),
    };

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `length` documented above.
    unsafe { length.write(string.len()) };
    string.as_ptr() as *const c_char
}

/// Gets the number of items of an array `value`, or of entries of an
/// object `value`.
///
/// The function panics if `value` is not an array or an object.
///
/// # Safety
///
/// The caller must ensure that `value` is a non-null pointer to a
/// `ServoJSValue` that is still valid, as documented on `ServoJSValue`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_js_value_length(value: *const ServoJSValue) -> usize {
    assert!(!value.is_null(), "value pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `value` documented above.
    match unsafe { &*value } {
        JSValue::Array(items) => items.len(),
        JSValue::Object(entries) => entries.len(),
        _ => panic!("value is not an array or an object"),
    }
}

/// Gets the item at `index` of an array `value`.
///
/// Returns a handle to the item, which is owned by `value` and valid
/// for as long as it is.
///
/// The function panics if `value` is not an array or `index` is not
/// below its length.
///
/// # Safety
///
/// The caller must ensure that `value` is a non-null pointer to a
/// `ServoJSValue` that is still valid, as documented on `ServoJSValue`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_js_value_array_item(
    value: *const ServoJSValue,
    index: usize,
) -> *const ServoJSValue {
    assert!(!value.is_null(), "value pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `value` documented above.
    match unsafe { &*value } {
        JSValue::Array(items) => &items[index],
        _ => panic!("value is not an array"),
    }
}

/// Gets the key of the entry at `index` of an object `value`. Entries are
/// in no particular order, but in the same order as for
/// `servo_js_value_object_value`.
///
/// Returns a pointer to UTF-8 bytes that are not NUL terminated and
/// writes their length in bytes to `length`. The bytes are owned by
/// `value` and valid for as long as it is.
///
/// The function panics if `value` is not an object or `index` is not
/// below its length.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `value` is a non-null pointer to a `ServoJSValue` that is still
///   valid, as documented on `ServoJSValue`.
/// - `length` is a non-null pointer to writable memory for the length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_js_value_object_key(
    value: *const ServoJSValue,
    index: usize,
    length: *mut usize,
) -> *const c_char {
    assert!(!value.is_null(), "value pointer must not be null");
    assert!(!length.is_null(), "length pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `value` documented above.
    let key = match unsafe { &*value } {
        JSValue::Object(entries) => entries.keys().nth(index).expect("index out of bounds"),
        _ => panic!("value is not an object"),
    };

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `length` documented above.
    unsafe { length.write(key.len()) };
    key.as_ptr() as *const c_char
}

/// Gets the value of the entry at `index` of an object `value`. Entries
/// are in no particular order, but in the same order as for
/// `servo_js_value_object_key`.
///
/// Returns a handle to the entry's value, which is owned by `value` and
/// valid for as long as it is.
///
/// The function panics if `value` is not an object or `index` is not
/// below its length.
///
/// # Safety
///
/// The caller must ensure that `value` is a non-null pointer to a
/// `ServoJSValue` that is still valid, as documented on `ServoJSValue`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_js_value_object_value(
    value: *const ServoJSValue,
    index: usize,
) -> *const ServoJSValue {
    assert!(!value.is_null(), "value pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `value` documented above.
    match unsafe { &*value } {
        JSValue::Object(entries) => entries.values().nth(index).expect("index out of bounds"),
        _ => panic!("value is not an object"),
    }
}
