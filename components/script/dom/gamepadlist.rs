/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::GamepadListBinding;
use dom::bindings::codegen::Bindings::GamepadListBinding::GamepadListMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::gamepad::Gamepad;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

// https://www.w3.org/TR/gamepad/
#[dom_struct]
pub struct GamepadList {
    reflector_: Reflector,
    list: DomRefCell<Vec<Dom<Gamepad>>>
}

impl GamepadList {
    fn new_inherited(list: &[&Gamepad]) -> GamepadList {
        GamepadList {
            reflector_: Reflector::new(),
            list: DomRefCell::new(list.iter().map(|g| Dom::from_ref(&**g)).collect())
        }
    }

    pub fn new(global: &GlobalScope, list: &[&Gamepad]) -> DomRoot<GamepadList> {
        reflect_dom_object(Box::new(GamepadList::new_inherited(list)),
                           global,
                           GamepadListBinding::Wrap)
    }

    pub fn add_if_not_exists(&self, gamepads: &[DomRoot<Gamepad>]) {
        for gamepad in gamepads {
            if !self.list.borrow().iter().any(|g| g.gamepad_id() == gamepad.gamepad_id()) {
                self.list.borrow_mut().push(Dom::from_ref(&*gamepad));
                // Ensure that the gamepad has the correct index
                gamepad.update_index(self.list.borrow().len() as i32 - 1);
            }
        }
    }
}

impl GamepadListMethods for GamepadList {
    // https://w3c.github.io/gamepad/#dom-navigator-getgamepads
    fn Length(&self) -> u32 {
        self.list.borrow().len() as u32
    }

    // https://w3c.github.io/gamepad/#dom-navigator-getgamepads
    fn Item(&self, index: u32) -> Option<DomRoot<Gamepad>> {
        self.list.borrow().get(index as usize).map(|gamepad| DomRoot::from_ref(&**gamepad))
    }

    // https://w3c.github.io/gamepad/#dom-navigator-getgamepads
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<Gamepad>> {
        self.Item(index)
    }
}
