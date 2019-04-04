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
    pub fn get_viewer_pose(&self, base_pose: &WebVRFrameData) -> RigidTransform3D<f64> {
        let pose = self.get_unoffset_viewer_pose(base_pose);

        // This may change, see https://github.com/immersive-web/webxr/issues/567
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
            // type. The pose does not depend on the base pose.
            RigidTransform3D::identity()
        }
    }

    /// Gets pose represented by this space
    ///
    /// The reference origin used is common between all
    /// get_pose calls for spaces from the same device, so this can be used to compare
    /// with other spaces
    pub fn get_pose(&self, _: &WebVRFrameData) -> RigidTransform3D<f64> {
        unimplemented!()
    }
}
