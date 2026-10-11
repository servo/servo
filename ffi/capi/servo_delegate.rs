/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{c_char, c_void};
use std::rc::Rc;

use servo_api::{AllowOrDenyRequest, Servo, ServoDelegate as RustServoDelegate};

/// A delegate that receives notifications about Servo-level events.
///
/// The function pointers can be set to `NULL` for callbacks that are not
/// needed. All callbacks receive the `user_data` pointer that was set in
/// this struct. The callbacks are invoked on the embedder thread that
/// calls `servo_spin_event_loop`.
///
/// Refer to the documentation of the corresponding
/// [`servo::ServoDelegate`] trait in the Rust API for more information.
///
/// [`servo::ServoDelegate`]: https://doc.servo.org/servo/trait.ServoDelegate.html
///
/// # Ownership of callback arguments
///
/// The `servo` argument passed to each callback is a temporary handle.
/// Its lifetime is limited to the duration of the callback. The
/// embedder must not retain or use handle after the callback returns.
/// The callback must not pass it to `servo_free` or any other
/// function that takes ownership of a `Servo`.
///
/// The `user_data` pointer is owned by the embedder.
/// The validity and lifetime of `user_data` is the embedder's responsibility.
///
/// # Safety
///
/// The embedder must ensure that for all function-pointer fields of
/// this struct:
///
/// - A non-null function pointer is a valid C ABI callback function
///   matching the exact signature shown.
/// - The function pointers and `user_data` remain valid for as long as
///   this delegate is associated with any `Servo`.
/// - The callback does not unwind across the FFI boundary.
/// - The `servo` argument is not retained or used after the callback
///   returns and is not passed to `servo_free` or any other
///   function that takes ownership of the `Servo`.
#[repr(C)]
pub struct ServoDelegate {
    /// An opaque pointer passed to all delegate callbacks. May be `NULL`.
    pub user_data: *mut c_void,

    /// Called when the DevTools server has started.
    ///
    /// `port` is the port number the DevTools server is listening on.
    /// `token` is a NUL terminated UTF-8 string that is only valid for the
    /// duration of the callback. The embedder must copy it if it needs to
    /// retain the value. The token can be used to bypass the permission prompt
    /// from the DevTools client.
    pub notify_devtools_server_started: Option<
        unsafe extern "C" fn(servo: *mut Servo, port: u16, token: *const c_char, user_data: *mut c_void),
    >,

    /// Called when a DevTools client requests a connection.
    ///
    /// Return `true` to allow the connection, `false` to deny it.
    /// The embedder should typically show a permissions prompt to the user
    /// before allowing the connection.
    pub request_devtools_connection: Option<
        unsafe extern "C" fn(servo: *mut Servo, user_data: *mut c_void) -> bool,
    >,
}

struct ServoDelegateWrapper {
    delegate: ServoDelegate,
}

impl RustServoDelegate for ServoDelegateWrapper {
    fn notify_devtools_server_started(&self, port: u16, token: String) {
        let Some(callback) = self.delegate.notify_devtools_server_started else {
            return;
        };

        // Guard against interior NUL bytes rather than panicking across the
        // FFI boundary.
        let Ok(token) = std::ffi::CString::new(token) else {
            return;
        };

        // SAFETY: The embedder is assumed to uphold the safety requirements of the
        // `ServoDelegate` struct.
        //
        // The `servo` raw pointer is derived from a valid `servo` handle.
        // `token` outlives the call.
        unsafe { callback(std::ptr::null_mut(), port, token.as_ptr(), self.delegate.user_data) };
    }

    fn request_devtools_connection(&self, request: AllowOrDenyRequest) {
        let Some(callback) = self.delegate.request_devtools_connection else {
            // No callback set, deny by default
            request.deny();
            return;
        };

        // SAFETY: The embedder is assumed to uphold the safety requirements of the
        // `ServoDelegate` struct.
        //
        // The `servo` raw pointer is derived from a valid `servo` handle.
        let allow = unsafe { callback(std::ptr::null_mut(), self.delegate.user_data) };

        if allow {
            request.allow();
        } else {
            request.deny();
        }
    }
}

/// Creates a boxed `ServoDelegate` trait object from the C delegate struct.
pub fn make_servo_delegate(delegate: ServoDelegate) -> Rc<dyn RustServoDelegate> {
    Rc::new(ServoDelegateWrapper { delegate })
}