/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};

use dom_struct::dom_struct;
use embedder_traits::{GamepadSupportedHapticEffects, GamepadUpdateType};
use js::rust::MutableHandleValue;

use super::gamepadbutton::GamepadButton;
use super::gamepadhapticactuator::GamepadHapticActuator;
use super::gamepadpose::GamepadPose;
use crate::dom::bindings::codegen::Bindings::GamepadBinding::{GamepadHand, GamepadMethods};
use crate::dom::bindings::frozenarray::CachedFrozenArray;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, DomSlice};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::gamepadevent::{GamepadEvent, GamepadEventType};
use crate::dom::window::Window;
use crate::script_runtime::{CanGc, JSContext};

// This value is for determining when to consider a gamepad as having a user gesture
// from an axis tilt. This matches the threshold in Chromium.
const AXIS_TILT_THRESHOLD: f64 = 0.5;
// This value is for determining when to consider a non-digital button "pressed".
// Like Gecko and Chromium it derives from the XInput trigger threshold.
const BUTTON_PRESS_THRESHOLD: f64 = 30.0 / 255.0;

#[dom_struct]
pub(crate) struct Gamepad {
    reflector_: Reflector,
    gamepad_id: u32,
    id: String,
    index: Cell<i32>,
    connected: Cell<bool>,
    timestamp: Cell<f64>,
    mapping_type: String,
    #[ignore_malloc_size_of = "mozjs"]
    frozen_buttons: CachedFrozenArray,
    buttons: Vec<Dom<GamepadButton>>,
    #[ignore_malloc_size_of = "mozjs"]
    frozen_axes: CachedFrozenArray,
    axes: RefCell<Vec<f64>>,
    pose: Option<Dom<GamepadPose>>,
    #[ignore_malloc_size_of = "Defined in rust-webvr"]
    hand: GamepadHand,
    axis_bounds: (f64, f64),
    button_bounds: (f64, f64),
    exposed: Cell<bool>,
    vibration_actuator: Dom<GamepadHapticActuator>,
}

impl Gamepad {
    #[allow(clippy::too_many_arguments)]
    fn new_inherited(
        gamepad_id: u32,
        id: String,
        index: i32,
        connected: bool,
        timestamp: f64,
        mapping_type: String,
        buttons: &[&GamepadButton],
        pose: Option<&GamepadPose>,
        hand: GamepadHand,
        axis_bounds: (f64, f64),
        button_bounds: (f64, f64),
        vibration_actuator: &GamepadHapticActuator,
    ) -> Gamepad {
        Self {
            reflector_: Reflector::new(),
            gamepad_id,
            id,
            index: Cell::new(index),
            connected: Cell::new(connected),
            timestamp: Cell::new(timestamp),
            mapping_type,
            frozen_buttons: CachedFrozenArray::new(),
            buttons: buttons
                .iter()
                .map(|button| Dom::from_ref(*button))
                .collect(),
            frozen_axes: CachedFrozenArray::new(),
            axes: RefCell::new(Vec::new()),
            pose: pose.map(Dom::from_ref),
            hand,
            axis_bounds,
            button_bounds,
            exposed: Cell::new(false),
            vibration_actuator: Dom::from_ref(vibration_actuator),
        }
    }

    /// When we construct a new gamepad, we initialize the number of buttons and
    /// axes corresponding to the "standard" gamepad mapping.
    /// The spec says UAs *may* do this for fingerprint mitigation, and it also
    /// happens to simplify implementation
    /// <https://www.w3.org/TR/gamepad/#fingerprinting-mitigation>
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        window: &Window,
        gamepad_id: u32,
        id: String,
        mapping_type: String,
        axis_bounds: (f64, f64),
        button_bounds: (f64, f64),
        supported_haptic_effects: GamepadSupportedHapticEffects,
        xr: bool,
        can_gc: CanGc,
    ) -> DomRoot<Gamepad> {
        let buttons = Gamepad::init_buttons(window, can_gc);
        rooted_vec!(let buttons <- buttons.iter().map(DomRoot::as_traced));
        let vibration_actuator =
            GamepadHapticActuator::new(window, gamepad_id, supported_haptic_effects, can_gc);
        let index = if xr { -1 } else { 0 };
        let gamepad = reflect_dom_object(
            Box::new(Gamepad::new_inherited(
                gamepad_id,
                id,
                index,
                true,
                0.,
                mapping_type,
                buttons.r(),
                None,
                GamepadHand::_empty,
                axis_bounds,
                button_bounds,
                &vibration_actuator,
            )),
            window,
            can_gc,
        );
        gamepad.init_axes();
        gamepad
    }
}

impl GamepadMethods<crate::DomTypeHolder> for Gamepad {
    /// <https://w3c.github.io/gamepad/#dom-gamepad-id>
    fn Id(&self) -> DOMString {
        DOMString::from(self.id.clone())
    }

    /// <https://w3c.github.io/gamepad/#dom-gamepad-index>
    fn Index(&self) -> i32 {
        self.index.get()
    }

    /// <https://w3c.github.io/gamepad/#dom-gamepad-connected>
    fn Connected(&self) -> bool {
        self.connected.get()
    }

    /// <https://w3c.github.io/gamepad/#dom-gamepad-timestamp>
    fn Timestamp(&self) -> Finite<f64> {
        Finite::wrap(self.timestamp.get())
    }

    /// <https://w3c.github.io/gamepad/#dom-gamepad-mapping>
    fn Mapping(&self) -> DOMString {
        DOMString::from(self.mapping_type.clone())
    }

    /// <https://w3c.github.io/gamepad/#dom-gamepad-axes>
    fn Axes(&self, cx: &mut js::context::JSContext, retval: MutableHandleValue) {
        self.frozen_axes.get_or_init(
            || self.axes.borrow().clone(),
            cx.into(),
            retval,
            CanGc::from_cx(cx),
        );
    }

    /// <https://w3c.github.io/gamepad/#dom-gamepad-buttons>
    fn Buttons(&self, cx: JSContext, retval: MutableHandleValue) {
        self.frozen_buttons.get_or_init(
            || {
                self.buttons
                    .iter()
                    .map(|b| DomRoot::from_ref(&**b))
                    .collect()
            },
            cx,
            retval,
            CanGc::deprecated_note(),
        );
    }

    /// <https://w3c.github.io/gamepad/#dom-gamepad-vibrationactuator>
    fn VibrationActuator(&self) -> DomRoot<GamepadHapticActuator> {
        DomRoot::from_ref(&*self.vibration_actuator)
    }

    /// <https://w3c.github.io/gamepad/extensions.html#gamepadhand-enum>
    fn Hand(&self) -> GamepadHand {
        self.hand
    }

    /// <https://w3c.github.io/gamepad/extensions.html#dom-gamepad-pose>
    fn GetPose(&self) -> Option<DomRoot<GamepadPose>> {
        self.pose.as_ref().map(|p| DomRoot::from_ref(&**p))
    }
}

#[expect(dead_code)]
impl Gamepad {
    pub(crate) fn gamepad_id(&self) -> u32 {
        self.gamepad_id
    }

    /// Initialize the standard buttons for a gamepad.
    /// <https://www.w3.org/TR/gamepad/#dfn-initializing-buttons>
    fn init_buttons(window: &Window, can_gc: CanGc) -> Vec<DomRoot<GamepadButton>> {
        vec![
            GamepadButton::new(window, false, false, can_gc), // Bottom button in right cluster
            GamepadButton::new(window, false, false, can_gc), // Right button in right cluster
            GamepadButton::new(window, false, false, can_gc), // Left button in right cluster
            GamepadButton::new(window, false, false, can_gc), // Top button in right cluster
            GamepadButton::new(window, false, false, can_gc), // Top left front button
            GamepadButton::new(window, false, false, can_gc), // Top right front button
            GamepadButton::new(window, false, false, can_gc), // Bottom left front button
            GamepadButton::new(window, false, false, can_gc), // Bottom right front button
            GamepadButton::new(window, false, false, can_gc), // Left button in center cluster
            GamepadButton::new(window, false, false, can_gc), // Right button in center cluster
            GamepadButton::new(window, false, false, can_gc), // Left stick pressed button
            GamepadButton::new(window, false, false, can_gc), // Right stick pressed button
            GamepadButton::new(window, false, false, can_gc), // Top button in left cluster
            GamepadButton::new(window, false, false, can_gc), // Bottom button in left cluster
            GamepadButton::new(window, false, false, can_gc), // Left button in left cluster
            GamepadButton::new(window, false, false, can_gc), // Right button in left cluster
            GamepadButton::new(window, false, false, can_gc), // Center button in center cluster
        ]
    }

    pub(crate) fn update_connected(&self, connected: bool, has_gesture: bool, can_gc: CanGc) {
        if self.connected.get() == connected {
            return;
        }
        self.connected.set(connected);

        let event_type = if connected {
            GamepadEventType::Connected
        } else {
            GamepadEventType::Disconnected
        };

        if has_gesture {
            self.notify_event(event_type, can_gc);
        }
    }

    pub(crate) fn index(&self) -> i32 {
        self.index.get()
    }

    pub(crate) fn update_index(&self, index: i32) {
        self.index.set(index);
    }

    pub(crate) fn update_timestamp(&self, timestamp: f64) {
        self.timestamp.set(timestamp);
    }

    pub(crate) fn notify_event(&self, event_type: GamepadEventType, can_gc: CanGc) {
        let event =
            GamepadEvent::new_with_type(self.global().as_window(), event_type, self, can_gc);
        event
            .upcast::<Event>()
            .fire(self.global().as_window().upcast::<EventTarget>(), can_gc);
    }

    /// Initialize the number of axes in the "standard" gamepad mapping.
    /// <https://www.w3.org/TR/gamepad/#dfn-initializing-axes>
    fn init_axes(&self) {
        *self.axes.borrow_mut() = vec![
            0., // Horizontal axis for left stick (negative left/positive right)
            0., // Vertical axis for left stick (negative up/positive down)
            0., // Horizontal axis for right stick (negative left/positive right)
            0., // Vertical axis for right stick (negative up/positive down)
        ];
    }

    /// <https://www.w3.org/TR/gamepad/#dfn-map-and-normalize-axes>
    pub(crate) fn map_and_normalize_axes(&self, axis_index: usize, value: f64) {
        // Let normalizedValue be 2 (logicalValue − logicalMinimum) / (logicalMaximum − logicalMinimum) − 1.
        let numerator = value - self.axis_bounds.0;
        let denominator = self.axis_bounds.1 - self.axis_bounds.0;
        if denominator != 0.0 && denominator.is_finite() {
            let normalized_value: f64 = 2.0 * numerator / denominator - 1.0;
            if normalized_value.is_finite() {
                self.axes.borrow_mut()[axis_index] = normalized_value;
                self.frozen_axes.clear();
            } else {
                warn!("Axis value is not finite!");
            }
        } else {
            warn!("Axis bounds difference is either 0 or non-finite!");
        }
    }

    /// <https://www.w3.org/TR/gamepad/#dfn-map-and-normalize-buttons>
    pub(crate) fn map_and_normalize_buttons(&self, button_index: usize, value: f64) {
        // Let normalizedValue be (logicalValue − logicalMinimum) / (logicalMaximum − logicalMinimum).
        let numerator = value - self.button_bounds.0;
        let denominator = self.button_bounds.1 - self.button_bounds.0;
        if denominator != 0.0 && denominator.is_finite() {
            let normalized_value: f64 = numerator / denominator;
            if normalized_value.is_finite() {
                let pressed = normalized_value >= BUTTON_PRESS_THRESHOLD;
                // TODO: Determine a way of getting touch capability for button
                if let Some(button) = self.buttons.get(button_index) {
                    button.update(pressed, /*touched*/ pressed, normalized_value);
                    self.frozen_buttons.clear();
                }
            } else {
                warn!("Button value is not finite!");
            }
        } else {
            warn!("Button bounds difference is either 0 or non-finite!");
        }
    }

    /// <https://www.w3.org/TR/gamepad/#dfn-exposed>
    pub(crate) fn exposed(&self) -> bool {
        self.exposed.get()
    }

    /// <https://www.w3.org/TR/gamepad/#dfn-exposed>
    pub(crate) fn set_exposed(&self, exposed: bool) {
        self.exposed.set(exposed);
    }

    pub(crate) fn vibration_actuator(&self) -> &GamepadHapticActuator {
        &self.vibration_actuator
    }
}

/// <https://www.w3.org/TR/gamepad/#dfn-gamepad-user-gesture>
pub(crate) fn contains_user_gesture(update_type: GamepadUpdateType) -> bool {
    match update_type {
        GamepadUpdateType::Axis(_, value) => value.abs() > AXIS_TILT_THRESHOLD,
        GamepadUpdateType::Button(_, value) => value > BUTTON_PRESS_THRESHOLD,
    }
}
