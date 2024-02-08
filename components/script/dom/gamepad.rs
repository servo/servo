/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::typedarray::{Float64, Float64Array};

use super::bindings::typedarrays::HeapTypedArray;
use crate::dom::bindings::codegen::Bindings::GamepadBinding::{GamepadHand, GamepadMethods};
use crate::dom::bindings::codegen::Bindings::GamepadButtonListBinding::GamepadButtonListMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, DomSlice};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::gamepadbutton::GamepadButton;
use crate::dom::gamepadbuttonlist::GamepadButtonList;
use crate::dom::gamepadevent::{GamepadEvent, GamepadEventType};
use crate::dom::gamepadpose::GamepadPose;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;

#[dom_struct]
pub struct Gamepad {
    reflector_: Reflector,
    gamepad_id: u32,
    id: String,
    index: Cell<i32>,
    connected: Cell<bool>,
    timestamp: Cell<f64>,
    mapping_type: String,
    #[ignore_malloc_size_of = "mozjs"]
    axes: HeapTypedArray<Float64>,
    buttons: Dom<GamepadButtonList>,
    pose: Option<Dom<GamepadPose>>,
    #[ignore_malloc_size_of = "Defined in rust-webvr"]
    hand: GamepadHand,
}

impl Gamepad {
    fn new_inherited(
        gamepad_id: u32,
        id: String,
        index: i32,
        connected: bool,
        timestamp: f64,
        mapping_type: String,
        buttons: &GamepadButtonList,
        pose: Option<&GamepadPose>,
        hand: GamepadHand,
    ) -> Gamepad {
        Self {
            reflector_: Reflector::new(),
            gamepad_id: gamepad_id,
            id: id,
            index: Cell::new(index),
            connected: Cell::new(connected),
            timestamp: Cell::new(timestamp),
            mapping_type: mapping_type,
            axes: HeapTypedArray::default(),
            buttons: Dom::from_ref(buttons),
            pose: pose.map(Dom::from_ref),
            hand: hand,
        }
    }

    pub fn new(global: &GlobalScope, gamepad_id: u32, id: String) -> DomRoot<Gamepad> {
        let gamepad = Self::new_with_proto(global, gamepad_id, id);
        gamepad.init_axes();
        gamepad
    }

    fn new_with_proto(global: &GlobalScope, gamepad_id: u32, id: String) -> DomRoot<Gamepad> {
        // https://www.w3.org/TR/gamepad/#dfn-initializing-buttons
        // https://www.w3.org/TR/gamepad/#fingerprinting-mitigation
        // Initialize the number of buttons in the "standard" gamepad mapping.
        // The spec says UAs *may* do this for fingerprint mitigation, and it also
        // happens to simplify implementation
        let standard_buttons = &[
            GamepadButton::new(global, false, false), // Bottom button in right cluster
            GamepadButton::new(global, false, false), // Right button in right cluster
            GamepadButton::new(global, false, false), // Left button in right cluster
            GamepadButton::new(global, false, false), // Top button in right cluster
            GamepadButton::new(global, false, false), // Top left front button
            GamepadButton::new(global, false, false), // Top right front button
            GamepadButton::new(global, false, false), // Bottom left front button
            GamepadButton::new(global, false, false), // Bottom right front button
            GamepadButton::new(global, false, false), // Left button in center cluster
            GamepadButton::new(global, false, false), // Right button in center cluster
            GamepadButton::new(global, false, false), // Left stick pressed button
            GamepadButton::new(global, false, false), // Right stick pressed button
            GamepadButton::new(global, false, false), // Top button in left cluster
            GamepadButton::new(global, false, false), // Bottom button in left cluster
            GamepadButton::new(global, false, false), // Left button in left cluster
            GamepadButton::new(global, false, false), // Right button in left cluster
            GamepadButton::new(global, false, false), // Center button in center cluster
        ];
        rooted_vec!(let buttons <- standard_buttons.iter().map(|button| DomRoot::from_ref(&**button)));
        let button_list = GamepadButtonList::new(global, buttons.r());
        let gamepad = reflect_dom_object_with_proto(
            Box::new(Gamepad::new_inherited(
                gamepad_id,
                id,
                0,
                false,
                0.,
                String::from("standard"),
                &button_list,
                None,
                GamepadHand::_empty,
            )),
            global,
            None,
        );
        gamepad
    }
}

impl GamepadMethods for Gamepad {
    // https://w3c.github.io/gamepad/#dom-gamepad-id
    fn Id(&self) -> DOMString {
        DOMString::from(self.id.clone())
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-index
    fn Index(&self) -> i32 {
        self.index.get()
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-connected
    fn Connected(&self) -> bool {
        self.connected.get()
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-timestamp
    fn Timestamp(&self) -> Finite<f64> {
        Finite::wrap(self.timestamp.get())
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-mapping
    fn Mapping(&self) -> DOMString {
        DOMString::from(self.mapping_type.clone())
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-axes
    fn Axes(&self, _cx: JSContext) -> Float64Array {
        self.axes
            .get_internal()
            .expect("Failed to get gamepad axes.")
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-buttons
    fn Buttons(&self) -> DomRoot<GamepadButtonList> {
        DomRoot::from_ref(&*self.buttons)
    }

    // https://w3c.github.io/gamepad/extensions.html#gamepadhand-enum
    fn Hand(&self) -> GamepadHand {
        self.hand
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepad-pose
    fn GetPose(&self) -> Option<DomRoot<GamepadPose>> {
        self.pose.as_ref().map(|p| DomRoot::from_ref(&**p))
    }
}

#[allow(dead_code)]
impl Gamepad {
    pub fn gamepad_id(&self) -> u32 {
        self.gamepad_id
    }

    pub fn update_connected(&self, connected: bool) {
        if self.connected.get() == connected {
            return;
        }
        self.connected.set(connected);

        let event_type = if connected {
            GamepadEventType::Connected
        } else {
            GamepadEventType::Disconnected
        };

        self.notify_event(event_type);
    }

    pub fn update_index(&self, index: i32) {
        self.index.set(index);
    }

    pub fn update_timestamp(&self, timestamp: f64) {
        self.timestamp.set(timestamp);
    }

    pub fn notify_event(&self, event_type: GamepadEventType) {
        let event = GamepadEvent::new_with_type(&self.global(), event_type, &self);
        event
            .upcast::<Event>()
            .fire(self.global().as_window().upcast::<EventTarget>());
    }

    // https://www.w3.org/TR/gamepad/#dfn-initializing-axes
    // https://www.w3.org/TR/gamepad/#fingerprinting-mitigation
    pub fn init_axes(&self) {
        // Initialize the number of axes in the "standard" gamepad mapping.
        // See above comment on buttons
        let initial_axes: Vec<f64> = vec![
            0., // Horizontal axis for left stick (negative left/positive right)
            0., // Vertical axis for left stick (negative up/positive down)
            0., // Horizontal axis for right stick (negative left/positive right)
            0., // Vertical axis for right stick (negative up/positive down)
        ];
        self.axes
            .set_data(GlobalScope::get_cx(), &initial_axes)
            .expect("Failed to set axes data on gamepad.")
    }

    #[allow(unsafe_code)]
    pub fn update_axis(&self, axis_index: usize, value: f32) {
        let mut axis_vec = self.axes.get_internal().unwrap();
        unsafe {
            axis_vec.as_mut_slice()[axis_index] = value as f64;
        }
    }

    pub fn update_button(&self, button_index: usize, pressed: bool, touched: bool, value: f32) {
        if let Some(button) = self.buttons.IndexedGetter(button_index as u32) {
            button.update(pressed, touched, value.into());
        }
    }
}
