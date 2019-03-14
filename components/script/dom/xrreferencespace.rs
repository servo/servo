/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding;
use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding::XRReferenceSpaceMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{DomRoot, MutDom};
use crate::dom::dompointreadonly::DOMPointReadOnly;
use crate::dom::window::Window;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::XRSession;
use crate::dom::xrspace::XRSpace;
use dom_struct::dom_struct;

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
    pub fn new(
        global: &Window,
        session: &XRSession,
        position: &DOMPointReadOnly,
        orientation: &DOMPointReadOnly,
    ) -> DomRoot<XRReferenceSpace> {
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
