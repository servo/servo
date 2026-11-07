/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use servo_api::SoftwareRenderingContext;

use crate::refresh_driver::ServoRefreshDriver;

/// An opaque struct that abstracts a rendering context used
/// for managing an OpenGL or GLES rendering context.
///
/// See the documentation of the correpsonding [servo::RenderingContext]
/// trait in the Rust API for more details.
/// [servo::RenderingContext]: https://doc.servo.org/servo/trait.RenderingContext.html
///
/// Handles to this object can be created using one of the
/// [`servo_rendering_context_create_*`] functions.
///
/// # Thread safety
/// The handle must be used only from the thread that created it.
// cbindgen:opaque
pub struct RenderingContext {
    pub(crate) inner: Rc<dyn servo_api::RenderingContext>,
}

/// Creates a new software rendering context for headless rendering.
///
/// Returns a newly allocated `RenderingContext` handle, or `NULL` on
/// failure.
/// The ownership of the returned handle is transferred to the caller,
/// who must free it with [`servo_rendering_context_free`] or consume it
/// by passing it to `servo_webview_builder_create`.
#[unsafe(no_mangle)]
pub extern "C" fn servo_rendering_context_create_software(
    width: u32,
    height: u32,
) -> *mut RenderingContext {
    let size = dpi::PhysicalSize::new(width, height);
    match SoftwareRenderingContext::new(size) {
        Ok(context) => Box::into_raw(Box::new(RenderingContext {
            inner: Rc::new(context),
        })),
        Err(error) => {
            log::error!("Failed to create SoftwareRenderingContext: {error:?}");
            std::ptr::null_mut()
        },
    }
}

/// Creates a new software rendering context for headless rendering, whose
/// frames are started by `refresh_driver` instead of Servo's own timer.
///
/// Returns a newly allocated `RenderingContext` handle, or `NULL` on
/// failure.
/// The ownership of the returned handle is transferred to the caller,
/// who must free it with [`servo_rendering_context_free`] or consume it
/// by passing it to `servo_webview_builder_create`.
/// The ownership of `refresh_driver` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `refresh_driver` was previously returned by `servo_refresh_driver_create`
///   and has not yet been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_rendering_context_create_software_with_refresh_driver(
    width: u32,
    height: u32,
    refresh_driver: *mut ServoRefreshDriver,
) -> *mut RenderingContext {
    assert!(
        !refresh_driver.is_null(),
        "refresh_driver pointer must not be null"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `refresh_driver` documented above.
    let refresh_driver = unsafe { &*refresh_driver };

    let size = dpi::PhysicalSize::new(width, height);
    match SoftwareRenderingContext::new_with_refresh_driver(size, refresh_driver.inner.clone()) {
        Ok(context) => Box::into_raw(Box::new(RenderingContext {
            inner: Rc::new(context),
        })),
        Err(error) => {
            log::error!("Failed to create SoftwareRenderingContext: {error:?}");
            std::ptr::null_mut()
        },
    }
}

/// Destroys `context` and frees its memory.
///
/// `context` is a handle to a `RenderingContext` object.
/// The ownership of `context` is transferred to the function. The caller
/// must not use or free `context` again.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `context` was previously returned by one of the
///   `servo_rendering_context_create_*` functions and has not yet been
///   freed nor passed to another API that takes ownership of it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_rendering_context_free(context: *mut RenderingContext) {
    assert!(!context.is_null(), "context pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `context` documented above.
    unsafe {
        let _ = Box::from_raw(context);
    }
}
