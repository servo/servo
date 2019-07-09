/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRViewBinding;
use crate::dom::bindings::codegen::Bindings::XRViewBinding::{XREye, XRViewMethods};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::vrframedata::create_typed_array;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::{cast_transform, ApiRigidTransform, XRSession};
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSContext, JSObject};
use std::ptr::NonNull;
use webxr_api::View;

#[dom_struct]
pub struct XRView {
    reflector_: Reflector,
    session: Dom<XRSession>,
    eye: XREye,
    #[ignore_malloc_size_of = "mozjs"]
    proj: Heap<*mut JSObject>,
    #[ignore_malloc_size_of = "mozjs"]
    view: Heap<*mut JSObject>,
    transform: Dom<XRRigidTransform>,
}

impl XRView {
    fn new_inherited(session: &XRSession, transform: &XRRigidTransform, eye: XREye) -> XRView {
        XRView {
            reflector_: Reflector::new(),
            session: Dom::from_ref(session),
            eye,
            proj: Heap::default(),
            view: Heap::default(),
            transform: Dom::from_ref(transform),
        }
    }

    #[allow(unsafe_code)]
    pub fn new<V: Copy>(
        global: &GlobalScope,
        session: &XRSession,
        view: &View<V>,
        eye: XREye,
        pose: &ApiRigidTransform,
    ) -> DomRoot<XRView> {
        // XXXManishearth compute and cache projection matrices on the Display
        let offset = cast_transform(view.transform);

        let transform = pose.post_mul(&offset.into());
        let transform = XRRigidTransform::new(global, transform);

        let ret = reflect_dom_object(
            Box::new(XRView::new_inherited(session, &transform, eye)),
            global,
            XRViewBinding::Wrap,
        );

        // row_major since euclid uses row vectors
        let proj = view.projection.to_row_major_array();
        let cx = global.get_cx();
        unsafe {
            create_typed_array(cx, &proj, &ret.proj);
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

    /// https://immersive-web.github.io/webxr/#dom-xrview-transform
    fn Transform(&self) -> DomRoot<XRRigidTransform> {
        DomRoot::from_ref(&self.transform)
    }
}
