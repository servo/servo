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
use euclid::{RigidTransform3D, Vector3D};
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
    pub fn get_unoffset_viewer_pose(&self, viewer_pose: &WebVRFrameData) -> RigidTransform3D<f64> {
        let viewer_pose = XRSpace::viewer_pose_from_frame_data(viewer_pose);
        // all math is in column-vector notation
        // we use the following equation to verify correctness here:
        // get_viewer_pose(space) = get_pose(space).inverse() * get_pose(viewer_space)
        match self.ty {
            XRStationaryReferenceSpaceSubtype::Eye_level => {
                // get_viewer_pose(eye_level) = get_pose(eye_level).inverse() * get_pose(viewer_space)
                //                            = I * viewer_pose
                //                            = viewer_pose

                // we get viewer poses in eye-level space by default
                viewer_pose
            },
            XRStationaryReferenceSpaceSubtype::Floor_level => {
                // XXXManishearth support getting floor info from stage parameters

                // get_viewer_pose(floor_level) = get_pose(floor_level).inverse() * get_pose(viewer_space)
                //                            = Translate(-2).inverse() * viewer_pose
                //                            = Translate(2) * viewer_pose

                // assume approximate user height of 2 meters
                let floor_to_eye: RigidTransform3D<f64> = Vector3D::new(0., 2., 0.).into();
                floor_to_eye.pre_mul(&viewer_pose)
            },
            XRStationaryReferenceSpaceSubtype::Position_disabled => {
                // get_viewer_pose(pos_disabled) = get_pose(pos_disabled).inverse() * get_pose(viewer_space)
                //                            = viewer_pose.translation.inverse() * viewer_pose
                //                            = viewer_pose.translation.inverse() * viewer_pose.translation
                //                                                                * viewer_pose.rotation
                //                            = viewer_pose.rotation

                // This space follows the user around, but does not mirror the user's orientation
                // Thus, the viewer's pose relative to this space is simply their orientation
                viewer_pose.rotation.into()
            },
        }
    }

    /// Gets pose represented by this space
    ///
    /// Does not apply originOffset, use get_pose on XRReferenceSpace instead
    pub fn get_unoffset_pose(&self, viewer_pose: &WebVRFrameData) -> RigidTransform3D<f64> {
        // XXXManishearth add floor-level transform for floor-level and disable position in position-disabled
        match self.ty {
            XRStationaryReferenceSpaceSubtype::Eye_level => {
                // The eye-level pose is basically whatever the headset pose was at t=0, which
                // for most devices is (0, 0, 0)
                RigidTransform3D::identity()
            },
            XRStationaryReferenceSpaceSubtype::Floor_level => {
                // XXXManishearth support getting floor info from stage parameters

                // Assume approximate height of 2m
                // the floor-level space is 2m below the eye-level space, which is (0, 0, 0)
                Vector3D::new(0., -2., 0.).into()
            },
            XRStationaryReferenceSpaceSubtype::Position_disabled => {
                // This space follows the user around, but does not mirror the user's orientation
                let viewer_pose = XRSpace::viewer_pose_from_frame_data(viewer_pose);
                viewer_pose.translation.into()
            },
        }
    }
}
