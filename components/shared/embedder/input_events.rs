/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::Vector2D;
use keyboard_types::{CompositionEvent, KeyboardEvent};
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use webrender_api::units::{DeviceIntPoint, DevicePixel, DevicePoint};

/// An input event that is sent from the embedder to Servo.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum InputEvent {
    EditingAction(EditingActionEvent),
    Gamepad(GamepadEvent),
    Ime(ImeEvent),
    Keyboard(KeyboardEvent),
    MouseButton(MouseButtonEvent),
    MouseMove(MouseMoveEvent),
    Touch(TouchEvent),
    Wheel(WheelEvent),
}

/// An editing action that should be performed on a `WebView`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EditingActionEvent {
    Copy,
    Cut,
    Paste,
}

impl InputEvent {
    pub fn point(&self) -> Option<DevicePoint> {
        match self {
            InputEvent::EditingAction(..) => None,
            InputEvent::Gamepad(..) => None,
            InputEvent::Ime(..) => None,
            InputEvent::Keyboard(..) => None,
            InputEvent::MouseButton(event) => Some(event.point),
            InputEvent::MouseMove(event) => Some(event.point),
            InputEvent::Touch(event) => Some(event.point),
            InputEvent::Wheel(event) => Some(event.point),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct MouseButtonEvent {
    pub action: MouseButtonAction,
    pub button: MouseButton,
    pub point: DevicePoint,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
    Other(u16),
}

impl From<u16> for MouseButton {
    fn from(value: u16) -> Self {
        match value {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            3 => MouseButton::Back,
            4 => MouseButton::Forward,
            _ => MouseButton::Other(value),
        }
    }
}

impl From<MouseButton> for i16 {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => 0,
            MouseButton::Middle => 1,
            MouseButton::Right => 2,
            MouseButton::Back => 3,
            MouseButton::Forward => 4,
            MouseButton::Other(value) => value as i16,
        }
    }
}

/// The types of mouse events
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum MouseButtonAction {
    /// Mouse button clicked
    Click,
    /// Mouse button down
    Down,
    /// Mouse button up
    Up,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct MouseMoveEvent {
    pub point: DevicePoint,
}

/// The type of input represented by a multi-touch event.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TouchEventType {
    /// A new touch point came in contact with the screen.
    Down,
    /// An existing touch point changed location.
    Move,
    /// A touch point was removed from the screen.
    Up,
    /// The system stopped tracking a touch point.
    Cancel,
}

/// The action to take in response to a touch event
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum TouchAction {
    /// Simulate a mouse click.
    Click(DevicePoint),
    /// Fling by the provided offset
    Flinging(Vector2D<f32, DevicePixel>, DeviceIntPoint),
    /// Scroll by the provided offset.
    Scroll(Vector2D<f32, DevicePixel>, DevicePoint),
    /// Zoom by a magnification factor and scroll by the provided offset.
    Zoom(f32, Vector2D<f32, DevicePixel>),
    /// Don't do anything.
    NoAction,
}

/// An opaque identifier for a touch point.
///
/// <http://w3c.github.io/touch-events/#widl-Touch-identifier>
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TouchId(pub i32);

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct TouchEvent {
    pub event_type: TouchEventType,
    pub id: TouchId,
    pub point: DevicePoint,
    pub action: TouchAction,
}

/// Mode to measure WheelDelta floats in
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum WheelMode {
    /// Delta values are specified in pixels
    DeltaPixel = 0x00,
    /// Delta values are specified in lines
    DeltaLine = 0x01,
    /// Delta values are specified in pages
    DeltaPage = 0x02,
}

/// The Wheel event deltas in every direction
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct WheelDelta {
    /// Delta in the left/right direction
    pub x: f64,
    /// Delta in the up/down direction
    pub y: f64,
    /// Delta in the direction going into/out of the screen
    pub z: f64,
    /// Mode to measure the floats in
    pub mode: WheelMode,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct WheelEvent {
    pub delta: WheelDelta,
    pub point: DevicePoint,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ImeEvent {
    Composition(CompositionEvent),
    Dismissed,
}

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd, Serialize,
)]
/// Index of gamepad in list of system's connected gamepads
pub struct GamepadIndex(pub usize);

#[derive(Clone, Debug, Deserialize, Serialize)]
/// The minimum and maximum values that can be reported for axis or button input from this gamepad
pub struct GamepadInputBounds {
    /// Minimum and maximum axis values
    pub axis_bounds: (f64, f64),
    /// Minimum and maximum button values
    pub button_bounds: (f64, f64),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// The haptic effects supported by this gamepad
pub struct GamepadSupportedHapticEffects {
    /// Gamepad support for dual rumble effects
    pub supports_dual_rumble: bool,
    /// Gamepad support for trigger rumble effects
    pub supports_trigger_rumble: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// The type of Gamepad event
pub enum GamepadEvent {
    /// A new gamepad has been connected
    /// <https://www.w3.org/TR/gamepad/#event-gamepadconnected>
    Connected(
        GamepadIndex,
        String,
        GamepadInputBounds,
        GamepadSupportedHapticEffects,
    ),
    /// An existing gamepad has been disconnected
    /// <https://www.w3.org/TR/gamepad/#event-gamepaddisconnected>
    Disconnected(GamepadIndex),
    /// An existing gamepad has been updated
    /// <https://www.w3.org/TR/gamepad/#receiving-inputs>
    Updated(GamepadIndex, GamepadUpdateType),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// The type of Gamepad input being updated
pub enum GamepadUpdateType {
    /// Axis index and input value
    /// <https://www.w3.org/TR/gamepad/#dfn-represents-a-standard-gamepad-axis>
    Axis(usize, f64),
    /// Button index and input value
    /// <https://www.w3.org/TR/gamepad/#dfn-represents-a-standard-gamepad-button>
    Button(usize, f64),
}
