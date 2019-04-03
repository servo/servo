/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRViewBinding::XREye;
use crate::dom::bindings::codegen::Bindings::XRViewerPoseBinding;
use crate::dom::bindings::codegen::Bindings::XRViewerPoseBinding::XRViewerPoseMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrpose::XRPose;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::XRSession;
use crate::dom::xrview::XRView;
use dom_struct::dom_struct;
use euclid::RigidTransform3D;
use js::conversions::ToJSValConvertible;
use js::jsapi::{Heap, JSContext};
use js::jsval::{JSVal, UndefinedValue};
use webvr_traits::WebVRFrameData;

#[dom_struct]
pub struct XRViewerPose {
    pose: XRPose,
    views: Heap<JSVal>,
}

impl XRViewerPose {
    fn new_inherited(transform: &XRRigidTransform) -> XRViewerPose {
        XRViewerPose {
            pose: XRPose::new_inherited(transform),
            views: Heap::default(),
        }
    }

    #[allow(unsafe_code)]
    pub fn new(
        global: &GlobalScope,
        session: &XRSession,
        pose: RigidTransform3D<f64>,
        data: &WebVRFrameData,
    ) -> DomRoot<XRViewerPose> {
        let left = XRView::new(global, session, XREye::Left, &pose, &data);
        let right = XRView::new(global, session, XREye::Right, &pose, &data);
        let transform = XRRigidTransform::new(&global.as_window(), pose);
        let pose = reflect_dom_object(
            Box::new(XRViewerPose::new_inherited(&transform)),
            global,
            XRViewerPoseBinding::Wrap,
        );

        unsafe {
            let cx = global.get_cx();
            rooted!(in(cx) let mut jsval = UndefinedValue());
            let vec = vec![left, right];
            vec.to_jsval(cx, jsval.handle_mut());
            pose.views.set(jsval.get());
        }

        pose
    }
}

impl XRViewerPoseMethods for XRViewerPose {
    /// https://immersive-web.github.io/webxr/#dom-xrviewerpose-views
    #[allow(unsafe_code)]
    unsafe fn Views(&self, _cx: *mut JSContext) -> JSVal {
        self.views.get()
    }
}
