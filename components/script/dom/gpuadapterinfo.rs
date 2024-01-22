/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webgpu::wgt::AdapterInfo;

use super::bindings::codegen::Bindings::WebGPUBinding::GPUAdapterInfoMethods;
use super::bindings::reflector::reflect_dom_object;
use super::bindings::root::DomRoot;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::globalscope::GlobalScope;
use crate::test::DOMString;

#[dom_struct]
pub struct GPUAdapterInfo {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in wgpu-types"]
    #[no_trace]
    info: AdapterInfo,
}

impl GPUAdapterInfo {
    fn new_inherited(info: AdapterInfo) -> Self {
        Self {
            reflector_: Reflector::new(),
            info,
        }
    }

    pub fn new(global: &GlobalScope, info: AdapterInfo) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(info)), global)
    }
}

// TODO: wgpu does not expose right fields right now
impl GPUAdapterInfoMethods for GPUAdapterInfo {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapterinfo-vendor>
    fn Vendor(&self) -> DOMString {
        DOMString::new()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapterinfo-architecture>
    fn Architecture(&self) -> DOMString {
        DOMString::new()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapterinfo-device>
    fn Device(&self) -> DOMString {
        DOMString::new()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapterinfo-description>
    fn Description(&self) -> DOMString {
        DOMString::from_string(self.info.driver_info.clone())
    }
}
