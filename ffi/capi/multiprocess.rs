/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::c_char;

#[cfg(feature = "multiprocess")]
use servo::{run_content_process, create_sandbox};

/// Called by the embedder in a separate process to run the content process.
///
/// This function must be called with the `multiprocess` feature enabled.
/// The `token` is an IPC connection token passed from the browser process
/// to the content process. The embedder is responsible for launching the
/// content process and passing the token (e.g., via command line argument
/// or environment variable).
///
/// # Arguments
///
/// * `token` - A NUL-terminated UTF-8 string containing the IPC token.
///   The pointer must not be null.
///
/// # Returns
///
/// The exit code of the content process. Returns -1 if the `multiprocess`
/// feature is not enabled, or if the token is null/invalid.
///
/// # Safety
///
/// The caller must ensure that:
/// - `token` is a non-null pointer to a valid C string that remains
///   unmodified for the duration of the call.
/// - This function is called from the main thread of the content process.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_run_content_process(token: *const c_char) -> i32 {
    if token.is_null() {
        return -1;
    }

    #[cfg(feature = "multiprocess")]
    {
        // SAFETY: The caller guarantees token is a valid non-null C string.
        let token_cstr = unsafe { std::ffi::CStr::from_ptr(token) };
        let token_str = match token_cstr.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return -1,
        };

        run_content_process(token_str);
        // run_content_process never returns (it runs the content process event loop)
        // but we return 0 for type compatibility.
        0
    }

    #[cfg(not(feature = "multiprocess"))]
    {
        -1
    }
}

/// Creates and activates the sandbox for the content process.
///
/// This function should be called early in the content process initialization,
/// before any other Servo initialization. It is automatically called by
/// `servo_run_content_process` if the sandbox option is enabled, but can
/// also be called manually by embedders who need more control over the
/// initialization sequence.
///
/// # Returns
///
/// `true` if the sandbox was successfully created and activated, `false`
/// if sandboxing is not supported on the current platform, if the
/// `multiprocess` feature is not enabled, or if sandbox activation fails.
///
/// # Safety
///
/// This function must be called from the main thread of the content process,
/// before any other threads are spawned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_create_sandbox() -> bool {
    #[cfg(feature = "multiprocess")]
    {
        // SAFETY: The caller guarantees this is called from the main thread
        // before other threads are spawned.
        // On Windows, in-process mitigation policies are applied via create_sandbox(),
        // but mitigations are not a true OS sandbox, so we report false.
        #[cfg(target_os = "windows")]
        {
            let _ = std::panic::catch_unwind(|| unsafe { create_sandbox() });
            return false;
        }

        #[cfg(not(target_os = "windows"))]
        {
            if !crate::servo_capi_is_sandboxing_supported() {
                return false;
            }
            let result = std::panic::catch_unwind(|| unsafe { create_sandbox() });
            result.is_ok()
        }
    }

    #[cfg(not(feature = "multiprocess"))]
    {
        false
    }
}