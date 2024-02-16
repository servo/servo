/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::GamepadButtonListBinding::GamepadButtonListMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, DomSlice};
use crate::dom::gamepadbutton::GamepadButton;
use crate::dom::globalscope::GlobalScope;

// https://w3c.github.io/gamepad/#gamepadbutton-interface
#[dom_struct]
pub struct GamepadButtonList {
    reflector_: Reflector,
    list: Vec<Dom<GamepadButton>>,
}

impl GamepadButtonList {
    #[allow(crown::unrooted_must_root)]
    fn new_inherited(list: &[&GamepadButton]) -> GamepadButtonList {
        GamepadButtonList {
            reflector_: Reflector::new(),
            list: list.iter().map(|button| Dom::from_ref(*button)).collect(),
        }
    }

    pub fn new(global: &GlobalScope, list: &[&GamepadButton]) -> DomRoot<GamepadButtonList> {
        reflect_dom_object(Box::new(GamepadButtonList::new_inherited(list)), global)
    }
}

impl GamepadButtonListMethods for GamepadButtonList {
    // https://w3c.github.io/gamepad/#dom-gamepad-buttons
    fn Length(&self) -> u32 {
        self.list.len() as u32
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-buttons
    fn Item(&self, index: u32) -> Option<DomRoot<GamepadButton>> {
        self.list
            .get(index as usize)
            .map(|button| DomRoot::from_ref(&**button))
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-buttons
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<GamepadButton>> {
        self.Item(index)
    }
}

impl GamepadButtonList {
    /// Initialize the number of buttons in the "standard" gamepad mapping.
    /// <https://www.w3.org/TR/gamepad/#dfn-initializing-buttons>
    pub fn init_buttons(global: &GlobalScope) -> DomRoot<GamepadButtonList> {
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
        Self::new(global, buttons.r())
    }
}
