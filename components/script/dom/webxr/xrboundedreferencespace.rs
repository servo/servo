/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::MutableHandleValue;

use crate::dom::bindings::codegen::Bindings::XRBoundedReferenceSpaceBinding::XRBoundedReferenceSpaceMethods;
use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding::XRReferenceSpaceType;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::dompointreadonly::DOMPointReadOnly;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::XRSession;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct XRBoundedReferenceSpace {
    reference_space: XRReferenceSpace,
    offset: Dom<XRRigidTransform>,
}

impl XRBoundedReferenceSpace {
    pub(crate) fn new_inherited(
        session: &XRSession,
        offset: &XRRigidTransform,
    ) -> XRBoundedReferenceSpace {
        XRBoundedReferenceSpace {
            reference_space: XRReferenceSpace::new_inherited(
                session,
                offset,
                XRReferenceSpaceType::Bounded_floor,
            ),
            offset: Dom::from_ref(offset),
        }
    }

    #[allow(unused)]
    pub(crate) fn new(
        global: &GlobalScope,
        session: &XRSession,
        can_gc: CanGc,
    ) -> DomRoot<XRBoundedReferenceSpace> {
        let offset = XRRigidTransform::identity(global, can_gc);
        Self::new_offset(global, session, &offset, can_gc)
    }

    #[allow(unused)]
    pub(crate) fn new_offset(
        global: &GlobalScope,
        session: &XRSession,
        offset: &XRRigidTransform,
        can_gc: CanGc,
    ) -> DomRoot<XRBoundedReferenceSpace> {
        reflect_dom_object(
            Box::new(XRBoundedReferenceSpace::new_inherited(session, offset)),
            global,
            can_gc,
        )
    }

    pub(crate) fn reference_space(&self) -> &XRReferenceSpace {
        &self.reference_space
    }
}

impl XRBoundedReferenceSpaceMethods<crate::DomTypeHolder> for XRBoundedReferenceSpace {
    /// <https://www.w3.org/TR/webxr/#dom-xrboundedreferencespace-boundsgeometry>
    fn BoundsGeometry(&self, cx: JSContext, can_gc: CanGc, retval: MutableHandleValue) {
        if let Some(bounds) = self.reference_space.get_bounds() {
            let points: Vec<DomRoot<DOMPointReadOnly>> = bounds
                .into_iter()
                .map(|point| {
                    DOMPointReadOnly::new(
                        &self.global(),
                        point.x.into(),
                        0.0,
                        point.y.into(),
                        1.0,
                        can_gc,
                    )
                })
                .collect();

            to_frozen_array(&points, cx, retval)
        } else {
            to_frozen_array::<DomRoot<DOMPointReadOnly>>(&[], cx, retval)
        }
    }
}
