/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::DOMPointBinding::DOMPointInit;
use crate::dom::bindings::codegen::Bindings::XRRigidTransformBinding;
use crate::dom::bindings::codegen::Bindings::XRRigidTransformBinding::XRRigidTransformMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::dompointreadonly::DOMPointReadOnly;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use euclid::{RigidTransform3D, Rotation3D, Vector3D};

#[dom_struct]
pub struct XRRigidTransform {
    reflector_: Reflector,
    position: MutNullableDom<DOMPointReadOnly>,
    orientation: MutNullableDom<DOMPointReadOnly>,
    #[ignore_malloc_size_of = "defined in euclid"]
    transform: RigidTransform3D<f64>,
    inverse: MutNullableDom<XRRigidTransform>,
}

impl XRRigidTransform {
    fn new_inherited(transform: RigidTransform3D<f64>) -> XRRigidTransform {
        XRRigidTransform {
            reflector_: Reflector::new(),
            position: MutNullableDom::default(),
            orientation: MutNullableDom::default(),
            transform,
            inverse: MutNullableDom::default(),
        }
    }

    pub fn new(
        global: &GlobalScope,
        transform: RigidTransform3D<f64>,
    ) -> DomRoot<XRRigidTransform> {
        reflect_dom_object(
            Box::new(XRRigidTransform::new_inherited(transform)),
            global,
            XRRigidTransformBinding::Wrap,
        )
    }

    pub fn identity(window: &GlobalScope) -> DomRoot<XRRigidTransform> {
        let transform = RigidTransform3D::identity();
        XRRigidTransform::new(window, transform)
    }

    // https://immersive-web.github.io/webxr/#dom-xrrigidtransform-xrrigidtransform
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

        let translate = Vector3D::new(position.x as f64, position.y as f64, position.z as f64);
        let rotate = Rotation3D::unit_quaternion(
            orientation.x as f64,
            orientation.y as f64,
            orientation.z as f64,
            orientation.w as f64,
        );
        let transform = RigidTransform3D::new(rotate, translate);
        Ok(XRRigidTransform::new(&window.global(), transform))
    }
}

impl XRRigidTransformMethods for XRRigidTransform {
    // https://immersive-web.github.io/webxr/#dom-xrrigidtransform-position
    fn Position(&self) -> DomRoot<DOMPointReadOnly> {
        self.position.or_init(|| {
            let t = &self.transform.translation;
            DOMPointReadOnly::new(&self.global(), t.x, t.y, t.z, 1.0)
        })
    }
    // https://immersive-web.github.io/webxr/#dom-xrrigidtransform-orientation
    fn Orientation(&self) -> DomRoot<DOMPointReadOnly> {
        self.position.or_init(|| {
            let r = &self.transform.rotation;
            DOMPointReadOnly::new(&self.global(), r.i, r.j, r.k, r.r)
        })
    }
    // https://immersive-web.github.io/webxr/#dom-xrrigidtransform-inverse
    fn Inverse(&self) -> DomRoot<XRRigidTransform> {
        self.inverse.or_init(|| {
            XRRigidTransform::new(&self.global(), self.transform.inverse())
        })
    }
}

impl XRRigidTransform {
    /// https://immersive-web.github.io/webxr/#dom-xrpose-transform
    pub fn transform(&self) -> RigidTransform3D<f64> {
        self.transform
    }
}
