/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRViewBinding;
use crate::dom::bindings::codegen::Bindings::XRViewBinding::{XREye, XRViewMethods};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::vrframedata::create_typed_array;
use crate::dom::xrsession::XRSession;
use dom_struct::dom_struct;
use euclid::{RigidTransform3D, Vector3D};
use js::jsapi::{Heap, JSContext, JSObject};
use std::ptr::NonNull;
use webvr_traits::WebVRFrameData;

#[dom_struct]
pub struct XRView {
    reflector_: Reflector,
    session: Dom<XRSession>,
    eye: XREye,
    proj: Heap<*mut JSObject>,
    view: Heap<*mut JSObject>,
}

impl XRView {
    fn new_inherited(session: &XRSession, eye: XREye) -> XRView {
        XRView {
            reflector_: Reflector::new(),
            session: Dom::from_ref(session),
            eye,
            proj: Heap::default(),
            view: Heap::default(),
        }
    }

    #[allow(unsafe_code)]
    pub fn new(
        global: &GlobalScope,
        session: &XRSession,
        eye: XREye,
        pose: &RigidTransform3D<f64>,
        data: &WebVRFrameData,
    ) -> DomRoot<XRView> {
        let ret = reflect_dom_object(
            Box::new(XRView::new_inherited(session, eye)),
            global,
            XRViewBinding::Wrap,
        );

        let vr_display = session.display();

        // XXXManishearth compute and cache projection matrices on the Display
        let (proj, offset) = if eye == XREye::Left {
            (
                &data.left_projection_matrix,
                vr_display.left_eye_params_offset(),
            )
        } else {
            (
                &data.right_projection_matrix,
                vr_display.right_eye_params_offset(),
            )
        };

        let offset = Vector3D::new(offset[0] as f64, offset[1] as f64, offset[2] as f64);
        let view = pose.post_mul(&offset.into()).to_transform().cast().to_column_major_array();

        let cx = global.get_cx();
        unsafe {
            create_typed_array(cx, proj, &ret.proj);
            create_typed_array(cx, &view, &ret.view);
        }
        ret
    }

    pub fn session(&self) -> &XRSession {
        &self.session
    }
}

impl XRViewMethods for XRView {
    /// https://immersive-web.github.io/webxr/#dom-xrview-eye
    fn Eye(&self) -> XREye {
        self.eye
    }

    #[allow(unsafe_code)]
    /// https://immersive-web.github.io/webxr/#dom-xrview-projectionmatrix
    unsafe fn ProjectionMatrix(&self, _cx: *mut JSContext) -> NonNull<JSObject> {
        NonNull::new(self.proj.get()).unwrap()
    }

    #[allow(unsafe_code)]
    /// https://immersive-web.github.io/webxr/#dom-xrview-projectionmatrix
    unsafe fn ViewMatrix(&self, _cx: *mut JSContext) -> NonNull<JSObject> {
        NonNull::new(self.view.get()).unwrap()
    }
}
