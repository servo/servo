/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::atomic::{AtomicUsize, Ordering};

use bitflags::bitflags;
use keyboard_types::{Code, CompositionEvent, Key, KeyState, Location, Modifiers};
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use webrender_api::ExternalScrollId;

use crate::WebViewPoint;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct InputEventId(usize);

static INPUT_EVENT_ID: AtomicUsize = AtomicUsize::new(0);

impl InputEventId {
    fn new() -> Self {
        Self(INPUT_EVENT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

bitflags! {
    #[derive(Clone, Copy, Default, Deserialize, PartialEq, Serialize)]
    pub struct InputEventResult: u8 {
        /// Whether or not this input event's default behavior was prevented via script.
        const DefaultPrevented = 1 << 0;
        /// Whether or not the WebView handled this event. Some events have default handlers in
        /// Servo, such as keyboard events that insert characters in `<input>` areas. When these
        /// handlers are triggered, this flag is included. This can be used to prevent triggering
        /// behavior (such as keybindings) when the WebView has already consumed the event for its
        /// own purpose.
        const Consumed = 1 << 1;
    }
}

/// An input event that is sent from the embedder to Servo.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum InputEvent {
    EditingAction(EditingActionEvent),
    #[cfg(feature = "gamepad")]
    Gamepad(GamepadEvent),
    Ime(ImeEvent),
    Keyboard(KeyboardEvent),
    MouseButton(MouseButtonEvent),
    MouseLeftViewport(MouseLeftViewportEvent),
    MouseMove(MouseMoveEvent),
    Scroll(ScrollEvent),
    Touch(TouchEvent),
    Wheel(WheelEvent),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InputEventAndId {
    pub event: InputEvent,
    pub id: InputEventId,
}

impl From<InputEvent> for InputEventAndId {
    fn from(event: InputEvent) -> Self {
        Self {
            event,
            id: InputEventId::new(),
        }
    }
}

/// An editing action that should be performed on a `WebView`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EditingActionEvent {
    Copy,
    Cut,
    Paste,
}

impl InputEvent {
    pub fn point(&self) -> Option<WebViewPoint> {
        match self {
            InputEvent::EditingAction(..) => None,
            #[cfg(feature = "gamepad")]
            InputEvent::Gamepad(..) => None,
            InputEvent::Ime(..) => None,
            InputEvent::Keyboard(..) => None,
            InputEvent::MouseButton(event) => Some(event.point),
            InputEvent::MouseMove(event) => Some(event.point),
            InputEvent::MouseLeftViewport(_) => None,
            InputEvent::Touch(event) => Some(event.point),
            InputEvent::Wheel(event) => Some(event.point),
            InputEvent::Scroll(..) => None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct KeyboardEvent {
    pub event: ::keyboard_types::KeyboardEvent,
}

impl KeyboardEvent {
    pub fn new(keyboard_event: ::keyboard_types::KeyboardEvent) -> Self {
        Self {
            event: keyboard_event,
        }
    }

    pub fn new_without_event(
        state: KeyState,
        key: Key,
        code: Code,
        location: Location,
        modifiers: Modifiers,
        repeat: bool,
        is_composing: bool,
    ) -> Self {
        Self::new(::keyboard_types::KeyboardEvent {
            state,
            key,
            code,
            location,
            modifiers,
            repeat,
            is_composing,
        })
    }

    pub fn from_state_and_key(state: KeyState, key: Key) -> Self {
        Self::new(::keyboard_types::KeyboardEvent {
            state,
            key,
            ..::keyboard_types::KeyboardEvent::default()
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct MouseButtonEvent {
    pub action: MouseButtonAction,
    pub button: MouseButton,
    pub point: WebViewPoint,
}

impl MouseButtonEvent {
    pub fn new(action: MouseButtonAction, button: MouseButton, point: WebViewPoint) -> Self {
        Self {
            action,
            button,
            point,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
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
    /// Mouse button down
    Down,
    /// Mouse button up
    Up,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct MouseMoveEvent {
    pub point: WebViewPoint,
    pub is_compatibility_event_for_touch: bool,
}

impl MouseMoveEvent {
    pub fn new(point: WebViewPoint) -> Self {
        Self {
            point,
            is_compatibility_event_for_touch: false,
        }
    }

    pub fn new_compatibility_for_touch(point: WebViewPoint) -> Self {
        Self {
            point,
            is_compatibility_event_for_touch: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct MouseLeftViewportEvent {
    pub focus_moving_to_another_iframe: bool,
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct TouchEvent {
    pub event_type: TouchEventType,
    pub id: TouchId,
    pub point: WebViewPoint,
    /// cancelable default value is true, once the first move has been processed by script disable it.
    cancelable: bool,
}

impl TouchEvent {
    pub fn new(event_type: TouchEventType, id: TouchId, point: WebViewPoint) -> Self {
        TouchEvent {
            event_type,
            id,
            point,
            cancelable: true,
        }
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

/// Phase of a wheel/trackpad scroll gesture.
///
/// This is used to determine the beginning and end of a scroll gesture,
/// which is useful for implementing momentum scrolling (fling) on platforms
/// that don't provide native momentum events.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum WheelPhase {
    /// The scroll gesture has started.
    Started,
    /// The scroll gesture is in progress.
    Moved,
    /// The scroll gesture has ended.
    Ended,
    /// The scroll gesture was cancelled.
    Cancelled,
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
    pub point: WebViewPoint,
    pub phase: WheelPhase,
}

impl WheelEvent {
    pub fn new(delta: WheelDelta, point: WebViewPoint, phase: WheelPhase) -> Self {
        WheelEvent {
            delta,
            point,
            phase,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ScrollEvent {
    pub external_id: ExternalScrollId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ImeEvent {
    Composition(CompositionEvent),
    Dismissed,
}

#[cfg(feature = "gamepad")]
#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd, Serialize,
)]
/// Index of gamepad in list of system's connected gamepads
pub struct GamepadIndex(pub usize);

#[cfg(feature = "gamepad")]
#[derive(Clone, Debug, Deserialize, Serialize)]
/// The minimum and maximum values that can be reported for axis or button input from this gamepad
pub struct GamepadInputBounds {
    /// Minimum and maximum axis values
    pub axis_bounds: (f64, f64),
    /// Minimum and maximum button values
    pub button_bounds: (f64, f64),
}

#[cfg(feature = "gamepad")]
#[derive(Clone, Debug, Deserialize, Serialize)]
/// The haptic effects supported by this gamepad
pub struct GamepadSupportedHapticEffects {
    /// Gamepad support for dual rumble effects
    pub supports_dual_rumble: bool,
    /// Gamepad support for trigger rumble effects
    pub supports_trigger_rumble: bool,
}

#[cfg(feature = "gamepad")]
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

#[cfg(feature = "gamepad")]
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
