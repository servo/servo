/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRRayBinding::XRRayMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::dompointreadonly::DOMPointReadOnly;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use webxr_api::{ApiSpace, Ray};

#[dom_struct]
pub struct XRRay {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webxr"]
    ray: Ray<ApiSpace>,
}

impl XRRay {
    fn new_inherited(ray: Ray<ApiSpace>) -> XRRay {
        XRRay {
            reflector_: Reflector::new(),
            ray,
        }
    }

    pub fn new(global: &GlobalScope, ray: Ray<ApiSpace>) -> DomRoot<XRRay> {
        reflect_dom_object(Box::new(XRRay::new_inherited(ray)), global)
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
}
