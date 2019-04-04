/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRStationaryReferenceSpaceBinding;
use crate::dom::bindings::codegen::Bindings::XRStationaryReferenceSpaceBinding::XRStationaryReferenceSpaceSubtype;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::XRSession;
use crate::dom::xrspace::XRSpace;
use dom_struct::dom_struct;
use euclid::RigidTransform3D;
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
        global: &GlobalScope,
        session: &XRSession,
        ty: XRStationaryReferenceSpaceSubtype,
    ) -> DomRoot<XRStationaryReferenceSpace> {
        let transform = XRRigidTransform::identity(global);
        reflect_dom_object(
            Box::new(XRStationaryReferenceSpace::new_inherited(
                session, ty, &transform,
            )),
            global,
            XRStationaryReferenceSpaceBinding::Wrap,
        )
    }
}

impl XRStationaryReferenceSpace {
    /// Gets pose of the viewer with respect to this space
    ///
    /// Does not apply originOffset, use get_viewer_pose on XRReferenceSpace instead
    pub fn get_unoffset_viewer_pose(&self, base_pose: &WebVRFrameData) -> RigidTransform3D<f64> {
        // XXXManishearth add floor-level transform for floor-level and disable position in position-disabled
        XRSpace::viewer_pose_from_frame_data(base_pose)
    }

    /// Gets pose represented by this space
    ///
    /// Does not apply originOffset, use get_pose on XRReferenceSpace instead
    pub fn get_unoffset_pose(&self, _: &WebVRFrameData) -> RigidTransform3D<f64> {
        // XXXManishearth add floor-level transform for floor-level and disable position in position-disabled

        // The eye-level pose is basically whatever the headset pose was at t=0, which
        // for most devices is (0, 0, 0)
        RigidTransform3D::identity()
    }
}
