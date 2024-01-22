/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::XRPoseBinding::XRPoseMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::ApiRigidTransform;

#[dom_struct]
pub struct XRPose {
    reflector_: Reflector,
    transform: Dom<XRRigidTransform>,
}

impl XRPose {
    pub fn new_inherited(transform: &XRRigidTransform) -> XRPose {
        XRPose {
            reflector_: Reflector::new(),
            transform: Dom::from_ref(transform),
        }
    }

    #[allow(unused)]
    pub fn new(global: &GlobalScope, transform: ApiRigidTransform) -> DomRoot<XRPose> {
        let transform = XRRigidTransform::new(global, transform);
        reflect_dom_object(Box::new(XRPose::new_inherited(&transform)), global)
    }
}

impl XRPoseMethods for XRPose {
    /// <https://immersive-web.github.io/webxr/#dom-xrpose-transform>
    fn Transform(&self) -> DomRoot<XRRigidTransform> {
        DomRoot::from_ref(&self.transform)
    }
}
