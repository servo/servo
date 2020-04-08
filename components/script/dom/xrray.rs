/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
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
