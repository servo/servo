/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{CStr, c_void};
use std::os::raw::{c_char, c_int};
use std::rc::Rc;

use keyboard_types::{Code, CompositionEvent, CompositionState, Key, KeyState, Location, Modifiers, NamedKey};
use servo_api::{Servo, WebView, WebViewBuilder};

use embedder_traits::{MediaSessionActionType, Scroll, WebViewPoint, WebViewVector};
use embedder_traits::input_events::{
    ImeEvent, InputEvent, InputEventId, KeyboardEvent, MouseButton, MouseButtonAction, MouseButtonEvent,
    MouseMoveEvent, TouchEvent, TouchEventType, TouchId, TouchPointerType, WheelDelta, WheelEvent,
    WheelMode,
};

use crate::ServoError;

/// Corresponds to [`servo_api::Theme::Light`].
pub const SERVO_THEME_LIGHT: c_int = 0;

/// Corresponds to [`servo_api::Theme::Dark`].
pub const SERVO_THEME_DARK: c_int = 1;

/// Key event types.
pub const SERVO_KEY_EVENT_DOWN: c_int = 0;
pub const SERVO_KEY_EVENT_UP: c_int = 1;

/// Mouse event types.
pub const SERVO_MOUSE_EVENT_DOWN: c_int = 0;
pub const SERVO_MOUSE_EVENT_UP: c_int = 1;
pub const SERVO_MOUSE_EVENT_MOVE: c_int = 2;
pub const SERVO_MOUSE_EVENT_LEAVE: c_int = 3;

/// Touch event types.
pub const SERVO_TOUCH_EVENT_DOWN: c_int = 0;
pub const SERVO_TOUCH_EVENT_MOVE: c_int = 1;
pub const SERVO_TOUCH_EVENT_UP: c_int = 2;
pub const SERVO_TOUCH_EVENT_CANCEL: c_int = 3;

/// Mouse button types.
pub const SERVO_MOUSE_BUTTON_NONE: c_int = -1;
pub const SERVO_MOUSE_BUTTON_PRIMARY: c_int = 0;
pub const SERVO_MOUSE_BUTTON_AUXILIARY: c_int = 1;
pub const SERVO_MOUSE_BUTTON_SECONDARY: c_int = 2;
pub const SERVO_MOUSE_BUTTON_BACK: c_int = 3;
pub const SERVO_MOUSE_BUTTON_FORWARD: c_int = 4;

/// Touch pointer types.
pub const SERVO_TOUCH_POINTER_TYPE_TOUCH: c_int = 0;
pub const SERVO_TOUCH_POINTER_TYPE_PEN: c_int = 1;

/// Modifier key bitflags (matches `keyboard_types::Modifiers`).
pub const SERVO_MODIFIER_ALT: u32 = 0x01;
pub const SERVO_MODIFIER_ALT_GRAPH: u32 = 0x02;
pub const SERVO_MODIFIER_CAPS_LOCK: u32 = 0x04;
pub const SERVO_MODIFIER_CONTROL: u32 = 0x08;
pub const SERVO_MODIFIER_FN: u32 = 0x10;
pub const SERVO_MODIFIER_FN_LOCK: u32 = 0x20;
pub const SERVO_MODIFIER_META: u32 = 0x40;
pub const SERVO_MODIFIER_NUM_LOCK: u32 = 0x80;
pub const SERVO_MODIFIER_SCROLL_LOCK: u32 = 0x100;
pub const SERVO_MODIFIER_SHIFT: u32 = 0x200;
pub const SERVO_MODIFIER_SYMBOL: u32 = 0x400;
pub const SERVO_MODIFIER_SYMBOL_LOCK: u32 = 0x800;

/// Wheel delta modes.
pub const SERVO_WHEEL_DELTA_PIXEL: c_int = 0x00;
pub const SERVO_WHEEL_DELTA_LINE: c_int = 0x01;
pub const SERVO_WHEEL_DELTA_PAGE: c_int = 0x02;

/// Media session action types matching `embedder_traits::MediaSessionActionType`.
pub const SERVO_MEDIA_SESSION_ACTION_PLAY: c_int = 0;
pub const SERVO_MEDIA_SESSION_ACTION_PAUSE: c_int = 1;
pub const SERVO_MEDIA_SESSION_ACTION_SEEK_BACKWARD: c_int = 2;
pub const SERVO_MEDIA_SESSION_ACTION_SEEK_FORWARD: c_int = 3;
pub const SERVO_MEDIA_SESSION_ACTION_PREVIOUS_TRACK: c_int = 4;
pub const SERVO_MEDIA_SESSION_ACTION_NEXT_TRACK: c_int = 5;
pub const SERVO_MEDIA_SESSION_ACTION_SKIP_AD: c_int = 6;
pub const SERVO_MEDIA_SESSION_ACTION_STOP: c_int = 7;

/// Writes `value` into a caller-provided NUL terminated UTF-8 buffer.
///
/// Returns the number of bytes required to store `value` including the
/// trailing NUL terminator. This allows the embedder to first query the
/// required length by passing `NULL` (or a too-small buffer) and then call
/// again with a large enough buffer.
///
/// # Safety
///
/// If `buffer` is non-`NULL`, it must point to at least `buffer_len` writable
/// bytes.
unsafe fn write_c_string(value: &str, buffer: *mut c_char, buffer_len: usize) -> usize {
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

use crate::rendering_context::RenderingContext;
use crate::user_content_manager::ServoUserContentManager;
use crate::webview_delegate::ServoWebViewDelegate;

/// An opaque struct representing a builder object for constructing new
/// `WebView`s.
///
/// Handles to this object can be created using [`servo_webview_builder_create`].
///
/// # Thread safety
///
/// The handle must be used only from the thread that created it.
/// It must also be created on the same thread that created the `Servo`
/// instance passed to `servo_webview_builder_create``.
// cbindgen:opaque
pub struct ServoWebViewBuilder {
    servo: Servo,
    rendering_context: Rc<dyn servo_api::RenderingContext>,
    url: Option<url::Url>,
    delegate: Option<ServoWebViewDelegate>,
    user_content_manager: Option<Rc<servo_api::UserContentManager>>,
    request_context: Option<servo_api::RequestContext>,
    hidpi_scale_factor: Option<f32>,
}

/// Creates a handle to a new `WebViewBuilder` object for the given
/// `servo` instance and rendering context.
///
/// `servo` is a handle to a `Servo` object.
/// The ownership of `servo` remains with the caller after the call.
///
/// `context` is a handle to a `RenderingContext` object.
/// The ownership of `context` is transferred to the function.
/// The caller must not use or free `context` again.
///
/// Returns a newly allocated `ServoWebViewBuilder` handle. The
/// ownership of the returned handle is transferred to the caller, who
/// must free it with [`servo_webview_builder_free`] or consume it by
/// passing it to `servo_webview_builder_build`. Returns `NULL` on failure.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `servo` is a non-null pointer to a `Servo` instance previously
///   returned by `servo_builder_build` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - `context` is a non-null pointer to a `RenderingContext` previously
///   returned by one of the `servo_rendering_context_create_*`
///   functions and has not yet been freed nor passed to another API
///   that takes ownership of it.
/// - The call is made from the same thread that created `servo` and
///   `context`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_builder_create(
    servo: *mut Servo,
    context: *mut RenderingContext,
) -> *mut ServoWebViewBuilder {
    ServoError::catch_ffi_ptr(|| {
        if servo.is_null() {
            return std::ptr::null_mut();
        }
        if context.is_null() {
            return std::ptr::null_mut();
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `servo` documented above.
        let servo = unsafe { &*servo };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `context` documented above. We take ownership here.
        let boxed_c_context = unsafe { Box::from_raw(context) };

        Box::into_raw(Box::new(ServoWebViewBuilder {
            servo: servo.clone(),
            rendering_context: boxed_c_context.inner,
            url: None,
            delegate: None,
            user_content_manager: None,
            request_context: None,
            hidpi_scale_factor: None,
        }))
    })
}

/// Sets the initial URL for the `WebView`.
///
/// `builder` is a handle to a `ServoWebViewBuilder` object.
/// The ownership of `builder` remains with the caller after the call.
///
/// `url` is a NUL terminated UTF-8 string.
/// The ownership of `url` remains with the caller after the call.
///
/// Returns `ServoError::Ok` (0) on success, `ServoError::InvalidUtf8` (-2)
/// if the URL is not valid UTF-8, or `ServoError::InvalidState` (-3) if the
/// URL could not be parsed.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `builder` is a non-null pointer to a `ServoWebViewBuilder`
///   previously returned by `servo_webview_builder_create` and has not
///   yet been freed nor passed to another API that takes ownership of
///   it.
/// - `url` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_builder_set_url(
    builder: *mut ServoWebViewBuilder,
    url: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if builder.is_null() {
            return ServoError::NullArgument;
        }
        if url.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `builder` documented above.
        let builder = unsafe { &mut *builder };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `url` documented above.
        let url_str = match unsafe { CStr::from_ptr(url) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8,
        };

        match url::Url::parse(url_str) {
            Ok(parsed) => {
                builder.url = Some(parsed);
                ServoError::Ok
            },
            Err(_) => ServoError::InvalidState,
        }
    })
}

/// Sets the delegate that will receive notification for `WebView` events.
///
/// `builder` is a handle to a `ServoWebViewBuilder` object.
/// The ownership of `builder` remains with the caller after the call.
///
/// `delegate` is a `ServoWebViewDelegate` struct with callbacks
/// for the notifications you are interested in.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `builder` is a non-null pointer to a `ServoWebViewBuilder`
///   previously returned by `servo_webview_builder_create` and has not
///   yet been freed nor passed to another API that takes ownership of
///   it.
/// - `delegate` must uphold the safety requirements documented on the
///    `ServoWebViewDelegate` type.
///
/// Servo copies the delegate's function pointers — they must remain
/// valid for the lifetime of the `WebView` (or until a new delegate is set).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_builder_set_delegate(
    builder: *mut ServoWebViewBuilder,
    delegate: ServoWebViewDelegate,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if builder.is_null() {
            return ServoError::NullArgument;
        }
        let builder = unsafe { &mut *builder };
        builder.delegate = Some(delegate);
        ServoError::Ok
    })
}

/// Sets the HiDPI scale factor for the `WebView` being built.
///
/// `builder` is a handle to a `ServoWebViewBuilder` object.
/// `scale` is the HiDPI scale factor (e.g., 1.0 for standard DPI, 2.0 for Retina/4K).
/// Must be positive and finite.
///
/// Returns `ServoError::Ok` on success, `ServoError::NullArgument` if `builder` is NULL,
/// or `ServoError::InvalidState` if `scale` is <= 0 or not finite.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_builder_set_hidpi_scale_factor(
    builder: *mut ServoWebViewBuilder,
    scale: f32,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if builder.is_null() {
            return ServoError::NullArgument;
        }
        if scale <= 0.0 || !scale.is_finite() {
            return ServoError::InvalidState;
        }
        unsafe { (*builder).hidpi_scale_factor = Some(scale); }
        ServoError::Ok
    })
}

/// Consumes `builder` and creates a new `WebView` instance.
///
/// `builder` is a handle to a `ServoWebViewBuilder` object.
/// The ownership of `builder` is transferred to the function. The
/// caller must not use or free `builder` again.
///
/// Returns a newly allocated `WebView` handle. The ownership of the
/// returned handle is transferred to the caller, who must free it with
/// [`servo_webview_free`]. Returns `NULL` on failure.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `builder` is a non-null pointer to a `ServoWebViewBuilder`
///   previously returned by `servo_webview_builder_create` and has not
///   yet been freed nor passed to another API that takes ownership of
///   it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_builder_build(
    builder: *mut ServoWebViewBuilder,
) -> *mut WebView {
    ServoError::catch_ffi_ptr(|| {
        if builder.is_null() {
            return std::ptr::null_mut();
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `builder` documented above. We take ownership here.
        let builder = unsafe { Box::from_raw(builder) };

        let mut webview_builder = WebViewBuilder::new(&builder.servo, builder.rendering_context);

        if let Some(url) = builder.url {
            webview_builder = webview_builder.url(url);
        }

        if let Some(delegate) = builder.delegate {
            webview_builder = webview_builder.delegate(std::rc::Rc::new(delegate));
        }

        if let Some(user_content_manager) = builder.user_content_manager {
            webview_builder = webview_builder.user_content_manager(user_content_manager);
        }

        if let Some(request_context) = builder.request_context {
            webview_builder = webview_builder.request_context(request_context);
        }

        if let Some(scale) = builder.hidpi_scale_factor {
            webview_builder = webview_builder.hidpi_scale_factor(euclid::Scale::new(scale));
        }

        Box::into_raw(Box::new(webview_builder.build()))
    })
}

/// Destroys `builder` and frees its memory.
///
/// `builder` is a handle to a `ServoWebViewBuilder` object.
/// The ownership of `builder` is transferred to the function. The
/// caller must not use or free `builder` again.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `builder` was previously returned by `servo_webview_builder_create`
///   and has not yet been freed nor passed to another API that takes
///   ownership of it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_builder_free(
    builder: *mut ServoWebViewBuilder,
) -> ServoError {
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

/// Paints the contents of the `WebView` to the rendering context's
/// surface.
///
/// Should be called when the embedder receives a
/// [`notify_new_frame_ready`] notification via [`ServoWebViewDelegate`]
/// or when a repaint is needed for other reasons.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `webview` is a non-null pointer to a `WebView` previously returned
///   by `servo_webview_builder_build` and not yet passed to
///   `servo_webview_free`. No other code may read or write `*webview`
///   for the duration of this call.
/// - The call is made from the same thread that originally created the
///   `WebView` via `servo_webview_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_paint(webview: *mut WebView) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `webview` documented above.
        let webview = unsafe { &*webview };

        webview.paint();

        ServoError::Ok
    })
}

/// Loads the given URL into the `WebView`.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `url` is a NUL terminated UTF-8 string.
/// The ownership of `url` remains with the caller after the call.
///
/// Returns `ServoError::Ok` (0) on success, `ServoError::InvalidUtf8` (-2)
/// if the URL is not valid UTF-8, or `ServoError::InvalidState` (-3) if the
/// URL could not be parsed.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `webview` is a non-null pointer to a `WebView` previously returned
///   by `servo_webview_builder_build` and not yet passed to
///   `servo_webview_free`. No other code may read or write `*webview`
///   for the duration of this call.
/// - `url` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
/// - The call is made from the same thread that originally created the
///   `WebView` via `servo_webview_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_load(
    webview: *mut WebView,
    url: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }
        if url.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `webview` documented above.
        let webview = unsafe { &*webview };

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `url` documented above.
        let url_str = match unsafe { CStr::from_ptr(url) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8,
        };

        match url::Url::parse(url_str) {
            Ok(parsed) => {
                webview.load(parsed);
                ServoError::Ok
            },
            Err(_) => ServoError::InvalidState,
        }
    })
}

/// Asynchronously takes a screenshot of the full `WebView` viewport.
///
/// The provided `callback` will be invoked with either the image data
/// when the screenshot is ready, or error information if Servo was not able
/// to handle the screenshot request successfully.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `callback` is a function pointer that will be invoked at
/// most once when the screenshot completes. The callback is invoked
/// asynchronously on the embedder thread that runs `servo_spin_event_loop`.
///
/// The callback receives:
/// - `data`  : pointer to raw pixel data, or `NULL` on error. Data is in
///             RGBA format with 4 bytes per pixel.
/// - `width` : image width in pixels or 0 on error.
/// - `height`: image height in pixels or 0 on error.
/// - `error` : error code represented by one of
///             the `SERVO_SCREENSHOT_CAPTURE_ERROR_*` constants.
///             Only valid when `data` is `NULL`.
/// - `user_data`: the user data pointer that was passed to`servo_webview_take_screenshot`.
///
/// The `data` pointer is valid only for the duration of the callback.
/// The callback must copy the data if it needs to be persist after the callback
/// returns. The ownership of `data` remains with Servo.
///
/// `user_data` is an opaque pointer that is passed to `callback` when it is invoked.
///  The ownership and validity of `user_data` is the caller's responsibility.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `webview` is a non-null pointer to a `WebView` previously returned
///   by `servo_webview_builder_build` and not yet passed to
///   `servo_webview_free`. No other code may read or write `*webview`
///   for the duration of this call.
/// - If `callback` is not null, it is a valid C ABI function matching
///   the exact signature shown, remains valid until it is invoked or
///   the `WebView` is freed, and does not unwind across the FFI
///   boundary.
/// - `user_data` is either null or remains valid until `callback` is
///   invoked or the `WebView` is freed.
/// - The call is made from the same thread that originally created the
///   `WebView` via `servo_webview_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_take_screenshot(
    webview: *mut WebView,
    callback: Option<
        unsafe extern "C" fn(
            data: *const u8,
            width: u32,
            height: u32,
            error: i32,
            user_data: *mut c_void,
        ),
    >,
    user_data: *mut c_void,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `webview` documented above.
        let webview = unsafe { &*webview };

        let Some(callback) = callback else {
            return ServoError::Ok;
        };

        webview.take_screenshot(None, move |result| {
            match result {
                Ok(image) => {
                    let (width, height) = image.dimensions();
                    let data = image.into_raw();
                    // SAFETY: The caller is assumed to uphold the safety
                    // requirements for `callback` documented above.
                    // `data.as_ptr()` is valid for the call as the backing `Vec`
                    // is dropped only after `callback` returns.
                    unsafe {
                        callback(data.as_ptr(), width, height, 0, user_data);
                    }
                },
                Err(error) => {
                    let error_code = error as _;
                    // SAFETY: The caller is assumed to uphold the safety
                    // requirements for `callback` documented above.
                    // `data` pointer is null as part of the contract.
                    unsafe {
                        callback(std::ptr::null(), 0, 0, error_code, user_data);
                    }
                },
            }
        });

        ServoError::Ok
    })
}

/// Destroys the `WebView` instance and frees its memory.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` is transferred to the function. The
/// caller must not use or free `webview` again.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `webview` was previously returned by `servo_webview_builder_build`
///   and has not yet been freed nor passed to another API that takes
///   ownership of it.
/// - The call is made from the same thread that originally created the
///   `WebView` via `servo_webview_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_free(webview: *mut WebView) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `webview` documented above.
        unsafe {
            let _ = Box::from_raw(webview);
        }

        ServoError::Ok
    })
}

/// Writes the URL currently loaded in `webview` into `buffer` as a NUL
/// terminated UTF-8 string.
///
/// Returns the number of bytes required to store the URL including the
/// trailing NUL terminator. If `buffer` is `NULL` or `buffer_len` is too
/// small, nothing is written and the required length is returned so the
/// embedder can resize its buffer and call again.
///
/// # Safety
///
/// - `webview` must be a non-null pointer to a `WebView` previously returned
///   by `servo_webview_builder_build` and not yet freed. No other code may
///   read or write `*webview` for the duration of this call.
/// - If `buffer` is non-null it must point to at least `buffer_len` writable
///   bytes.
/// - The call must be made from the thread that created the `WebView`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_get_url(
    webview: *const WebView,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    if webview.is_null() {
        return 0;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements for
    // `webview` documented above.
    let webview = unsafe { &*webview };
    let url = webview.url().map(|url| url.to_string()).unwrap_or_default();

    // SAFETY: The caller is assumed to uphold the safety requirements for
    // `buffer` documented above.
    unsafe { write_c_string(&url, buffer, buffer_len) }
}

/// Writes the title of the page loaded in `webview` into `buffer` as a NUL
/// terminated UTF-8 string.
///
/// Returns the number of bytes required to store the title including the
/// trailing NUL terminator. If `buffer` is `NULL` or `buffer_len` is too
/// small, nothing is written and the required length is returned.
///
/// # Safety
///
/// See [`servo_webview_get_url`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_get_title(
    webview: *const WebView,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    if webview.is_null() {
        return 0;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements for
    // `webview` documented above.
    let webview = unsafe { &*webview };
    let title = webview.page_title().unwrap_or_default();

    // SAFETY: The caller is assumed to uphold the safety requirements for
    // `buffer` documented above.
    unsafe { write_c_string(&title, buffer, buffer_len) }
}

/// Writes the status text of the `WebView` (for example the target of the link
/// currently under the pointer) into `buffer` as a NUL terminated UTF-8 string.
///
/// Returns the number of bytes required to store the text including the
/// trailing NUL terminator. If `buffer` is `NULL` or `buffer_len` is too
/// small, nothing is written and the required length is returned.
///
/// # Safety
///
/// See [`servo_webview_get_url`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_get_status_text(
    webview: *const WebView,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    if webview.is_null() {
        return 0;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements for
    // `webview` documented above.
    let webview = unsafe { &*webview };
    let status = webview.status_text().unwrap_or_default();

    // SAFETY: The caller is assumed to uphold the safety requirements for
    // `buffer` documented above.
    unsafe { write_c_string(&status, buffer, buffer_len) }
}

/// Returns the load status of `webview`.
///
/// The returned value is one of the `SERVO_LOAD_STATUS_*` constants. Note that
/// the load status can change between calls: for the most reliable results,
/// observe [`ServoWebViewDelegate::notify_load_status_changed`] instead.
///
/// # Safety
///
/// See [`servo_webview_get_url`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_get_load_status(webview: *const WebView) -> c_int {
    if webview.is_null() {
        return 0;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements for
    // `webview` documented above.
    unsafe { &*webview }.load_status() as c_int
}

/// Writes the size of the `WebView`'s viewport in device pixels into
/// `out_width` and `out_height`.
///
/// Either out parameter may be `NULL` if the embedder is only interested in
/// the other dimension.
///
/// # Safety
///
/// - `webview` must be a non-null pointer to a `WebView` previously returned
///   by `servo_webview_builder_build` and not yet freed.
/// - If `out_width` / `out_height` are non-null they must be valid for writes.
/// - The call must be made from the thread that created the `WebView`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_get_size(
    webview: *const WebView,
    out_width: *mut u32,
    out_height: *mut u32,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let size = unsafe { &*webview }.size();

        if !out_width.is_null() {
            // SAFETY: The caller guarantees `out_width` is valid for writes.
            unsafe { *out_width = size.width.max(0.0) as u32 };
        }

        if !out_height.is_null() {
            // SAFETY: The caller guarantees `out_height` is valid for writes.
            unsafe { *out_height = size.height.max(0.0) as u32 };
        }

        ServoError::Ok
    })
}

/// Returns `true` if `webview` can navigate backwards in its session history.
///
/// # Safety
///
/// See [`servo_webview_get_url`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_can_go_back(webview: *const WebView) -> bool {
    if webview.is_null() {
        return false;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements for
    // `webview` documented above.
    unsafe { &*webview }.can_go_back()
}

/// Returns `true` if `webview` can navigate forwards in its session history.
///
/// # Safety
///
/// See [`servo_webview_get_url`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_can_go_forward(webview: *const WebView) -> bool {
    if webview.is_null() {
        return false;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements for
    // `webview` documented above.
    unsafe { &*webview }.can_go_forward()
}

/// Navigates `webview` backwards by `amount` entries in its session history.
///
/// This is a no-op if there are fewer than `amount` entries to go back to.
///
/// # Safety
///
/// - `webview` must be a non-null pointer to a `WebView` previously returned
///   by `servo_webview_builder_build` and not yet freed. No other code may
///   read or write `*webview` for the duration of this call.
/// - The call must be made from the thread that created the `WebView`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_go_back(webview: *mut WebView, amount: usize) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let _ = unsafe { &*webview }.go_back(amount);

        ServoError::Ok
    })
}

/// Navigates `webview` forwards by `amount` entries in its session history.
///
/// This is a no-op if there are fewer than `amount` entries to go forward to.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_go_forward(
    webview: *mut WebView,
    amount: usize,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let _ = unsafe { &*webview }.go_forward(amount);

        ServoError::Ok
    })
}

/// Reloads the current page in `webview`.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_reload(webview: *mut WebView) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        unsafe { &*webview }.reload();

        ServoError::Ok
    })
}

/// Clears the session history of `webview`.
///
/// The currently loaded page is retained, but navigating back and forward is
/// no longer possible until new navigations occur.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_clear_session_history(webview: *mut WebView) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        unsafe { &*webview }.clear_session_history();

        ServoError::Ok
    })
}

/// Sets the page zoom factor of `webview`.
///
/// The value is clamped by Servo to the range `[0.1, 10.0]`. A value of `1.0`
/// corresponds to the default, unzoomed state.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_set_page_zoom(
    webview: *mut WebView,
    zoom: f32,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        unsafe { &*webview }.set_page_zoom(zoom);

        ServoError::Ok
    })
}

/// Returns the current page zoom factor of `webview`.
///
/// # Safety
///
/// See [`servo_webview_get_url`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_page_zoom(webview: *const WebView) -> f32 {
    if webview.is_null() {
        return 1.0;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements for
    // `webview` documented above.
    unsafe { &*webview }.page_zoom()
}

/// Resizes the `WebView` viewport to `width` by `height` device pixels.
///
/// The embedder is responsible for resizing the associated
/// `RenderingContext` to the same size.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_resize(
    webview: *mut WebView,
    width: u32,
    height: u32,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        unsafe { &*webview }.resize(dpi::PhysicalSize::new(width, height));

        ServoError::Ok
    })
}

/// Notifies `webview` that its window gained or lost input focus.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_set_focused(
    webview: *mut WebView,
    focused: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        unsafe { &*webview }.set_focused(focused);

        ServoError::Ok
    })
}

/// Returns whether `webview` is currently focused.
///
/// # Safety
///
/// See [`servo_webview_get_url`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_focused(webview: *const WebView) -> bool {
    if webview.is_null() {
        return false;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements for
    // `webview` documented above.
    unsafe { &*webview }.focused()
}

/// Marks `webview` as visible so that it resumes producing frames.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_show(webview: *mut WebView) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        unsafe { &*webview }.show();

        ServoError::Ok
    })
}

/// Marks `webview` as hidden so that Servo can stop producing frames for it.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_hide(webview: *mut WebView) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        unsafe { &*webview }.hide();

        ServoError::Ok
    })
}

/// Notifies `webview` that the platform color scheme changed.
///
/// `theme` must be one of `SERVO_THEME_LIGHT` or `SERVO_THEME_DARK`. Invalid
/// values are ignored and a warning is logged.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_notify_theme_change(
    webview: *mut WebView,
    theme: c_int,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        let theme = match theme {
            SERVO_THEME_LIGHT => servo_api::Theme::Light,
            SERVO_THEME_DARK => servo_api::Theme::Dark,
            other => {
                log::warn!("Invalid theme value: {other}");
                return ServoError::Ok;
            },
        };

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        unsafe { &*webview }.notify_theme_change(theme);

        ServoError::Ok
    })
}

/// Notifies `webview` of a keyboard event.
///
/// `out_event_id` will be set to the `InputEventId` that can be used with
/// `servo_webview_notify_input_event_handled` to determine the result of
/// processing the event. May be `NULL` if the caller does not need the ID.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview`, `key`, or `code` is null.
/// Returns `ServoError::InvalidUtf8` if `key` or `code` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_notify_key_event(
    webview: *mut WebView,
    event_type: c_int,
    key: *const c_char,
    code: *const c_char,
    location: c_int,
    modifiers: u32,
    repeat: bool,
    is_composing: bool,
    out_event_id: *mut u64,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }
        if key.is_null() {
            return ServoError::NullArgument;
        }
        if code.is_null() {
            return ServoError::NullArgument;
        }

        let key_str = match unsafe { CStr::from_ptr(key) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8,
        };
        let code_str = match unsafe { CStr::from_ptr(code) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8,
        };

        let state = match event_type {
            SERVO_KEY_EVENT_DOWN => KeyState::Down,
            SERVO_KEY_EVENT_UP => KeyState::Up,
            _ => {
                log::warn!("Invalid key event type: {event_type}");
                return ServoError::InvalidArgument;
            },
        };

        let location = match location {
            0 => Location::Standard,
            1 => Location::Left,
            2 => Location::Right,
            3 => Location::Numpad,
            _ => {
                log::warn!("Invalid key location: {location}");
                Location::Standard
            },
        };

        let modifiers = Modifiers::from_bits_truncate(modifiers);

        let key = if key_str.len() == 1 && key_str.chars().next().is_some_and(|c| !c.is_control()) {
            Key::Character(key_str.to_string())
        } else {
            Key::Named(NamedKey::from_str(key_str).unwrap_or(NamedKey::Unidentified))
        };

        let code = Code::from_str(code_str).unwrap_or(Code::Unidentified);

        let keyboard_event = keyboard_types::KeyboardEvent {
            state,
            key,
            code,
            location,
            modifiers,
            repeat,
            is_composing,
        };

        let event = InputEvent::Keyboard(KeyboardEvent::new(keyboard_event));
        let event_id = unsafe { &*webview }.notify_input_event(event);

        if !out_event_id.is_null() {
            unsafe { *out_event_id = event_id.0 as u64 };
        }

        ServoError::Ok
    })
}

/// Notifies `webview` of a mouse button event.
///
/// `out_event_id` will be set to the `InputEventId` that can be used with
/// `servo_webview_notify_input_event_handled` to determine the result of
/// processing the event. May be `NULL` if the caller does not need the ID.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_notify_mouse_button_event(
    webview: *mut WebView,
    event_type: c_int,
    button: c_int,
    x: f64,
    y: f64,
    modifiers: u32,
    out_event_id: *mut u64,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        let action = match event_type {
            SERVO_MOUSE_EVENT_DOWN => MouseButtonAction::Down,
            SERVO_MOUSE_EVENT_UP => MouseButtonAction::Up,
            _ => {
                log::warn!("Invalid mouse button event type: {event_type}");
                return ServoError::InvalidArgument;
            },
        };

        let button = match button {
            SERVO_MOUSE_BUTTON_NONE => MouseButton::None,
            SERVO_MOUSE_BUTTON_PRIMARY => MouseButton::Primary,
            SERVO_MOUSE_BUTTON_AUXILIARY => MouseButton::Auxiliary,
            SERVO_MOUSE_BUTTON_SECONDARY => MouseButton::Secondary,
            SERVO_MOUSE_BUTTON_BACK => MouseButton::Back,
            SERVO_MOUSE_BUTTON_FORWARD => MouseButton::Forward,
            other => MouseButton::Other(other as u16),
        };

        let point = WebViewPoint::Device(dpi::DevicePoint::new(x as f32, y as f32));

        let event = InputEvent::MouseButton(MouseButtonEvent::new(action, button, point));
        let event_id = unsafe { &*webview }.notify_input_event(event);

        if !out_event_id.is_null() {
            unsafe { *out_event_id = event_id.0 as u64 };
        }

        ServoError::Ok
    })
}

/// Notifies `webview` of a mouse move event.
///
/// `out_event_id` will be set to the `InputEventId` that can be used with
/// `servo_webview_notify_input_event_handled` to determine the result of
/// processing the event. May be `NULL` if the caller does not need the ID.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_notify_mouse_move_event(
    webview: *mut WebView,
    x: f64,
    y: f64,
    modifiers: u32,
    out_event_id: *mut u64,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        let point = WebViewPoint::Device(dpi::DevicePoint::new(x as f32, y as f32));

        let event = InputEvent::MouseMove(MouseMoveEvent::new(point));
        let event_id = unsafe { &*webview }.notify_input_event(event);

        if !out_event_id.is_null() {
            unsafe { *out_event_id = event_id.0 as u64 };
        }

        ServoError::Ok
    })
}

/// Notifies `webview` that the mouse left the viewport.
///
/// `out_event_id` will be set to the `InputEventId` that can be used with
/// `servo_webview_notify_input_event_handled` to determine the result of
/// processing the event. May be `NULL` if the caller does not need the ID.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_notify_mouse_leave_event(
    webview: *mut WebView,
    focus_moving_to_another_iframe: bool,
    out_event_id: *mut u64,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        let event = InputEvent::MouseLeftViewport(
            embedder_traits::input_events::MouseLeftViewportEvent {
                focus_moving_to_another_iframe,
            },
        );
        let event_id = unsafe { &*webview }.notify_input_event(event);

        if !out_event_id.is_null() {
            unsafe { *out_event_id = event_id.0 as u64 };
        }

        ServoError::Ok
    })
}

/// Notifies `webview` of a touch event.
///
/// `out_event_id` will be set to the `InputEventId` that can be used with
/// `servo_webview_notify_input_event_handled` to determine the result of
/// processing the event. May be `NULL` if the caller does not need the ID.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_notify_touch_event(
    webview: *mut WebView,
    event_type: c_int,
    touch_id: i32,
    x: f64,
    y: f64,
    pointer_type: c_int,
    out_event_id: *mut u64,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        let event_type = match event_type {
            SERVO_TOUCH_EVENT_DOWN => TouchEventType::Down,
            SERVO_TOUCH_EVENT_MOVE => TouchEventType::Move,
            SERVO_TOUCH_EVENT_UP => TouchEventType::Up,
            SERVO_TOUCH_EVENT_CANCEL => TouchEventType::Cancel,
            _ => {
                log::warn!("Invalid touch event type: {event_type}");
                return ServoError::InvalidArgument;
            },
        };

        let pointer_type = match pointer_type {
            SERVO_TOUCH_POINTER_TYPE_TOUCH => TouchPointerType::Touch,
            SERVO_TOUCH_POINTER_TYPE_PEN => TouchPointerType::Pen,
            _ => {
                log::warn!("Invalid touch pointer type: {pointer_type}");
                TouchPointerType::Touch
            },
        };

        let point = WebViewPoint::Device(dpi::DevicePoint::new(x as f32, y as f32));

        let event = InputEvent::Touch(TouchEvent::new(
            event_type,
            TouchId(touch_id),
            point,
            pointer_type,
        ));
        let event_id = unsafe { &*webview }.notify_input_event(event);

        if !out_event_id.is_null() {
            unsafe { *out_event_id = event_id.0 as u64 };
        }

        ServoError::Ok
    })
}

/// Notifies `webview` of a wheel event.
///
/// `out_event_id` will be set to the `InputEventId` that can be used with
/// `servo_webview_notify_input_event_handled` to determine the result of
/// processing the event. May be `NULL` if the caller does not need the ID.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_notify_wheel_event(
    webview: *mut WebView,
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    delta_mode: c_int,
    x: f64,
    y: f64,
    out_event_id: *mut u64,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        let mode = match delta_mode {
            SERVO_WHEEL_DELTA_PIXEL => WheelMode::DeltaPixel,
            SERVO_WHEEL_DELTA_LINE => WheelMode::DeltaLine,
            SERVO_WHEEL_DELTA_PAGE => WheelMode::DeltaPage,
            _ => {
                log::warn!("Invalid wheel delta mode: {delta_mode}");
                WheelMode::DeltaPixel
            },
        };

        let delta = WheelDelta {
            x: delta_x,
            y: delta_y,
            z: delta_z,
            mode,
        };

        let point = WebViewPoint::Device(dpi::DevicePoint::new(x as f32, y as f32));

        let event = InputEvent::Wheel(WheelEvent::new(delta, point));
        let event_id = unsafe { &*webview }.notify_input_event(event);

        if !out_event_id.is_null() {
            unsafe { *out_event_id = event_id.0 as u64 };
        }

        ServoError::Ok
    })
}

/// Notifies `webview` that an input event has been handled by the embedder.
///
/// This should be called in response to the `notify_input_event_handled`
/// delegate callback to inform Servo that the embedder has finished processing
/// the event outcome.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_notify_input_event_handled(
    webview: *mut WebView,
    event_id: u64,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        let event_id = InputEventId(event_id as usize);
        unsafe { &*webview }.notify_input_event_handled(event_id);

        ServoError::Ok
    })
}

/// Requests that the `WebView` begin a cooperative close sequence.
///
/// This is equivalent to the page calling `window.close()`. Servo will
/// initiate the close process, which includes firing `beforeunload` and
/// `unload` events, and will eventually invoke the
/// `ServoWebViewDelegate::notify_closed` callback when the close is
/// complete. The embedder should call `servo_webview_free` after
/// receiving `notify_closed` (or immediately if it does not wait for
/// the callback) to release the `WebView` resources.
///
/// `webview` is a handle to a `WebView` object.
/// - The call is made from the same thread that originally created the
///   `WebView` via `servo_webview_builder_build`.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_close(webview: *mut WebView) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let webview = unsafe { &*webview };
        webview.close();

        ServoError::Ok
    })
}

/// Notifies `webview` that its window has been resized.
///
/// The embedder should call this when the containing window's size changes.
/// This will notify the delegate via `ServoWebViewDelegate::on_window_resized`.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `width` and `height` are the new window size in device pixels.
///
/// Returns `ServoError::Ok` (0) on success.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_was_resized(
    webview: *mut WebView,
    width: u32,
    height: u32,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let webview = unsafe { &*webview };
        webview.was_resized(width, height);

        ServoError::Ok
    })
}

/// Notifies `webview` that its window has been moved.
///
/// The embedder should call this when the containing window's position changes.
/// This will notify the delegate via `ServoWebViewDelegate::on_window_moved`.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `x` and `y` are the new window position in device pixels.
///
/// Returns `ServoError::Ok` (0) on success.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_was_moved(
    webview: *mut WebView,
    x: i32,
    y: i32,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let webview = unsafe { &*webview };
        webview.was_moved(x, y);

        ServoError::Ok
    })
}

/// Notifies `webview` that its window has been hidden (minimized).
///
/// The embedder should call this when the containing window is minimized.
/// This will notify the delegate via `ServoWebViewDelegate::on_window_minimized`.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// Returns `ServoError::Ok` (0) on success.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_was_hidden(webview: *mut WebView) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let webview = unsafe { &*webview };
        webview.was_hidden();

        ServoError::Ok
    })
}

/// Notifies `webview` that its window has been shown (restored).
///
/// The embedder should call this when the containing window is restored
/// from minimized or maximized state.
/// This will notify the delegate via `ServoWebViewDelegate::on_window_restored`.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// Returns `ServoError::Ok` (0) on success.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_was_shown(webview: *mut WebView) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let webview = unsafe { &*webview };
        webview.was_shown();

        ServoError::Ok
    })
}

/// Notifies `webview` that its window has been maximized.
///
/// The embedder should call this when the containing window is maximized.
/// This will notify the delegate via `ServoWebViewDelegate::on_window_maximized`.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// Returns `ServoError::Ok` (0) on success.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_was_maximized(webview: *mut WebView) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let webview = unsafe { &*webview };
        webview.was_maximized();

        ServoError::Ok
    })
}

/// Notifies `webview` that a window move or resize operation has started.
///
/// The embedder should call this when the user begins dragging the window
/// title bar or resize handle.
/// This will notify the delegate via `ServoWebViewDelegate::on_window_move_or_resize_started`.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// Returns `ServoError::Ok` (0) on success.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_notify_move_or_resize_started(
    webview: *mut WebView,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let webview = unsafe { &*webview };
        webview.notify_move_or_resize_started();

        ServoError::Ok
    })
}

/// Sets the fullscreen state of the window containing `webview`.
///
/// The embedder should call this to programmatically enter or exit fullscreen mode.
/// This will notify the delegate via `ServoWebViewDelegate::on_window_fullscreen_changed`.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `fullscreen` is `true` to enter fullscreen, `false` to exit.
///
/// Returns `ServoError::Ok` (0) on success.
///
/// # Safety
///
/// See [`servo_webview_go_back`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_set_fullscreen(
    webview: *mut WebView,
    fullscreen: bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `webview` documented above.
        let webview = unsafe { &*webview };
        webview.set_fullscreen(fullscreen);

        ServoError::Ok
    })
}

/// Returns whether the window containing `webview` is currently in fullscreen mode.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// Returns `true` if fullscreen, `false` otherwise.
///
/// # Safety
///
/// See [`servo_webview_get_url`] for the safety requirements, which apply
/// identically here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_is_fullscreen(webview: *const WebView) -> bool {
    if webview.is_null() {
        return false;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements for
    // `webview` documented above.
    unsafe { &*webview }.is_fullscreen()
}

/// Gets the window bounds (position and size) of the window containing `webview`.
///
/// The bounds are returned in device pixels via out parameters.
/// The embedder must implement `screen_geometry` in the delegate to provide this information.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `x`, `y`, `width`, `height` are out parameters. Any may be `NULL` if not needed.
///
/// # Safety
///
/// - `webview` must be a non-null pointer to a `WebView` previously returned
///   by `servo_webview_builder_build` and not yet freed.
/// - If `x`, `y`, `width`, `height` are non-null they must be valid for writes.
/// - The call must be made from the thread that created the `WebView`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_get_window_bounds(
    webview: *const WebView,
    x: *mut i32,
    y: *mut i32,
    width: *mut u32,
    height: *mut u32,
) {
    if webview.is_null() {
        return;
    }

    // SAFETY: The caller is assumed to uphold the safety requirements for
    // `webview` documented above.
    let webview = unsafe { &*webview };
    if let Some((wx, wy, ww, wh)) = webview.window_bounds() {
        if !x.is_null() {
            unsafe { *x = wx };
        }
        if !y.is_null() {
            unsafe { *y = wy };
        }
        if !width.is_null() {
            unsafe { *width = ww };
        }
        if !height.is_null() {
            unsafe { *height = wh };
        }
    }
}

/// Updates the HiDPI scale factor for `webview`.
///
/// `webview` is a handle to a `WebView` object.
/// `scale` is the new HiDPI scale factor. Must be positive and finite.
///
/// Returns `ServoError::Ok` on success, `ServoError::NullArgument` if `webview` is NULL,
/// or `ServoError::InvalidState` if `scale` is <= 0 or not finite.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_set_hidpi_scale_factor(
    webview: *mut WebView,
    scale: f32,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }
        if scale <= 0.0 || !scale.is_finite() {
            return ServoError::InvalidState;
        }
        unsafe { &*webview }.set_hidpi_scale_factor(euclid::Scale::new(scale));
        ServoError::Ok
    })
}

/// Reads the current HiDPI scale factor of `webview`.
///
/// `webview` is a handle to a `WebView` object.
/// `out_scale` is a pointer to write the scale factor to.
///
/// Returns `ServoError::Ok` on success, or `ServoError::NullArgument` if any argument is NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_get_hidpi_scale_factor(
    webview: *const WebView,
    out_scale: *mut f32,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() || out_scale.is_null() {
            return ServoError::NullArgument;
        }
        unsafe { *out_scale = (&*webview).hidpi_scale_factor().get(); }
        ServoError::Ok
    })
}

/// Checks whether `webview` is currently animating (e.g., CSS animations, requestAnimationFrame).
///
/// When animating, the embedder should regularly pump the event loop via `servo_spin_event_loop`.
///
/// Returns `ServoError::Ok` on success, or `ServoError::NullArgument` if any pointer is NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_is_animating(
    webview: *const WebView,
    out_animating: *mut bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() || out_animating.is_null() {
            return ServoError::NullArgument;
        }
        unsafe { *out_animating = (&*webview).animating(); }
        ServoError::Ok
    })
}

/// Reads the current pinch zoom factor of `webview`.
///
/// Returns `ServoError::Ok` on success, or `ServoError::NullArgument` if any argument is NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_get_pinch_zoom(
    webview: *const WebView,
    out_zoom: *mut f32,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() || out_zoom.is_null() {
            return ServoError::NullArgument;
        }
        unsafe { *out_zoom = (&*webview).pinch_zoom(); }
        ServoError::Ok
    })
}

/// Adjusts the pinch zoom factor of `webview` by `pinch_zoom_delta` centered at (`center_x`, `center_y`).
///
/// Returns `ServoError::Ok` on success, or `ServoError::NullArgument` if `webview` is NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_adjust_pinch_zoom(
    webview: *mut WebView,
    pinch_zoom_delta: f32,
    center_x: f32,
    center_y: f32,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }
        let center = euclid::Point2D::new(center_x, center_y);
        unsafe { &*webview }.adjust_pinch_zoom(pinch_zoom_delta, center);
        ServoError::Ok
    })
}

/// Requests `webview` to exit fullscreen mode.
///
/// Returns `ServoError::Ok` on success, or `ServoError::NullArgument` if `webview` is NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_exit_fullscreen(webview: *mut WebView) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }
        unsafe { &*webview }.exit_fullscreen();
        ServoError::Ok
    })
}

/// Loads a URL in `webview` with custom HTTP headers.
///
/// `webview` is a handle to a `WebView` object.
/// `url` is a NUL-terminated UTF-8 string of the URL to load.
/// `headers` is an array of NUL-terminated strings formatted as `"Header-Name: Header-Value"`.
/// `headers_count` is the number of headers in the array. Can be 0 if `headers` is NULL.
///
/// Returns `ServoError::Ok` on success, `ServoError::NullArgument` if `webview` or `url` is NULL,
/// or `ServoError::InvalidUtf8` if strings are not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_load_request(
    webview: *mut WebView,
    url: *const c_char,
    headers: *const *const c_char,
    headers_count: usize,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() || url.is_null() {
            return ServoError::NullArgument;
        }

        let url_str = match unsafe { CStr::from_ptr(url) }.to_str() {
            Ok(s) => s,
            Err(_) => return ServoError::InvalidUtf8,
        };

        let parsed_url = match url::Url::parse(url_str) {
            Ok(u) => u,
            Err(_) => return ServoError::InvalidState,
        };

        let mut header_map = http::HeaderMap::new();
        if !headers.is_null() && headers_count > 0 {
            for i in 0..headers_count {
                let h_ptr = unsafe { *headers.add(i) };
                if h_ptr.is_null() {
                    continue;
                }
                let h_str = match unsafe { CStr::from_ptr(h_ptr) }.to_str() {
                    Ok(s) => s,
                    Err(_) => return ServoError::InvalidUtf8,
                };
                if let Some((name, val)) = h_str.split_once(':') {
                    if let (Ok(name), Ok(val)) = (
                        http::header::HeaderName::from_bytes(name.trim().as_bytes()),
                        http::header::HeaderValue::from_str(val.trim()),
                    ) {
                        header_map.insert(name, val);
                    }
                }
            }
        }

        let url_request = embedder_traits::UrlRequest {
            url: parsed_url.into(),
            headers: header_map,
        };

        unsafe { &*webview }.load_request(url_request);
        ServoError::Ok
    })
}

/// Scrolls the scrollable area of `webview` by (`dx`, `dy`) CSS pixels.
///
/// # Safety
///
/// `webview` must be a valid, non-null pointer to a `WebView` instance.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_scroll_by(
    webview: *mut WebView,
    dx: f32,
    dy: f32,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        let delta = WebViewVector::new(dx, dy);
        unsafe { &*webview }.notify_scroll_event(
            Scroll::Delta(delta),
            WebViewPoint::zero(),
        );

        ServoError::Ok
    })
}

/// Scrolls the scrollable area of `webview` to the top / start.
///
/// # Safety
///
/// `webview` must be a valid, non-null pointer to a `WebView` instance.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_scroll_to_start(
    webview: *mut WebView,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        unsafe { &*webview }.notify_scroll_event(
            Scroll::Start,
            WebViewPoint::zero(),
        );

        ServoError::Ok
    })
}

/// Scrolls the scrollable area of `webview` to the bottom / end.
///
/// # Safety
///
/// `webview` must be a valid, non-null pointer to a `WebView` instance.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_scroll_to_end(
    webview: *mut WebView,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        unsafe { &*webview }.notify_scroll_event(
            Scroll::End,
            WebViewPoint::zero(),
        );

        ServoError::Ok
    })
}

/// Notifies `webview` that IME text composition has started.
///
/// # Safety
///
/// `webview` must be a valid, non-null pointer to a `WebView` instance.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_notify_ime_composition_start(
    webview: *mut WebView,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        let event = InputEvent::Ime(ImeEvent::Composition(
            CompositionEvent {
                state: CompositionState::Start,
                data: String::new(),
            },
        ));
        unsafe { &*webview }.notify_input_event(event);

        ServoError::Ok
    })
}

/// Notifies `webview` of an IME text composition update (preedit text).
///
/// # Safety
///
/// `webview` must be a valid, non-null pointer to a `WebView` instance.
/// `text` must point to a valid NUL-terminated UTF-8 string.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` or `text` is null.
/// Returns `ServoError::InvalidUtf8` if `text` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_notify_ime_composition_update(
    webview: *mut WebView,
    text: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() || text.is_null() {
            return ServoError::NullArgument;
        }

        let text_str = match unsafe { CStr::from_ptr(text) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };

        let event = InputEvent::Ime(ImeEvent::Composition(
            CompositionEvent {
                state: CompositionState::Update,
                data: text_str,
            },
        ));
        unsafe { &*webview }.notify_input_event(event);

        ServoError::Ok
    })
}

/// Notifies `webview` that IME text composition has ended (committed text).
///
/// # Safety
///
/// `webview` must be a valid, non-null pointer to a `WebView` instance.
/// `text` must point to a valid NUL-terminated UTF-8 string.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` or `text` is null.
/// Returns `ServoError::InvalidUtf8` if `text` is not valid UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_notify_ime_composition_end(
    webview: *mut WebView,
    text: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() || text.is_null() {
            return ServoError::NullArgument;
        }

        let text_str = match unsafe { CStr::from_ptr(text) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };

        let wv = unsafe { &*webview };
        wv.notify_input_event(InputEvent::Ime(
            ImeEvent::Composition(
                CompositionEvent {
                    state: CompositionState::Update,
                    data: text_str.clone(),
                },
            ),
        ));
        wv.notify_input_event(InputEvent::Ime(
            ImeEvent::Composition(
                CompositionEvent {
                    state: CompositionState::End,
                    data: text_str,
                },
            ),
        ));

        ServoError::Ok
    })
}

/// Notifies `webview` that IME has been dismissed / canceled.
///
/// # Safety
///
/// `webview` must be a valid, non-null pointer to a `WebView` instance.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_notify_ime_dismissed(
    webview: *mut WebView,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        let event = InputEvent::Ime(ImeEvent::Dismissed);
        unsafe { &*webview }.notify_input_event(event);

        ServoError::Ok
    })
}

/// Notifies `webview` of a media session action (e.g. play, pause, next track).
///
/// `action` is one of `SERVO_MEDIA_SESSION_ACTION_*`.
///
/// # Safety
///
/// `webview` must be a valid, non-null pointer to a `WebView` instance.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if `webview` is null.
/// Returns `ServoError::InvalidArgument` if `action` is not recognized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_notify_media_session_action(
    webview: *mut WebView,
    action: c_int,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() {
            return ServoError::NullArgument;
        }

        let action_type = match action {
            SERVO_MEDIA_SESSION_ACTION_PLAY => MediaSessionActionType::Play,
            SERVO_MEDIA_SESSION_ACTION_PAUSE => MediaSessionActionType::Pause,
            SERVO_MEDIA_SESSION_ACTION_SEEK_BACKWARD => {
                MediaSessionActionType::SeekBackward
            },
            SERVO_MEDIA_SESSION_ACTION_SEEK_FORWARD => {
                MediaSessionActionType::SeekForward
            },
            SERVO_MEDIA_SESSION_ACTION_PREVIOUS_TRACK => {
                MediaSessionActionType::PreviousTrack
            },
            SERVO_MEDIA_SESSION_ACTION_NEXT_TRACK => {
                MediaSessionActionType::NextTrack
            },
            SERVO_MEDIA_SESSION_ACTION_SKIP_AD => MediaSessionActionType::SkipAd,
            SERVO_MEDIA_SESSION_ACTION_STOP => MediaSessionActionType::Stop,
            _ => return ServoError::InvalidArgument,
        };

        unsafe { &*webview }.notify_media_session_action_event(action_type);

        ServoError::Ok
    })
}
