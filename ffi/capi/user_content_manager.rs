/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! UserContentManager C API for managing user scripts and styles.

use std::ffi::{CStr, c_char};
use std::path::PathBuf;
use std::rc::Rc;

use embedder_traits::user_contents::{UserScript, UserStyleSheet};
use servo_api::{Servo, UserContentManager};
use url::Url;

use crate::ServoError;

/// An opaque struct representing a UserContentManager.
///
/// Handles to this object can be created using [`servo_user_content_manager_create`].
///
/// # Thread safety
///
/// The handle must be used only from the thread that created it.
/// It must also be created on the same thread that created the `Servo`
/// instance passed to `servo_user_content_manager_create`.
///
/// # Ownership
///
/// The ownership of the returned handle is transferred to the caller, who
/// must free it with [`servo_user_content_manager_free`] or transfer
/// ownership by passing it to `servo_webview_builder_set_user_content_manager`.
///
/// Note: Once passed to `servo_webview_builder_set_user_content_manager`,
/// the WebView builder takes ownership. Do not free the handle yourself.
/// It will be freed when the WebView is built (or the builder is freed).
// cbindgen:opaque
pub struct ServoUserContentManager {
    inner: Rc<UserContentManager>,
    _servo: Servo, // Keep Servo alive
}

/// An opaque handle to a user script.
///
/// Handles to this object can be created using [`servo_user_script_create`].
/// Unlike a `ServoUserContentManager`, ownership is always retained by the
/// caller, so the same handle can be added to (or removed from) any number of
/// `ServoUserContentManager`s and must be released with
/// [`servo_user_script_free`].
///
/// # Thread safety
///
/// The handle must be used only from the thread that created it.
// cbindgen:opaque
pub struct ServoUserScript {
    inner: Rc<UserScript>,
}

/// An opaque handle to a user stylesheet.
///
/// Handles to this object can be created using
/// [`servo_user_style_sheet_create`] and must be released with
/// [`servo_user_style_sheet_free`].
///
/// # Thread safety
///
/// The handle must be used only from the thread that created it.
// cbindgen:opaque
pub struct ServoUserStyleSheet {
    inner: Rc<UserStyleSheet>,
}

/// Creates a new user script from JavaScript `source`.
///
/// `source` is a NUL-terminated UTF-8 string containing the JavaScript to
/// inject. `source_file` is an optional NUL-terminated UTF-8 string naming the
/// file the script came from (used for diagnostics); pass `NULL` when the
/// script has no associated file.
///
/// Returns a newly allocated `ServoUserScript` handle owned by the caller, or
/// `NULL` if `source` is `NULL` or not valid UTF-8.
///
/// # Safety
///
/// The caller must ensure that `source` and (if non-`NULL`) `source_file` are
/// non-null pointers to valid, NUL-terminated C strings that remain unmodified
/// for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_user_script_create(
    source: *const c_char,
    source_file: *const c_char,
) -> *mut ServoUserScript {
    ServoError::catch_ffi_ptr(|| {
        if source.is_null() {
            return std::ptr::null_mut();
        }

        // SAFETY: The caller is assumed to uphold the safety requirements above.
        let source = match unsafe { CStr::from_ptr(source) }.to_str() {
            Ok(source) => source.to_string(),
            Err(_) => return std::ptr::null_mut(),
        };

        let source_file = if source_file.is_null() {
            None
        } else {
            // SAFETY: The caller is assumed to uphold the safety requirements above.
            match unsafe { CStr::from_ptr(source_file) }.to_str() {
                Ok(path) => Some(PathBuf::from(path)),
                Err(_) => return std::ptr::null_mut(),
            }
        };

        Box::into_raw(Box::new(ServoUserScript {
            inner: Rc::new(UserScript::new(source, source_file)),
        }))
    })
}

/// Frees a `ServoUserScript` handle.
///
/// The ownership of `script` is transferred to the function. The caller must
/// not use or free `script` again.
///
/// # Safety
///
/// `script` must have been previously returned by [`servo_user_script_create`]
/// and not yet freed.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `script` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_user_script_free(script: *mut ServoUserScript) -> ServoError {
    ServoError::catch_ffi(|| {
        if script.is_null() {
            return ServoError::NullArgument;
        }
        // SAFETY: The caller is assumed to uphold the safety requirements above.
        unsafe {
            let _ = Box::from_raw(script);
        }
        ServoError::Ok
    })
}

/// Creates a new user stylesheet from CSS `source`.
///
/// `source` is a NUL-terminated UTF-8 string containing the CSS to inject.
/// `url` is a NUL-terminated UTF-8 string representing the stylesheet's base
/// URL (used to resolve relative `url()` references); it must be a valid
/// absolute URL such as `"about:blank"` or a `file:`/`https:` URL.
///
/// Returns a newly allocated `ServoUserStyleSheet` handle owned by the caller,
/// or `NULL` if an argument is `NULL`, not valid UTF-8, or `url` is not a valid
/// URL.
///
/// # Safety
///
/// The caller must ensure that `source` and `url` are non-null pointers to
/// valid, NUL-terminated C strings that remain unmodified for the duration of
/// the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_user_style_sheet_create(
    source: *const c_char,
    url: *const c_char,
) -> *mut ServoUserStyleSheet {
    ServoError::catch_ffi_ptr(|| {
        if source.is_null() || url.is_null() {
            return std::ptr::null_mut();
        }

        // SAFETY: The caller is assumed to uphold the safety requirements above.
        let source = match unsafe { CStr::from_ptr(source) }.to_str() {
            Ok(source) => source.to_string(),
            Err(_) => return std::ptr::null_mut(),
        };
        // SAFETY: The caller is assumed to uphold the safety requirements above.
        let url = match unsafe { CStr::from_ptr(url) }.to_str() {
            Ok(url) => match Url::parse(url) {
                Ok(url) => url,
                Err(_) => return std::ptr::null_mut(),
            },
            Err(_) => return std::ptr::null_mut(),
        };

        Box::into_raw(Box::new(ServoUserStyleSheet {
            inner: Rc::new(UserStyleSheet::new(source, url)),
        }))
    })
}

/// Frees a `ServoUserStyleSheet` handle.
///
/// The ownership of `stylesheet` is transferred to the function. The caller
/// must not use or free `stylesheet` again.
///
/// # Safety
///
/// `stylesheet` must have been previously returned by
/// [`servo_user_style_sheet_create`] and not yet freed.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `stylesheet` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_user_style_sheet_free(
    stylesheet: *mut ServoUserStyleSheet,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if stylesheet.is_null() {
            return ServoError::NullArgument;
        }
        // SAFETY: The caller is assumed to uphold the safety requirements above.
        unsafe {
            let _ = Box::from_raw(stylesheet);
        }
        ServoError::Ok
    })
}

/// Registers a user script with a `ServoUserContentManager`.
///
/// The script is injected into every page managed by the manager. Like all
/// user content, the change only takes effect after the page is reloaded.
///
/// The ownership of both handles remains with the caller: the same
/// `ServoUserScript` can be added to several managers, and passing it here does
/// not transfer ownership. Use [`servo_user_content_manager_remove_script`]
/// with the same handle to remove it.
///
/// # Safety
///
/// `user_content_manager` and `script` must be non-null pointers previously
/// returned by [`servo_user_content_manager_create`] and
/// [`servo_user_script_create`] respectively, and must not have been freed.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if either argument is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_user_content_manager_add_script(
    user_content_manager: *mut ServoUserContentManager,
    script: *mut ServoUserScript,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if user_content_manager.is_null() || script.is_null() {
            return ServoError::NullArgument;
        }
        // SAFETY: The caller is assumed to uphold the safety requirements above.
        let user_content_manager = unsafe { &*user_content_manager };
        let script = unsafe { &*script };
        user_content_manager.inner.add_script(script.inner.clone());
        ServoError::Ok
    })
}

/// Unregisters a user script previously added to a `ServoUserContentManager`.
///
/// The `script` handle must be the same one passed to
/// [`servo_user_content_manager_add_script`]. The change only takes effect
/// after the page is reloaded.
///
/// # Safety
///
/// `user_content_manager` and `script` must be non-null pointers previously
/// returned by [`servo_user_content_manager_create`] and
/// [`servo_user_script_create`] respectively, and must not have been freed.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if either argument is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_user_content_manager_remove_script(
    user_content_manager: *mut ServoUserContentManager,
    script: *mut ServoUserScript,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if user_content_manager.is_null() || script.is_null() {
            return ServoError::NullArgument;
        }
        // SAFETY: The caller is assumed to uphold the safety requirements above.
        let user_content_manager = unsafe { &*user_content_manager };
        let script = unsafe { &*script };
        user_content_manager.inner.remove_script(script.inner.clone());
        ServoError::Ok
    })
}

/// Registers a user stylesheet with a `ServoUserContentManager`.
///
/// The stylesheet is injected into every page managed by the manager. Like all
/// user content, the change only takes effect after the page is reloaded.
///
/// The ownership of both handles remains with the caller: passing `stylesheet`
/// here does not transfer ownership. Use
/// [`servo_user_content_manager_remove_style_sheet`] with the same handle to
/// remove it.
///
/// # Safety
///
/// `user_content_manager` and `stylesheet` must be non-null pointers previously
/// returned by [`servo_user_content_manager_create`] and
/// [`servo_user_style_sheet_create`] respectively, and must not have been
/// freed.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if either argument is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_user_content_manager_add_style_sheet(
    user_content_manager: *mut ServoUserContentManager,
    stylesheet: *mut ServoUserStyleSheet,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if user_content_manager.is_null() || stylesheet.is_null() {
            return ServoError::NullArgument;
        }
        // SAFETY: The caller is assumed to uphold the safety requirements above.
        let user_content_manager = unsafe { &*user_content_manager };
        let stylesheet = unsafe { &*stylesheet };
        user_content_manager.inner.add_stylesheet(stylesheet.inner.clone());
        ServoError::Ok
    })
}

/// Unregisters a user stylesheet previously added to a
/// `ServoUserContentManager`.
///
/// The `stylesheet` handle must be the same one passed to
/// [`servo_user_content_manager_add_style_sheet`]. The change only takes effect
/// after the page is reloaded.
///
/// # Safety
///
/// `user_content_manager` and `stylesheet` must be non-null pointers previously
/// returned by [`servo_user_content_manager_create`] and
/// [`servo_user_style_sheet_create`] respectively, and must not have been
/// freed.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if either argument is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_user_content_manager_remove_style_sheet(
    user_content_manager: *mut ServoUserContentManager,
    stylesheet: *mut ServoUserStyleSheet,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if user_content_manager.is_null() || stylesheet.is_null() {
            return ServoError::NullArgument;
        }
        // SAFETY: The caller is assumed to uphold the safety requirements above.
        let user_content_manager = unsafe { &*user_content_manager };
        let stylesheet = unsafe { &*stylesheet };
        user_content_manager.inner.remove_stylesheet(stylesheet.inner.clone());
        ServoError::Ok
    })
}

/// Creates a new `UserContentManager` for the given `Servo` instance.
///
/// `servo` is a handle to a `Servo` object.
/// The ownership of `servo` remains with the caller after the call.
///
/// Returns a newly allocated `ServoUserContentManager` handle. The
/// ownership of the returned handle is transferred to the caller, who
/// must free it with [`servo_user_content_manager_free`] or transfer
/// ownership by passing it to `servo_webview_builder_set_user_content_manager`.
/// Returns `NULL` on failure.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `servo` is a non-null pointer to a `Servo` instance previously
///   returned by `servo_builder_build` and has not yet been freed.
/// - The call is made from the same thread that created the `Servo`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_user_content_manager_create(
    servo: *mut Servo,
) -> *mut ServoUserContentManager {
    ServoError::catch_ffi_ptr(|| {
        if servo.is_null() {
            return std::ptr::null_mut();
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `servo` documented above.
        let servo = unsafe { &*servo };

        let ucm = UserContentManager::new(servo);
        Box::into_raw(Box::new(ServoUserContentManager {
            inner: Rc::new(ucm),
            _servo: servo.clone(),
        }))
    })
}

/// Frees a `ServoUserContentManager` handle.
///
/// `user_content_manager` is a handle to a `ServoUserContentManager` object.
/// The ownership of `user_content_manager` is transferred to the function.
/// The caller must not use or free `user_content_manager` again.
///
/// Note: If the `UserContentManager` was passed to
/// `servo_webview_builder_set_user_content_manager`, the builder owns it
/// and this function should NOT be called (it would be a double-free).
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `user_content_manager` was previously returned by
///   `servo_user_content_manager_create` and has not yet been freed nor
///   passed to another API that takes ownership of it.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `user_content_manager` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_user_content_manager_free(
    user_content_manager: *mut ServoUserContentManager,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if user_content_manager.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `user_content_manager` documented above.
        unsafe {
            let _ = Box::from_raw(user_content_manager);
        }

        ServoError::Ok
    })
}

/// Enable the message router on a `UserContentManager`.
///
/// This injects the `window.servoMessageRouter` bootstrap script into all
/// pages managed by this `UserContentManager`. When enabled, you can use
/// `servo_webview_post_message`, `servo_webview_add_message_handler`, and
/// the JS-side `window.servoMessageRouter` API for communication between
/// native and JavaScript.
///
/// `user_content_manager` is a handle to a `ServoUserContentManager` object.
/// The ownership of `user_content_manager` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `user_content_manager` is a non-null pointer to a `ServoUserContentManager`
///   previously returned by `servo_user_content_manager_create` and not yet freed.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `user_content_manager` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_user_content_manager_enable_message_router(
    user_content_manager: *mut ServoUserContentManager,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if user_content_manager.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `user_content_manager` documented above.
        let ucm = unsafe { &*user_content_manager };
        ucm.inner.inject_message_router();

        ServoError::Ok
    })
}

/// Set the `UserContentManager` on a `WebViewBuilder`.
///
/// `builder` is a handle to a `ServoWebViewBuilder` object.
/// The ownership of `builder` remains with the caller after the call.
///
/// `user_content_manager` is a handle to a `ServoUserContentManager` object.
/// The ownership of `user_content_manager` is transferred to the function.
/// The caller must not use or free `user_content_manager` again.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `builder` is a non-null pointer to a `ServoWebViewBuilder`
///   previously returned by `servo_webview_builder_create` and has not
///   yet been freed nor passed to another API that takes ownership of it.
/// - `user_content_manager` is a non-null pointer to a `ServoUserContentManager`
///   previously returned by `servo_user_content_manager_create` and has not
///   yet been freed nor passed to another API that takes ownership of it.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `builder` or `user_content_manager` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_builder_set_user_content_manager(
    builder: *mut crate::webview::ServoWebViewBuilder,
    user_content_manager: *mut ServoUserContentManager,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if builder.is_null() {
            return ServoError::NullArgument;
        }
        if user_content_manager.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `builder` documented above.
        let builder = unsafe { &mut *builder };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `user_content_manager` documented above. We take ownership here.
        let ucm_box = unsafe { Box::from_raw(user_content_manager) };
        builder.user_content_manager = Some(ucm_box.inner);

        ServoError::Ok
    })
}
