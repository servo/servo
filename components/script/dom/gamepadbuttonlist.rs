/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::GamepadButtonListBinding;
use dom::bindings::codegen::Bindings::GamepadButtonListBinding::GamepadButtonListMethods;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::gamepadbutton::GamepadButton;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use webvr_traits::WebVRGamepadButton;

// https://www.w3.org/TR/GamepadButton/
#[dom_struct]
pub struct GamepadButtonList {
    reflector_: Reflector,
    list: Vec<JS<GamepadButton>>
}

impl GamepadButtonList {
    #[allow(unrooted_must_root)]
    fn new_inherited(list: Vec<JS<GamepadButton>>) -> GamepadButtonList {
        GamepadButtonList {
            reflector_: Reflector::new(),
            list: list
        }
    }

    pub fn new_from_vr(global: &GlobalScope, buttons: &[WebVRGamepadButton]) -> Root<GamepadButtonList> {
        let list: Vec<Root<GamepadButton>> = buttons.iter()
                                                    .map(|btn| GamepadButton::new(&global, btn.pressed, btn.touched))
                                                    .collect();
        reflect_dom_object(box GamepadButtonList::new_inherited(list.iter().map(|g| JS::from_ref(&**g)).collect()),
                           global,
                           GamepadButtonListBinding::Wrap)
    }

    pub fn sync_from_vr(&self, vr_buttons: &[WebVRGamepadButton]) {
        let mut index = 0;
        for btn in vr_buttons {
            self.list.get(index).as_ref().unwrap().update(btn.pressed, btn.touched);
            index += 1;
        }
    }
}

impl GamepadButtonListMethods for GamepadButtonList {
    // https://w3c.github.io/gamepad/#dom-gamepad-buttons
    fn Length(&self) -> u32 {
        self.list.len() as u32
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-buttons
    fn Item(&self, index: u32) -> Option<Root<GamepadButton>> {
        self.list.get(index as usize).map(|button| Root::from_ref(&**button))
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-buttons
    fn IndexedGetter(&self, index: u32) -> Option<Root<GamepadButton>> {
        self.Item(index)
    }
}
