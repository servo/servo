/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::str::FromStr;

use log::warn;
use servo_api::{
    Code, DevicePoint, InputEvent, Key, KeyState, KeyboardEvent, Location, Modifiers, MouseButton,
    MouseButtonAction, MouseButtonEvent, MouseMoveEvent, WebView, WheelDelta, WheelEvent,
    WheelMode,
};

/// The action of a mouse button event.
#[repr(i32)]
pub enum ServoMouseButtonAction {
    /// The button was pressed.
    Down,
    /// The button was released.
    Up,
}

/// The state of a key in a keyboard event.
#[repr(i32)]
pub enum ServoKeyState {
    /// The key was pressed.
    Down,
    /// The key was released.
    Up,
}

/// The unit of the deltas of a wheel event.
#[repr(i32)]
pub enum ServoWheelMode {
    /// The deltas are in pixels.
    Pixel,
    /// The deltas are in lines.
    Line,
    /// The deltas are in pages.
    Page,
}

/// The modifier keys that can be active during a keyboard event, as bit
/// flags that can be combined.
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum ServoKeyModifier {
    Alt = 1 << 0,
    AltGraph = 1 << 1,
    CapsLock = 1 << 2,
    Control = 1 << 3,
    Fn = 1 << 4,
    FnLock = 1 << 5,
    Meta = 1 << 6,
    NumLock = 1 << 7,
    ScrollLock = 1 << 8,
    Shift = 1 << 9,
    Symbol = 1 << 10,
    SymbolLock = 1 << 11,
}

const MODIFIERS: [(ServoKeyModifier, Modifiers); 12] = [
    (ServoKeyModifier::Alt, Modifiers::ALT),
    (ServoKeyModifier::AltGraph, Modifiers::ALT_GRAPH),
    (ServoKeyModifier::CapsLock, Modifiers::CAPS_LOCK),
    (ServoKeyModifier::Control, Modifiers::CONTROL),
    (ServoKeyModifier::Fn, Modifiers::FN),
    (ServoKeyModifier::FnLock, Modifiers::FN_LOCK),
    (ServoKeyModifier::Meta, Modifiers::META),
    (ServoKeyModifier::NumLock, Modifiers::NUM_LOCK),
    (ServoKeyModifier::ScrollLock, Modifiers::SCROLL_LOCK),
    (ServoKeyModifier::Shift, Modifiers::SHIFT),
    (ServoKeyModifier::Symbol, Modifiers::SYMBOL),
    (ServoKeyModifier::SymbolLock, Modifiers::SYMBOL_LOCK),
];

/// Notifies the `WebView` that the mouse moved to (`x`, `y`), in device
/// pixels relative to the top left corner of the `WebView`.
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
pub unsafe extern "C" fn servo_webview_notify_mouse_move(webview: *mut WebView, x: f32, y: f32) {
    assert!(!webview.is_null(), "webview pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `webview` documented above.
    let webview = unsafe { &*webview };

    webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(
        DevicePoint::new(x, y).into(),
    )));
}

/// Notifies the `WebView` that a mouse button was pressed or released at
/// (`x`, `y`), in device pixels relative to the top left corner of the
/// `WebView`.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `action` must be one of the `SERVO_MOUSE_BUTTON_ACTION_*` constants.
///
/// `button` is the number of the button as in the DOM's `MouseEvent.button`:
/// 0 for the primary button, 1 for the auxiliary button, 2 for the secondary
/// button, 3 for the back button and 4 for the forward button.
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
pub unsafe extern "C" fn servo_webview_notify_mouse_button(
    webview: *mut WebView,
    action: c_int,
    button: i16,
    x: f32,
    y: f32,
) {
    assert!(!webview.is_null(), "webview pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `webview` documented above.
    let webview = unsafe { &*webview };

    let action = match action {
        action if action == ServoMouseButtonAction::Down as c_int => MouseButtonAction::Down,
        action if action == ServoMouseButtonAction::Up as c_int => MouseButtonAction::Up,
        _ => {
            warn!("invalid mouse button action value: {action}");
            return;
        },
    };

    webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
        action,
        MouseButton::from(button),
        DevicePoint::new(x, y).into(),
    )));
}

/// Notifies the `WebView` that the mouse wheel was turned at (`x`, `y`), in
/// device pixels relative to the top left corner of the `WebView`.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `delta_x`, `delta_y` and `delta_z` are the deltas as in the DOM's
/// `WheelEvent`, in the unit given by `mode`, which must be one of the
/// `SERVO_WHEEL_MODE_*` constants.
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
pub unsafe extern "C" fn servo_webview_notify_wheel(
    webview: *mut WebView,
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    mode: c_int,
    x: f32,
    y: f32,
) {
    assert!(!webview.is_null(), "webview pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `webview` documented above.
    let webview = unsafe { &*webview };

    let mode = match mode {
        mode if mode == ServoWheelMode::Pixel as c_int => WheelMode::DeltaPixel,
        mode if mode == ServoWheelMode::Line as c_int => WheelMode::DeltaLine,
        mode if mode == ServoWheelMode::Page as c_int => WheelMode::DeltaPage,
        _ => {
            warn!("invalid wheel mode value: {mode}");
            return;
        },
    };
    let delta = WheelDelta {
        x: delta_x,
        y: delta_y,
        z: delta_z,
        mode,
    };

    webview.notify_input_event(InputEvent::Wheel(WheelEvent::new(
        delta,
        DevicePoint::new(x, y).into(),
    )));
}

/// Notifies the `WebView` that a key was pressed or released.
///
/// `webview` is a handle to a `WebView` object.
/// The ownership of `webview` remains with the caller after the call.
///
/// `state` must be one of the `SERVO_KEY_STATE_*` constants.
///
/// `key` is a NUL terminated UTF-8 string with the value of the key as in
/// the DOM's `KeyboardEvent.key`, such as `a`, `A` or `Enter`.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `key` remains with the caller after the call.
///
/// `code` is a NUL terminated UTF-8 string with the physical key as in the
/// DOM's `KeyboardEvent.code`, such as `KeyA` or `Enter`, or `NULL` if it
/// is not known.
/// The function panics if it is not a valid UTF-8 string.
/// The ownership of `code` remains with the caller after the call.
///
/// `modifiers` is a combination of the `SERVO_KEY_MODIFIER_*` constants of
/// the modifier keys that are active.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `webview` is a non-null pointer to a `WebView` previously returned
///   by `servo_webview_builder_build` and not yet passed to
///   `servo_webview_free`. No other code may read or write `*webview`
///   for the duration of this call.
/// - `key` is a non-null pointer to a C string that remains unmodified
///   for the duration of the call.
/// - `code` is either null or a pointer to a C string that remains
///   unmodified for the duration of the call.
/// - The call is made from the same thread that originally created the
///   `WebView` via `servo_webview_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_notify_keyboard(
    webview: *mut WebView,
    state: c_int,
    key: *const c_char,
    code: *const c_char,
    modifiers: u32,
) {
    assert!(!webview.is_null(), "webview pointer must not be null");
    assert!(!key.is_null(), "key pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `webview` documented above.
    let webview = unsafe { &*webview };

    let state = match state {
        state if state == ServoKeyState::Down as c_int => KeyState::Down,
        state if state == ServoKeyState::Up as c_int => KeyState::Up,
        _ => {
            warn!("invalid key state value: {state}");
            return;
        },
    };

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `key` documented above.
    let key = unsafe { CStr::from_ptr(key) }.to_str().unwrap();
    let Ok(key) = Key::from_str(key) else {
        warn!("invalid key value: {key}");
        return;
    };

    let code = if code.is_null() {
        Code::Unidentified
    } else {
        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `code` documented above.
        let code = unsafe { CStr::from_ptr(code) }.to_str().unwrap();
        Code::from_str(code).unwrap_or(Code::Unidentified)
    };

    let modifiers = MODIFIERS
        .into_iter()
        .filter(|(modifier, _)| modifiers & (*modifier as u32) != 0)
        .fold(Modifiers::empty(), |all, (_, modifier)| all | modifier);

    webview.notify_input_event(InputEvent::Keyboard(KeyboardEvent::new_without_event(
        state,
        key,
        code,
        Location::Standard,
        modifiers,
        false,
        false,
    )));
}
