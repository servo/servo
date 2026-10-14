/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use raw_window_handle::{DisplayHandle, RawDisplayHandle, RawWindowHandle, WindowHandle};
use servo_api::{SoftwareRenderingContext, WindowRenderingContext};

use crate::ServoError;

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
    pub(crate) window_context: Option<Rc<WindowRenderingContext>>,
}

/// Rendering context type constants for future type queries.
#[unsafe(no_mangle)]
pub static SERVO_RENDERING_CONTEXT_SOFTWARE: u32 = 0;

#[unsafe(no_mangle)]
pub static SERVO_RENDERING_CONTEXT_WINDOW: u32 = 1;

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
    ServoError::catch_ffi_ptr(|| {
        let size = dpi::PhysicalSize::new(width, height);
        match SoftwareRenderingContext::new(size) {
            Ok(context) => Box::into_raw(Box::new(RenderingContext {
                inner: Rc::new(context),
                window_context: None,
            })),
            Err(error) => {
                log::error!("Failed to create SoftwareRenderingContext: {error:?}");
                std::ptr::null_mut()
            },
        }
    })
}

/// Creates a new windowed rendering context for GPU-accelerated rendering to a native window.
///
/// The `display_handle` and `window_handle` parameters are raw pointers to
/// `raw_window_handle::RawDisplayHandle` and `raw_window_handle::RawWindowHandle`
/// respectively, as provided by windowing libraries like winit, SDL, or GLFW.
///
/// Returns a newly allocated `RenderingContext` handle, or `NULL` on
/// failure (including invalid handles or zero-sized requests).
/// The ownership of the returned handle is transferred to the caller,
/// who must free it with [`servo_rendering_context_free`] or consume it
/// by passing it to `servo_webview_builder_create`.
///
/// # Safety
///
/// The caller must ensure that:
/// - `display_handle` and `window_handle` are valid pointers to the respective
///   `raw_window_handle` types for the current platform.
/// - The window and display handles remain valid for the lifetime of the
///   returned `RenderingContext`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_rendering_context_create_window(
    display_handle: *mut std::ffi::c_void,
    window_handle: *mut std::ffi::c_void,
    width: u32,
    height: u32,
) -> *mut RenderingContext {
    ServoError::catch_ffi_ptr(|| {
        if display_handle.is_null() || window_handle.is_null() {
            log::error!("display_handle and window_handle must not be null");
            return std::ptr::null_mut();
        }

        let size = dpi::PhysicalSize::new(width, height);
        if size.width == 0 || size.height == 0 {
            log::error!("WindowRenderingContext size must be at least 1x1");
            return std::ptr::null_mut();
        }

        // SAFETY: The caller guarantees these are valid pointers to the correct types.
        let display_handle = unsafe { &*(display_handle as *const RawDisplayHandle) };
        let window_handle = unsafe { &*(window_handle as *const RawWindowHandle) };

        let display_handle = unsafe { DisplayHandle::borrow_raw(*display_handle) };
        let window_handle = unsafe { WindowHandle::borrow_raw(*window_handle) };

        match WindowRenderingContext::new(display_handle, window_handle, size) {
            Ok(context) => {
                let rc = Rc::new(context);
                Box::into_raw(Box::new(RenderingContext {
                    inner: rc.clone(),
                    window_context: Some(rc),
                }))
            },
            Err(error) => {
                log::error!("Failed to create WindowRenderingContext: {error:?}");
                std::ptr::null_mut()
            },
        }
    })
}

/// Creates a new offscreen rendering context for GPU-accelerated offscreen
/// rendering (OSR / FBO composition).
///
/// The `parent` parameter must be a `RenderingContext` created by
/// `servo_rendering_context_create_window`. The offscreen context will share
/// the parent's OpenGL context and render to an FBO that can be composited
/// into the parent window.
///
/// Returns a newly allocated `RenderingContext` handle, or `NULL` on
/// failure (including null parent, non-window parent, or zero-sized requests).
/// The ownership of the returned handle is transferred to the caller,
/// who must free it with [`servo_rendering_context_free`] or consume it
/// by passing it to `servo_webview_builder_create`.
///
/// # Safety
///
/// The caller must ensure that:
/// - `parent` is a non-null pointer to a `RenderingContext` previously
///   returned by `servo_rendering_context_create_window` and has not yet been
///   freed nor passed to another API that takes ownership of it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_rendering_context_create_offscreen(
    parent: *mut RenderingContext,
    width: u32,
    height: u32,
) -> *mut RenderingContext {
    ServoError::catch_ffi_ptr(|| {
        if parent.is_null() {
            log::error!("parent RenderingContext must not be null");
            return std::ptr::null_mut();
        }

        let size = dpi::PhysicalSize::new(width, height);
        if size.width == 0 || size.height == 0 {
            log::error!("OffscreenRenderingContext size must be at least 1x1");
            return std::ptr::null_mut();
        }

        // SAFETY: The caller guarantees `parent` is a valid pointer to a RenderingContext.
        let parent_ctx = unsafe { &*parent };

        let Some(window_ctx) = &parent_ctx.window_context else {
            log::error!(
                "parent RenderingContext must be a WindowRenderingContext (created by servo_rendering_context_create_window)"
            );
            return std::ptr::null_mut();
        };

        // Create the offscreen context as a child of the window context.
        let offscreen_ctx = window_ctx.offscreen_context(size);

        Box::into_raw(Box::new(RenderingContext {
            inner: Rc::new(offscreen_ctx),
            window_context: None,
        }))
    })
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
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `context` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_rendering_context_free(
    context: *mut RenderingContext,
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
