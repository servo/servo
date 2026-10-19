/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! System clipboard access for a C host.
//!
//! `servo_api::ClipboardDelegate` lets an embedder implement copy/paste for a
//! `WebView`. This module mirrors that Rust delegate as a C struct, and hands
//! the text request back to the embedder through an opaque handle that it
//! resolves with [`servo_webview_clipboard_request_success`] or
//! [`servo_webview_clipboard_request_failure`].

use std::ffi::{CStr, CString, c_char, c_void};

use servo_api::{ClipboardDelegate, StringRequest, WebView};

use crate::ServoError;

/// Callback type for [`ServoClipboardDelegate::clear`].
///
/// The embedder should erase the contents of the system clipboard. There is no
/// response: the engine does not wait for the clipboard to be cleared.
pub type ServoClipboardClearCallback =
    Option<unsafe extern "C" fn(webview: *mut WebView, user_data: *mut c_void)>;

/// Callback type for [`ServoClipboardDelegate::get_text`].
///
/// `request` is an opaque handle to the pending request. The embedder must
/// resolve it by calling [`servo_webview_clipboard_request_success`] with the
/// clipboard text or [`servo_webview_clipboard_request_failure`] with a
/// message. The handle is consumed by whichever of those is called, and must
/// not be used afterwards. Until one of them is called the page that asked for
/// the clipboard text stays blocked, so the embedder should resolve the request
/// from within the event loop.
pub type ServoClipboardGetTextCallback = Option<
    unsafe extern "C" fn(
        webview: *mut WebView,
        request: *mut ServoClipboardStringRequest,
        user_data: *mut c_void,
    ),
>;

/// Callback type for [`ServoClipboardDelegate::set_text`].
///
/// `text` is a NUL terminated UTF-8 string that is only valid for the duration
/// of the callback. There is no response: the engine does not wait for the
/// clipboard to be updated.
pub type ServoClipboardSetTextCallback =
    Option<unsafe extern "C" fn(webview: *mut WebView, text: *const c_char, user_data: *mut c_void)>;

/// A delegate that gives a C host control over the system clipboard.
///
/// Set it on a `WebView` with `servo_webview_builder_set_clipboard_delegate`
/// before calling `servo_webview_builder_build`. When a callback is `NULL` the
/// corresponding request is ignored: an unhandled `get_text` is failed
/// immediately so the page does not block forever.
///
/// When no clipboard delegate is set at all, Servo uses its built-in
/// implementation, which reads and writes the system clipboard directly.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ServoClipboardDelegate {
    /// Size of this struct in bytes, as seen by the caller.
    ///
    /// The caller must set this to `sizeof(ServoClipboardDelegate)`. Servo
    /// rejects delegates whose `struct_size` is smaller than the size it was
    /// compiled with, returning `ServoError::InvalidArgument`. See
    /// `ABI_STABILITY.md` §3.3.
    pub struct_size: u32,

    /// Reserved for future use; must be `0`.
    pub _reserved: u32,

    /// An opaque pointer passed to all delegate callbacks. May be `NULL`.
    pub user_data: *mut c_void,

    /// Called when the clipboard should be cleared. May be `NULL`.
    pub clear: ServoClipboardClearCallback,

    /// Called when the clipboard's text contents are requested. May be `NULL`.
    pub get_text: ServoClipboardGetTextCallback,

    /// Called when the clipboard's text contents should be replaced. May be
    /// `NULL`.
    pub set_text: ServoClipboardSetTextCallback,
}

/// An opaque handle to a pending call to `ServoClipboardDelegate::get_text`.
///
/// The embedder receives one in the `get_text` callback and must resolve it
/// with [`servo_webview_clipboard_request_success`] or
/// [`servo_webview_clipboard_request_failure`]. Passing it to either of those
/// transfers ownership; the handle must not be used afterwards.
// cbindgen:opaque
pub struct ServoClipboardStringRequest {
    request: Option<StringRequest>,
}

/// Resolves a clipboard text request with the clipboard's current text.
///
/// `request` is a handle received from
/// `ServoClipboardDelegate::get_text`. Ownership of `request` is transferred to
/// this function, which frees it; the caller must not use it again.
///
/// `text` is a NUL terminated UTF-8 string. A `NULL` `text` is treated as an
/// empty string.
///
/// Returns `ServoError::Ok` on success, `ServoError::NullArgument` if `request`
/// is `NULL`, `ServoError::InvalidUtf8` if `text` is not valid UTF-8, or
/// `ServoError::InvalidState` if the request was already resolved.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `request` is a non-null pointer previously received from a
///   `ServoClipboardDelegate::get_text` callback, and has not already been
///   passed to this function or to
///   [`servo_webview_clipboard_request_failure`].
/// - `text` is either `NULL` or a non-null pointer to a C string that remains
///   unmodified for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_clipboard_request_success(
    request: *mut ServoClipboardStringRequest,
    text: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if request.is_null() {
            return ServoError::NullArgument;
        }
        let text = if text.is_null() {
            String::new()
        } else {
            // SAFETY: The caller is assumed to uphold the safety requirements
            // for `text` documented above.
            match unsafe { CStr::from_ptr(text) }.to_str() {
                Ok(text) => text.to_owned(),
                Err(_) => return ServoError::InvalidUtf8,
            }
        };

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `request` documented above. We take ownership here, so the box is
        // freed when this call returns.
        let mut request = unsafe { Box::from_raw(request) };
        let Some(request) = request.request.take() else {
            return ServoError::InvalidState;
        };
        request.success(text);
        ServoError::Ok
    })
}

/// Resolves a clipboard text request with a failure message.
///
/// `request` is a handle received from
/// `ServoClipboardDelegate::get_text`. Ownership of `request` is transferred to
/// this function, which frees it; the caller must not use it again. The page
/// sees the request fail, exactly as if the clipboard had been unavailable.
///
/// `message` is a NUL terminated UTF-8 string. A `NULL` `message` is treated as
/// an empty string.
///
/// Returns `ServoError::Ok` on success, `ServoError::NullArgument` if `request`
/// is `NULL`, `ServoError::InvalidUtf8` if `message` is not valid UTF-8, or
/// `ServoError::InvalidState` if the request was already resolved.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `request` is a non-null pointer previously received from a
///   `ServoClipboardDelegate::get_text` callback, and has not already been
///   passed to this function or to
///   [`servo_webview_clipboard_request_success`].
/// - `message` is either `NULL` or a non-null pointer to a C string that
///   remains unmodified for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_clipboard_request_failure(
    request: *mut ServoClipboardStringRequest,
    message: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if request.is_null() {
            return ServoError::NullArgument;
        }
        let message = if message.is_null() {
            String::new()
        } else {
            // SAFETY: The caller is assumed to uphold the safety requirements
            // for `message` documented above.
            match unsafe { CStr::from_ptr(message) }.to_str() {
                Ok(message) => message.to_owned(),
                Err(_) => return ServoError::InvalidUtf8,
            }
        };

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `request` documented above. We take ownership here, so the box is
        // freed when this call returns.
        let mut request = unsafe { Box::from_raw(request) };
        let Some(request) = request.request.take() else {
            return ServoError::InvalidState;
        };
        request.failure(message);
        ServoError::Ok
    })
}

impl ClipboardDelegate for ServoClipboardDelegate {
    fn clear(&self, mut webview: WebView) {
        let Some(callback) = self.clear else {
            // No handler: there is nothing to clear.
            return;
        };
        // SAFETY: The embedder is assumed to uphold the safety requirements of
        // the `ServoClipboardDelegate` struct. `webview` is a valid handle and
        // only needs to outlive the call.
        unsafe { callback(&mut webview as *mut WebView, self.user_data) };
    }

    fn get_text(&self, mut webview: WebView, request: StringRequest) {
        let Some(callback) = self.get_text else {
            // No handler: drop the request. Its `Drop` impl fails it, so the
            // page does not block forever waiting for an answer.
            return;
        };
        let request = Box::into_raw(Box::new(ServoClipboardStringRequest {
            request: Some(request),
        }));
        // SAFETY: The embedder is assumed to uphold the safety requirements of
        // the `ServoClipboardDelegate` struct, including resolving `request`
        // with one of the request functions above.
        unsafe { callback(&mut webview as *mut WebView, request, self.user_data) };
    }

    fn set_text(&self, mut webview: WebView, new_contents: String) {
        let Some(callback) = self.set_text else {
            // No handler: there is nothing to update.
            return;
        };
        let Ok(text) = CString::new(new_contents) else {
            // The new contents contain an interior NUL, which cannot be
            // represented as a C string. There is nothing to hand over.
            return;
        };
        // SAFETY: The embedder is assumed to uphold the safety requirements of
        // the `ServoClipboardDelegate` struct. `text` outlives the call.
        unsafe { callback(&mut webview as *mut WebView, text.as_ptr(), self.user_data) };
    }
}
