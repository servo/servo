/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! UserContentManager C API for managing user scripts and styles.

use std::rc::Rc;

use servo_api::{Servo, UserContentManager};

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
