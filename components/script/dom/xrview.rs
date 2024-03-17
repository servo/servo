/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use euclid::RigidTransform3D;
use js::typedarray::{Float32, Float32Array};
use webxr_api::{ApiSpace, View};

use super::bindings::buffer_source::HeapBufferSource;
use crate::dom::bindings::codegen::Bindings::XRViewBinding::{XREye, XRViewMethods};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::{cast_transform, BaseSpace, BaseTransform, XRSession};
use crate::script_runtime::JSContext;

#[dom_struct]
pub struct XRView {
    reflector_: Reflector,
    session: Dom<XRSession>,
    eye: XREye,
    viewport_index: usize,
    #[ignore_malloc_size_of = "mozjs"]
    proj: HeapBufferSource<Float32>,
    #[ignore_malloc_size_of = "defined in rust-webxr"]
    #[no_trace]
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
            proj: HeapBufferSource::default(),
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
        let transform: RigidTransform3D<f32, V, BaseSpace> = view.transform.then(to_base);
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
    /// <https://immersive-web.github.io/webxr/#dom-xrview-eye>
    fn Eye(&self) -> XREye {
        self.eye
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrview-projectionmatrix>
    fn ProjectionMatrix(&self, _cx: JSContext) -> Float32Array {
        if !self.proj.is_initialized() {
            let cx = GlobalScope::get_cx();
            // row_major since euclid uses row vectors
            let proj = self.view.projection.to_array();
            self.proj
                .set_data(cx, &proj)
                .expect("Failed to set projection matrix.")
        }
        self.proj
            .get_buffer()
            .expect("Failed to get projection matrix.")
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrview-transform>
    fn Transform(&self) -> DomRoot<XRRigidTransform> {
        DomRoot::from_ref(&self.transform)
    }
}
