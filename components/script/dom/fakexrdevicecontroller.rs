/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::FakeXRDeviceControllerBinding;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

#[dom_struct]
pub struct FakeXRDeviceController {
    reflector: Reflector,
}

impl FakeXRDeviceController {
    pub fn new_inherited() -> FakeXRDeviceController {
        FakeXRDeviceController {
            reflector: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<FakeXRDeviceController> {
        reflect_dom_object(
            Box::new(FakeXRDeviceController::new_inherited()),
            global,
            FakeXRDeviceControllerBinding::Wrap,
        )
    }
}
