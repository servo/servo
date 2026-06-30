/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;

use dom_struct::dom_struct;
use js::context::JSContext;
use jstraceable_derive::JSTraceable;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUAdapterInfoMethods, GPUAdapterInfoWrap,
};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx_and_wrap};

use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;

#[dom_struct]
pub struct GPUAdapterInfo<D: DomTypes> {
    reflector_: Reflector,
    vendor: DOMString,
    architecture: DOMString,
    device: DOMString,
    description: DOMString,
    subgroup_min_size: u32,
    subgroup_max_size: u32,
    is_fallback_adapter: bool,
    #[no_trace = "Phantom data is not real"]
    phantom: PhantomData<D>,
}

impl<D> GPUAdapterInfo<D>
where
    D: DomTypes<GPUAdapterInfo = GPUAdapterInfo<D>>,
{
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
            phantom: PhantomData,
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        vendor: DOMString,
        architecture: DOMString,
        device: DOMString,
        description: DOMString,
        subgroup_min_size: u32,
        subgroup_max_size: u32,
        is_fallback_adapter: bool,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx_and_wrap::<D, _, _>(
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
            cx,
            GPUAdapterInfoWrap::<D>,
        )
    }

    pub fn clone_from(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        info: &GPUAdapterInfo<D>,
    ) -> DomRoot<Self> {
        Self::new(
            cx,
            global,
            info.vendor.clone(),
            info.architecture.clone(),
            info.device.clone(),
            info.description.clone(),
            info.subgroup_min_size,
            info.subgroup_max_size,
            info.is_fallback_adapter,
        )
    }
}

impl<D: DomTypes> GPUAdapterInfoMethods<D> for GPUAdapterInfo<D> {
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
