/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use euclid::RigidTransform3D;
use webxr_api::{BaseSpace, Frame, InputId, Joint, JointFrame, Space};

use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrsession::{ApiPose, XRSession};
use crate::dom::xrspace::XRSpace;

#[dom_struct]
pub struct XRJointSpace {
    xrspace: XRSpace,
    #[ignore_malloc_size_of = "defined in rust-webxr"]
    #[no_trace]
    input: InputId,
    #[ignore_malloc_size_of = "defined in rust-webxr"]
    #[no_trace]
    joint: Joint,
}

impl XRJointSpace {
    pub fn new_inherited(session: &XRSession, input: InputId, joint: Joint) -> XRJointSpace {
        XRJointSpace {
            xrspace: XRSpace::new_inherited(session),
            input,
            joint,
        }
    }

    #[allow(unused)]
    pub fn new(
        global: &GlobalScope,
        session: &XRSession,
        input: InputId,
        joint: Joint,
    ) -> DomRoot<XRJointSpace> {
        reflect_dom_object(Box::new(Self::new_inherited(session, input, joint)), global)
    }

    pub fn space(&self) -> Space {
        let base = BaseSpace::Joint(self.input, self.joint);
        let offset = RigidTransform3D::identity();
        Space { base, offset }
    }

    pub fn frame<'a>(&self, frame: &'a Frame) -> Option<&'a JointFrame> {
        frame
            .inputs
            .iter()
            .find(|i| i.id == self.input)
            .and_then(|i| i.hand.as_ref())
            .and_then(|h| h.get(self.joint))
    }

    pub fn get_pose(&self, frame: &Frame) -> Option<ApiPose> {
        self.frame(frame).map(|f| f.pose).map(|t| t.cast_unit())
    }
}
