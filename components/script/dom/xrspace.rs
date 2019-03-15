/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRSpaceBinding;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRSpace {
    eventtarget: EventTarget,
}

impl XRSpace {
    pub fn new_inherited() -> XRSpace {
        XRSpace {
            eventtarget: EventTarget::new_inherited(),
        }
    }

    #[allow(unused)]
    pub fn new(global: &GlobalScope) -> DomRoot<XRSpace> {
        reflect_dom_object(
            Box::new(XRSpace::new_inherited()),
            global,
            XRSpaceBinding::Wrap,
        )
    }
}
