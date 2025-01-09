/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use euclid::RigidTransform3D;
use webxr_api::{BaseSpace, Frame, InputId, Joint, JointFrame, Space};

use crate::dom::bindings::codegen::Bindings::XRHandBinding::XRHandJoint;
use crate::dom::bindings::codegen::Bindings::XRJointSpaceBinding::XRJointSpaceMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrsession::{ApiPose, XRSession};
use crate::dom::xrspace::XRSpace;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct XRJointSpace {
    xrspace: XRSpace,
    #[ignore_malloc_size_of = "defined in rust-webxr"]
    #[no_trace]
    input: InputId,
    #[ignore_malloc_size_of = "defined in rust-webxr"]
    #[no_trace]
    joint: Joint,
    hand_joint: XRHandJoint,
}

impl XRJointSpace {
    pub(crate) fn new_inherited(
        session: &XRSession,
        input: InputId,
        joint: Joint,
        hand_joint: XRHandJoint,
    ) -> XRJointSpace {
        XRJointSpace {
            xrspace: XRSpace::new_inherited(session),
            input,
            joint,
            hand_joint,
        }
    }

    #[allow(unused)]
    pub(crate) fn new(
        global: &GlobalScope,
        session: &XRSession,
        input: InputId,
        joint: Joint,
        hand_joint: XRHandJoint,
    ) -> DomRoot<XRJointSpace> {
        reflect_dom_object(
            Box::new(Self::new_inherited(session, input, joint, hand_joint)),
            global,
            CanGc::note(),
        )
    }

    pub(crate) fn space(&self) -> Space {
        let base = BaseSpace::Joint(self.input, self.joint);
        let offset = RigidTransform3D::identity();
        Space { base, offset }
    }

    pub(crate) fn frame<'a>(&self, frame: &'a Frame) -> Option<&'a JointFrame> {
        frame
            .inputs
            .iter()
            .find(|i| i.id == self.input)
            .and_then(|i| i.hand.as_ref())
            .and_then(|h| h.get(self.joint))
    }

    pub(crate) fn get_pose(&self, frame: &Frame) -> Option<ApiPose> {
        self.frame(frame).map(|f| f.pose).map(|t| t.cast_unit())
    }
}

impl XRJointSpaceMethods<crate::DomTypeHolder> for XRJointSpace {
    /// <https://www.w3.org/TR/webxr-hand-input-1/#xrjointspace-jointname>
    fn JointName(&self) -> XRHandJoint {
        self.hand_joint
    }
}
