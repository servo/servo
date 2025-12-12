/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use euclid::{RigidTransform3D, Rotation3D, Vector3D};
use js::rust::HandleObject;
use js::typedarray::{Float32, HeapFloat32Array};
use script_bindings::trace::RootedTraceableBox;

use crate::dom::bindings::buffer_source::HeapBufferSource;
use crate::dom::bindings::codegen::Bindings::DOMPointBinding::DOMPointInit;
use crate::dom::bindings::codegen::Bindings::XRRigidTransformBinding::XRRigidTransformMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::dompointreadonly::DOMPointReadOnly;
use crate::dom::window::Window;
use crate::dom::xrsession::ApiRigidTransform;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct XRRigidTransform {
    reflector_: Reflector,
    position: MutNullableDom<DOMPointReadOnly>,
    orientation: MutNullableDom<DOMPointReadOnly>,
    #[ignore_malloc_size_of = "defined in euclid"]
    #[no_trace]
    transform: ApiRigidTransform,
    inverse: MutNullableDom<XRRigidTransform>,
    #[ignore_malloc_size_of = "defined in mozjs"]
    matrix: HeapBufferSource<Float32>,
}

impl XRRigidTransform {
    fn new_inherited(transform: ApiRigidTransform) -> XRRigidTransform {
        XRRigidTransform {
            reflector_: Reflector::new(),
            position: MutNullableDom::default(),
            orientation: MutNullableDom::default(),
            transform,
            inverse: MutNullableDom::default(),
            matrix: HeapBufferSource::default(),
        }
    }

    pub(crate) fn new(
        window: &Window,
        transform: ApiRigidTransform,
        can_gc: CanGc,
    ) -> DomRoot<XRRigidTransform> {
        Self::new_with_proto(window, None, transform, can_gc)
    }

    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        transform: ApiRigidTransform,
        can_gc: CanGc,
    ) -> DomRoot<XRRigidTransform> {
        reflect_dom_object_with_proto(
            Box::new(XRRigidTransform::new_inherited(transform)),
            window,
            proto,
            can_gc,
        )
    }

    pub(crate) fn identity(window: &Window, can_gc: CanGc) -> DomRoot<XRRigidTransform> {
        let transform = RigidTransform3D::identity();
        XRRigidTransform::new(window, transform, can_gc)
    }
}

impl XRRigidTransformMethods<crate::DomTypeHolder> for XRRigidTransform {
    /// <https://immersive-web.github.io/webxr/#dom-xrrigidtransform-xrrigidtransform>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        position: &DOMPointInit,
        orientation: &DOMPointInit,
    ) -> Fallible<DomRoot<Self>> {
        if position.w != 1.0 {
            return Err(Error::Type(format!(
                "XRRigidTransform must be constructed with a position that has a w value of of 1.0, not {}",
                position.w
            )));
        }

        if !position.x.is_finite() ||
            !position.y.is_finite() ||
            !position.z.is_finite() ||
            !position.w.is_finite()
        {
            return Err(Error::Type(
                "Position must not contain non-finite values".into(),
            ));
        }

        if !orientation.x.is_finite() ||
            !orientation.y.is_finite() ||
            !orientation.z.is_finite() ||
            !orientation.w.is_finite()
        {
            return Err(Error::Type(
                "Orientation must not contain non-finite values".into(),
            ));
        }

        let translate = Vector3D::new(position.x as f32, position.y as f32, position.z as f32);
        let rotate = Rotation3D::unit_quaternion(
            orientation.x as f32,
            orientation.y as f32,
            orientation.z as f32,
            orientation.w as f32,
        );

        if !rotate.i.is_finite() {
            // if quaternion has zero norm, we'll get an infinite or NaN
            // value for each element. This is preferable to checking for zero.
            return Err(Error::InvalidState(None));
        }
        let transform = RigidTransform3D::new(rotate, translate);
        Ok(XRRigidTransform::new_with_proto(
            window, proto, transform, can_gc,
        ))
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrrigidtransform-position>
    fn Position(&self, can_gc: CanGc) -> DomRoot<DOMPointReadOnly> {
        self.position.or_init(|| {
            let t = &self.transform.translation;
            DOMPointReadOnly::new(
                &self.global(),
                t.x.into(),
                t.y.into(),
                t.z.into(),
                1.0,
                can_gc,
            )
        })
    }
    /// <https://immersive-web.github.io/webxr/#dom-xrrigidtransform-orientation>
    fn Orientation(&self, can_gc: CanGc) -> DomRoot<DOMPointReadOnly> {
        self.orientation.or_init(|| {
            let r = &self.transform.rotation;
            DOMPointReadOnly::new(
                &self.global(),
                r.i.into(),
                r.j.into(),
                r.k.into(),
                r.r.into(),
                can_gc,
            )
        })
    }
    /// <https://immersive-web.github.io/webxr/#dom-xrrigidtransform-inverse>
    fn Inverse(&self, can_gc: CanGc) -> DomRoot<XRRigidTransform> {
        self.inverse.or_init(|| {
            let transform =
                XRRigidTransform::new(self.global().as_window(), self.transform.inverse(), can_gc);
            transform.inverse.set(Some(self));
            transform
        })
    }
    /// <https://immersive-web.github.io/webxr/#dom-xrrigidtransform-matrix>
    fn Matrix(&self, _cx: JSContext, can_gc: CanGc) -> RootedTraceableBox<HeapFloat32Array> {
        if !self.matrix.is_initialized() {
            self.matrix
                .set_data(_cx, &self.transform.to_transform().to_array(), can_gc)
                .expect("Failed to set on data on transform's internal matrix.")
        }

        self.matrix
            .get_typed_array()
            .expect("Failed to get transform's internal matrix.")
    }
}

impl XRRigidTransform {
    /// <https://immersive-web.github.io/webxr/#dom-xrpose-transform>
    pub(crate) fn transform(&self) -> ApiRigidTransform {
        self.transform
    }
}
