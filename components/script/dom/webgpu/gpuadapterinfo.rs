/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUAdapterInfoMethods;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUAdapterInfo {
    reflector_: Reflector,
    vendor: DOMString,
    architecture: DOMString,
    device: DOMString,
    description: DOMString,
    subgroup_min_size: u32,
    subgroup_max_size: u32,
    is_fallback_adapter: bool,
}

impl GPUAdapterInfo {
    fn new_inherited(
        vendor: DOMString,
        architecture: DOMString,
        device: DOMString,
        description: DOMString,
        subgroup_min_size: u32,
        subgroup_max_size: u32,
        is_fallback_adapter: bool,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            vendor,
            architecture,
            device,
            description,
            subgroup_min_size,
            subgroup_max_size,
            is_fallback_adapter,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        global: &GlobalScope,
        vendor: DOMString,
        architecture: DOMString,
        device: DOMString,
        description: DOMString,
        subgroup_min_size: u32,
        subgroup_max_size: u32,
        is_fallback_adapter: bool,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited(
                vendor,
                architecture,
                device,
                description,
                subgroup_min_size,
                subgroup_max_size,
                is_fallback_adapter,
            )),
            global,
            can_gc,
        )
    }

    pub(crate) fn clone_from(
        global: &GlobalScope,
        info: &GPUAdapterInfo,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        Self::new(
            global,
            info.vendor.clone(),
            info.architecture.clone(),
            info.device.clone(),
            info.description.clone(),
            info.subgroup_min_size,
            info.subgroup_max_size,
            info.is_fallback_adapter,
            can_gc,
        )
    }
}

impl GPUAdapterInfoMethods<crate::DomTypeHolder> for GPUAdapterInfo {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapterinfo-vendor>
    fn Vendor(&self) -> DOMString {
        self.vendor.clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapterinfo-architecture>
    fn Architecture(&self) -> DOMString {
        self.architecture.clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapterinfo-device>
    fn Device(&self) -> DOMString {
        self.device.clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapterinfo-description>
    fn Description(&self) -> DOMString {
        self.description.clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapterinfo-subgroupminsize>
    fn SubgroupMinSize(&self) -> u32 {
        self.subgroup_min_size
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapterinfo-subgroupmaxsize>
    fn SubgroupMaxSize(&self) -> u32 {
        self.subgroup_max_size
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapterinfo-isfallbackadapter>
    fn IsFallbackAdapter(&self) -> bool {
        self.is_fallback_adapter
    }
}
