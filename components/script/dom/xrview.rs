/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRViewBinding::{XREye, XRViewMethods};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::utils::create_typed_array;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::{cast_transform, BaseSpace, BaseTransform, XRSession};
use crate::script_runtime::JSContext;
use dom_struct::dom_struct;
use euclid::RigidTransform3D;
use js::jsapi::{Heap, JSObject};
use std::ptr::NonNull;
use webxr_api::{ApiSpace, View};

#[dom_struct]
pub struct XRView {
    reflector_: Reflector,
    session: Dom<XRSession>,
    eye: XREye,
    viewport_index: usize,
    #[ignore_malloc_size_of = "mozjs"]
    proj: Heap<*mut JSObject>,
    #[ignore_malloc_size_of = "defined in rust-webxr"]
    view: View<ApiSpace>,
    transform: Dom<XRRigidTransform>,
}

impl XRView {
    fn new_inherited(
        session: &XRSession,
        transform: &XRRigidTransform,
        eye: XREye,
        viewport_index: usize,
        view: View<ApiSpace>,
    ) -> XRView {
        XRView {
            reflector_: Reflector::new(),
            session: Dom::from_ref(session),
            eye,
            viewport_index,
            proj: Heap::default(),
            view,
            transform: Dom::from_ref(transform),
        }
    }

    pub fn new<V: Copy>(
        global: &GlobalScope,
        session: &XRSession,
        view: &View<V>,
        eye: XREye,
        viewport_index: usize,
        to_base: &BaseTransform,
    ) -> DomRoot<XRView> {
        let transform: RigidTransform3D<f32, V, BaseSpace> = to_base.pre_transform(&view.transform);
        let transform = XRRigidTransform::new(global, cast_transform(transform));

        reflect_dom_object(
            Box::new(XRView::new_inherited(
                session,
                &transform,
                eye,
                viewport_index,
                view.cast_unit(),
            )),
            global,
        )
    }

    pub fn session(&self) -> &XRSession {
        &self.session
    }

    pub fn viewport_index(&self) -> usize {
        self.viewport_index
    }
}

impl XRViewMethods for XRView {
    /// https://immersive-web.github.io/webxr/#dom-xrview-eye
    fn Eye(&self) -> XREye {
        self.eye
    }

    /// https://immersive-web.github.io/webxr/#dom-xrview-projectionmatrix
    fn ProjectionMatrix(&self, _cx: JSContext) -> NonNull<JSObject> {
        if self.proj.get().is_null() {
            let cx = self.global().get_cx();
            // row_major since euclid uses row vectors
            let proj = self.view.projection.to_row_major_array();
            create_typed_array(cx, &proj, &self.proj);
        }
        NonNull::new(self.proj.get()).unwrap()
    }

    /// https://immersive-web.github.io/webxr/#dom-xrview-transform
    fn Transform(&self) -> DomRoot<XRRigidTransform> {
        DomRoot::from_ref(&self.transform)
    }
}
