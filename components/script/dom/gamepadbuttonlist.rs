/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::GamepadButtonListBinding::GamepadButtonListMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, DomSlice};
use crate::dom::gamepadbutton::GamepadButton;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

// https://w3c.github.io/gamepad/#gamepadbutton-interface
#[dom_struct]
pub(crate) struct GamepadButtonList {
    reflector_: Reflector,
    list: Vec<Dom<GamepadButton>>,
}

impl GamepadButtonList {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(list: &[&GamepadButton]) -> GamepadButtonList {
        GamepadButtonList {
            reflector_: Reflector::new(),
            list: list.iter().map(|button| Dom::from_ref(*button)).collect(),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        list: &[&GamepadButton],
        can_gc: CanGc,
    ) -> DomRoot<GamepadButtonList> {
        reflect_dom_object(
            Box::new(GamepadButtonList::new_inherited(list)),
            global,
            can_gc,
        )
    }
}

impl GamepadButtonListMethods<crate::DomTypeHolder> for GamepadButtonList {
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
    pub(crate) fn init_buttons(global: &GlobalScope) -> DomRoot<GamepadButtonList> {
        let standard_buttons = &[
            GamepadButton::new(global, false, false, CanGc::note()), // Bottom button in right cluster
            GamepadButton::new(global, false, false, CanGc::note()), // Right button in right cluster
            GamepadButton::new(global, false, false, CanGc::note()), // Left button in right cluster
            GamepadButton::new(global, false, false, CanGc::note()), // Top button in right cluster
            GamepadButton::new(global, false, false, CanGc::note()), // Top left front button
            GamepadButton::new(global, false, false, CanGc::note()), // Top right front button
            GamepadButton::new(global, false, false, CanGc::note()), // Bottom left front button
            GamepadButton::new(global, false, false, CanGc::note()), // Bottom right front button
            GamepadButton::new(global, false, false, CanGc::note()), // Left button in center cluster
            GamepadButton::new(global, false, false, CanGc::note()), // Right button in center cluster
            GamepadButton::new(global, false, false, CanGc::note()), // Left stick pressed button
            GamepadButton::new(global, false, false, CanGc::note()), // Right stick pressed button
            GamepadButton::new(global, false, false, CanGc::note()), // Top button in left cluster
            GamepadButton::new(global, false, false, CanGc::note()), // Bottom button in left cluster
            GamepadButton::new(global, false, false, CanGc::note()), // Left button in left cluster
            GamepadButton::new(global, false, false, CanGc::note()), // Right button in left cluster
            GamepadButton::new(global, false, false, CanGc::note()), // Center button in center cluster
        ];
        rooted_vec!(let buttons <- standard_buttons.iter().map(DomRoot::as_traced));
        Self::new(global, buttons.r(), CanGc::note())
    }
}
