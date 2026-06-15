/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::reflect_dom_object_with_cx;

use crate::dom::bindings::codegen::Bindings::XRJointPoseBinding::XRJointPoseMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;
use crate::dom::xrpose::XRPose;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::ApiRigidTransform;

#[dom_struct]
pub(crate) struct XRJointPose {
    pose: XRPose,
    radius: Option<f32>,
}

impl XRJointPose {
    fn new_inherited(transform: &XRRigidTransform, radius: Option<f32>) -> XRJointPose {
        XRJointPose {
            pose: XRPose::new_inherited(transform),
            radius,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        pose: ApiRigidTransform,
        radius: Option<f32>,
    ) -> DomRoot<XRJointPose> {
        let transform = XRRigidTransform::new(cx, window, pose);
        reflect_dom_object_with_cx(
            Box::new(XRJointPose::new_inherited(&transform, radius)),
            window,
            cx,
        )
    }
}

impl XRJointPoseMethods<crate::DomTypeHolder> for XRJointPose {
    /// <https://immersive-web.github.io/webxr/#dom-XRJointPose-views>
    fn GetRadius(&self) -> Option<Finite<f32>> {
        self.radius.map(Finite::wrap)
    }
}
