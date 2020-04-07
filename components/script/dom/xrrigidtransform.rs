/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::DOMPointBinding::DOMPointInit;
use crate::dom::bindings::codegen::Bindings::XRRigidTransformBinding::XRRigidTransformMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::utils::create_typed_array;
use crate::dom::dompointreadonly::DOMPointReadOnly;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::dom::xrsession::ApiRigidTransform;
use crate::script_runtime::JSContext;
use dom_struct::dom_struct;
use euclid::{RigidTransform3D, Rotation3D, Vector3D};
use js::jsapi::{Heap, JSObject};
use std::ptr::NonNull;

#[dom_struct]
pub struct XRRigidTransform {
    reflector_: Reflector,
    position: MutNullableDom<DOMPointReadOnly>,
    orientation: MutNullableDom<DOMPointReadOnly>,
    #[ignore_malloc_size_of = "defined in euclid"]
    transform: ApiRigidTransform,
    inverse: MutNullableDom<XRRigidTransform>,
    #[ignore_malloc_size_of = "defined in mozjs"]
    matrix: Heap<*mut JSObject>,
}

impl XRRigidTransform {
    fn new_inherited(transform: ApiRigidTransform) -> XRRigidTransform {
        XRRigidTransform {
            reflector_: Reflector::new(),
            position: MutNullableDom::default(),
            orientation: MutNullableDom::default(),
            transform,
            inverse: MutNullableDom::default(),
            matrix: Heap::default(),
        }
    }

    pub fn new(global: &GlobalScope, transform: ApiRigidTransform) -> DomRoot<XRRigidTransform> {
        reflect_dom_object(Box::new(XRRigidTransform::new_inherited(transform)), global)
    }

    pub fn identity(window: &GlobalScope) -> DomRoot<XRRigidTransform> {
        let transform = RigidTransform3D::identity();
        XRRigidTransform::new(window, transform)
    }

    // https://immersive-web.github.io/webxr/#dom-xrrigidtransform-xrrigidtransform
    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        position: &DOMPointInit,
        orientation: &DOMPointInit,
    ) -> Fallible<DomRoot<Self>> {
        if position.w != 1.0 {
            return Err(Error::Type(format!(
                "XRRigidTransform must be constructed with a position that has a w value of of 1.0, not {}",
                position.w
            )));
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
            return Err(Error::InvalidState);
        }
        let transform = RigidTransform3D::new(rotate, translate);
        Ok(XRRigidTransform::new(&window.global(), transform))
    }
}

impl XRRigidTransformMethods for XRRigidTransform {
    // https://immersive-web.github.io/webxr/#dom-xrrigidtransform-position
    fn Position(&self) -> DomRoot<DOMPointReadOnly> {
        self.position.or_init(|| {
            let t = &self.transform.translation;
            DOMPointReadOnly::new(&self.global(), t.x.into(), t.y.into(), t.z.into(), 1.0)
        })
    }
    // https://immersive-web.github.io/webxr/#dom-xrrigidtransform-orientation
    fn Orientation(&self) -> DomRoot<DOMPointReadOnly> {
        self.orientation.or_init(|| {
            let r = &self.transform.rotation;
            DOMPointReadOnly::new(
                &self.global(),
                r.i.into(),
                r.j.into(),
                r.k.into(),
                r.r.into(),
            )
        })
    }
    // https://immersive-web.github.io/webxr/#dom-xrrigidtransform-inverse
    fn Inverse(&self) -> DomRoot<XRRigidTransform> {
        self.inverse.or_init(|| {
            let transform = XRRigidTransform::new(&self.global(), self.transform.inverse());
            transform.inverse.set(Some(self));
            transform
        })
    }
    // https://immersive-web.github.io/webxr/#dom-xrrigidtransform-matrix
    fn Matrix(&self, _cx: JSContext) -> NonNull<JSObject> {
        if self.matrix.get().is_null() {
            let cx = self.global().get_cx();
            // According to the spec all matrices are column-major,
            // however euclid uses row vectors so we use .to_row_major_array()
            let arr = self.transform.to_transform().to_row_major_array();
            create_typed_array(cx, &arr, &self.matrix);
        }
        NonNull::new(self.matrix.get()).unwrap()
    }
}

impl XRRigidTransform {
    /// https://immersive-web.github.io/webxr/#dom-xrpose-transform
    pub fn transform(&self) -> ApiRigidTransform {
        self.transform
    }
}
