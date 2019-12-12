/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding;
use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding::XRReferenceSpaceMethods;
use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding::XRReferenceSpaceType;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::{cast_transform, ApiPose, ApiViewerPose, XRSession};
use crate::dom::xrspace::XRSpace;
use dom_struct::dom_struct;
use euclid::RigidTransform3D;
use webxr_api::Frame;

#[dom_struct]
pub struct XRReferenceSpace {
    xrspace: XRSpace,
    offset: Dom<XRRigidTransform>,
    ty: XRReferenceSpaceType,
}

impl XRReferenceSpace {
    pub fn new_inherited(
        session: &XRSession,
        offset: &XRRigidTransform,
        ty: XRReferenceSpaceType,
    ) -> XRReferenceSpace {
        XRReferenceSpace {
            xrspace: XRSpace::new_inherited(session),
            offset: Dom::from_ref(offset),
            ty,
        }
    }

    #[allow(unused)]
    pub fn new(
        global: &GlobalScope,
        session: &XRSession,
        ty: XRReferenceSpaceType,
    ) -> DomRoot<XRReferenceSpace> {
        let offset = XRRigidTransform::identity(global);
        Self::new_offset(global, session, ty, &offset)
    }

    #[allow(unused)]
    pub fn new_offset(
        global: &GlobalScope,
        session: &XRSession,
        ty: XRReferenceSpaceType,
        offset: &XRRigidTransform,
    ) -> DomRoot<XRReferenceSpace> {
        reflect_dom_object(
            Box::new(XRReferenceSpace::new_inherited(session, &offset, ty)),
            global,
            XRReferenceSpaceBinding::Wrap,
        )
    }
}

impl XRReferenceSpaceMethods for XRReferenceSpace {
    /// https://immersive-web.github.io/webxr/#dom-xrreferencespace-getoffsetreferencespace
    fn GetOffsetReferenceSpace(&self, new: &XRRigidTransform) -> DomRoot<Self> {
        let offset = self.offset.transform().pre_transform(&new.transform());
        let offset = XRRigidTransform::new(&self.global(), offset);
        Self::new_offset(
            &self.global(),
            self.upcast::<XRSpace>().session(),
            self.ty,
            &offset,
        )
    }
}

impl XRReferenceSpace {
    /// Gets pose of the viewer with respect to this space
    ///
    /// This is equivalent to `get_pose(self).inverse() * get_pose(viewerSpace)` (in column vector notation),
    /// however we specialize it to be efficient
    pub fn get_viewer_pose(&self, base_pose: &Frame) -> Option<ApiViewerPose> {
        let pose = self.get_unoffset_viewer_pose(base_pose)?;
        // in column-vector notation,
        // get_viewer_pose(space) = get_pose(space).inverse() * get_pose(viewer_space)
        //                        = (get_unoffset_pose(space) * offset).inverse() * get_pose(viewer_space)
        //                        = offset.inverse() * get_unoffset_pose(space).inverse() * get_pose(viewer_space)
        //                        = offset.inverse() * get_unoffset_viewer_pose(space)
        let offset = self.offset.transform();
        let inverse = offset.inverse();
        Some(inverse.pre_transform(&pose))
    }

    /// Gets pose of the viewer with respect to this space
    ///
    /// Does not apply originOffset, use get_viewer_pose instead if you need it
    pub fn get_unoffset_viewer_pose(&self, base_pose: &Frame) -> Option<ApiViewerPose> {
        // all math is in column-vector notation
        // we use the following equation to verify correctness here:
        // get_viewer_pose(space) = get_pose(space).inverse() * get_pose(viewer_space)
        match self.ty {
            XRReferenceSpaceType::Local => {
                // get_viewer_pose(eye_level) = get_pose(eye_level).inverse() * get_pose(viewer_space)
                //                            = I * viewer_pose
                //                            = viewer_pose
                let viewer_pose: ApiViewerPose = cast_transform(base_pose.transform?);

                // we get viewer poses in eye-level space by default
                Some(viewer_pose)
            },
            XRReferenceSpaceType::Local_floor => {
                // get_viewer_pose(floor_level) = get_pose(floor_level).inverse() * get_pose(viewer_space)
                //                            = floor_to_native.inverse() * viewer_pose
                //                            = native_to_floor * viewer_pose
                let viewer_pose = base_pose.transform?;
                let native_to_floor = self
                    .upcast::<XRSpace>()
                    .session()
                    .with_session(|s| s.floor_transform())?;

                Some(cast_transform(native_to_floor.pre_transform(&viewer_pose)))
            },
            XRReferenceSpaceType::Viewer => {
                // This reference space follows the viewer around, so the viewer is
                // always at an identity transform with respect to it
                Some(RigidTransform3D::identity())
            },
            _ => unimplemented!(),
        }
    }

    /// Gets pose represented by this space
    ///
    /// The reference origin used is common between all
    /// get_pose calls for spaces from the same device, so this can be used to compare
    /// with other spaces
    pub fn get_pose(&self, base_pose: &Frame) -> Option<ApiPose> {
        let pose = self.get_unoffset_pose(base_pose)?;
        let offset = self.offset.transform();
        // pose is a transform from the unoffset space to native space,
        // offset is a transform from offset space to unoffset space,
        // we want a transform from unoffset space to native space,
        // which is pose * offset in column vector notation
        Some(pose.pre_transform(&offset))
    }

    /// Gets pose represented by this space
    ///
    /// Does not apply originOffset, use get_viewer_pose instead if you need it
    pub fn get_unoffset_pose(&self, base_pose: &Frame) -> Option<ApiPose> {
        match self.ty {
            XRReferenceSpaceType::Local => {
                // The eye-level pose is basically whatever the headset pose was at t=0, which
                // for most devices is (0, 0, 0)
                Some(RigidTransform3D::identity())
            },
            XRReferenceSpaceType::Local_floor => {
                let native_to_floor = self
                    .upcast::<XRSpace>()
                    .session()
                    .with_session(|s| s.floor_transform())?;
                Some(cast_transform(native_to_floor.inverse()))
            },
            XRReferenceSpaceType::Viewer => base_pose.transform.map(cast_transform),
            _ => unimplemented!(),
        }
    }
}
