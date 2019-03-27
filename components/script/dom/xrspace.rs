/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRSpaceBinding;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrsession::XRSession;
use dom_struct::dom_struct;
use euclid::Transform3D;
use webvr_traits::WebVRFrameData;

#[dom_struct]
pub struct XRSpace {
    eventtarget: EventTarget,
    session: Dom<XRSession>,
}

impl XRSpace {
    pub fn new_inherited(session: &XRSession) -> XRSpace {
        XRSpace {
            eventtarget: EventTarget::new_inherited(),
            session: Dom::from_ref(session),
        }
    }

    #[allow(unused)]
    pub fn new(global: &GlobalScope, session: &XRSession) -> DomRoot<XRSpace> {
        reflect_dom_object(
            Box::new(XRSpace::new_inherited(session)),
            global,
            XRSpaceBinding::Wrap,
        )
    }
}

impl XRSpace {
    /// Gets viewer pose represented by this space
    #[allow(unused)]
    pub fn get_viewer_pose(&self, base_pose: &WebVRFrameData) -> Transform3D<f64> {
        if let Some(reference) = self.downcast::<XRReferenceSpace>() {
            reference.get_viewer_pose(base_pose)
        } else {
            unreachable!()
        }
    }

    /// Gets pose represented by this space
    ///
    /// Does not apply originOffset, use get_viewer_pose instead if you need it
    #[allow(unused)]
    pub fn get_pose(&self, base_pose: &WebVRFrameData) -> Transform3D<f64> {
        if let Some(reference) = self.downcast::<XRReferenceSpace>() {
            reference.get_pose(base_pose)
        } else {
            unreachable!()
        }
    }
}
