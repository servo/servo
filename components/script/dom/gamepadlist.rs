/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::GamepadListBinding;
use dom::bindings::codegen::Bindings::GamepadListBinding::GamepadListMethods;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::gamepad::Gamepad;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

// https://www.w3.org/TR/gamepad/
#[dom_struct]
pub struct GamepadList {
    reflector_: Reflector,
    list: DOMRefCell<Vec<JS<Gamepad>>>
}

impl GamepadList {
    #[allow(unrooted_must_root)]
    fn new_inherited(list: Vec<JS<Gamepad>>) -> GamepadList {
        GamepadList {
            reflector_: Reflector::new(),
            list: DOMRefCell::new(list)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(global: &GlobalScope, list: Vec<Root<Gamepad>>) -> Root<GamepadList> {
        reflect_dom_object(box GamepadList::new_inherited(list.iter().map(|r| JS::from_ref(&**r)).collect()),
                           global,
                           GamepadListBinding::Wrap)
    }

    pub fn add(&self, gamepad: &Gamepad) {
        self.list.borrow_mut().push(JS::from_ref(&*gamepad));
    }

    pub fn sync(&self, gamepads: &[Root<Gamepad>]) {
        for gamepad in gamepads {
            if self.list.borrow().iter().find(|g| g.gamepad_id() == gamepad.gamepad_id()).is_none() {
                self.add(gamepad);
            }
        }
    }
}

impl GamepadListMethods for GamepadList {
    // https://www.w3.org/TR/gamepad/#dom-navigator-getgamepads
    fn Length(&self) -> u32 {
        self.list.borrow().len() as u32
    }

    // https://www.w3.org/TR/gamepad/#dom-navigator-getgamepads
    fn Item(&self, index: u32) -> Option<Root<Gamepad>> {
        if (index as usize) < self.list.borrow().len() {
            Some(Root::from_ref(&*(self.list.borrow()[index as usize])))
        } else {
            None
        }
    }

    // https://www.w3.org/TR/gamepad/#dom-navigator-getgamepads
    fn IndexedGetter(&self, index: u32) -> Option<Root<Gamepad>> {
        self.Item(index)
    }
}
