/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use servo_api::SoftwareRenderingContext;

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
        Ok(ctx) => Box::into_raw(Box::new(RenderingContext {
            inner: Rc::new(ctx),
        })),
        Err(error) => {
            log::error!("Failed to create SoftwareRenderingContext: {error:?}");
            std::ptr::null_mut()
        },
    }
}

/// Destroys `ctx` and frees its memory.
///
/// `ctx` is a handle to a `RenderingContext` object.
/// The ownership of `ctx` is transferred to the function. The caller
/// must not use or free `ctx` again.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `ctx` was previously returned by one of the
///   `servo_rendering_context_create_*` functions and has not yet been
///   freed nor passed to another API that takes ownership of it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_rendering_context_free(ctx: *mut RenderingContext) {
    assert!(!ctx.is_null(), "ctx pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `ctx` documented above.
    unsafe {
        let _ = Box::from_raw(ctx);
    }
}
