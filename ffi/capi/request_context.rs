/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{CStr, c_char};
use std::path::PathBuf;

use servo_api::RequestContext;

use crate::ServoError;

/// Proxy configuration for a request context.
///
/// This struct mirrors the Rust `ProxyConfig` and matches CEF's proxy
/// configuration model.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ServoProxyConfig {
    /// Size of this struct in bytes, as seen by the caller.
    ///
    /// The caller must set this to `sizeof(ServoProxyConfig)`. Servo rejects
    /// configurations whose `struct_size` is smaller than the size it was
    /// compiled with, returning `ServoError::InvalidArgument`. See
    /// `ABI_STABILITY.md` §3.3.
    pub struct_size: u32,

    /// Reserved for future use; must be `0`.
    pub _reserved: u32,

    /// The proxy mode. One of `SERVO_PROXY_*` constants.
    pub mode: i32,
    /// The proxy server address in "host:port" format.
    /// Only used when mode is `SERVO_PROXY_FIXED_SERVERS`.
    pub server: *const c_char,
    /// Comma-separated list of hosts to bypass the proxy.
    /// Only used when mode is `SERVO_PROXY_FIXED_SERVERS`.
    pub bypass_list: *const c_char,
}

/// Proxy mode constants.
/// These match CEF's `cef_proxy_type_t` values.
pub const SERVO_PROXY_DIRECT: i32 = 0;
pub const SERVO_PROXY_AUTO_DETECT: i32 = 1;
pub const SERVO_PROXY_PAC_SCRIPT: i32 = 2;
pub const SERVO_PROXY_FIXED_SERVERS: i32 = 3;
pub const SERVO_PROXY_SYSTEM: i32 = 4;

/// Opaque handle for a RequestContext.
///
/// A `RequestContext` provides profile/cookie isolation, proxy configuration,
/// cache path selection, and accept language settings. Multiple `WebView`s
/// can share a `RequestContext` to share cookies, cache, and other site data.
///
/// # Thread safety
///
/// The handle must be used only from the thread that created the `Servo` instance.
// cbindgen:opaque
pub struct ServoRequestContext {
    inner: RequestContext,
}

/// Creates a new `ServoRequestContext` with default values.
///
/// Returns a newly allocated `ServoRequestContext` handle. The ownership of
/// the returned handle is transferred to the caller, who must free it with
/// [`servo_request_context_free`]. Returns `NULL` on allocation failure.
#[unsafe(no_mangle)]
pub extern "C" fn servo_request_context_create() -> *mut ServoRequestContext {
    ServoError::catch_ffi_ptr(|| {
        let ctx = RequestContext::new();
        Box::into_raw(Box::new(ServoRequestContext { inner: ctx }))
    })
}

/// Frees the `ServoRequestContext` handle.
///
/// `context` is a handle to a `ServoRequestContext` object.
/// The ownership of `context` is transferred to the function. The caller
/// must not use or free `context` again.
///
/// # Safety
///
/// The caller must ensure that `context` was previously returned by
/// `servo_request_context_create` and has not yet been freed.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `context` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_request_context_free(
    context: *mut ServoRequestContext,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if context.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `context` documented above.
        unsafe {
            let _ = Box::from_raw(context);
        }

        ServoError::Ok
    })
}

/// Sets the cache path for the request context.
///
/// `context` is a handle to a `ServoRequestContext` object.
/// The ownership of `context` remains with the caller after the call.
///
/// `path` is a NUL-terminated UTF-8 string representing the path to the
/// HTTP cache directory. If `NULL`, the cache will be stored in memory only.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `context` is a non-null pointer to a `ServoRequestContext` previously
///   returned by `servo_request_context_create` and has not yet been freed.
/// - If `path` is non-null, it points to a valid C string that remains
///   unmodified for the duration of the call.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `context` is null.
/// Returns `ServoError::InvalidUtf8` if `path` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_request_context_set_cache_path(
    context: *mut ServoRequestContext,
    path: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if context.is_null() {
            return ServoError::NullArgument;
        }

        let cache_path = if path.is_null() {
            None
        } else {
            // SAFETY: The caller is assumed to uphold the safety requirements
            // for `path` documented above.
            let Ok(path_str) = (unsafe { CStr::from_ptr(path) }).to_str() else {
                return ServoError::InvalidUtf8;
            };
            Some(PathBuf::from(path_str))
        };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `context` documented above.
        unsafe {
            (*context).inner.cache_path = cache_path;
        }

        ServoError::Ok
    })
}

/// Sets the cookie store path for the request context.
///
/// `context` is a handle to a `ServoRequestContext` object.
/// The ownership of `context` remains with the caller after the call.
///
/// `path` is a NUL-terminated UTF-8 string representing the path to the
/// cookie store directory. If `NULL`, cookies will be stored in memory only.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `context` is a non-null pointer to a `ServoRequestContext` previously
///   returned by `servo_request_context_create` and has not yet been freed.
/// - If `path` is non-null, it points to a valid C string that remains
///   unmodified for the duration of the call.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `context` is null.
/// Returns `ServoError::InvalidUtf8` if `path` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_request_context_set_cookie_path(
    context: *mut ServoRequestContext,
    path: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if context.is_null() {
            return ServoError::NullArgument;
        }

        let cookie_path = if path.is_null() {
            None
        } else {
            // SAFETY: The caller is assumed to uphold the safety requirements
            // for `path` documented above.
            let Ok(path_str) = (unsafe { CStr::from_ptr(path) }).to_str() else {
                return ServoError::InvalidUtf8;
            };
            Some(PathBuf::from(path_str))
        };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `context` documented above.
        unsafe {
            (*context).inner.cookie_store_path = cookie_path;
        }

        ServoError::Ok
    })
}

/// Sets the proxy configuration for the request context.
///
/// `context` is a handle to a `ServoRequestContext` object.
/// The ownership of `context` remains with the caller after the call.
///
/// `config` is a pointer to a `ServoProxyConfig` struct.
/// The struct is copied by value.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `context` is a non-null pointer to a `ServoRequestContext` previously
///   returned by `servo_request_context_create` and has not yet been freed.
/// - `config` is a non-null pointer to a valid `ServoProxyConfig` struct.
/// - If `config.mode` is `SERVO_PROXY_FIXED_SERVERS`, `config.server` must be
///   a valid NUL-terminated UTF-8 string.
/// - If `config.bypass_list` is non-null, it must be a valid NUL-terminated
///   UTF-8 string.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `context` or `config` is null.
/// Returns `ServoError::InvalidArgument` if `config.struct_size` is smaller
/// than the size of `ServoProxyConfig` compiled into this library.
/// Returns `ServoError::InvalidUtf8` if any string is not valid UTF-8.
/// Returns `ServoError::InvalidState` if the proxy mode is invalid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_request_context_set_proxy_config(
    context: *mut ServoRequestContext,
    config: *const ServoProxyConfig,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if context.is_null() {
            return ServoError::NullArgument;
        }
        if config.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `config` documented above.
        let config_ref = unsafe { &*config };

        // W2.5/`ABI_STABILITY.md` §3.3: a caller built against an older header
        // would pass a smaller struct, which this function would read past.
        if !crate::is_compatible_struct_size(
            config_ref.struct_size,
            std::mem::size_of::<ServoProxyConfig>(),
        ) {
            return ServoError::InvalidArgument;
        }

        let mode = match servo_api::ProxyMode::from_i32(config_ref.mode) {
            Some(m) => m,
            None => return ServoError::InvalidState,
        };

        let server = if config_ref.server.is_null() {
            None
        } else {
            // SAFETY: The caller is assumed to uphold the safety requirements
            // for `config.server` documented above.
            let Ok(s) = (unsafe { CStr::from_ptr(config_ref.server) }).to_str() else {
                return ServoError::InvalidUtf8;
            };
            Some(s.to_string())
        };

        let bypass_list = if config_ref.bypass_list.is_null() {
            None
        } else {
            // SAFETY: The caller is assumed to uphold the safety requirements
            // for `config.bypass_list` documented above.
            let Ok(s) = (unsafe { CStr::from_ptr(config_ref.bypass_list) }).to_str() else {
                return ServoError::InvalidUtf8;
            };
            Some(s.to_string())
        };

        let proxy_config = servo_api::ProxyConfig {
            mode,
            server,
            bypass_list,
        };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `context` documented above.
        unsafe {
            (*context).inner.proxy_config = proxy_config;
        }

        ServoError::Ok
    })
}

/// Sets the accept language for the request context.
///
/// `context` is a handle to a `ServoRequestContext` object.
/// The ownership of `context` remains with the caller after the call.
///
/// `lang` is a NUL-terminated UTF-8 string representing the Accept-Language
/// header value (e.g., "en-US,en;q=0.9"). If `NULL`, the default accept
/// language will be used.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `context` is a non-null pointer to a `ServoRequestContext` previously
///   returned by `servo_request_context_create` and has not yet been freed.
/// - If `lang` is non-null, it points to a valid C string that remains
///   unmodified for the duration of the call.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `context` is null.
/// Returns `ServoError::InvalidUtf8` if `lang` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_request_context_set_accept_language(
    context: *mut ServoRequestContext,
    lang: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if context.is_null() {
            return ServoError::NullArgument;
        }

        let accept_language = if lang.is_null() {
            None
        } else {
            // SAFETY: The caller is assumed to uphold the safety requirements
            // for `lang` documented above.
            let Ok(lang_str) = (unsafe { CStr::from_ptr(lang) }).to_str() else {
                return ServoError::InvalidUtf8;
            };
            Some(lang_str.to_string())
        };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `context` documented above.
        unsafe {
            (*context).inner.accept_language = accept_language;
        }

        ServoError::Ok
    })
}

/// Sets the request context on a `ServoBuilder`.
///
/// `builder` is a handle to a `ServoBuilder` object.
/// The ownership of `builder` remains with the caller after the call.
///
/// `context` is a handle to a `ServoRequestContext` object.
/// The ownership of `context` is transferred to the function.
/// The caller must not use or free `context` again.
/// This function will free the previously set request context, if any.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `builder` is a non-null pointer to a `ServoBuilder` previously
///   returned by `servo_builder_create` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - `context` is a non-null pointer to a `ServoRequestContext` previously
///   returned by `servo_request_context_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `builder` or `context` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_set_request_context(
    builder: *mut crate::ServoBuilder,
    context: *mut ServoRequestContext,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if builder.is_null() {
            return ServoError::NullArgument;
        }
        if context.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `builder` and `context` documented above.
        unsafe {
            let ctx = Box::from_raw(context);
            (*builder).request_context = Some(ctx.inner);
        }

        ServoError::Ok
    })
}

/// Sets the request context on a `WebViewBuilder`.
///
/// `builder` is a handle to a `WebViewBuilder` object.
/// The ownership of `builder` remains with the caller after the call.
///
/// `context` is a handle to a `ServoRequestContext` object.
/// The ownership of `context` is transferred to the function.
/// The caller must not use or free `context` again.
/// This function will free the previously set request context, if any.
///
/// This overrides the request context set on the `ServoBuilder` for this
/// specific `WebView`.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `builder` is a non-null pointer to a `WebViewBuilder` previously
///   returned by `servo_webview_builder_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
/// - `context` is a non-null pointer to a `ServoRequestContext` previously
///   returned by `servo_request_context_create` and has not yet been freed
///   nor passed to another API that takes ownership of it.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `builder` or `context` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_builder_set_request_context(
    builder: *mut crate::webview::ServoWebViewBuilder,
    context: *mut ServoRequestContext,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if builder.is_null() {
            return ServoError::NullArgument;
        }
        if context.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `builder` and `context` documented above.
        unsafe {
            let ctx = Box::from_raw(context);
            (*builder).request_context = Some(ctx.inner);
        }

        ServoError::Ok
    })
}
