/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::reflector::Reflector;
use crate::script_runtime::JSContext;
use dom_struct::dom_struct;

use js::jsval::JSVal;
use webgpu::{
    identity::WebGPUOpResult, wgpu::resource, wgt, WebGPU, WebGPURequest, WebGPUTexture,
    WebGPUTextureView,
};

use super::bindings::codegen::Bindings::GPUCompilationInfoBinding::GPUCompilationInfoMethods;
use super::types::GPUCompilationMessage;

#[dom_struct]
pub struct GPUCompilationInfo {
    reflector_: Reflector,
    // #[ignore_malloc_size_of = "defined in wgpu-types"]
    //msg: GPUCompilationMessage,
}

// TODO: wgpu does not expose right fields right now
impl GPUCompilationInfoMethods for GPUCompilationInfo {
    /// https://gpuweb.github.io/gpuweb/#dom-gpucompilationinfo-messages
    fn Messages(&self, cx: JSContext) -> JSVal {
        todo!()
    }
}
