/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::XRPoseBinding::XRPoseMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::dompointreadonly::DOMPointReadOnly;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::ApiRigidTransform;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct XRPose {
    reflector_: Reflector,
    transform: Dom<XRRigidTransform>,
}

impl XRPose {
    pub(crate) fn new_inherited(transform: &XRRigidTransform) -> XRPose {
        XRPose {
            reflector_: Reflector::new(),
            transform: Dom::from_ref(transform),
        }
    }

    #[allow(unused)]
    pub(crate) fn new(
        global: &GlobalScope,
        transform: ApiRigidTransform,
        can_gc: CanGc,
    ) -> DomRoot<XRPose> {
        let transform = XRRigidTransform::new(global, transform, can_gc);
        reflect_dom_object(
            Box::new(XRPose::new_inherited(&transform)),
            global,
            CanGc::note(),
        )
    }
}

impl XRPoseMethods<crate::DomTypeHolder> for XRPose {
    /// <https://immersive-web.github.io/webxr/#dom-xrpose-transform>
    fn Transform(&self) -> DomRoot<XRRigidTransform> {
        DomRoot::from_ref(&self.transform)
    }

    /// <https://www.w3.org/TR/webxr/#dom-xrpose-linearvelocity>
    fn GetLinearVelocity(&self) -> Option<DomRoot<DOMPointReadOnly>> {
        // TODO: Expose from webxr crate
        None
    }

    /// <https://www.w3.org/TR/webxr/#dom-xrpose-angularvelocity>
    fn GetAngularVelocity(&self) -> Option<DomRoot<DOMPointReadOnly>> {
        // TODO: Expose from webxr crate
        None
    }

    /// <https://www.w3.org/TR/webxr/#dom-xrpose-emulatedposition>
    fn EmulatedPosition(&self) -> bool {
        // There are currently no instances in which we would need to rely
        // on emulation for reporting pose, so return false.
        false
    }
}
