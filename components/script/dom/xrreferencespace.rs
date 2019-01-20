/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrspace::XRSpace;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRReferenceSpace {
    xrspace: XRSpace,
}

impl XRReferenceSpace {
    pub fn new_inherited() -> XRReferenceSpace {
        XRReferenceSpace {
            xrspace: XRSpace::new_inherited(),
        }
    }

    #[allow(unused)]
    pub fn new(global: &GlobalScope) -> DomRoot<XRReferenceSpace> {
        reflect_dom_object(
            Box::new(XRReferenceSpace::new_inherited()),
            global,
            XRReferenceSpaceBinding::Wrap,
        )
    }
}
