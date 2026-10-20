/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::c_char;

#[cfg(feature = "multiprocess")]
use servo::{run_content_process, create_sandbox};

/// Runs a content process that this binary was re-executed to create.
///
/// Servo launches content processes itself: the constellation spawns
/// `current_exe()` with exactly `--content-process <token>` as its arguments
/// (`components/constellation/sandboxing.rs::setup_common`), so no token ever
/// reaches the embedder and there is no process for it to launch. The embedder's
/// `main` is expected to route that argument to this function as its first
/// action; see the "Multi-process support" section of the C API guide. Nothing
/// else has to be initialised first.
///
/// # Arguments
///
/// * `token` - A NUL-terminated UTF-8 string containing the IPC token, as
///   passed on the command line by the browser process. The pointer must not
///   be null.
///
/// # Returns
///
/// The exit code of the content process, or -1 if `token` is null or not valid
/// UTF-8, if the `multiprocess` feature is not enabled, or if starting the
/// content process fails or panics. A stale or wrong token is a refused call,
/// not a crash: the handshake unwraps internally, so the panic is caught here
/// rather than reaching the FFI boundary.
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

        // `run_content_process` unwraps its IPC handshake, so a token nobody is
        // listening on panics. The hook prints the panic before unwinding; what
        // must not happen is the unwind reaching this `extern "C"` function,
        // which would abort the process. Catch it and report -1 like every other
        // refusal, the same way `servo_create_sandbox` below does. CI builds
        // `servo-capi` without this feature, so this branch is compiled there
        // but only executed by a real multiprocess embedder.
        match std::panic::catch_unwind(|| run_content_process(token_str)) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    #[cfg(not(feature = "multiprocess"))]
    {
        -1
    }
}

/// Creates and activates the sandbox for the content process.
///
/// Calling this manually is optional. `servo_run_content_process` does it
/// itself, after the options arrive from the browser process, so that the
/// decision matches what was asked for; reach for this only when the embedder
/// needs to place the step itself (a log line, a probe), and then it must be
/// the first thing the content process does, before any other Servo
/// initialization. On platforms without an OS sandbox this applies the
/// in-process mitigation policies instead and returns `false`.
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