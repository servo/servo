/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

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
        reflect_dom_object(Box::new(GPUOutOfMemoryError::new_inherited()), global)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuoutofmemoryerror-gpuoutofmemoryerror
    #[allow(non_snake_case)]
    pub fn Constructor(global: &GlobalScope) -> DomRoot<Self> {
        GPUOutOfMemoryError::new(global)
    }
}
