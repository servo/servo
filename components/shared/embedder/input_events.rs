/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use keyboard_types::{CompositionEvent, KeyboardEvent};
use log::error;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use webrender_api::units::DevicePoint;

use crate::WebDriverMessageId;

/// An input event that is sent from the embedder to Servo.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum InputEvent {
    EditingAction(EditingActionEvent),
    Gamepad(GamepadEvent),
    Ime(ImeEvent),
    Keyboard(KeyboardEventWithWebDriverId),
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

    pub fn webdriver_message_id(&self) -> Option<WebDriverMessageId> {
        match self {
            InputEvent::EditingAction(..) => None,
            InputEvent::Gamepad(..) => None,
            InputEvent::Ime(..) => None,
            InputEvent::Keyboard(event) => event.webdriver_id,
            InputEvent::MouseButton(event) => event.webdriver_id,
            InputEvent::MouseMove(event) => event.webdriver_id,
            InputEvent::Touch(..) => None,
            InputEvent::Wheel(event) => event.webdriver_id,
        }
    }

    pub fn with_webdriver_message_id(mut self, webdriver_id: Option<WebDriverMessageId>) -> Self {
        match self {
            InputEvent::EditingAction(..) => {},
            InputEvent::Gamepad(..) => {},
            InputEvent::Ime(..) => {},
            InputEvent::Keyboard(ref mut event) => {
                event.webdriver_id = webdriver_id;
            },
            InputEvent::MouseButton(ref mut event) => {
                event.webdriver_id = webdriver_id;
            },
            InputEvent::MouseMove(ref mut event) => {
                event.webdriver_id = webdriver_id;
            },
            InputEvent::Touch(..) => {},
            InputEvent::Wheel(ref mut event) => {
                event.webdriver_id = webdriver_id;
            },
        };

        self
    }
}

/// Wrap the KeyboardEvent from crate to pair it with webdriver_id,
/// which is used for webdriver action synchronization.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KeyboardEventWithWebDriverId {
    pub event: KeyboardEvent,
    webdriver_id: Option<WebDriverMessageId>,
}

impl KeyboardEventWithWebDriverId {
    pub fn new(keyboard_event: KeyboardEvent) -> Self {
        Self {
            event: keyboard_event,
            webdriver_id: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct MouseButtonEvent {
    pub action: MouseButtonAction,
    pub button: MouseButton,
    pub point: DevicePoint,
    webdriver_id: Option<WebDriverMessageId>,
}

impl MouseButtonEvent {
    pub fn new(action: MouseButtonAction, button: MouseButton, point: DevicePoint) -> Self {
        Self {
            action,
            button,
            point,
            webdriver_id: None,
        }
    }
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

impl<T: Into<u64>> From<T> for MouseButton {
    fn from(value: T) -> Self {
        let value = value.into();
        match value {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            3 => MouseButton::Back,
            4 => MouseButton::Forward,
            _ => MouseButton::Other(value as u16),
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
    webdriver_id: Option<WebDriverMessageId>,
}

impl MouseMoveEvent {
    pub fn new(point: DevicePoint) -> Self {
        Self {
            point,
            webdriver_id: None,
        }
    }
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

/// An opaque identifier for a touch point.
///
/// <http://w3c.github.io/touch-events/#widl-Touch-identifier>
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TouchId(pub i32);

/// An ID for a sequence of touch events between a `Down` and the `Up` or `Cancel` event.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TouchSequenceId(u32);

impl TouchSequenceId {
    pub const fn new() -> Self {
        Self(0)
    }

    /// Increments the ID for the next touch sequence.
    ///
    /// The increment is wrapping, since we can assume that the touch handler
    /// script for touch sequence N will have finished processing by the time
    /// we have wrapped around.
    pub fn next(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct TouchEvent {
    pub event_type: TouchEventType,
    pub id: TouchId,
    pub point: DevicePoint,
    /// cancelable default value is true, once the first move has been processed by script disable it.
    cancelable: bool,
    /// The sequence_id will be set by servo's touch handler.
    sequence_id: Option<TouchSequenceId>,
}

impl TouchEvent {
    pub fn new(event_type: TouchEventType, id: TouchId, point: DevicePoint) -> Self {
        TouchEvent {
            event_type,
            id,
            point,
            sequence_id: None,
            cancelable: true,
        }
    }
    /// Embedders should ignore this.
    #[doc(hidden)]
    pub fn init_sequence_id(&mut self, sequence_id: TouchSequenceId) {
        if self.sequence_id.is_none() {
            self.sequence_id = Some(sequence_id);
        } else {
            // We could allow embedders to set the sequence ID.
            error!("Sequence ID already set.");
        }
    }

    #[doc(hidden)]
    pub fn expect_sequence_id(&self) -> TouchSequenceId {
        self.sequence_id.expect("Sequence ID not initialized")
    }

    #[doc(hidden)]
    pub fn disable_cancelable(&mut self) {
        self.cancelable = false;
    }

    #[doc(hidden)]
    pub fn is_cancelable(&self) -> bool {
        self.cancelable
    }
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
    webdriver_id: Option<WebDriverMessageId>,
}

impl WheelEvent {
    pub fn new(delta: WheelDelta, point: DevicePoint) -> Self {
        WheelEvent {
            delta,
            point,
            webdriver_id: None,
        }
    }
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
