/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::MutableHandleValue;
use script_bindings::reflector::reflect_dom_object_with_cx;

use crate::dom::bindings::codegen::Bindings::XRBoundedReferenceSpaceBinding::XRBoundedReferenceSpaceMethods;
use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding::XRReferenceSpaceType;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::dompointreadonly::DOMPointReadOnly;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::XRSession;

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

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        session: &XRSession,
    ) -> DomRoot<XRBoundedReferenceSpace> {
        let offset = XRRigidTransform::identity(cx, window);
        let global = window.global();
        Self::new_offset(cx, &global, session, &offset)
    }

    pub(crate) fn new_offset(
        cx: &mut JSContext,
        global: &GlobalScope,
        session: &XRSession,
        offset: &XRRigidTransform,
    ) -> DomRoot<XRBoundedReferenceSpace> {
        reflect_dom_object_with_cx(
            Box::new(XRBoundedReferenceSpace::new_inherited(session, offset)),
            global,
            cx,
        )
    }

    pub(crate) fn reference_space(&self) -> &XRReferenceSpace {
        &self.reference_space
    }
}

impl XRBoundedReferenceSpaceMethods<crate::DomTypeHolder> for XRBoundedReferenceSpace {
    /// <https://www.w3.org/TR/webxr/#dom-xrboundedreferencespace-boundsgeometry>
    fn BoundsGeometry(&self, cx: &mut JSContext, retval: MutableHandleValue) {
        if let Some(bounds) = self.reference_space.get_bounds() {
            let points: Vec<DomRoot<DOMPointReadOnly>> = bounds
                .into_iter()
                .map(|point| {
                    DOMPointReadOnly::new(
                        cx,
                        &self.global(),
                        point.x.into(),
                        0.0,
                        point.y.into(),
                        1.0,
                    )
                })
                .collect();

            to_frozen_array(cx, &points, retval)
        } else {
            to_frozen_array::<DomRoot<DOMPointReadOnly>>(cx, &[], retval)
        }
    }
}
