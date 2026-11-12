/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;

/// An opaque struct that lets the embedder decide when Servo starts preparing
/// the next frame, for instance on display vsync, instead of on Servo's own
/// timer.
///
/// See the documentation of the corresponding [servo::RefreshDriver]
/// trait in the Rust API for more details.
/// [servo::RefreshDriver]: https://doc.servo.org/servo/trait.RefreshDriver.html
///
/// Handles to this object are created using `servo_refresh_driver_create`.
///
/// # Thread safety
/// The handle must be used only from the thread that created it.
// cbindgen:opaque
pub struct ServoRefreshDriver {
    pub(crate) inner: Rc<EmbedderRefreshDriver>,
}

pub(crate) struct EmbedderRefreshDriver {
    user_data: *mut c_void,
    frame_requested: Option<unsafe extern "C" fn(user_data: *mut c_void)>,
    start_frame_callbacks: RefCell<Vec<Box<dyn Fn() + Send + 'static>>>,
}

impl servo_api::RefreshDriver for EmbedderRefreshDriver {
    fn observe_next_frame(&self, start_frame_callback: Box<dyn Fn() + Send + 'static>) {
        let mut start_frame_callbacks = self.start_frame_callbacks.borrow_mut();
        start_frame_callbacks.push(start_frame_callback);
        if start_frame_callbacks.len() > 1 {
            return;
        }
        drop(start_frame_callbacks);

        if let Some(frame_requested) = self.frame_requested {
            // SAFETY: The embedder is assumed to uphold the safety requirements
            // documented on `servo_refresh_driver_create`.
            unsafe { frame_requested(self.user_data) };
        }
    }
}

/// Creates a new refresh driver.
///
/// `frame_requested` is called with `user_data` when Servo starts waiting for
/// a frame, so that the embedder can start it, for instance on the next vsync.
/// It may be `NULL` for an embedder that starts frames on a schedule of its
/// own. It is called on the thread that runs `servo_spin_event_loop`.
///
/// Returns a newly allocated `ServoRefreshDriver` handle. The ownership of the
/// returned handle is transferred to the caller, who must free it with
/// [`servo_refresh_driver_free`].
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `frame_requested` is either null or a valid C ABI function matching
///   the signature shown, which does not unwind across the FFI boundary.
/// - `frame_requested` and `user_data` remain valid for as long as a
///   `RenderingContext` created with the refresh driver is in use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_refresh_driver_create(
    user_data: *mut c_void,
    frame_requested: Option<unsafe extern "C" fn(user_data: *mut c_void)>,
) -> *mut ServoRefreshDriver {
    Box::into_raw(Box::new(ServoRefreshDriver {
        inner: Rc::new(EmbedderRefreshDriver {
            user_data,
            frame_requested,
            start_frame_callbacks: Default::default(),
        }),
    }))
}

/// Starts a frame of the `WebView`s whose rendering contexts were created with
/// `refresh_driver`. Servo then prepares and paints the frames that were
/// waiting to start as the embedder spins the event loop.
///
/// `refresh_driver` is a handle to a `ServoRefreshDriver` object.
/// The ownership of `refresh_driver` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `refresh_driver` was previously returned by `servo_refresh_driver_create`
///   and has not yet been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_refresh_driver_start_frame(refresh_driver: *mut ServoRefreshDriver) {
    assert!(
        !refresh_driver.is_null(),
        "refresh_driver pointer must not be null"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `refresh_driver` documented above.
    let refresh_driver = unsafe { &*refresh_driver };

    let start_frame_callbacks = refresh_driver.inner.start_frame_callbacks.take();
    for start_frame_callback in start_frame_callbacks {
        start_frame_callback();
    }
}

/// Destroys `refresh_driver` and frees its memory. Rendering contexts
/// created with it keep using it until they are freed.
///
/// `refresh_driver` is a handle to a `ServoRefreshDriver` object.
/// The ownership of `refresh_driver` is transferred to the function. The
/// caller must not use or free `refresh_driver` again.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `refresh_driver` was previously returned by `servo_refresh_driver_create`
///   and has not yet been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_refresh_driver_free(refresh_driver: *mut ServoRefreshDriver) {
    assert!(
        !refresh_driver.is_null(),
        "refresh_driver pointer must not be null"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `refresh_driver` documented above.
    unsafe {
        let _ = Box::from_raw(refresh_driver);
    }
}
