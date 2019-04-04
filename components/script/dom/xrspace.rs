/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRSpaceBinding;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrsession::XRSession;
use dom_struct::dom_struct;
use euclid::{RigidTransform3D, Rotation3D, Vector3D};
use webvr_traits::WebVRFrameData;

#[dom_struct]
pub struct XRSpace {
    eventtarget: EventTarget,
    session: Dom<XRSession>,
}

impl XRSpace {
    pub fn new_inherited(session: &XRSession) -> XRSpace {
        XRSpace {
            eventtarget: EventTarget::new_inherited(),
            session: Dom::from_ref(session),
        }
    }

    #[allow(unused)]
    pub fn new(global: &GlobalScope, session: &XRSession) -> DomRoot<XRSpace> {
        reflect_dom_object(
            Box::new(XRSpace::new_inherited(session)),
            global,
            XRSpaceBinding::Wrap,
        )
    }
}

impl XRSpace {
    /// Gets pose represented by this space
    ///
    /// The reference origin used is common between all
    /// get_pose calls for spaces from the same device, so this can be used to compare
    /// with other spaces
    pub fn get_pose(&self, base_pose: &WebVRFrameData) -> RigidTransform3D<f64> {
        if let Some(reference) = self.downcast::<XRReferenceSpace>() {
            reference.get_pose(base_pose)
        } else {
            unreachable!()
        }
    }

    pub fn viewer_pose_from_frame_data(data: &WebVRFrameData) -> RigidTransform3D<f64> {
        let pos = data.pose.position.unwrap_or([0., 0., 0.]);
        let translation = Vector3D::new(pos[0] as f64, pos[1] as f64, pos[2] as f64);
        let orient = data.pose.orientation.unwrap_or([0., 0., 0., 0.]);
        let rotation = Rotation3D::quaternion(
            orient[0] as f64,
            orient[1] as f64,
            orient[2] as f64,
            orient[3] as f64,
        );
        RigidTransform3D::new(rotation, translation)
    }

    pub fn session(&self) -> &XRSession {
        &self.session
    }
}
