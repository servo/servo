/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRViewerPoseBinding;
use crate::dom::bindings::codegen::Bindings::XRViewerPoseBinding::XRViewerPoseMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrview::XRView;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRViewerPose {
    reflector_: Reflector,
    left: Dom<XRView>,
    right: Dom<XRView>,
}

impl XRViewerPose {
    fn new_inherited(left: &XRView, right: &XRView) -> XRViewerPose {
        XRViewerPose {
            reflector_: Reflector::new(),
            left: Dom::from_ref(left),
            right: Dom::from_ref(right),
        }
    }

    pub fn new(global: &GlobalScope, left: &XRView, right: &XRView) -> DomRoot<XRViewerPose> {
        reflect_dom_object(
            Box::new(XRViewerPose::new_inherited(left, right)),
            global,
            XRViewerPoseBinding::Wrap,
        )
    }
}

impl XRViewerPoseMethods for XRViewerPose {
    /// https://immersive-web.github.io/webxr/#dom-xrviewerpose-views
    fn Views(&self) -> Vec<DomRoot<XRView>> {
        vec![
            DomRoot::from_ref(&self.left),
            DomRoot::from_ref(&self.right),
        ]
    }
}
