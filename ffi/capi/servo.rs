/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servo_api::Servo;

/// Destroys the `Servo` instance returned by `servo_builder_build` and
/// frees its memory.
///
/// `servo` is a handle to a `Servo` object.
/// The ownership of `servo` is transferred to the function. The caller
/// must not use or free `servo` again.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `servo` was previously returned by `servo_builder_build` and has
///   not yet been freed nor passed to another API that takes ownership
///   of it.
/// - The call is made from the same thread that originally created the
///   `Servo` instance via `servo_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_free(servo: *mut Servo) {
    assert!(!servo.is_null(), "servo pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `servo` documented above.
    unsafe {
        let _: Box<Servo> = Box::from_raw(servo);
    }
}

/// Spin the Servo event loop once. The embedder should call this
/// periodically to process incoming messages and perform rendering
/// updates.
///
/// `servo` is a handle to a `Servo` object.
/// The ownership of `servo` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `servo` is a non-null pointer to a `Servo` instance previously
///   returned by `servo_builder_build` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - The call is made from the same thread that originally created the
///   `Servo` instance via `servo_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_spin_event_loop(servo: *mut Servo) {
    assert!(!servo.is_null(), "servo pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `servo` documented above.
    let servo = unsafe { &*servo };

    servo.spin_event_loop();
}

/// Initialize logging for the Servo instance.
///
/// `servo` is a handle to a `Servo` object.
/// The ownership of `servo` remains with the caller after the call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `servo` is a non-null pointer to a `Servo` instance previously
///   returned by `servo_builder_build` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - The call is made from the same thread that originally created the
///   `Servo` instance via `servo_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_setup_logging(servo: *mut Servo) {
    assert!(!servo.is_null(), "servo pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `servo` documented above.
    let servo = unsafe { &*servo };

    servo.setup_logging();
}
