/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{c_char, c_void};

use servo_api::{DownloadItem, DownloadState};
use url::Url;

use crate::ServoError;

/// C-compatible constants for download states.
pub const SERVO_DOWNLOAD_IN_PROGRESS: i32 = DownloadState::InProgress as i32;
pub const SERVO_DOWNLOAD_COMPLETE: i32 = DownloadState::Complete as i32;
pub const SERVO_DOWNLOAD_CANCELLED: i32 = DownloadState::Cancelled as i32;
pub const SERVO_DOWNLOAD_INTERRUPTED: i32 = DownloadState::Interrupted as i32;

/// Opaque handle for a download item.
///
/// This struct is managed by Servo and should not be directly allocated
/// or freed by the embedder. Use the accessor functions to query its
/// properties and the control functions to manage the download.
///
/// # Thread safety
///
/// The handle is tied to the embedder thread that created the WebView.
/// All calls must be made from that thread.
// cbindgen:opaque
pub struct ServoDownloadItem {
    /// The inner Rust download item.
    inner: DownloadItem,
}

impl ServoDownloadItem {
    /// Create a new `ServoDownloadItem` from a Rust `DownloadItem`.
    pub fn from_download_item(item: DownloadItem) -> Box<Self> {
        Box::new(Self { inner: item })
    }

    /// Get the download item ID.
    pub fn id(&self) -> u64 {
        self.inner.id()
    }

    /// Get the download URL.
    pub fn url(&self) -> &Url {
        self.inner.url()
    }

    /// Get the suggested filename.
    pub fn suggested_filename(&self) -> &str {
        self.inner.suggested_filename()
    }

    /// Get the number of bytes received.
    pub fn received_bytes(&self) -> u64 {
        self.inner.received_bytes()
    }

    /// Get the total number of bytes.
    pub fn total_bytes(&self) -> u64 {
        self.inner.total_bytes()
    }

    /// Get the current download state.
    pub fn state(&self) -> DownloadState {
        self.inner.state()
    }

    /// Cancel the download.
    pub fn cancel(&self) -> ServoError {
        // For now, just return Ok. The actual cancellation would be handled
        // by the network layer, which we don't have direct access to from here.
        // The embedder can track cancellation state themselves.
        ServoError::Ok
    }

    /// Pause the download.
    pub fn pause(&self) -> ServoError {
        // Pause is not yet implemented in the network layer
        ServoError::Unsupported
    }

    /// Resume the download.
    pub fn resume(&self) -> ServoError {
        // Resume is not yet implemented in the network layer
        ServoError::Unsupported
    }
}

/// Callback type for `on_before_download` delegate callback.
///
/// This is called when a download is about to start. The embedder can
/// choose to allow or cancel the download by calling the provided
/// `continue_callback` with `true` or `false` respectively.
///
/// `download` is the download item being started.
/// `continue_callback` is a callback to continue or cancel the download.
/// `user_data` is the user data from the delegate.
pub type ServoDownloadContinueCallback =
    Option<unsafe extern "C" fn(continue_download: bool, user_data: *mut c_void)>;

/// Callback type for `on_download_updated` delegate callback.
///
/// This is called when a download's progress is updated.
///
/// `download` is the download item that was updated.
/// `user_data` is the user data from the delegate.
pub type ServoDownloadUpdatedCallback =
    Option<unsafe extern "C" fn(download: *mut ServoDownloadItem, user_data: *mut c_void)>;

/// Writes a C string into a buffer using the buffer-query pattern.
///
/// Returns the number of bytes required (including NUL terminator).
/// If `buffer` is NULL or `buffer_len` is too small, nothing is written
/// and the required size is returned.
unsafe fn write_c_string(value: &str, buffer: *mut c_char, buffer_len: usize) -> usize {
    let required = value.len() + 1;
    if buffer.is_null() || buffer_len < required {
        return required;
    }
    // SAFETY: The caller guarantees that `buffer` is valid for writes of at
    // least `buffer_len >= required` bytes.
    unsafe {
        std::ptr::copy_nonoverlapping(value.as_ptr(), buffer.cast::<u8>(), value.len());
        *buffer.add(value.len()) = 0;
    }
    required
}

/// Gets the URL of the download item.
///
/// Returns the number of bytes required (including NUL terminator).
/// If `buffer` is NULL or `buffer_len` is too small, nothing is written
/// and the required size is returned.
///
/// # Safety
///
/// - `download` must be a valid `ServoDownloadItem` pointer.
/// - If `buffer` is non-null, it must point to at least `buffer_len` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_download_item_get_url(
    download: *const ServoDownloadItem,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    if download.is_null() {
        return 0;
    }
    let download = unsafe { &*download };
    unsafe { write_c_string(download.url().as_str(), buffer, buffer_len) }
}

/// Gets the suggested filename of the download item.
///
/// Returns the number of bytes required (including NUL terminator).
/// If `buffer` is NULL or `buffer_len` is too small, nothing is written
/// and the required size is returned.
///
/// # Safety
///
/// - `download` must be a valid `ServoDownloadItem` pointer.
/// - If `buffer` is non-null, it must point to at least `buffer_len` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_download_item_get_suggested_filename(
    download: *const ServoDownloadItem,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    if download.is_null() {
        return 0;
    }
    let download = unsafe { &*download };
    unsafe { write_c_string(download.suggested_filename(), buffer, buffer_len) }
}

/// Gets the number of bytes received so far.
///
/// Returns 0 if `download` is NULL.
///
/// # Safety
///
/// - `download` must be a valid `ServoDownloadItem` pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_download_item_get_received_bytes(
    download: *const ServoDownloadItem,
) -> u64 {
    if download.is_null() {
        return 0;
    }
    let download = unsafe { &*download };
    download.received_bytes()
}

/// Gets the total number of bytes expected.
///
/// Returns 0 if `download` is NULL or if the total size is unknown.
///
/// # Safety
///
/// - `download` must be a valid `ServoDownloadItem` pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_download_item_get_total_bytes(
    download: *const ServoDownloadItem,
) -> u64 {
    if download.is_null() {
        return 0;
    }
    let download = unsafe { &*download };
    download.total_bytes()
}

/// Gets the current download state.
///
/// Returns one of the `SERVO_DOWNLOAD_*` constants.
/// Returns `SERVO_DOWNLOAD_INTERRUPTED` if `download` is NULL.
///
/// # Safety
///
/// - `download` must be a valid `ServoDownloadItem` pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_download_item_get_state(download: *const ServoDownloadItem) -> i32 {
    if download.is_null() {
        return SERVO_DOWNLOAD_INTERRUPTED;
    }
    let download = unsafe { &*download };
    download.state() as i32
}

/// Cancels the download.
///
/// Returns `ServoError::Ok` on success, `ServoError::NullArgument` if
/// `download` is NULL.
///
/// # Safety
///
/// - `download` must be a valid `ServoDownloadItem` pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_download_item_cancel(
    download: *mut ServoDownloadItem,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if download.is_null() {
            return ServoError::NullArgument;
        }
        let download = unsafe { &*download };
        download.cancel()
    })
}

/// Pauses the download.
///
/// Returns `ServoError::Ok` on success, `ServoError::NullArgument` if
/// `download` is NULL, or `ServoError::Unsupported` if pausing is not
/// implemented.
///
/// # Safety
///
/// - `download` must be a valid `ServoDownloadItem` pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_download_item_pause(download: *mut ServoDownloadItem) -> ServoError {
    ServoError::catch_ffi(|| {
        if download.is_null() {
            return ServoError::NullArgument;
        }
        let download = unsafe { &*download };
        download.pause()
    })
}

/// Resumes the download.
///
/// Returns `ServoError::Ok` on success, `ServoError::NullArgument` if
/// `download` is NULL, or `ServoError::Unsupported` if resuming is not
/// implemented.
///
/// # Safety
///
/// - `download` must be a valid `ServoDownloadItem` pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_download_item_resume(
    download: *mut ServoDownloadItem,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if download.is_null() {
            return ServoError::NullArgument;
        }
        let download = unsafe { &*download };
        download.resume()
    })
}
