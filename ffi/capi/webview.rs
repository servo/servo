/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{CStr, c_void};
use std::os::raw::c_char;
use std::rc::Rc;

use servo_api::{Servo, WebView, WebViewBuilder};

use crate::rendering_context::RenderingContext;
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
/// passing it to `servo_webview_builder_build`.
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
    assert!(!servo.is_null(), "servo pointer must not be null");
    assert!(!context.is_null(), "context pointer must not be null");

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
    }))
}

/// Sets the initial URL for the `WebView`.
///
/// `builder` is a handle to a `ServoWebViewBuilder` object.
/// The ownership of `builder` remains with the caller after the call.
///
/// `url` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `url` remains with the caller after the call.
///
/// Returns 0 on success, or -1 if the URL could not be parsed.
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
) -> i32 {
    assert!(!builder.is_null(), "builder pointer must not be null");
    assert!(!url.is_null(), "url pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `builder` documented above.
    let builder = unsafe { &mut *builder };

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `url` documented above.
    let url_str = unsafe { CStr::from_ptr(url) }.to_str().unwrap();

    match url::Url::parse(url_str) {
        Ok(parsed) => {
            builder.url = Some(parsed);
            0
        },
        Err(_) => -1,
    }
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
) {
    assert!(!builder.is_null(), "builder pointer must not be null");
    let builder = unsafe { &mut *builder };
    builder.delegate = Some(delegate);
}

/// Consumes `builder` and creates a new `WebView` instance.
///
/// `builder` is a handle to a `ServoWebViewBuilder` object.
/// The ownership of `builder` is transferred to the function. The
/// caller must not use or free `builder` again.
///
/// Returns a newly allocated `WebView` handle. The ownership of the
/// returned handle is transferred to the caller, who must free it with
/// [`servo_webview_free`].
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
    assert!(!builder.is_null(), "builder pointer must not be null");

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

    Box::into_raw(Box::new(webview_builder.build()))
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
pub unsafe extern "C" fn servo_webview_builder_free(builder: *mut ServoWebViewBuilder) {
    assert!(!builder.is_null(), "builder pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `builder` documented above.
    unsafe {
        let _ = Box::from_raw(builder);
    }
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
pub unsafe extern "C" fn servo_webview_paint(webview: *mut WebView) {
    assert!(!webview.is_null(), "webview pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `webview` documented above.
    let webview = unsafe { &*webview };

    webview.paint();
}

/// Loads the given URL into the `WebView`.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `url` is a NUL terminated UTF-8 string.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `url` remains with the caller after the call.
///
/// Returns 0 on success, or -1 if the URL could not be parsed.
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
pub unsafe extern "C" fn servo_webview_load(webview: *mut WebView, url: *const c_char) -> i32 {
    assert!(!webview.is_null(), "webview pointer must not be null");
    assert!(!url.is_null(), "url pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `webview` documented above.
    let webview = unsafe { &*webview };

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `url` documented above.
    let url_str = unsafe { CStr::from_ptr(url) }.to_str().unwrap();

    match url::Url::parse(url_str) {
        Ok(parsed) => {
            webview.load(parsed);
            0
        },
        Err(_) => -1,
    }
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
) {
    assert!(!webview.is_null(), "webview pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `webview` documented above.
    let webview = unsafe { &*webview };

    let Some(callback) = callback else {
        return;
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
pub unsafe extern "C" fn servo_webview_free(webview: *mut WebView) {
    assert!(!webview.is_null(), "webview pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `webview` documented above.
    unsafe {
        let _ = Box::from_raw(webview);
    }
}
