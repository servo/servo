/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Message router C API for bidirectional native↔JS messaging.

use std::ffi::{CStr, c_char, c_void};

use servo_api::WebView;

use crate::ServoError;

/// A callback invoked when a message is received from JavaScript.
///
/// `name` is the message name (NUL-terminated UTF-8 string).
/// `data_json` is the message data as a JSON string (NUL-terminated UTF-8 string).
/// `user_data` is the opaque pointer passed to `servo_webview_add_message_handler`.
///
/// The strings are owned by Servo and are only valid for the duration of the
/// callback. The embedder must copy any data it needs to retain.
///
/// # Safety
///
/// If non-`NULL`, `callback` must be a valid C ABI function matching this
/// signature, must remain valid until the handler is removed or the `WebView`
/// is freed, and must not unwind across the FFI boundary.
pub type ServoMessageCallback = Option<
    unsafe extern "C" fn(name: *const c_char, data_json: *const c_char, user_data: *mut c_void),
>;

/// Register a handler for messages from JavaScript (JS→native).
///
/// When JavaScript calls `window.servoMessageRouter.postMessage(name, data)`,
/// `callback` will be invoked with the message name and data serialized as JSON.
///
/// Multiple handlers can be registered for the same message name; they will be
/// called in registration order.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `name` is a NUL-terminated UTF-8 string identifying the message type.
///
/// `callback` is the function to invoke when a message with the given name
/// is received from JavaScript. If `NULL`, the request is ignored.
///
/// `user_data` is an opaque pointer passed to `callback`. Its ownership and
/// validity are the caller's responsibility; it must remain valid until the
/// handler is removed or the `WebView` is freed.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `webview` is a non-null pointer to a `WebView` previously returned by
///   `servo_webview_builder_build` and not yet freed. No other code may read
///   or write `*webview` for the duration of this call.
/// - `name` is a non-null pointer to a C string that remains unmodified for
///   the duration of the call.
/// - If `callback` is not `NULL`, it is a valid C ABI function matching the
///   signature, remains valid until the handler is removed or the `WebView`
///   is freed, and does not unwind across the FFI boundary.
/// - `user_data` is either null or remains valid until the handler is removed
///   or the `WebView` is freed.
/// - The call is made from the same thread that created the `WebView`.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` or `name` is null.
/// Returns `ServoError::InvalidUtf8` if `name` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_add_message_handler(
    webview: *mut WebView,
    name: *const c_char,
    callback: ServoMessageCallback,
    user_data: *mut c_void,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }
        if name.is_null() {
            return ServoError::NullArgument;
        }

        let Some(callback) = callback else {
            return ServoError::Ok;
        };

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let webview = unsafe { &*webview };

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `name` documented above.
        let name = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };

        let message_router = webview.message_router();
        message_router.add_message_handler(name, move |msg_name, data_json| {
            // SAFETY: The caller is assumed to uphold the safety requirements
            // for `callback` and `user_data` documented above.
            unsafe {
                let name_c = std::ffi::CString::new(msg_name).unwrap_or_default();
                let data_c = std::ffi::CString::new(data_json).unwrap_or_default();
                callback(name_c.as_ptr(), data_c.as_ptr(), user_data);
            }
        });

        ServoError::Ok
    })
}

/// Remove all handlers for a given message name.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `name` is a NUL-terminated UTF-8 string identifying the message type.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `webview` is a non-null pointer to a `WebView` previously returned by
///   `servo_webview_builder_build` and not yet freed. No other code may read
///   or write `*webview` for the duration of this call.
/// - `name` is a non-null pointer to a C string that remains unmodified for
///   the duration of the call.
/// - The call is made from the same thread that created the `WebView`.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` or `name` is null.
/// Returns `ServoError::InvalidUtf8` if `name` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_remove_message_handler(
    webview: *mut WebView,
    name: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }
        if name.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let webview = unsafe { &*webview };

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `name` documented above.
        let name = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8,
        };

        webview.remove_message_handler(name);

        ServoError::Ok
    })
}

/// Send a message from native to JavaScript (native→JS).
///
/// This dispatches a `CustomEvent` named `servo-message` on `window`
/// with `detail: { name, data }`. JavaScript code can listen for these
/// messages via `window.servoMessageRouter.addEventListener(name, handler)`.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `name` is a NUL-terminated UTF-8 string identifying the message type.
///
/// `data_json` is a NUL-terminated UTF-8 string containing the message data
/// serialized as JSON.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `webview` is a non-null pointer to a `WebView` previously returned by
///   `servo_webview_builder_build` and not yet freed. No other code may read
///   or write `*webview` for the duration of this call.
/// - `name` and `data_json` are non-null pointers to C strings that remain
///   unmodified for the duration of the call.
/// - The call is made from the same thread that created the `WebView`.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview`, `name`, or `data_json` is null.
/// Returns `ServoError::InvalidUtf8` if `name` or `data_json` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_post_message(
    webview: *mut WebView,
    name: *const c_char,
    data_json: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }
        if name.is_null() {
            return ServoError::NullArgument;
        }
        if data_json.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let webview = unsafe { &*webview };

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `name` and `data_json` documented above.
        let name = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8,
        };
        let data_json = match unsafe { CStr::from_ptr(data_json) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8,
        };

        webview.post_message(name, data_json);

        ServoError::Ok
    })
}

/// Poll for messages from JavaScript (JS→native).
///
/// This should be called periodically by the embedder (e.g., during the event loop)
/// to retrieve messages that JavaScript has sent via `window.servoMessageRouter.postMessage`.
/// The messages are dispatched to handlers registered via `servo_webview_add_message_handler`.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `webview` is a non-null pointer to a `WebView` previously returned by
///   `servo_webview_builder_build` and not yet freed. No other code may read
///   or write `*webview` for the duration of this call.
/// - The call is made from the same thread that created the `WebView`.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_poll_js_messages(webview: *mut WebView) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let webview = unsafe { &*webview };

        webview.poll_js_messages();

        ServoError::Ok
    })
}
