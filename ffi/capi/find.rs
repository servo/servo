/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{c_char, c_void};
use std::os::raw::c_int;

use euclid::Rect;
use servo_api::WebView;
use servo_constellation_traits::FindResult as ConstellationFindResult;

use crate::ServoError;

/// Opaque struct representing a find-in-page result.
///
/// Contains information about the active match and total match count.
/// Created by Servo and passed to the `on_find_result` delegate callback.
/// The embedder must not free this struct; it is owned by Servo and
/// valid only for the duration of the callback.
// cbindgen:opaque
pub struct ServoFindResult {
    identifier: i32,
    active_match_ordinal: i32,
    number_of_matches: i32,
    selection_rect: Option<Rect<f32, euclid::UnknownUnit>>,
}

impl From<ConstellationFindResult> for ServoFindResult {
    fn from(result: ConstellationFindResult) -> Self {
        Self {
            identifier: result.identifier,
            active_match_ordinal: result.active_match_ordinal,
            number_of_matches: result.number_of_matches,
            selection_rect: result.selection_rect,
        }
    }
}

/// Get the unique identifier for this find request.
///
/// Returns the identifier that was returned by `servo_webview_find`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_find_result_get_identifier(result: *const ServoFindResult) -> c_int {
    if result.is_null() {
        return 0;
    }
    unsafe { (*result).identifier }
}

/// Get the ordinal of the active match (1-based).
///
/// Returns the 1-based index of the currently active match within
/// the total matches, or 0 if no match is active.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_find_result_get_active_match_ordinal(
    result: *const ServoFindResult,
) -> c_int {
    if result.is_null() {
        return 0;
    }
    unsafe { (*result).active_match_ordinal }
}

/// Get the total number of matches found.
///
/// Returns the total number of matches for the search text, or 0 if none.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_find_result_get_number_of_matches(
    result: *const ServoFindResult,
) -> c_int {
    if result.is_null() {
        return 0;
    }
    unsafe { (*result).number_of_matches }
}

/// Get the selection rectangle of the active match in CSS pixels.
///
/// The rectangle is relative to the viewport origin. If there is no active
/// match, all output parameters are set to 0.
///
/// `x`, `y`, `width`, `height` are output parameters. Any may be `NULL`
/// if the caller is not interested in that value.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_find_result_get_selection_rect(
    result: *const ServoFindResult,
    x: *mut f32,
    y: *mut f32,
    width: *mut f32,
    height: *mut f32,
) {
    if result.is_null() {
        if !x.is_null() {
            unsafe { *x = 0.0 };
        }
        if !y.is_null() {
            unsafe { *y = 0.0 };
        }
        if !width.is_null() {
            unsafe { *width = 0.0 };
        }
        if !height.is_null() {
            unsafe { *height = 0.0 };
        }
        return;
    }

    let result = unsafe { &*result };
    if let Some(rect) = result.selection_rect {
        if !x.is_null() {
            unsafe { *x = rect.origin.x };
        }
        if !y.is_null() {
            unsafe { *y = rect.origin.y };
        }
        if !width.is_null() {
            unsafe { *width = rect.size.width };
        }
        if !height.is_null() {
            unsafe { *height = rect.size.height };
        }
    } else {
        if !x.is_null() {
            unsafe { *x = 0.0 };
        }
        if !y.is_null() {
            unsafe { *y = 0.0 };
        }
        if !width.is_null() {
            unsafe { *width = 0.0 };
        }
        if !height.is_null() {
            unsafe { *height = 0.0 };
        }
    }
}

/// Starts a find-in-page search for the given text.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `text` is a NUL terminated UTF-8 string containing the search text.
/// The ownership of `text` remains with the caller after the call.
///
/// `forward` specifies the search direction: `true` for forward, `false` for backward.
///
/// `match_case` specifies whether the search should be case-sensitive.
///
/// Returns a unique identifier for this find request on success (positive integer),
/// or a negative `ServoError` code on failure. The identifier can be used with
/// `servo_webview_find_next` and `servo_webview_find_clear`.
///
/// The results are delivered asynchronously via the `on_find_result` delegate callback.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `webview` is a non-null pointer to a `WebView` previously returned
///   by `servo_webview_builder_build` and not yet passed to `servo_webview_free`.
/// - `text` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
/// - The call is made from the same thread that originally created the
///   `WebView` via `servo_webview_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_find(
    webview: *mut WebView,
    text: *const c_char,
    forward: bool,
    match_case: bool,
) -> c_int {
    ServoError::catch_ffi_i32(|| {
        if webview.is_null() {
            return ServoError::NullArgument as c_int;
        }
        if text.is_null() {
            return ServoError::NullArgument as c_int;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `webview` documented above.
        let webview = unsafe { &*webview };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `text` documented above.
        let text_str = match unsafe { std::ffi::CStr::from_ptr(text) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8 as c_int,
        };

        webview.find(text_str, forward, match_case)
    })
}

/// Finds the next match for an existing find request.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `identifier` is the identifier returned by a previous call to `servo_webview_find`.
///
/// `forward` specifies the search direction: `true` for forward, `false` for backward.
///
/// Returns `ServoError::Ok` (0) on success, or a negative `ServoError` code on failure.
///
/// The results are delivered asynchronously via the `on_find_result` delegate callback.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `webview` is a non-null pointer to a `WebView` previously returned
///   by `servo_webview_builder_build` and not yet passed to `servo_webview_free`.
/// - The call is made from the same thread that originally created the
///   `WebView` via `servo_webview_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_find_next(
    webview: *mut WebView,
    identifier: c_int,
    forward: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `webview` documented above.
        let webview = unsafe { &*webview };

        webview.find_next(identifier, forward);

        ServoError::Ok
    })
}

/// Clears the find results for the given identifier.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `identifier` is the identifier returned by a previous call to `servo_webview_find`.
///
/// Returns `ServoError::Ok` (0) on success, or a negative `ServoError` code on failure.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `webview` is a non-null pointer to a `WebView` previously returned
///   by `servo_webview_builder_build` and not yet passed to `servo_webview_free`.
/// - The call is made from the same thread that originally created the
///   `WebView` via `servo_webview_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_find_clear(
    webview: *mut WebView,
    identifier: c_int,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `webview` documented above.
        let webview = unsafe { &*webview };

        webview.clear_find(identifier);

        ServoError::Ok
    })
}

/// Callback type for find-in-page results.
///
/// `identifier` is the identifier of the find request.
/// `count` is the total number of matches found.
/// `result` is a pointer to a `ServoFindResult` struct containing details
/// about the active match, or `NULL` if no matches were found.
/// `user_data` is the opaque pointer that was set in the delegate.
///
/// The `result` pointer is valid only for the duration of the callback.
/// The embedder must not retain or free it.
pub type ServoFindResultCallback = Option<
    unsafe extern "C" fn(
        webview: *mut WebView,
        identifier: c_int,
        count: c_int,
        result: *const ServoFindResult,
        user_data: *mut c_void,
    ),
>;
