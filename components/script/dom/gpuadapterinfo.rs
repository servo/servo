/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::trace::JSTraceable;
use crate::test::DOMString;
use dom_struct::dom_struct;
use js::jsapi::JSTracer;

use webgpu::{
    identity::WebGPUOpResult, wgpu::resource, wgt, WebGPU, WebGPURequest, WebGPUTexture,
    WebGPUTextureView,
};

use super::bindings::codegen::Bindings::GPUAdapterInfoBinding::GPUAdapterInfoMethods;
pub struct AdapterInfo(wgt::AdapterInfo);

#[allow(unsafe_code)]
unsafe impl JSTraceable for AdapterInfo {
    unsafe fn trace(&self, _: *mut JSTracer) {
        // do nothing
    }
}

#[dom_struct]
pub struct GPUAdapterInfo {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in wgpu-types"]
    info: AdapterInfo,
}

// TODO: wgpu does not expose right fields right now
impl GPUAdapterInfoMethods for GPUAdapterInfo {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuadapterinfo-vendor
    fn Vendor(&self) -> DOMString {
        DOMString::new()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuadapterinfo-architecture
    fn Architecture(&self) -> DOMString {
        DOMString::new()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuadapterinfo-device
    fn Device(&self) -> DOMString {
        DOMString::new()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuadapterinfo-description
    fn Description(&self) -> DOMString {
        DOMString::from_string(self.info.0.driver_info.clone())
    }
}
