/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use euclid::RigidTransform3D;
use webxr_api::{self, Frame, Space};

use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding::{
    XRReferenceSpaceMethods, XRReferenceSpaceType,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::{cast_transform, ApiPose, BaseTransform, XRSession};
use crate::dom::xrspace::XRSpace;

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
            Box::new(XRReferenceSpace::new_inherited(session, offset, ty)),
            global,
        )
    }

    pub fn space(&self) -> Space {
        let base = match self.ty {
            XRReferenceSpaceType::Local => webxr_api::BaseSpace::Local,
            XRReferenceSpaceType::Viewer => webxr_api::BaseSpace::Viewer,
            XRReferenceSpaceType::Local_floor => webxr_api::BaseSpace::Floor,
            _ => panic!("unsupported reference space found"),
        };
        let offset = self.offset.transform();
        Space { base, offset }
    }
}

impl XRReferenceSpaceMethods for XRReferenceSpace {
    /// <https://immersive-web.github.io/webxr/#dom-xrreferencespace-getoffsetreferencespace>
    fn GetOffsetReferenceSpace(&self, new: &XRRigidTransform) -> DomRoot<Self> {
        let offset = new.transform().then(&self.offset.transform());
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
    /// Get a transform that can be used to locate the base space
    ///
    /// This is equivalent to `get_pose(self).inverse()` (in column vector notation),
    /// but with better types
    pub fn get_base_transform(&self, base_pose: &Frame) -> Option<BaseTransform> {
        let pose = self.get_pose(base_pose)?;
        Some(pose.inverse().cast_unit())
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
        Some(offset.then(&pose))
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
            XRReferenceSpaceType::Viewer => {
                Some(cast_transform(base_pose.pose.as_ref()?.transform))
            },
            _ => unimplemented!(),
        }
    }
}
