/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding;
use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding::XRReferenceSpaceMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{DomRoot, MutDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::XRSession;
use crate::dom::xrspace::XRSpace;
use crate::dom::xrstationaryreferencespace::XRStationaryReferenceSpace;
use dom_struct::dom_struct;
use euclid::RigidTransform3D;
use webvr_traits::WebVRFrameData;

#[dom_struct]
pub struct XRReferenceSpace {
    xrspace: XRSpace,
    transform: MutDom<XRRigidTransform>,
}

impl XRReferenceSpace {
    pub fn new_inherited(session: &XRSession, transform: &XRRigidTransform) -> XRReferenceSpace {
        XRReferenceSpace {
            xrspace: XRSpace::new_inherited(session),
            transform: MutDom::new(transform),
        }
    }

    #[allow(unused)]
    pub fn identity(global: &GlobalScope, session: &XRSession) -> DomRoot<XRReferenceSpace> {
        let transform = XRRigidTransform::identity(global);
        reflect_dom_object(
            Box::new(XRReferenceSpace::new_inherited(session, &transform)),
            global,
            XRReferenceSpaceBinding::Wrap,
        )
    }
}

impl XRReferenceSpaceMethods for XRReferenceSpace {
    /// https://immersive-web.github.io/webxr/#dom-xrreferencespace-originoffset
    fn SetOriginOffset(&self, transform: &XRRigidTransform) {
        self.transform.set(transform);
    }

    /// https://immersive-web.github.io/webxr/#dom-xrreferencespace-originoffset
    fn OriginOffset(&self) -> DomRoot<XRRigidTransform> {
        self.transform.get()
    }
}

impl XRReferenceSpace {
    /// Gets pose of the viewer with respect to this space
    ///
    /// This is equivalent to `get_pose(self).inverse() * get_pose(viewerSpace)` (in column vector notation),
    /// however we specialize it to be efficient
    pub fn get_viewer_pose(&self, base_pose: &WebVRFrameData) -> RigidTransform3D<f64> {
        let pose = self.get_unoffset_viewer_pose(base_pose);

        // This may change, see https://github.com/immersive-web/webxr/issues/567

        // in column-vector notation,
        // get_viewer_pose(space) = get_pose(space).inverse() * get_pose(viewer_space)
        //                        = (get_unoffset_pose(space) * offset).inverse() * get_pose(viewer_space)
        //                        = offset.inverse() * get_unoffset_pose(space).inverse() * get_pose(viewer_space)
        //                        = offset.inverse() * get_unoffset_viewer_pose(space)
        let offset = self.transform.get().transform();
        let inverse = offset.inverse();
        inverse.pre_mul(&pose)
    }

    /// Gets pose of the viewer with respect to this space
    ///
    /// Does not apply originOffset, use get_viewer_pose instead if you need it
    pub fn get_unoffset_viewer_pose(&self, base_pose: &WebVRFrameData) -> RigidTransform3D<f64> {
        if let Some(stationary) = self.downcast::<XRStationaryReferenceSpace>() {
            stationary.get_unoffset_viewer_pose(base_pose)
        } else {
            // non-subclassed XRReferenceSpaces exist, obtained via the "identity"
            // type. These poses are equivalent to the viewer pose and follow the headset
            // around, so the viewer is always at an identity transform with respect to them
            RigidTransform3D::identity()
        }
    }

    /// Gets pose represented by this space
    ///
    /// The reference origin used is common between all
    /// get_pose calls for spaces from the same device, so this can be used to compare
    /// with other spaces
    pub fn get_pose(&self, base_pose: &WebVRFrameData) -> RigidTransform3D<f64> {
        let pose = self.get_unoffset_pose(base_pose);

        // This may change, see https://github.com/immersive-web/webxr/issues/567
        let offset = self.transform.get().transform();
        offset.post_mul(&pose)
    }

    /// Gets pose represented by this space
    ///
    /// Does not apply originOffset, use get_viewer_pose instead if you need it
    pub fn get_unoffset_pose(&self, base_pose: &WebVRFrameData) -> RigidTransform3D<f64> {
        if let Some(stationary) = self.downcast::<XRStationaryReferenceSpace>() {
            stationary.get_unoffset_pose(base_pose)
        } else {
            // non-subclassed XRReferenceSpaces exist, obtained via the "identity"
            // type. These are equivalent to the viewer pose and follow the headset
            // around
            XRSpace::viewer_pose_from_frame_data(base_pose)
        }
    }
}
