/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::DOMPointBinding::DOMPointInit;
use crate::dom::bindings::codegen::Bindings::XRRayBinding::{XRRayDirectionInit, XRRayMethods};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::utils::create_typed_array;
use crate::dom::dompointreadonly::DOMPointReadOnly;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::script_runtime::JSContext;
use dom_struct::dom_struct;
use euclid::{Angle, RigidTransform3D, Rotation3D, Vector3D};
use js::jsapi::{Heap, JSObject};
use std::ptr::NonNull;
use webxr_api::{ApiSpace, Ray};

#[dom_struct]
pub struct XRRay {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webxr"]
    ray: Ray<ApiSpace>,
    #[ignore_malloc_size_of = "defined in mozjs"]
    matrix: Heap<*mut JSObject>,
}

impl XRRay {
    fn new_inherited(ray: Ray<ApiSpace>) -> XRRay {
        XRRay {
            reflector_: Reflector::new(),
            ray,
            matrix: Heap::default(),
        }
    }

    pub fn new(global: &GlobalScope, ray: Ray<ApiSpace>) -> DomRoot<XRRay> {
        reflect_dom_object(Box::new(XRRay::new_inherited(ray)), global)
    }

    #[allow(non_snake_case)]
    /// https://immersive-web.github.io/hit-test/#dom-xrray-xrray
    pub fn Constructor(
        window: &Window,
        origin: &DOMPointInit,
        direction: &XRRayDirectionInit,
    ) -> Fallible<DomRoot<Self>> {
        if origin.w != 1.0 {
            return Err(Error::Type("Origin w coordinate must be 1".into()));
        }
        if *direction.w != 0.0 {
            return Err(Error::Type("Direction w coordinate must be 0".into()));
        }
        if *direction.x == 0.0 && *direction.y == 0.0 && *direction.z == 0.0 {
            return Err(Error::Type(
                "Direction vector cannot have zero length".into(),
            ));
        }

        let origin = Vector3D::new(origin.x as f32, origin.y as f32, origin.z as f32);
        let direction = Vector3D::new(
            *direction.x as f32,
            *direction.y as f32,
            *direction.z as f32,
        )
        .normalize();

        Ok(Self::new(&window.global(), Ray { origin, direction }))
    }

    #[allow(non_snake_case)]
    /// https://immersive-web.github.io/hit-test/#dom-xrray-xrray-transform
    pub fn Constructor_(window: &Window, transform: &XRRigidTransform) -> Fallible<DomRoot<Self>> {
        let transform = transform.transform();
        let origin = transform.translation;
        let direction = transform
            .rotation
            .transform_vector3d(Vector3D::new(0., 0., -1.));

        Ok(Self::new(&window.global(), Ray { origin, direction }))
    }

    pub fn ray(&self) -> Ray<ApiSpace> {
        self.ray
    }
}

impl XRRayMethods for XRRay {
    /// https://immersive-web.github.io/hit-test/#dom-xrray-origin
    fn Origin(&self) -> DomRoot<DOMPointReadOnly> {
        DOMPointReadOnly::new(
            &self.global(),
            self.ray.origin.x as f64,
            self.ray.origin.y as f64,
            self.ray.origin.z as f64,
            1.,
        )
    }

    /// https://immersive-web.github.io/hit-test/#dom-xrray-direction
    fn Direction(&self) -> DomRoot<DOMPointReadOnly> {
        DOMPointReadOnly::new(
            &self.global(),
            self.ray.direction.x as f64,
            self.ray.direction.y as f64,
            self.ray.direction.z as f64,
            0.,
        )
    }

    /// https://immersive-web.github.io/hit-test/#dom-xrray-matrix
    fn Matrix(&self, _cx: JSContext) -> NonNull<JSObject> {
        // https://immersive-web.github.io/hit-test/#xrray-obtain-the-matrix
        // Step 1
        if self.matrix.get().is_null() {
            let cx = self.global().get_cx();
            // Step 2
            let z = Vector3D::new(0., 0., -1.);
            // Step 3
            let axis = z.cross(self.ray.direction);
            // Step 4
            let cos_angle = z.dot(self.ray.direction);
            // Step 5
            let rotation = if cos_angle > -1. && cos_angle < 1. {
                Rotation3D::around_axis(axis, Angle::radians(cos_angle.acos()))
            } else if cos_angle == -1. {
                let axis = Vector3D::new(1., 0., 0.);
                Rotation3D::around_axis(axis, Angle::radians(cos_angle.acos()))
            } else {
                Rotation3D::identity()
            };
            // Step 6
            let translation = self.ray.origin;
            // Step 7
            // According to the spec all matrices are column-major,
            // however euclid uses row vectors so we use .to_row_major_array()
            let arr = RigidTransform3D::new(rotation, translation)
                .to_transform()
                .to_row_major_array();
            create_typed_array(cx, &arr, &self.matrix);
        }
        NonNull::new(self.matrix.get()).unwrap()
    }
}
