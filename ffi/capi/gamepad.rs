/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Gamepad haptics for a C host.
//!
//! `servo_api::GamepadDelegate` lets an embedder play and stop rumble effects
//! on a gamepad the page asked for. This module mirrors that Rust delegate as a
//! C struct, and describes the pending effect through
//! [`ServoGamepadHapticEffectInfo`]. The host reports the outcome back with
//! [`servo_webview_gamepad_haptic_effect_request_succeeded`] or
//! [`servo_webview_gamepad_haptic_effect_request_failed`].
//!
//! The delegate is only consulted when the engine was built with the `gamepad`
//! feature. Without a gamepad connected to the host no request is ever made, so
//! the round trip cannot be exercised in CI; see `C_API_GUIDE.md`.

use std::ffi::c_void;
use std::os::raw::c_int;

use embedder_traits::GamepadHapticEffectType;
use servo_api::{
    GamepadDelegate, GamepadHapticEffectRequest, GamepadHapticEffectRequestType,
};

use crate::ServoError;

/// The request was to stop whatever effect is playing.
pub const SERVO_GAMEPAD_HAPTIC_EFFECT_STOP: c_int = 0;

/// The request was a `dual-rumble` effect. The `duration`, `start_delay`,
/// `strong_magnitude` and `weak_magnitude` fields of
/// `ServoGamepadHapticEffectInfo` carry its parameters.
pub const SERVO_GAMEPAD_HAPTIC_EFFECT_DUAL_RUMBLE: c_int = 1;

/// Callback type for [`ServoGamepadDelegate::handle_haptic_effect_request`].
///
/// `request` is an opaque handle describing the effect. The embedder must
/// resolve it by calling [`servo_webview_gamepad_haptic_effect_request_succeeded`]
/// or [`servo_webview_gamepad_haptic_effect_request_failed`]. The handle is
/// consumed by whichever of those is called, and must not be used afterwards.
/// Until one of them is called the page that asked for the effect waits, so the
/// embedder should resolve the request from within the event loop.
///
/// Unlike the other delegate callbacks this one receives no `WebView`: the
/// engine's own `GamepadDelegate` does not supply one, because a rumble request
/// is addressed to a gamepad rather than to a page. The effect is already
/// associated with the `WebView` the delegate was installed on.
pub type ServoGamepadHapticEffectCallback = Option<
    unsafe extern "C" fn(request: *mut ServoGamepadHapticEffectRequest, user_data: *mut c_void),
>;

/// A delegate that gives a C host control over gamepad haptics.
///
/// Set it on a `WebView` with `servo_webview_builder_set_gamepad_delegate`
/// before calling `servo_webview_builder_build`. When the callback is `NULL` the
/// request is failed immediately, so the page does not wait for an effect that
/// no one is going to play.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ServoGamepadDelegate {
    /// Size of this struct in bytes, as seen by the caller.
    ///
    /// The caller must set this to `sizeof(ServoGamepadDelegate)`. Servo
    /// rejects delegates whose `struct_size` is smaller than the size it was
    /// compiled with, returning `ServoError::InvalidArgument`. See
    /// `ABI_STABILITY.md` §3.3.
    pub struct_size: u32,

    /// Reserved for future use; must be `0`.
    pub _reserved: u32,

    /// An opaque pointer passed to the delegate callback. May be `NULL`.
    pub user_data: *mut c_void,

    /// Called when the page asks for a haptic effect. May be `NULL`.
    pub handle_haptic_effect_request: ServoGamepadHapticEffectCallback,
}

/// Describes the effect a [`ServoGamepadHapticEffectRequest`] is asking for.
///
/// Read it with [`servo_webview_gamepad_haptic_effect_request_info`]. Like the
/// other output structs it carries a `struct_size`, which the caller sets so
/// that a future, larger version of this struct cannot be written past the end
/// of a buffer compiled against an older header.
#[repr(C)]
pub struct ServoGamepadHapticEffectInfo {
    /// Size of this struct in bytes, as seen by the caller. Set this to
    /// `sizeof(ServoGamepadHapticEffectInfo)`.
    pub struct_size: u32,

    /// Reserved for future use; must be `0`.
    pub _reserved: u32,

    /// Index of the gamepad the effect is destined for.
    pub gamepad_index: usize,

    /// One of `SERVO_GAMEPAD_HAPTIC_EFFECT_STOP` or
    /// `SERVO_GAMEPAD_HAPTIC_EFFECT_DUAL_RUMBLE`.
    pub effect_type: c_int,

    /// Effect duration in milliseconds. Only meaningful for dual-rumble.
    pub duration: f64,

    /// Delay before the effect starts, in milliseconds. Only meaningful for
    /// dual-rumble.
    pub start_delay: f64,

    /// Intensity of the strong rumble motor, `0.0` to `1.0`. Only meaningful
    /// for dual-rumble.
    pub strong_magnitude: f64,

    /// Intensity of the weak rumble motor, `0.0` to `1.0`. Only meaningful for
    /// dual-rumble.
    pub weak_magnitude: f64,
}

/// An opaque handle to a pending call to
/// `ServoGamepadDelegate::handle_haptic_effect_request`.
///
/// The embedder receives one in the callback and must resolve it with
/// [`servo_webview_gamepad_haptic_effect_request_succeeded`] or
/// [`servo_webview_gamepad_haptic_effect_request_failed`]. Passing it to either
/// of those transfers ownership; the handle must not be used afterwards. If it
/// is dropped unresolved - which happens if it is leaked back to Rust, or never
/// resolved - the effect is reported as failed.
// cbindgen:opaque
pub struct ServoGamepadHapticEffectRequest {
    request: Option<GamepadHapticEffectRequest>,
}

/// Fills `out_info` with a description of the effect `request` asks for.
///
/// `request` is a handle received from
/// `ServoGamepadDelegate::handle_haptic_effect_request`. This does not resolve
/// the request and does not take ownership of it: the embedder still has to
/// call [`servo_webview_gamepad_haptic_effect_request_succeeded`] or
/// [`servo_webview_gamepad_haptic_effect_request_failed`].
///
/// `out_info` must have its `struct_size` set to `sizeof(ServoGamepadHapticEffectInfo)`.
///
/// Returns `ServoError::Ok` on success, `ServoError::NullArgument` if either
/// argument is `NULL`, or `ServoError::InvalidArgument` if `out_info`'s
/// `struct_size` does not match the size this library was compiled with.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `request` is a non-null pointer previously received from a
///   `ServoGamepadDelegate::handle_haptic_effect_request` callback, has not
///   been resolved, and outlives the call.
/// - `out_info` is a non-null pointer to a `ServoGamepadHapticEffectInfo` the
///   caller owns, whose `struct_size` has been set.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_gamepad_haptic_effect_request_info(
    request: *const ServoGamepadHapticEffectRequest,
    out_info: *mut ServoGamepadHapticEffectInfo,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if request.is_null() || out_info.is_null() {
            return ServoError::NullArgument;
        }
        if !crate::is_compatible_struct_size(
            unsafe { &*out_info }.struct_size,
            std::mem::size_of::<ServoGamepadHapticEffectInfo>(),
        ) {
            return ServoError::InvalidArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `request` documented above; it is a valid, unresolved handle.
        let request = unsafe { &*request };
        let Some(pending) = request.request.as_ref() else {
            return ServoError::InvalidState;
        };

        let (effect_type, duration, start_delay, strong_magnitude, weak_magnitude) =
            match pending.request_type() {
                GamepadHapticEffectRequestType::Stop => {
                    (SERVO_GAMEPAD_HAPTIC_EFFECT_STOP, 0.0, 0.0, 0.0, 0.0)
                },
                GamepadHapticEffectRequestType::Play(GamepadHapticEffectType::DualRumble(params)) => (
                    SERVO_GAMEPAD_HAPTIC_EFFECT_DUAL_RUMBLE,
                    params.duration,
                    params.start_delay,
                    params.strong_magnitude,
                    params.weak_magnitude,
                ),
            };

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `out_info` documented above, and its `struct_size` was just checked
        // against the size of this struct.
        let out_info = unsafe { &mut *out_info };
        out_info.gamepad_index = pending.gamepad_index();
        out_info.effect_type = effect_type;
        out_info.duration = duration;
        out_info.start_delay = start_delay;
        out_info.strong_magnitude = strong_magnitude;
        out_info.weak_magnitude = weak_magnitude;
        ServoError::Ok
    })
}

/// Reports to the page that the requested haptic effect was carried out.
///
/// `request` is a handle received from
/// `ServoGamepadDelegate::handle_haptic_effect_request`. Ownership of `request`
/// is transferred to this function, which frees it; the caller must not use it
/// again.
///
/// Returns `ServoError::Ok` on success, `ServoError::NullArgument` if `request`
/// is `NULL`, or `ServoError::InvalidState` if the request was already resolved.
///
/// # Safety
///
/// The caller must ensure that `request` is a non-null pointer previously
/// received from a `ServoGamepadDelegate::handle_haptic_effect_request`
/// callback, and has not already been passed to this function or to
/// [`servo_webview_gamepad_haptic_effect_request_failed`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_gamepad_haptic_effect_request_succeeded(
    request: *mut ServoGamepadHapticEffectRequest,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if request.is_null() {
            return ServoError::NullArgument;
        }
        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `request` documented above. We take ownership here, so the box is
        // freed when this call returns.
        let mut request = unsafe { Box::from_raw(request) };
        let Some(request) = request.request.take() else {
            return ServoError::InvalidState;
        };
        request.succeeded();
        ServoError::Ok
    })
}

/// Reports to the page that the requested haptic effect was not carried out.
///
/// `request` is a handle received from
/// `ServoGamepadDelegate::handle_haptic_effect_request`. Ownership of `request`
/// is transferred to this function, which frees it; the caller must not use it
/// again. The page sees the effect fail, exactly as if the gamepad had refused
/// it.
///
/// Returns `ServoError::Ok` on success, `ServoError::NullArgument` if `request`
/// is `NULL`, or `ServoError::InvalidState` if the request was already resolved.
///
/// # Safety
///
/// The caller must ensure that `request` is a non-null pointer previously
/// received from a `ServoGamepadDelegate::handle_haptic_effect_request`
/// callback, and has not already been passed to this function or to
/// [`servo_webview_gamepad_haptic_effect_request_succeeded`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_gamepad_haptic_effect_request_failed(
    request: *mut ServoGamepadHapticEffectRequest,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if request.is_null() {
            return ServoError::NullArgument;
        }
        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `request` documented above. We take ownership here, so the box is
        // freed when this call returns.
        let mut request = unsafe { Box::from_raw(request) };
        let Some(request) = request.request.take() else {
            return ServoError::InvalidState;
        };
        request.failed();
        ServoError::Ok
    })
}

impl GamepadDelegate for ServoGamepadDelegate {
    fn handle_haptic_effect_request(&self, request: GamepadHapticEffectRequest) {
        let Some(callback) = self.handle_haptic_effect_request else {
            // No handler: drop the request. Its `Drop` impl fails it, so the
            // page is told the effect did not play instead of waiting forever.
            return;
        };
        let request = Box::into_raw(Box::new(ServoGamepadHapticEffectRequest {
            request: Some(request),
        }));
        // SAFETY: The embedder is assumed to uphold the safety requirements of
        // the `ServoGamepadDelegate` struct, including resolving `request` with
        // one of the request functions above.
        unsafe { callback(request, self.user_data) };
    }
}
