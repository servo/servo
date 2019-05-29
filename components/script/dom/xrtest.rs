/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRTestBinding;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRTest {
    reflector: Reflector,
}

impl XRTest {
    pub fn new_inherited() -> XRTest {
        XRTest {
            reflector: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<XRTest> {
        reflect_dom_object(
            Box::new(XRTest::new_inherited()),
            global,
            XRTestBinding::Wrap,
        )
    }
}
