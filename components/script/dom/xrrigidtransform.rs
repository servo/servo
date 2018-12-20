/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRRigidTransformBinding;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRRigidTransform {
    reflector_: Reflector,
}

impl XRRigidTransform {
    fn new_inherited() -> XRRigidTransform {
        XRRigidTransform {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<XRRigidTransform> {
        reflect_dom_object(
            Box::new(XRRigidTransform::new_inherited()),
            global,
            XRRigidTransformBinding::Wrap,
        )
    }
}
