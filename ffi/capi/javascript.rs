/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! JavaScript evaluation support for the Servo C API.

use std::ffi::{CStr, CString, c_char, c_void};

use servo_api::{JSValue, WebView};

use crate::{ServoError, ServoValue};

/// A callback invoked when a JavaScript evaluation started by
/// [`servo_webview_evaluate_javascript`] completes.
///
/// Exactly one of `result_json` and `error_message` is non-`NULL`:
///
/// - On success, `result_json` points to a NUL terminated UTF-8 string
///   containing the serialized result as JSON. JavaScript `undefined` and
///   `null` are both serialized as the JSON literal `null`. `error_message`
///   is `NULL`.
/// - On failure, `result_json` is `NULL` and `error_message` points to a NUL
///   terminated UTF-8 string describing the error.
///
/// The strings are owned by Servo and are only valid for the duration of the
/// callback. The embedder must copy any data it needs to retain.
///
/// # Safety
///
/// If non-`NULL`, `callback` must be a valid C ABI function matching this
/// signature, must remain valid until it is invoked, and must not unwind
/// across the FFI boundary.
pub type ServoJavaScriptCallback = Option<
    unsafe extern "C" fn(
        result_json: *const c_char,
        error_message: *const c_char,
        user_data: *mut c_void,
    ),
>;

/// A callback invoked when a JavaScript evaluation started by
/// [`servo_webview_evaluate_javascript_v2`] completes.
///
/// On success, `result_value` points to a `ServoValue` handle representing the
/// result. The caller owns this handle and must free it with `servo_value_free`.
/// `error_message` is `NULL`.
/// On failure, `result_value` is `NULL` and `error_message` points to a NUL
/// terminated UTF-8 string describing the error.
///
/// The `error_message` string is owned by Servo and is only valid for the
/// duration of the callback. The embedder must copy any data it needs to retain.
///
/// # Safety
///
/// If non-`NULL`, `callback` must be a valid C ABI function matching this
/// signature, must remain valid until it is invoked, and must not unwind
/// across the FFI boundary.
pub type ServoJavaScriptCallbackV2 = Option<
    unsafe extern "C" fn(
        result_value: *mut ServoValue,
        error_message: *const c_char,
        user_data: *mut c_void,
    ),
>;

/// Asynchronously evaluates `script` in the context of the document currently
/// loaded in `webview`.
///
/// **Deprecated.** Prefer [`servo_webview_evaluate_javascript_v2`], which
/// delivers the result as a `ServoValue` (matching CEF's value-returning
/// `ExecuteJavaScript`) instead of a serialized JSON string. This function is
/// kept for ABI compatibility: the symbol is not removed, but new embedders
/// should use the `_v2` form.
///
/// `script` is a NUL terminated UTF-8 string. The ownership of `script`
/// remains with the caller after the call.
///
/// `callback` is invoked at most once, on the embedder thread that runs
/// `servo_spin_event_loop`, when the evaluation completes. If `callback` is
/// `NULL` the request is ignored.
///
/// `user_data` is an opaque pointer passed to `callback`. Its ownership and
/// validity are the caller's responsibility; it must remain valid until the
/// callback is invoked or the `WebView` is freed.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `webview` is a non-null pointer to a `WebView` previously returned by
///   `servo_webview_builder_build` and not yet freed. No other code may read
///   or write `*webview` for the duration of this call.
/// - `script` is a non-null pointer to a C string that remains unmodified for
///   the duration of the call.
/// - The safety requirements documented on [`ServoJavaScriptCallback`] are
///   upheld for `callback` and `user_data`.
/// - The call is made from the same thread that created the `WebView`.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` or `script` is null.
/// Returns `ServoError::InvalidUtf8` if `script` is not valid UTF-8.
#[deprecated(note = "use servo_webview_evaluate_javascript_v2, which returns a ServoValue")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_evaluate_javascript(
    webview: *mut WebView,
    script: *const c_char,
    callback: ServoJavaScriptCallback,
    user_data: *mut c_void,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }
        if script.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `script` documented above.
        let script = match unsafe { CStr::from_ptr(script) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };

        let Some(callback) = callback else {
            return ServoError::Ok;
        };

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let webview = unsafe { &*webview };

        let callback = move |result: Result<JSValue, servo_api::JavaScriptEvaluationError>| {
            match result {
                Ok(value) => {
                    let json = cstring_or_empty(jsvalue_to_json_string(&value));
                    // SAFETY: The caller is assumed to uphold the safety
                    // requirements for `callback` and `user_data` documented
                    // above. `json` is valid for the duration of the call and
                    // `error_message` is null as part of the contract.
                    unsafe { callback(json.as_ptr(), std::ptr::null(), user_data) };
                },
                Err(error) => {
                    let message = cstring_or_empty(format!("{error:?}"));
                    // SAFETY: As above, with `result_json` null as part of the
                    // contract.
                    unsafe { callback(std::ptr::null(), message.as_ptr(), user_data) };
                },
            }
        };

        webview.evaluate_javascript(script, callback);

        ServoError::Ok
    })
}

/// Asynchronously evaluates `script` in the context of the document currently
/// loaded in `webview`, returning a `ServoValue` instead of a JSON string.
///
/// This is the v2 API that returns structured value containers (CEF-compatible
/// CefValue) instead of JSON strings.
///
/// `script` is a NUL terminated UTF-8 string. The ownership of `script`
/// remains with the caller after the call.
///
/// `callback` is invoked at most once, on the embedder thread that runs
/// `servo_spin_event_loop`, when the evaluation completes. If `callback` is
/// `NULL` the request is ignored.
///
/// `user_data` is an opaque pointer passed to `callback`. Its ownership and
/// validity are the caller's responsibility; it must remain valid until the
/// callback is invoked or the `WebView` is freed.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `webview` is a non-null pointer to a `WebView` previously returned by
///   `servo_webview_builder_build` and not yet freed. No other code may read
///   or write `*webview` for the duration of this call.
/// - `script` is a non-null pointer to a C string that remains unmodified for
///   the duration of the call.
/// - The safety requirements documented on [`ServoJavaScriptCallbackV2`] are
///   upheld for `callback` and `user_data`.
/// - The call is made from the same thread that created the `WebView`.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` or `script` is null.
/// Returns `ServoError::InvalidUtf8` if `script` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_evaluate_javascript_v2(
    webview: *mut WebView,
    script: *const c_char,
    callback: ServoJavaScriptCallbackV2,
    user_data: *mut c_void,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }
        if script.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `script` documented above.
        let script = match unsafe { CStr::from_ptr(script) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };

        let Some(callback) = callback else {
            return ServoError::Ok;
        };

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let webview = unsafe { &*webview };

        let callback = move |result: Result<JSValue, servo_api::JavaScriptEvaluationError>| {
            match result {
                Ok(value) => {
                    let servo_value = Box::new(ServoValue::from_jsvalue(&value));
                    // SAFETY: The caller is assumed to uphold the safety
                    // requirements for `callback` and `user_data` documented
                    // above. `servo_value` is valid for the duration of the call
                    // and `error_message` is null as part of the contract.
                    unsafe { callback(Box::into_raw(servo_value), std::ptr::null(), user_data) };
                },
                Err(error) => {
                    let message = cstring_or_empty(format!("{error:?}"));
                    // SAFETY: As above, with `result_value` null as part of the
                    // contract.
                    unsafe { callback(std::ptr::null_mut(), message.as_ptr(), user_data) };
                },
            }
        };

        webview.evaluate_javascript(script, callback);

        ServoError::Ok
    })
}

/// Serializes a [`JSValue`] into a JSON string.
///
/// JavaScript `undefined` and `null` are both serialized as `null`, and
/// non-finite numbers are serialized as `null` to keep the output valid JSON.
fn jsvalue_to_json_string(value: &JSValue) -> String {
    let mut output = String::new();
    write_json_value(value, &mut output);
    output
}

fn write_json_value(value: &JSValue, output: &mut String) {
    match value {
        JSValue::Undefined | JSValue::Null => output.push_str("null"),
        JSValue::Boolean(value) => output.push_str(if *value { "true" } else { "false" }),
        JSValue::Number(value) if value.is_finite() => output.push_str(&value.to_string()),
        JSValue::Number(_) => output.push_str("null"),
        JSValue::String(value)
        | JSValue::Element(value)
        | JSValue::ShadowRoot(value)
        | JSValue::Frame(value)
        | JSValue::Window(value) => write_json_string(value, output),
        JSValue::Array(items) => {
            output.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_json_value(item, output);
            }
            output.push(']');
        },
        JSValue::Object(entries) => {
            output.push('{');
            for (index, (key, value)) in entries.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_json_string(key, output);
                output.push(':');
                write_json_value(value, output);
            }
            output.push('}');
        },
    }
}

fn write_json_string(value: &str, output: &mut String) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if (character as u32) < 0x20 => {
                output.push_str(&format!("\\u{:04x}", character as u32));
            },
            character => output.push(character),
        }
    }
    output.push('"');
}

/// Builds a `CString` from `value`, replacing any interior NUL bytes so that
/// the conversion can never fail.
fn cstring_or_empty(value: String) -> CString {
    match CString::new(value) {
        Ok(value) => value,
        Err(error) => {
            let bytes = error
                .into_vec()
                .into_iter()
                .map(|byte| if byte == 0 { b' ' } else { byte })
                .collect::<Vec<u8>>();
            // CString::new("") cannot fail (empty string has no interior NUL)
            CString::new(bytes).unwrap_or_default()
        },
    }
}
