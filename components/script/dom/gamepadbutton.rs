/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::GamepadButtonBinding;
use dom::bindings::codegen::Bindings::GamepadButtonBinding::GamepadButtonMethods;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use std::cell::Cell;
use typeholder::TypeHolderTrait;
use std::marker::PhantomData;

#[dom_struct]
pub struct GamepadButton<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    pressed: Cell<bool>,
    touched: Cell<bool>,
    value: Cell<f64>,
    _p: PhantomData<TH>,
}

impl<TH: TypeHolderTrait> GamepadButton<TH> {
    pub fn new_inherited(pressed: bool, touched: bool) -> GamepadButton<TH> {
        Self {
            reflector_: Reflector::new(),
            pressed: Cell::new(pressed),
            touched: Cell::new(touched),
            value: Cell::new(0.0),
            _p: Default::default(),
        }
    }

    pub fn new(global: &GlobalScope<TH>,pressed: bool, touched: bool) -> DomRoot<GamepadButton<TH>> {
        reflect_dom_object(Box::new(GamepadButton::new_inherited(pressed, touched)),
                           global,
                           GamepadButtonBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> GamepadButtonMethods for GamepadButton<TH> {
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

impl<TH: TypeHolderTrait> GamepadButton<TH> {
    pub fn update(&self, pressed: bool, touched: bool) {
        self.pressed.set(pressed);
        self.touched.set(touched);
    }
}
