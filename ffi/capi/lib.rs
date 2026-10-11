/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod cert_error;
mod cookie_manager;
mod download;
mod find;
mod javascript;
mod message_router;
// mod multiprocess;
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

use std::ffi::c_char;
use std::rc::Rc;

use options::ServoOptions;
use preferences::ServoPreferences;
use servo_api::protocol_handler::ProtocolRegistry;
use servo_delegate::{ServoDelegate, make_servo_delegate};

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
    ServoDeleteCookieCallback, ServoFlushStoreCallback, ServoSiteDataManager,
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
pub use javascript::{
    ServoJavaScriptCallback, ServoJavaScriptCallbackV2, servo_webview_evaluate_javascript,
    servo_webview_evaluate_javascript_v2,
};

// Re-export user content manager types
pub use user_content_manager::{
    ServoUserContentManager, servo_user_content_manager_create,
    servo_user_content_manager_enable_message_router,
    servo_user_content_manager_free,
    servo_webview_builder_set_user_content_manager,
};

// Re-export message router types
pub use message_router::{
    ServoMessageCallback, servo_webview_add_message_handler,
    servo_webview_poll_js_messages, servo_webview_post_message,
    servo_webview_remove_message_handler,
};

// Re-export protocol handler types
pub use protocol_handler::{
    ServoProtocolHandler, ServoProtocolHandlerCallback, ServoWebResourceRequest,
    ServoWebResourceResponse, servo_builder_register_protocol_handler,
    servo_web_resource_request_get_body, servo_web_resource_request_get_header,
    servo_web_resource_request_get_method, servo_web_resource_request_get_url,
    servo_web_resource_response_set_body, servo_web_resource_response_set_header,
    servo_web_resource_response_set_status, servo_web_resource_response_set_url,
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
}

impl ServoError {
    /// Creates a `catch_unwind` boundary for FFI functions.
    ///
    /// This ensures that no panic can cross the `extern "C"` boundary,
    /// which would be undefined behavior. If a panic occurs, it is caught
    /// and `ServoError::InternalPanic` is returned.
    #[inline]
    pub fn catch_ffi<F: FnOnce() -> ServoError>(f: F) -> ServoError {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or(ServoError::InternalPanic)
    }

    /// Creates a `catch_unwind` boundary for FFI functions that return a raw
    /// `i32` (for example, an identifier). Panics are caught and converted to
    /// `ServoError::InternalPanic` as an `i32`.
    #[inline]
    pub fn catch_ffi_i32<F: FnOnce() -> i32>(f: F) -> i32 {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or(ServoError::InternalPanic as i32)
    }

    /// Creates a `catch_unwind` boundary for FFI functions that return a raw pointer.
    /// Panics are caught and `NULL` (`null_mut()`) is returned.
    #[inline]
    pub fn catch_ffi_ptr<T, F: FnOnce() -> *mut T>(f: F) -> *mut T {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or(std::ptr::null_mut())
    }

    /// Creates a `catch_unwind` boundary for FFI functions that return a const pointer.
    /// Panics are caught and `NULL` (`null()`) is returned.
    #[inline]
    pub fn catch_ffi_const_ptr<T, F: FnOnce() -> *const T>(f: F) -> *const T {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or(std::ptr::null())
    }
}

/// Major version of the Servo C API.
///
/// The C API follows semantic versioning independently of the Rust `servo`
/// crate. See `docs/embedding/ABI_STABILITY.md` for the compatibility policy.
pub const SERVO_CAPI_VERSION_MAJOR: u32 = 0;

/// Minor version of the Servo C API.
pub const SERVO_CAPI_VERSION_MINOR: u32 = 1;

/// Patch version of the Servo C API.
pub const SERVO_CAPI_VERSION_PATCH: u32 = 0;

/// Returns the version of the Servo C API as a NUL terminated UTF-8 string.
///
/// The returned pointer refers to static memory and must not be freed or
/// modified by the caller. It remains valid for the lifetime of the process.
///
/// Embedders can use this to check runtime compatibility against the version
/// the header they compiled against reports.
#[unsafe(no_mangle)]
pub extern "C" fn servo_capi_version() -> *const c_char {
    c"0.1.0".as_ptr()
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
    pub wake_callback: extern "C" fn(),
}

/// A no-op default `wake_callback` used when the embedder has not set
/// one explicitly.
pub extern "C" fn default_event_loop_waker_wake_callback() {}

impl Default for ServoEventLoopWaker {
    fn default() -> Self {
        Self {
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
/// Returns `ServoError::NullArgument` if `builder` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_set_event_loop_waker(
    builder: *mut ServoBuilder,
    event_loop_waker: ServoEventLoopWaker,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if builder.is_null() {
            return ServoError::NullArgument;
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
/// Returns `ServoError::NullArgument` if `builder` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_set_delegate(
    builder: *mut ServoBuilder,
    delegate: ServoDelegate,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if builder.is_null() {
            return ServoError::NullArgument;
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
