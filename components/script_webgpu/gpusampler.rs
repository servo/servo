/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUSamplerDescriptor, GPUSamplerMethods, GPUSamplerWrap,
};
use script_bindings::reflector::{DomGlobalGeneric, Reflector, reflect_dom_object_with_wrap};
use webgpu_traits::{WebGPU, WebGPUDevice, WebGPURequest, WebGPUSampler};
use wgpu_core::resource::SamplerDescriptor;

use crate::JSTraceable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::gpuconvert::WebGPUConvert;
use crate::traits::{Equivalence, GPUDeviceTrait, WebGPUGlobalTrait};

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPUSampler {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    sampler: WebGPUSampler,
}

impl Drop for DroppableGPUSampler {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropSampler(self.sampler.0))
        {
            warn!("Failed to send DropSampler ({:?}) ({})", self.sampler.0, e);
        }
    }
}

#[dom_struct]
pub struct GPUSampler<D: DomTypes> {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    #[no_trace]
    device: WebGPUDevice,
    compare_enable: bool,
    dropppable: DroppableGPUSampler,
    #[no_trace = "PhantomData does not exist"]
    phantom: PhantomData<D>,
}

impl<D: Equivalence> GPUSampler<D> {
    fn new_inherited(
        channel: WebGPU,
        device: WebGPUDevice,
        compare_enable: bool,
        sampler: WebGPUSampler,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            device,
            compare_enable,
            dropppable: DroppableGPUSampler { channel, sampler },
            phantom: PhantomData,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        channel: WebGPU,
        device: WebGPUDevice,
        compare_enable: bool,
        sampler: WebGPUSampler,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPUSampler::new_inherited(
                channel,
                device,
                compare_enable,
                sampler,
                label,
            )),
            global,
            cx,
            GPUSamplerWrap::<D>,
        )
    }
}

impl<D> GPUSampler<D>
where
    D: Equivalence,
    D::GlobalScope: WebGPUGlobalTrait,
    D::GPUDevice: GPUDeviceTrait<D>,
{
    pub(crate) fn id(&self) -> WebGPUSampler {
        self.dropppable.sampler
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createsampler>
    pub fn create(
        cx: &mut JSContext,
        device: &D::GPUDevice,
        descriptor: &GPUSamplerDescriptor,
    ) -> DomRoot<GPUSampler<D>> {
        let sampler_id = device
            .global_from_reflector()
            .global_wgpu_id_hub()
            .create_sampler_id();
        let compare_enable = descriptor.compare.is_some();
        let desc = SamplerDescriptor {
            label: (&descriptor.parent).convert(),
            address_modes: [
                descriptor.addressModeU.convert(),
                descriptor.addressModeV.convert(),
                descriptor.addressModeW.convert(),
            ],
            mag_filter: descriptor.magFilter.convert(),
            min_filter: descriptor.minFilter.convert(),
            mipmap_filter: descriptor.mipmapFilter.convert(),
            lod_min_clamp: *descriptor.lodMinClamp,
            lod_max_clamp: *descriptor.lodMaxClamp,
            compare: descriptor.compare.map(WebGPUConvert::convert),
            anisotropy_clamp: 1,
            border_color: None,
        };

        device
            .channel()
            .0
            .send(WebGPURequest::CreateSampler {
                device_id: device.id().0,
                sampler_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU sampler");

        let sampler = WebGPUSampler(sampler_id);

        GPUSampler::new(
            cx,
            &*device.global_from_reflector(),
            device.channel(),
            device.id(),
            compare_enable,
            sampler,
            descriptor.parent.label.clone(),
        )
    }
}

impl<D: DomTypes> GPUSamplerMethods<D> for GPUSampler<D> {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
    }
}
