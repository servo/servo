/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::GamepadButtonListBinding;
use dom::bindings::codegen::Bindings::GamepadButtonListBinding::GamepadButtonListMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot, RootedReference};
use dom::gamepadbutton::GamepadButton;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;
use webvr_traits::WebVRGamepadButton;

// https://w3c.github.io/gamepad/#gamepadbutton-interface
#[dom_struct]
pub struct GamepadButtonList<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    list: Vec<Dom<GamepadButton<TH>>>
}

impl<TH: TypeHolderTrait> GamepadButtonList<TH> {
    #[allow(unrooted_must_root)]
    fn new_inherited(list: &[&GamepadButton<TH>]) -> GamepadButtonList<TH> {
        GamepadButtonList {
            reflector_: Reflector::new(),
            list: list.iter().map(|button| Dom::from_ref(*button)).collect(),
        }
    }

    pub fn new_from_vr(global: &GlobalScope<TH>, buttons: &[WebVRGamepadButton]) -> DomRoot<GamepadButtonList<TH>> {
        rooted_vec!(let list <- buttons.iter()
                                       .map(|btn| GamepadButton::new(&global, btn.pressed, btn.touched)));

        reflect_dom_object(Box::new(GamepadButtonList::new_inherited(list.r())),
                           global,
                           GamepadButtonListBinding::Wrap)
    }

    pub fn sync_from_vr(&self, vr_buttons: &[WebVRGamepadButton]) {
        for (gp_btn, btn) in self.list.iter().zip(vr_buttons.iter()) {
            gp_btn.update(btn.pressed, btn.touched);
        }
    }
}

impl<TH: TypeHolderTrait> GamepadButtonListMethods<TH> for GamepadButtonList<TH> {
    // https://w3c.github.io/gamepad/#dom-gamepad-buttons
    fn Length(&self) -> u32 {
        self.list.len() as u32
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-buttons
    fn Item(&self, index: u32) -> Option<DomRoot<GamepadButton<TH>>> {
        self.list.get(index as usize).map(|button| DomRoot::from_ref(&**button))
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-buttons
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<GamepadButton<TH>>> {
        self.Item(index)
    }
}
