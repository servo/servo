/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webgpu::wgt::AdapterInfo;

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUAdapterInfoMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUAdapterInfo {
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

    pub(crate) fn new(global: &GlobalScope, info: AdapterInfo) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(info)), global, CanGc::note())
    }
}

// TODO: wgpu does not expose right fields right now
impl GPUAdapterInfoMethods<crate::DomTypeHolder> for GPUAdapterInfo {
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
