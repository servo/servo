/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::GamepadButtonBinding::GamepadButtonMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct GamepadButton {
    reflector_: Reflector,
    pressed: Cell<bool>,
    touched: Cell<bool>,
    value: Cell<f64>,
}

impl GamepadButton {
    pub fn new_inherited(pressed: bool, touched: bool) -> GamepadButton {
        Self {
            reflector_: Reflector::new(),
            pressed: Cell::new(pressed),
            touched: Cell::new(touched),
            value: Cell::new(0.0),
        }
    }

    pub fn new(global: &GlobalScope, pressed: bool, touched: bool) -> DomRoot<GamepadButton> {
        reflect_dom_object(
            Box::new(GamepadButton::new_inherited(pressed, touched)),
            global,
        )
    }
}

impl GamepadButtonMethods for GamepadButton {
    // https://www.w3.org/TR/gamepad/#widl-GamepadButton-pressed
    fn Pressed(&self) -> bool {
        self.pressed.get()
    }

    // https://www.w3.org/TR/gamepad/#widl-GamepadButton-touched
    fn Touched(&self) -> bool {
        self.touched.get()
    }

    // https://www.w3.org/TR/gamepad/#widl-GamepadButton-value
    fn Value(&self) -> Finite<f64> {
        Finite::wrap(self.value.get())
    }
}

impl GamepadButton {
    pub fn update(&self, pressed: bool, touched: bool, value: f64) {
        self.pressed.set(pressed);
        self.touched.set(touched);
        self.value.set(value);
    }
}
