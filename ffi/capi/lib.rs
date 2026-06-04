/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod options;
mod preferences;
mod rendering_context;
mod servo;
mod webview;
mod webview_delegate;

extern crate servo as servo_api;

use options::ServoOptions;
use preferences::ServoPreferences;

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
/// [`servo_builder_build`].
#[unsafe(no_mangle)]
pub extern "C" fn servo_builder_create() -> *mut ServoBuilder {
    Box::into_raw(Box::new(ServoBuilder::default()))
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_set_preferences(
    builder: *mut ServoBuilder,
    preferences: *mut ServoPreferences,
) {
    assert!(!builder.is_null(), "builder pointer must not be null");
    assert!(!preferences.is_null(), "preferences pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `builder` and `preferences` documented above.
    unsafe {
        (*builder).preferences = Some(Box::from_raw(preferences));
    }
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_set_options(
    builder: *mut ServoBuilder,
    options: *mut ServoOptions,
) {
    assert!(!builder.is_null(), "builder pointer must not be null");
    assert!(!options.is_null(), "options pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `builder` and `options` documented above.
    unsafe {
        (*builder).options = Some(Box::from_raw(options));
    }
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
///   uphelp by `event_loop_waker`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_set_event_loop_waker(
    builder: *mut ServoBuilder,
    event_loop_waker: ServoEventLoopWaker,
) {
    assert!(!builder.is_null(), "builder pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `builder` documented above.
    unsafe {
        (*builder).event_loop_waker = event_loop_waker;
    }
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
///
/// # Safety
///
/// The caller must ensure that `builder` was previously returned by
/// `servo_builder_create` and has not yet been freed nor passed to
/// another API that takes ownership of it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_build(builder: *mut ServoBuilder) -> *mut servo_api::Servo {
    assert!(!builder.is_null(), "builder pointer must not be null");
    let mut rust_builder = servo_api::ServoBuilder::default();

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `builder` documented above.
    let builder = unsafe { &mut *builder };

    if let Some(options) = builder.options.take() {
        rust_builder = rust_builder.opts(*options);
    }

    if let Some(preferences) = builder.preferences.take() {
        rust_builder = rust_builder.preferences(*preferences);
    }

    rust_builder = rust_builder.event_loop_waker(Box::new(builder.event_loop_waker));

    let servo = rust_builder.build();
    Box::into_raw(Box::new(servo))
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
pub unsafe extern "C" fn servo_builder_free(builder: *mut ServoBuilder) {
    assert!(!builder.is_null(), "builder pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `builder` documented above.
    unsafe {
        let _ = Box::from_raw(builder);
    }
}
