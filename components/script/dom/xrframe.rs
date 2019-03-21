/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRFrameBinding;
use crate::dom::bindings::codegen::Bindings::XRFrameBinding::XRFrameMethods;
use crate::dom::bindings::codegen::Bindings::XRViewBinding::XREye;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrsession::XRSession;
use crate::dom::xrview::XRView;
use crate::dom::xrviewerpose::XRViewerPose;
use dom_struct::dom_struct;
use webvr_traits::WebVRFrameData;

#[dom_struct]
pub struct XRFrame {
    reflector_: Reflector,
    session: Dom<XRSession>,
    #[ignore_malloc_size_of = "defined in rust-webvr"]
    data: WebVRFrameData,
}

impl XRFrame {
    fn new_inherited(session: &XRSession, data: WebVRFrameData) -> XRFrame {
        XRFrame {
            reflector_: Reflector::new(),
            session: Dom::from_ref(session),
            data,
        }
    }

    pub fn new(
        global: &GlobalScope,
        session: &XRSession,
        data: WebVRFrameData,
    ) -> DomRoot<XRFrame> {
        reflect_dom_object(
            Box::new(XRFrame::new_inherited(session, data)),
            global,
            XRFrameBinding::Wrap,
        )
    }
}

impl XRFrameMethods for XRFrame {
    /// https://immersive-web.github.io/webxr/#dom-xrframe-session
    fn Session(&self) -> DomRoot<XRSession> {
        DomRoot::from_ref(&self.session)
    }

    /// https://immersive-web.github.io/webxr/#dom-xrframe-getviewerpose
    fn GetViewerPose(&self, reference: &XRReferenceSpace) -> Option<DomRoot<XRViewerPose>> {
        let pose = reference.get_viewer_pose(&self.data);
        let left = XRView::new(
            &self.global(),
            &self.session,
            XREye::Left,
            &pose,
            &self.data,
        );
        let right = XRView::new(
            &self.global(),
            &self.session,
            XREye::Right,
            &pose,
            &self.data,
        );
        Some(XRViewerPose::new(&self.global(), &left, &right))
    }
}
