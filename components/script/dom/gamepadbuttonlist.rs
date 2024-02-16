/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::GamepadButtonListBinding::GamepadButtonListMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
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
        reflect_dom_object(
            Box::new(GamepadButtonList::new_inherited(list)),
            global,
        )
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
