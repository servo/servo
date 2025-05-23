/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use crate::dom::virtualmethods::VirtualMethods;

#[dom_struct]
pub(crate) struct SVGAnimatedNumber {
    anim_val: f32,
    base_val: Cell<f32>,
}

impl SVGAnimatedNumber {
    // attribute float baseVal;
    pub fn base_val(&self) -> f32 {
        self.base_val.get()
    }

    pub fn set_base_val(&self, value: f32) {
        self.base_val.set(value);
    }

    // readonly attribute float animVal;
    pub fn anim_val(&self) -> f32 {
        self.anim_val
    }
}

impl VirtualMethods for SVGAnimatedNumber {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        None
    }
}
