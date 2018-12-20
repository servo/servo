/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRViewerPoseBinding;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRViewerPose {
    reflector_: Reflector,
}

impl XRViewerPose {
    fn new_inherited() -> XRViewerPose {
        XRViewerPose {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<XRViewerPose> {
        reflect_dom_object(
            Box::new(XRViewerPose::new_inherited()),
            global,
            XRViewerPoseBinding::Wrap,
        )
    }
}
