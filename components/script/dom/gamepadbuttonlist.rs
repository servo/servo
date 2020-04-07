/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::GamepadButtonListBinding::GamepadButtonListMethods;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::gamepadbutton::GamepadButton;
use dom_struct::dom_struct;

// https://w3c.github.io/gamepad/#gamepadbutton-interface
#[dom_struct]
pub struct GamepadButtonList {
    reflector_: Reflector,
    list: Vec<Dom<GamepadButton>>,
}

// TODO: support gamepad discovery
#[allow(dead_code)]
impl GamepadButtonList {
    #[allow(unrooted_must_root)]
    fn new_inherited(list: &[&GamepadButton]) -> GamepadButtonList {
        GamepadButtonList {
            reflector_: Reflector::new(),
            list: list.iter().map(|button| Dom::from_ref(*button)).collect(),
        }
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
