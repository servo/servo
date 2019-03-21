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
