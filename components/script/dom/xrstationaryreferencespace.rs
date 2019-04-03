/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRStationaryReferenceSpaceBinding;
use crate::dom::bindings::codegen::Bindings::XRStationaryReferenceSpaceBinding::XRStationaryReferenceSpaceSubtype;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::XRSession;
use dom_struct::dom_struct;
use euclid::{Rotation3D, RigidTransform3D, Vector3D};
use webvr_traits::WebVRFrameData;

#[dom_struct]
pub struct XRStationaryReferenceSpace {
    xrreferencespace: XRReferenceSpace,
    ty: XRStationaryReferenceSpaceSubtype,
}

#[allow(unused)]
impl XRStationaryReferenceSpace {
    pub fn new_inherited(
        session: &XRSession,
        ty: XRStationaryReferenceSpaceSubtype,
        transform: &XRRigidTransform,
    ) -> XRStationaryReferenceSpace {
        XRStationaryReferenceSpace {
            xrreferencespace: XRReferenceSpace::new_inherited(session, transform),
            ty,
        }
    }

    pub fn new(
        window: &Window,
        session: &XRSession,
        ty: XRStationaryReferenceSpaceSubtype,
    ) -> DomRoot<XRStationaryReferenceSpace> {
        let transform = XRRigidTransform::identity(window);
        reflect_dom_object(
            Box::new(XRStationaryReferenceSpace::new_inherited(
                session, ty, &transform,
            )),
            window,
            XRStationaryReferenceSpaceBinding::Wrap,
        )
    }
}

impl XRStationaryReferenceSpace {
    /// Gets pose represented by this space
    ///
    /// Does not apply originOffset, use get_viewer_pose instead
    pub fn get_pose(&self, base_pose: &WebVRFrameData) -> RigidTransform3D<f64> {
        // XXXManishearth add floor-level transform for floor-level and disable position in position-disabled
        let pos = base_pose.pose.position.unwrap_or([0., 0., 0.]);
        let translation = Vector3D::new(pos[0] as f64, pos[1] as f64, pos[2] as f64);
        let orient = base_pose.pose.orientation.unwrap_or([0., 0., 0., 0.]);
        let rotation = Rotation3D::quaternion(
            orient[0] as f64,
            orient[1] as f64,
            orient[2] as f64,
            orient[3] as f64,
        );
        RigidTransform3D::new(rotation, translation)
    }
}
