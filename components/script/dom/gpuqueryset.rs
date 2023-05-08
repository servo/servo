/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::reflector::Reflector;
use dom_struct::dom_struct;

use webgpu::{
    identity::WebGPUOpResult, wgpu::resource, wgt, WebGPU, WebGPURequest, WebGPUTexture,
    WebGPUTextureView,
};

use super::bindings::codegen::Bindings::GPUQuerySetBinding::GPUQuerySetMethods;
use super::bindings::str::USVString;

#[dom_struct]
pub struct GPUQuerySet {
    reflector_: Reflector,
    // #[ignore_malloc_size_of = "defined in wgpu-types"]
}

// TODO: wgpu does not expose right fields right now
impl GPUQuerySetMethods for GPUQuerySet {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuqueryset-destroy
    fn Destroy(&self) {
        todo!()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<USVString> {
        todo!()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<USVString>) {
        todo!()
    }
}
