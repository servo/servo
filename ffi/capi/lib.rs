/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod cert_error;
mod clipboard;
mod cookie_manager;
mod download;
mod find;
mod gamepad;
mod javascript;
mod message_router;
mod multiprocess;
mod network_manager;
mod options;
mod preferences;
mod protocol_handler;
mod rendering_context;
mod request_context;
mod servo;
mod servo_delegate;
mod site_data;
mod user_content_manager;
mod value;
mod webview;
mod webview_delegate;

extern crate servo as servo_api;

use std::ffi::{CStr, c_char};
use std::rc::Rc;

use options::ServoOptions;
pub use preferences::ServoPreferences;
use servo_api::protocol_handler::ProtocolRegistry;
use servo_delegate::{ServoDelegate, make_servo_delegate};

/// Writes `value` into a caller-provided NUL-terminated UTF-8 buffer using the
/// buffer-query pattern shared by the C API's string getters.
///
/// Returns the number of bytes required to store `value` including the trailing
/// NUL terminator, so an embedder can first query the required length by
/// passing `NULL` (or a too-small buffer) and then call again with a big enough
/// buffer. Nothing is written when `buffer` is `NULL` or `buffer_len` is smaller
/// than the required size.
///
/// # Safety
///
/// If `buffer` is non-`NULL`, it must point to at least `buffer_len` writable
/// bytes. This is the single implementation behind every string getter in this
/// crate, so the safety contract only has to be upheld in one place.
pub(crate) unsafe fn write_c_string(value: &str, buffer: *mut c_char, buffer_len: usize) -> usize {
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

// Re-export cursor and console constants from webview_delegate
pub use webview_delegate::{
    SERVO_CONSOLE_LOG_LEVEL_DEBUG, SERVO_CONSOLE_LOG_LEVEL_ERROR, SERVO_CONSOLE_LOG_LEVEL_INFO,
    SERVO_CONSOLE_LOG_LEVEL_LOG, SERVO_CONSOLE_LOG_LEVEL_WARN, SERVO_CURSOR_ALIAS,
    SERVO_CURSOR_AUTO, SERVO_CURSOR_CELL, SERVO_CURSOR_COLUMN_RESIZE, SERVO_CURSOR_CONTEXT_MENU,
    SERVO_CURSOR_COPY, SERVO_CURSOR_CROSSHAIR, SERVO_CURSOR_CUSTOM, SERVO_CURSOR_EAST_PANNING,
    SERVO_CURSOR_EAST_RESIZE, SERVO_CURSOR_EAST_WEST_RESIZE, SERVO_CURSOR_GRAB,
    SERVO_CURSOR_GRABBING, SERVO_CURSOR_HAND, SERVO_CURSOR_HELP, SERVO_CURSOR_IBEAM,
    SERVO_CURSOR_MIDDLE_PANNING, SERVO_CURSOR_MOVE, SERVO_CURSOR_NO_DROP, SERVO_CURSOR_NONE,
    SERVO_CURSOR_NORTH_EAST_PANNING, SERVO_CURSOR_NORTH_EAST_RESIZE,
    SERVO_CURSOR_NORTH_EAST_SOUTH_WEST_RESIZE, SERVO_CURSOR_NORTH_PANNING,
    SERVO_CURSOR_NORTH_RESIZE, SERVO_CURSOR_NORTH_SOUTH_RESIZE, SERVO_CURSOR_NORTH_WEST_PANNING,
    SERVO_CURSOR_NORTH_WEST_RESIZE, SERVO_CURSOR_NORTH_WEST_SOUTH_EAST_RESIZE,
    SERVO_CURSOR_NOT_ALLOWED, SERVO_CURSOR_POINTER, SERVO_CURSOR_PROGRESS, SERVO_CURSOR_ROW_RESIZE,
    SERVO_CURSOR_SOUTH_EAST_PANNING, SERVO_CURSOR_SOUTH_EAST_RESIZE, SERVO_CURSOR_SOUTH_PANNING,
    SERVO_CURSOR_SOUTH_RESIZE, SERVO_CURSOR_SOUTH_WEST_PANNING, SERVO_CURSOR_SOUTH_WEST_RESIZE,
    SERVO_CURSOR_VERTICAL_TEXT, SERVO_CURSOR_WAIT, SERVO_CURSOR_WEST_PANNING,
    SERVO_CURSOR_WEST_RESIZE, ServoFilePickerCallback,
    ServoContextMenuAction, ServoContextMenuCallback, ServoContextMenuItem,
    ServoContextMenuParams, SERVO_CONTEXT_MENU_ACTION_DISMISS,
    SERVO_CONTEXT_MENU_ACTION_GO_BACK, SERVO_CONTEXT_MENU_ACTION_GO_FORWARD,
    SERVO_CONTEXT_MENU_ACTION_RELOAD, SERVO_CONTEXT_MENU_ACTION_COPY_LINK,
    SERVO_CONTEXT_MENU_ACTION_OPEN_LINK_IN_NEW_WEBVIEW,
    SERVO_CONTEXT_MENU_ACTION_COPY_IMAGE_LINK,
    SERVO_CONTEXT_MENU_ACTION_OPEN_IMAGE_IN_NEW_VIEW, SERVO_CONTEXT_MENU_ACTION_CUT,
    SERVO_CONTEXT_MENU_ACTION_COPY, SERVO_CONTEXT_MENU_ACTION_PASTE,
    SERVO_CONTEXT_MENU_ACTION_SELECT_ALL, SERVO_CONTEXT_MENU_FLAG_LINK,
    SERVO_CONTEXT_MENU_FLAG_IMAGE, SERVO_CONTEXT_MENU_FLAG_EDITABLE_TEXT,
    SERVO_CONTEXT_MENU_FLAG_SELECTION,
};

// Re-export media session constants from webview
pub use webview::{
    SERVO_MEDIA_SESSION_ACTION_NEXT_TRACK, SERVO_MEDIA_SESSION_ACTION_PAUSE,
    SERVO_MEDIA_SESSION_ACTION_PLAY, SERVO_MEDIA_SESSION_ACTION_PREVIOUS_TRACK,
    SERVO_MEDIA_SESSION_ACTION_SEEK_BACKWARD, SERVO_MEDIA_SESSION_ACTION_SEEK_FORWARD,
    SERVO_MEDIA_SESSION_ACTION_SKIP_AD, SERVO_MEDIA_SESSION_ACTION_STOP,
};

// Re-export certificate error types and constants
pub use cert_error::{
    SERVO_CERT_ERROR_COMMON_NAME_INVALID, SERVO_CERT_ERROR_EXPIRED,
    SERVO_CERT_ERROR_INCOMPLETE_CHAIN, SERVO_CERT_ERROR_INVALID, SERVO_CERT_ERROR_INVALID_USAGE,
    SERVO_CERT_ERROR_NONE, SERVO_CERT_ERROR_NOT_YET_VALID, SERVO_CERT_ERROR_OTHER,
    SERVO_CERT_ERROR_REVOKED, SERVO_CERT_ERROR_SELF_SIGNED, SERVO_CERT_ERROR_SYMANTEC_LEGACY,
    SERVO_CERT_ERROR_TRANSPARENCY_REQUIRED, SERVO_CERT_ERROR_UNKNOWN_CA, SERVO_CERT_ERROR_WEAK_KEY,
    SERVO_CERT_ERROR_WEAK_SIGNATURE_ALGORITHM, ServoCertErrorCallback, ServoCertErrorCode,
    ServoCertificateError, servo_certificate_error_continue,
};

// Re-export clipboard types
pub use clipboard::{
    ServoClipboardClearCallback, ServoClipboardDelegate, ServoClipboardGetTextCallback,
    ServoClipboardSetTextCallback, ServoClipboardStringRequest,
    servo_webview_clipboard_request_failure, servo_webview_clipboard_request_success,
};

// Re-export gamepad types
pub use gamepad::{
    SERVO_GAMEPAD_HAPTIC_EFFECT_DUAL_RUMBLE, SERVO_GAMEPAD_HAPTIC_EFFECT_STOP,
    ServoGamepadDelegate, ServoGamepadHapticEffectCallback, ServoGamepadHapticEffectInfo,
    ServoGamepadHapticEffectRequest, servo_webview_gamepad_haptic_effect_request_failed,
    servo_webview_gamepad_haptic_effect_request_info,
    servo_webview_gamepad_haptic_effect_request_succeeded,
};

// Re-export download constants
pub use download::{
    SERVO_DOWNLOAD_CANCELLED, SERVO_DOWNLOAD_COMPLETE, SERVO_DOWNLOAD_IN_PROGRESS,
    SERVO_DOWNLOAD_INTERRUPTED,
};

// Re-export find types
pub use find::{ServoFindResult, ServoFindResultCallback};

// Re-export site data types
pub use site_data::{
    SERVO_COOKIE_SAME_SITE_LAX, SERVO_COOKIE_SAME_SITE_NONE, SERVO_COOKIE_SAME_SITE_STRICT,
    ServoCookie, ServoCookieFilter, ServoCookieFilterFlags, ServoCookieVisitor,
    ServoDeleteCookieCallback, ServoFlushStoreCallback, ServoSiteData, ServoSiteDataManager,
    ServoSiteDataVisitor, ServoSitesVisitor, ServoStorageType,
};

// Re-export request context types and constants
pub use request_context::{
    SERVO_PROXY_AUTO_DETECT, SERVO_PROXY_DIRECT, SERVO_PROXY_FIXED_SERVERS, SERVO_PROXY_PAC_SCRIPT,
    SERVO_PROXY_SYSTEM, ServoProxyConfig, ServoRequestContext, servo_builder_set_request_context,
    servo_request_context_create, servo_request_context_free,
    servo_request_context_set_accept_language, servo_request_context_set_cache_path,
    servo_request_context_set_cookie_path, servo_request_context_set_proxy_config,
    servo_webview_builder_set_request_context,
};

// Re-export value types and constants
pub use value::{
    ServoDictionaryValue, ServoListValue, ServoValue, ServoValueType,
    servo_dictionary_create, servo_dictionary_free, servo_dictionary_get_key,
    servo_dictionary_get_keys, servo_dictionary_get_size, servo_dictionary_has_key,
    servo_dictionary_set_key, servo_list_create, servo_list_free, servo_list_get_size,
    servo_list_get_value, servo_list_set_value, servo_value_create, servo_value_free,
    servo_value_from_json, servo_value_get_binary, servo_value_get_bool, servo_value_get_double,
    servo_value_get_int, servo_value_get_string, servo_value_get_type, servo_value_set_binary,
    servo_value_set_bool, servo_value_set_double, servo_value_set_int, servo_value_set_string,
    servo_value_to_json,
};

// Re-export JavaScript evaluation types
// `servo_webview_evaluate_javascript` is deprecated (see its docs); it is still
// re-exported so cbindgen keeps emitting the symbol for ABI compatibility.
#[allow(deprecated)]
pub use javascript::{
    ServoJavaScriptCallback, ServoJavaScriptCallbackV2, servo_webview_evaluate_javascript,
    servo_webview_evaluate_javascript_v2,
};

// Re-export user content manager types
pub use user_content_manager::{
    ServoUserContentManager, ServoUserScript, ServoUserStyleSheet,
    servo_user_content_manager_add_script, servo_user_content_manager_add_style_sheet,
    servo_user_content_manager_create, servo_user_content_manager_enable_message_router,
    servo_user_content_manager_free, servo_user_content_manager_remove_script,
    servo_user_content_manager_remove_style_sheet, servo_user_script_create,
    servo_user_script_free, servo_user_style_sheet_create, servo_user_style_sheet_free,
    servo_webview_builder_set_user_content_manager,
};

// Re-export message router types
pub use message_router::{
    ServoMessageCallback, ServoScriptMessageCallback, servo_webview_add_message_handler,
    servo_webview_add_script_message_handler, servo_webview_poll_js_messages,
    servo_webview_post_message, servo_webview_remove_message_handler,
    servo_webview_remove_script_message_handler,
};

// Re-export protocol handler types
pub use protocol_handler::{
    ServoProtocolHandler, ServoProtocolHandlerCallback, ServoWebResourceLoad,
    ServoWebResourceRequest, ServoWebResourceResponse, servo_builder_register_protocol_handler,
    servo_web_resource_load_cancel, servo_web_resource_load_finish, servo_web_resource_load_free,
    servo_web_resource_load_get_request, servo_web_resource_load_intercept,
    servo_web_resource_load_send_body_data, servo_web_resource_request_free,
    servo_web_resource_request_get_body, servo_web_resource_request_get_header,
    servo_web_resource_request_get_method, servo_web_resource_request_get_url,
    servo_web_resource_response_free, servo_web_resource_response_set_body,
    servo_web_resource_response_set_header, servo_web_resource_response_set_status,
    servo_web_resource_response_set_url,
};

// Re-export multiprocess functions
pub use multiprocess::{servo_create_sandbox, servo_run_content_process};

// Re-export network manager types and helpers
pub use network_manager::{
    ServoNetworkManager, servo_get_network_manager, servo_network_manager_clear_cache,
    servo_network_manager_free, servo_network_manager_get_cache_entries,
};

// Re-export preferences types and helpers
pub use preferences::{
    servo_preferences_create, servo_preferences_free,
    servo_preferences_get_bool, servo_preferences_get_devtools_server_enabled,
    servo_preferences_get_devtools_server_port, servo_preferences_get_i64,
    servo_preferences_get_string, servo_preferences_get_u64, servo_preferences_set_bool,
    servo_preferences_set_devtools_server, servo_preferences_set_i64,
    servo_preferences_set_string, servo_preferences_set_u64,
};

/// Error codes returned by the Servo C API.
///
/// All fallible functions return `ServoError` (as `int32_t`). `0` indicates
/// success; negative values indicate specific error conditions.
///
/// The enum uses a fixed `i32` representation to ensure a stable ABI.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServoError {
    /// Operation completed successfully.
    Ok = 0,

    /// A required pointer argument was `NULL`.
    NullArgument = -1,

    /// A string argument was not valid UTF-8.
    InvalidUtf8 = -2,

    /// The operation is not valid in the current state (e.g., WebView is
    /// shutting down, or a builder has already been consumed).
    InvalidState = -3,

    /// A requested resource was not found.
    NotFound = -4,

    /// A caller-provided buffer was too small.
    Capacity = -5,

    /// The requested operation is not supported on this platform or build
    /// configuration.
    Unsupported = -6,

    /// An internal error occurred in Servo (likely a bug).
    Internal = -7,

    /// A panic was caught at the FFI boundary.
    InternalPanic = -8,

    /// An invalid argument was provided.
    InvalidArgument = -9,

    /// The call was made from a thread other than the one that first entered
    /// the C API.
    ///
    /// The C API is single-threaded from the embedder's point of view: every
    /// handle is affine to the thread that first called into the library. This
    /// is enforced at the FFI boundary (see `docs/embedding/ABI_STABILITY.md`
    /// section 5) so a contract violation returns this error instead of
    /// performing unsynchronized access.
    WrongThread = -10,
}

impl ServoError {
    /// Creates a `catch_unwind` boundary for FFI functions.
    ///
    /// The boundary does two things, in order:
    ///
    /// 1. It enforces the C API's single-threaded contract by rejecting calls
    ///    from a thread other than the one that first entered the library
    ///    (`ServoError::WrongThread`). This check cannot panic and runs in
    ///    release builds too.
    /// 2. It ensures that no panic can cross the `extern "C"` boundary, which
    ///    would be undefined behavior. A panic is caught and converted to
    ///    `ServoError::InternalPanic`.
    #[inline]
    pub fn catch_ffi<F: FnOnce() -> ServoError>(f: F) -> ServoError {
        if let Err(error) = entry_checks::check_main_thread() {
            return error;
        }
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
            .unwrap_or(ServoError::InternalPanic)
    }

    /// Like [`ServoError::catch_ffi`], for FFI functions that return a raw `i32`
    /// (for example, an identifier). An off-thread call returns
    /// `ServoError::WrongThread` as an `i32`; panics are converted to
    /// `ServoError::InternalPanic` as an `i32`.
    #[inline]
    pub fn catch_ffi_i32<F: FnOnce() -> i32>(f: F) -> i32 {
        if entry_checks::check_main_thread().is_err() {
            return ServoError::WrongThread as i32;
        }
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
            .unwrap_or(ServoError::InternalPanic as i32)
    }

    /// Like [`ServoError::catch_ffi`], for FFI functions that return a raw
    /// pointer. An off-thread call or a panic returns `NULL`.
    #[inline]
    pub fn catch_ffi_ptr<T, F: FnOnce() -> *mut T>(f: F) -> *mut T {
        if entry_checks::check_main_thread().is_err() {
            return std::ptr::null_mut();
        }
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or(std::ptr::null_mut())
    }

    /// Like [`ServoError::catch_ffi`], for FFI functions that return a const
    /// pointer. An off-thread call or a panic returns `NULL`.
    #[inline]
    pub fn catch_ffi_const_ptr<T, F: FnOnce() -> *const T>(f: F) -> *const T {
        if entry_checks::check_main_thread().is_err() {
            return std::ptr::null();
        }
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or(std::ptr::null())
    }
}

/// Entry-point checks shared by the FFI boundary helpers.
///
/// The C API is single-threaded from the embedder's point of view: every handle
/// is `Rc`-based and affine to the thread that first entered the API. This
/// check turns a contract violation into `ServoError::WrongThread` instead of
/// unsynchronized access. It is compiled into release builds as well -- a check
/// that vanishes in release is a footgun, because release is where embedders
/// actually ship.
pub(crate) mod entry_checks {
    use std::sync::Mutex;
    use std::thread::ThreadId;

    use crate::ServoError;

    /// The thread that first called through the FFI boundary, or `None` until
    /// the first call.
    static MAIN_THREAD: Mutex<Option<ThreadId>> = Mutex::new(None);

    /// Returns `Err(ServoError::WrongThread)` when the current thread is not the
    /// thread that first called into the C API.
    ///
    /// The first caller wins: the API is single-threaded, so whoever enters
    /// first defines the owning thread. If the tracking lock is ever poisoned
    /// the check fails open (returns `Ok`): thread affinity is a correctness
    /// guard, and an unrelated poisoned lock must not wedge every entry point.
    pub fn check_main_thread() -> Result<(), ServoError> {
        let current = std::thread::current().id();
        let Ok(mut main_thread) = MAIN_THREAD.lock() else {
            return Ok(());
        };
        match *main_thread {
            None => {
                *main_thread = Some(current);
                Ok(())
            },
            Some(id) if id == current => Ok(()),
            Some(_) => Err(ServoError::WrongThread),
        }
    }
}

/// Tracks the opaque handles that are currently alive (W2.5, `HARDENING_PLAN.md`
/// §B.1).
///
/// The C API hands out raw `Box` pointers. Misusing one — using it after
/// freeing it, freeing it twice, or passing a handle of the wrong type — is
/// undefined behaviour by contract, and the pointer is not dereferenced until
/// after the embedder's mistake has already happened, so the engine cannot
/// notice it today.
///
/// This registry keys on the *address* of every live handle, so a check is a
/// lookup rather than a dereference: a stale or bogus pointer is rejected
/// without touching memory that may no longer belong to us. A handle that was
/// never registered (garbage) or that was unregistered by its own destructor
/// fails the check, and the entry point returns
/// [`ServoError::InvalidArgument`] instead of corrupting memory.
///
/// Limits, stated honestly:
///
/// Entries are `(address, kind)` pairs, so passing a handle of the wrong type
/// is a miss as well — otherwise a live handle of another type would satisfy
/// the lookup and be interpreted as the wrong struct.
///
/// Limits, stated honestly:
///
/// - Address reuse defeats it. If the allocator hands the same address to a
///   later handle of the same kind, a stale pointer to the earlier one looks
///   live again.
/// - It only knows about the handle types that have adopted it. See
///   `HARDENING_PLAN.md` §B.1 for the list and for what is still uncovered.
///
/// Like [`entry_checks::check_main_thread`], a poisoned lock **fails open**
/// (the handle is treated as live): a lock problem elsewhere in the process
/// must not wedge every entry point. Unlike the main-thread check this is
/// compiled into release builds too, so it protects shipping embedders rather
/// than only the test run; the lock is only taken on the entry points of the
/// handle types that have adopted it.
pub(crate) mod handle_registry {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};

    /// The handle types that have adopted the registry. One variant per type,
    /// so a handle cannot be used through an entry point of another type.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub(crate) enum HandleKind {
        /// A `ServoOptions` handle.
        Options,
    }

    fn live_handles() -> &'static Mutex<HashSet<(usize, HandleKind)>> {
        static LIVE: OnceLock<Mutex<HashSet<(usize, HandleKind)>>> = OnceLock::new();
        LIVE.get_or_init(|| Mutex::new(HashSet::new()))
    }

    /// Records `handle` as alive. Call it from the constructor that hands the
    /// handle to the embedder.
    pub(crate) fn register<T>(handle: *const T, kind: HandleKind) {
        if let Ok(mut live) = live_handles().lock() {
            live.insert((handle as usize, kind));
        }
    }

    /// Forgets `handle`. Call it from the destructor that takes ownership of
    /// it.
    ///
    /// Returns `false` when `handle` was not alive, which is what a double free
    /// looks like. The caller must check the return value **before** dropping
    /// the box: dropping it twice is the corruption this exists to prevent.
    pub(crate) fn unregister<T>(handle: *const T, kind: HandleKind) -> bool {
        match live_handles().lock() {
            Ok(mut live) => live.remove(&(handle as usize, kind)),
            Err(_) => true,
        }
    }

    /// Returns `true` when `handle` is a live handle of this API with this
    /// kind.
    pub(crate) fn is_live<T>(handle: *const T, kind: HandleKind) -> bool {
        match live_handles().lock() {
            Ok(live) => live.contains(&(handle as usize, kind)),
            Err(_) => true,
        }
    }
}

/// Returns `true` when the `struct_size` field supplied by an embedder is
/// compatible with the size of the corresponding Rust type.
///
/// The delegate-like input structs in this API are passed **by value**. A
/// caller built against an older header would pass a smaller struct, which the
/// by-value ABI would read out of bounds. Requiring `struct_size` to be at
/// least the compiled-in size turns that undefined behavior into a clean
/// `ServoError::InvalidArgument`. A larger `struct_size` (a caller built
/// against a *newer* header) is accepted, since Servo only reads the fields it
/// knows about. See `ABI_STABILITY.md` §3.3.
#[inline]
pub(crate) fn is_compatible_struct_size(struct_size: u32, rust_size: usize) -> bool {
    (struct_size as usize) >= rust_size
}

/// Major version of the Servo C API.
///
/// The C API follows semantic versioning independently of the Rust `servo`
/// crate. See `docs/embedding/ABI_STABILITY.md` for the compatibility policy.
pub const SERVO_CAPI_VERSION_MAJOR: u32 = 0;

/// Minor version of the Servo C API.
///
/// 0.3.0 is a breaking MINOR release: `ServoCookieFilter` and
/// `ServoProxyConfig`, the two `#[repr(C)]` structs the embedder fills in and
/// passes by pointer, gain the `struct_size` field required by
/// `ABI_STABILITY.md` §3.3. Both structs were unusable before — no in-tree
/// caller used them — so the change costs embedders a recompile (they must set
/// `struct_size`) and buys forward compatibility for every future field.
pub const SERVO_CAPI_VERSION_MINOR: u32 = 3;

/// Patch version of the Servo C API.
pub const SERVO_CAPI_VERSION_PATCH: u32 = 0;

/// The Servo C API version packed into a single integer.
///
/// The value is `(MAJOR << 16) | (MINOR << 8) | PATCH` and matches the
/// `SERVO_CAPI_VERSION` macro in the generated header.
pub const SERVO_CAPI_VERSION: u32 =
    (SERVO_CAPI_VERSION_MAJOR << 16) | (SERVO_CAPI_VERSION_MINOR << 8) | SERVO_CAPI_VERSION_PATCH;

/// The version of the Servo C API as a NUL terminated string.
///
/// This must stay in sync with [`SERVO_CAPI_VERSION_MAJOR`],
/// [`SERVO_CAPI_VERSION_MINOR`] and [`SERVO_CAPI_VERSION_PATCH`]. The C test
/// `test_capi_version` (`tests/capi/c/test_api.c`) compares this string against
/// the `MAJOR.MINOR.PATCH` macros expanded from the generated header, so a
/// mismatch fails CI instead of shipping a header/library version skew.
const SERVO_CAPI_VERSION_STRING: &CStr = c"0.3.0";

/// Returns the version of the Servo C API as a NUL terminated UTF-8 string.
///
/// The returned pointer refers to static memory and must not be freed or
/// modified by the caller. It remains valid for the lifetime of the process.
///
/// Embedders can use this to check runtime compatibility against the version
/// the header they compiled against reports.
#[unsafe(no_mangle)]
pub extern "C" fn servo_capi_version() -> *const c_char {
    SERVO_CAPI_VERSION_STRING.as_ptr()
}

/// Returns the version of the Servo C API as a packed integer.
///
/// The value is `(MAJOR << 16) | (MINOR << 8) | PATCH`, matching the
/// `SERVO_CAPI_VERSION` macro. Embedders can use this to detect a mismatched
/// library at load time without parsing the version string, for example
/// `servo_capi_get_version() != SERVO_CAPI_VERSION`.
#[unsafe(no_mangle)]
pub extern "C" fn servo_capi_get_version() -> u32 {
    SERVO_CAPI_VERSION
}

/// Returns the name of the platform this build targets.
///
/// The returned pointer refers to static memory and must not be freed or
/// modified by the caller. Possible values are `"linux"`, `"macos"`,
/// `"windows"`, `"android"`, `"ios"`, `"ohos"`, or `"unknown"`.
///
/// Part of the runtime capability query: an embedder should not assume that a
/// platform-specific capability (see [`servo_capi_is_sandboxing_supported`]) is
/// present just because the build succeeds.
#[allow(unreachable_code)]
#[unsafe(no_mangle)]
pub extern "C" fn servo_capi_platform_name() -> *const c_char {
    #[cfg(target_os = "windows")]
    return c"windows".as_ptr();
    #[cfg(target_os = "macos")]
    return c"macos".as_ptr();
    #[cfg(target_os = "linux")]
    return c"linux".as_ptr();
    #[cfg(target_os = "android")]
    return c"android".as_ptr();
    #[cfg(target_os = "ios")]
    return c"ios".as_ptr();
    #[cfg(target_env = "ohos")]
    return c"ohos".as_ptr();

    // Fallback for targets not enumerated above. On the targets handled above
    // this line is unreachable, hence the `allow` on the function.
    c"unknown".as_ptr()
}

/// Returns whether this build was compiled with multi-process support.
///
/// Multi-process is a build-time `multiprocess` feature and is independent of
/// whether the content-process sandbox can actually be activated on this
/// platform at runtime (see [`servo_capi_is_sandboxing_supported`]).
#[unsafe(no_mangle)]
pub extern "C" fn servo_capi_is_multiprocess_enabled() -> bool {
    cfg!(feature = "multiprocess")
}

/// Returns whether the content-process sandbox can be created on this platform
/// and build.
///
/// Returns `false` unless the crate was built with the `multiprocess` feature
/// **and** the target supports sandboxing. As of this writing sandboxing is not
/// available on Windows, iOS, Android, OpenHarmony, or any ARM/AArch64 or
/// RISC-V target (which includes Apple-Silicon macOS and ARM64 Linux). An
/// embedder on those platforms runs without renderer sandboxing and must not
/// assume otherwise — this is a real security property, not a formality.
///
/// Note: On Windows, in-process mitigation policies (DEP, ASLR, strict handles)
/// are applied when `--sandbox` is enabled, but mitigations are not an OS sandbox
/// (no broker, token restriction, or AppContainer).
///
/// This predicate mirrors the platforms where a true sandbox profile is activated.
#[unsafe(no_mangle)]
pub extern "C" fn servo_capi_is_sandboxing_supported() -> bool {
    cfg!(feature = "multiprocess")
        && cfg!(not(any(
            target_os = "windows",
            target_os = "ios",
            target_os = "android",
            target_arch = "arm",
            target_arch = "aarch64",
            target_arch = "riscv32",
            target_arch = "riscv64",
            target_env = "ohos",
        )))
}

/// An opaque struct representing builder for a `Servo` instance.
/// Refer to the documentation of the corresponding
/// [servo::ServoBuilder] struct in Rust API for more information.
/// [servo::ServoBuilder]: https://doc.servo.org/servo/struct.ServoBuilder.html
///
/// # Thread safety
///
/// `ServoBuilder` has no internal synchronization, so the embedder is
/// responsible for serializing access if the handle is shared between
/// threads. All calls to `servo_builder_set_*` should be made from the
/// same thread on which `servo_builder_build` will eventually invoked.
/// This is usually the embedder's main thread that will drive the event
/// loop by calling `servo_spin_event_loop`.
// cbindgen:opaque
#[derive(Default)]
pub struct ServoBuilder {
    options: Option<Box<ServoOptions>>,
    event_loop_waker: ServoEventLoopWaker,
    preferences: Option<Box<ServoPreferences>>,
    protocol_registry: Option<ProtocolRegistry>,
    delegate: Option<Rc<dyn servo_api::ServoDelegate>>,
    request_context: Option<servo_api::RequestContext>,
}

/// A callback used by Servo to wake the embedder thread when
/// Servo has new work to process. The embedder is expected to
/// pump Servo's event loop in response to this callback.
///
/// # Safety
///
/// The embedder must ensure that:
///
/// - `wake_callback` is a valid C ABI function matching the signature
///   shown.
/// - `wake_callback` remains valid for the lifetime of the `Servo`
///   instance the waker is associated with.
/// - `wake_callback` does not unwind across the FFI boundary.
/// - `wake_callback` is safe to invoke from any thread, since Servo may
///   call it from internal threads.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ServoEventLoopWaker {
    /// Size of this struct in bytes, as seen by the caller.
    ///
    /// The caller must set this to `sizeof(ServoEventLoopWaker)` (C) or
    /// `std::mem::size_of::<ServoEventLoopWaker>()` (Rust). Servo rejects
    /// wakers whose `struct_size` is smaller than the size it was compiled
    /// with, which catches callers built against an older header instead of
    /// letting the by-value ABI read out of bounds. See `ABI_STABILITY.md` §3.3.
    pub struct_size: u32,

    /// Reserved for future use; must be `0`.
    pub _reserved: u32,

    /// Called when Servo wants the embedder's event loop to wake up.
    pub wake_callback: extern "C" fn(),
}

/// A no-op default `wake_callback` used when the embedder has not set
/// one explicitly.
pub extern "C" fn default_event_loop_waker_wake_callback() {}

impl Default for ServoEventLoopWaker {
    fn default() -> Self {
        Self {
            struct_size: std::mem::size_of::<ServoEventLoopWaker>() as u32,
            _reserved: 0,
            wake_callback: default_event_loop_waker_wake_callback,
        }
    }
}

impl servo_api::EventLoopWaker for ServoEventLoopWaker {
    fn clone_box(&self) -> Box<dyn servo_api::EventLoopWaker> {
        Box::new(*self)
    }

    fn wake(&self) {
        (self.wake_callback)()
    }
}

/// Creates a new `ServoBuilder` populated with default values.
///
/// Returns a newly allocated `ServoBuilder` handle. The ownership of
/// the returned handle is transferred to the caller, who must free it
/// with [`servo_builder_free`] or consume it by passing it to
/// [`servo_builder_build`]. Returns `NULL` on allocation failure.
#[unsafe(no_mangle)]
pub extern "C" fn servo_builder_create() -> *mut ServoBuilder {
    ServoError::catch_ffi_ptr(|| Box::into_raw(Box::new(ServoBuilder::default())))
}

/// Sets the preferences to be used by the `Servo` instance.
///
/// `builder` is a handle to a `ServoBuilder` object.
/// The ownership of `builder` remains with the caller after the call.
///
/// `preferences` is a handle to a `ServoPreferences` object.
/// The ownership of `preferences` is transferred to the function.
/// The caller must not use or free `preferences` again.
/// This function will free the previously set preferences, if any.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `builder` is a non-null pointer to a `ServoBuilder` previously
///   returned by `servo_builder_create` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - `preferences` is a non-null pointer to a `ServoPreferences` previously
///   returned by `servo_preferences_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `builder` or `preferences` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_set_preferences(
    builder: *mut ServoBuilder,
    preferences: *mut ServoPreferences,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if builder.is_null() {
            return ServoError::NullArgument;
        }
        if preferences.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `builder` and `preferences` documented above.
        unsafe {
            (*builder).preferences = Some(Box::from_raw(preferences));
        }

        ServoError::Ok
    })
}

/// Sets the options to be used by the `Servo` instance.
///
/// `builder` is a handle to a `ServoBuilder` object.
/// The ownership of `builder` remains with the caller after the call.
///
/// `options` is a handle to a `ServoOptions` object.
/// The ownership of `options` is transferred to the function. The
/// caller must not use or free `options` again. This function will
/// free the previously set options, if any.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `builder` is a non-null pointer to a `ServoBuilder` previously
///   returned by `servo_builder_create` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - `options` is a non-null pointer to a `ServoOptions` previously
///   returned by `servo_options_create` and has not yet been freed nor
///   passed to another API that takes ownership of it.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `builder` or `options` is null.
/// Returns `ServoError::InvalidArgument` if `options` is not a live handle:
/// never created, already freed, or already consumed by another call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_set_options(
    builder: *mut ServoBuilder,
    options: *mut ServoOptions,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if builder.is_null() {
            return ServoError::NullArgument;
        }
        if options.is_null() {
            return ServoError::NullArgument;
        }

        // W2.5: this call consumes the `options` handle, so it must also forget
        // it in the handle registry. A handle that is no longer live is a
        // double consume or a use after a previous free, and taking ownership
        // of it again would free the same allocation twice.
        if !crate::handle_registry::unregister(options, crate::handle_registry::HandleKind::Options)
        {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `builder` and `options` documented above.
        unsafe {
            (*builder).options = Some(Box::from_raw(options));
        }

        ServoError::Ok
    })
}

/// Sets the callback used to wake the embedder's event loop when Servo
/// has new work to process (e.g, rendering updates).
///
/// `builder` is a handle to a `ServoBuilder` object.
/// The ownership of `builder` remains with the caller after the call.
///
/// `event_loop_waker` is a `ServoEventLoopWaker` struct that is copied
/// by value. See [`ServoEventLoopWaker`] for the safety requirements on
/// its fields.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `builder` is a non-null pointer to a `ServoBuilder` previously
///   returned by `servo_builder_create` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - The safety requirements documented on [`ServoEventLoopWaker`] are
///   upheld by `event_loop_waker`.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `builder` is null, or
/// `ServoError::InvalidArgument` if `event_loop_waker.struct_size` is smaller
/// than the size of `ServoEventLoopWaker` compiled into this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_set_event_loop_waker(
    builder: *mut ServoBuilder,
    event_loop_waker: ServoEventLoopWaker,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if builder.is_null() {
            return ServoError::NullArgument;
        }

        if !is_compatible_struct_size(
            event_loop_waker.struct_size,
            std::mem::size_of::<ServoEventLoopWaker>(),
        ) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `builder` documented above.
        unsafe {
            (*builder).event_loop_waker = event_loop_waker;
        }

        ServoError::Ok
    })
}

/// Sets the delegate to receive Servo-level notifications.
///
/// `builder` is a handle to a `ServoBuilder` object.
/// The ownership of `builder` remains with the caller after the call.
///
/// `delegate` is a `ServoDelegate` struct that is copied by value.
/// See [`ServoDelegate`] for the safety requirements on its fields.
///
/// This function takes ownership of the delegate's callbacks and user_data.
/// The delegate must remain valid for the lifetime of the resulting `Servo`
/// instance.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `builder` is a non-null pointer to a `ServoBuilder` previously
///   returned by `servo_builder_create` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - The safety requirements documented on [`ServoDelegate`] are
///   upheld by `delegate`.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `builder` is null, or
/// `ServoError::InvalidArgument` if `delegate.struct_size` is smaller than the
/// size of `ServoDelegate` compiled into this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_set_delegate(
    builder: *mut ServoBuilder,
    delegate: ServoDelegate,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if builder.is_null() {
            return ServoError::NullArgument;
        }

        if !is_compatible_struct_size(delegate.struct_size, std::mem::size_of::<ServoDelegate>()) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `builder` and `delegate` documented above.
        let delegate = make_servo_delegate(delegate);
        unsafe {
            (*builder).delegate = Some(delegate);
        }

        ServoError::Ok
    })
}

/// Consumes the builder and creates a new `Servo` instance.
///
/// `builder` is a handle to a `ServoBuilder` object.
/// The ownership of `builder` is transferred to the function. The
/// caller must not use or free `builder` again.
///
/// Returns a newly allocated `Servo` handle. The ownership of the
/// returned handle is transferred to the caller, who must free it with
/// [`servo_free`]. The resulting `Servo` instance is tied to the thread
/// that called this function and must only be used from that thread.
/// Returns `NULL` on failure.
///
/// # Safety
///
/// The caller must ensure that `builder` was previously returned by
/// `servo_builder_create` and has not yet been freed nor passed to
/// another API that takes ownership of it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_build(builder: *mut ServoBuilder) -> *mut servo_api::Servo {
    ServoError::catch_ffi_ptr(|| {
        if builder.is_null() {
            return std::ptr::null_mut();
        }
        let mut rust_builder = servo_api::ServoBuilder::default();

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `builder` documented above. We take ownership here.
        let mut builder = unsafe { Box::from_raw(builder) };

        if let Some(options) = builder.options.take() {
            rust_builder = rust_builder.opts(*options);
        }

        if let Some(preferences) = builder.preferences.take() {
            rust_builder = rust_builder.preferences(*preferences);
        }

        rust_builder = rust_builder.event_loop_waker(Box::new(builder.event_loop_waker));

        if let Some(request_context) = builder.request_context.take() {
            rust_builder = rust_builder.request_context(request_context);
        }

        if let Some(protocol_registry) = builder.protocol_registry.take() {
            rust_builder = rust_builder.protocol_registry(protocol_registry);
        }

        let servo = rust_builder.build();
        if let Some(delegate) = builder.delegate.take() {
            servo.set_delegate(delegate);
        }
        Box::into_raw(Box::new(servo))
    })
}

/// Destroys `builder` and frees its memory.
///
/// `builder` is a handle to a `ServoBuilder` object.
/// The ownership of `builder` is transferred to the function. The
/// caller must not use or free `builder` again.
///
/// # Safety
///
/// The caller must ensure that `builder` was previously returned by
/// `servo_builder_create` and has not yet been freed nor passed to
/// another API that takes ownership of it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_free(builder: *mut ServoBuilder) -> ServoError {
    ServoError::catch_ffi(|| {
        if builder.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `builder` documented above.
        unsafe {
            let _ = Box::from_raw(builder);
        }

        ServoError::Ok
    })
}
