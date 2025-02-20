/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use euclid::RigidTransform3D;
use js::typedarray::{Float32, Float32Array};
use webxr_api::{ApiSpace, View};

use crate::dom::bindings::buffer_source::HeapBufferSource;
use crate::dom::bindings::codegen::Bindings::XRViewBinding::{XREye, XRViewMethods};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::{cast_transform, BaseSpace, BaseTransform, XRSession};
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct XRView {
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
    requested_viewport_scale: Cell<f64>,
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
            requested_viewport_scale: Cell::new(1.0),
        }
    }

    pub(crate) fn new<V: Copy>(
        global: &GlobalScope,
        session: &XRSession,
        view: &View<V>,
        eye: XREye,
        viewport_index: usize,
        to_base: &BaseTransform,
        can_gc: CanGc,
    ) -> DomRoot<XRView> {
        let transform: RigidTransform3D<f32, V, BaseSpace> = view.transform.then(to_base);
        let transform = XRRigidTransform::new(global, cast_transform(transform), can_gc);

        reflect_dom_object(
            Box::new(XRView::new_inherited(
                session,
                &transform,
                eye,
                viewport_index,
                view.cast_unit(),
            )),
            global,
            can_gc,
        )
    }

    pub(crate) fn session(&self) -> &XRSession {
        &self.session
    }

    pub(crate) fn viewport_index(&self) -> usize {
        self.viewport_index
    }
}

impl XRViewMethods<crate::DomTypeHolder> for XRView {
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

    /// <https://www.w3.org/TR/webxr/#dom-xrview-recommendedviewportscale>
    fn GetRecommendedViewportScale(&self) -> Option<Finite<f64>> {
        // Just return 1.0 since we currently will always use full-sized viewports
        Finite::new(1.0)
    }

    /// <https://www.w3.org/TR/webxr/#dom-xrview-requestviewportscale>
    fn RequestViewportScale(&self, scale: Option<Finite<f64>>) {
        if let Some(scale) = scale {
            if *scale > 0.0 {
                let clamped_scale = scale.clamp(0.0, 1.0);
                self.requested_viewport_scale.set(clamped_scale);
            }
        }
    }

    /// <https://www.w3.org/TR/webxr-ar-module-1/#dom-xrview-isfirstpersonobserver>
    fn IsFirstPersonObserver(&self) -> bool {
        // Servo is not currently supported anywhere that supports this, so return false
        false
    }
}
