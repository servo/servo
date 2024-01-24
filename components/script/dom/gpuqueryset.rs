/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(dead_code)] // this file is stub

use dom_struct::dom_struct;

use super::bindings::codegen::Bindings::WebGPUBinding::GPUQuerySetMethods;
use super::bindings::str::USVString;
use crate::dom::bindings::reflector::Reflector;

#[dom_struct]
pub struct GPUQuerySet {
    reflector_: Reflector,
    // #[ignore_malloc_size_of = "defined in wgpu-types"]
}

// TODO: wgpu does not expose right fields right now
impl GPUQuerySetMethods for GPUQuerySet {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuqueryset-destroy>
    fn Destroy(&self) {
        todo!()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        todo!()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, _value: USVString) {
        todo!()
    }
}
