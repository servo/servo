/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dom_struct::dom_struct;
use js::gc::MutableHandleValue;
use script_bindings::domstring::DOMString;
use script_bindings::reflector::Reflector;

use crate::dom::bindings::codegen::Bindings::ServoTestUtilsBinding::LayoutResultMethods;
use crate::dom::bindings::import::base::SafeJSContext;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct LayoutResult {
    reflector_: Reflector,
    phases: Vec<DOMString>,
}

impl LayoutResult {
    pub(crate) fn new_inherited(phases: Vec<DOMString>) -> Self {
        Self {
            reflector_: Reflector::new(),
            phases,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        phases: Vec<DOMString>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(phases)), global, can_gc)
    }
}

impl LayoutResultMethods<crate::DomTypeHolder> for LayoutResult {
    fn Phases(&self, cx: SafeJSContext, can_gc: CanGc, return_value: MutableHandleValue) {
        to_frozen_array(&self.phases, cx, return_value, can_gc);
    }
}
