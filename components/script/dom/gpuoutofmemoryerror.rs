/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct GPUOutOfMemoryError {
    reflector_: Reflector,
}

impl GPUOutOfMemoryError {
    fn new_inherited() -> Self {
        Self {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<Self> {
        Self::new_with_proto(global, None)
    }

    fn new_with_proto(global: &GlobalScope, proto: Option<HandleObject>) -> DomRoot<Self> {
        reflect_dom_object_with_proto(
            Box::new(GPUOutOfMemoryError::new_inherited()),
            global,
            proto,
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuoutofmemoryerror-gpuoutofmemoryerror
    #[allow(non_snake_case)]
    pub fn Constructor(global: &GlobalScope, proto: Option<HandleObject>) -> DomRoot<Self> {
        GPUOutOfMemoryError::new_with_proto(global, proto)
    }
}
