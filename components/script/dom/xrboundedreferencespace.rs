/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsval::JSVal;

use crate::dom::bindings::codegen::Bindings::XRBoundedReferenceSpaceBinding::XRBoundedReferenceSpaceMethods;
use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding::XRReferenceSpaceType;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::dompointreadonly::DOMPointReadOnly;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::XRSession;
use crate::script_runtime::JSContext;

#[dom_struct]
pub struct XRBoundedReferenceSpace {
    xrspace: XRReferenceSpace,
    offset: Dom<XRRigidTransform>,
}

impl XRBoundedReferenceSpace {
    pub fn new_inherited(
        session: &XRSession,
        offset: &XRRigidTransform,
    ) -> XRBoundedReferenceSpace {
        XRBoundedReferenceSpace {
            xrspace: XRReferenceSpace::new_inherited(
                session,
                offset,
                XRReferenceSpaceType::Bounded_floor,
            ),
            offset: Dom::from_ref(offset),
        }
    }

    #[allow(unused)]
    pub fn new(global: &GlobalScope, session: &XRSession) -> DomRoot<XRBoundedReferenceSpace> {
        let offset = XRRigidTransform::identity(global);
        Self::new_offset(global, session, &offset)
    }

    #[allow(unused)]
    pub fn new_offset(
        global: &GlobalScope,
        session: &XRSession,
        offset: &XRRigidTransform,
    ) -> DomRoot<XRBoundedReferenceSpace> {
        reflect_dom_object(
            Box::new(XRBoundedReferenceSpace::new_inherited(session, offset)),
            global,
        )
    }
}

impl XRBoundedReferenceSpaceMethods for XRBoundedReferenceSpace {
    /// <https://www.w3.org/TR/webxr/#dom-xrboundedreferencespace-boundsgeometry>
    fn BoundsGeometry(&self, cx: JSContext) -> JSVal {
        if let Some(bounds) = self.xrspace.get_bounds() {
            let point1 = DOMPointReadOnly::new(
                &self.global(),
                bounds.min.x.into(),
                0.0,
                bounds.min.y.into(),
                1.0,
            );
            let point2 = DOMPointReadOnly::new(
                &self.global(),
                bounds.min.x.into(),
                0.0,
                bounds.max.y.into(),
                1.0,
            );
            let point3 = DOMPointReadOnly::new(
                &self.global(),
                bounds.max.x.into(),
                0.0,
                bounds.max.y.into(),
                1.0,
            );
            let point4 = DOMPointReadOnly::new(
                &self.global(),
                bounds.max.x.into(),
                0.0,
                bounds.min.y.into(),
                1.0,
            );

            to_frozen_array(&[point1, point2, point3, point4], cx)
        } else {
            to_frozen_array::<DomRoot<DOMPointReadOnly>>(&[], cx)
        }
    }
}
