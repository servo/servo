/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use js::cell::JSCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use webgpu_traits::{WebGPU, WebGPUDevice, WebGPURequest, WebGPUSampler};
use wgpu_core::resource::SamplerDescriptor;

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUSamplerDescriptor, GPUSamplerMethods,
};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webgpu::gpudevice::GPUDevice;

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
pub(crate) struct GPUSampler {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "JSCell is hard to measure"]
    label: JSCell<USVString>,
    #[no_trace]
    device: WebGPUDevice,
    compare_enable: bool,
    dropppable: DroppableGPUSampler,
}

impl GPUSampler {
    fn new_inherited(
        channel: WebGPU,
        device: WebGPUDevice,
        compare_enable: bool,
        sampler: WebGPUSampler,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: JSCell::new(label),
            device,
            compare_enable,
            dropppable: DroppableGPUSampler { channel, sampler },
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        channel: WebGPU,
        device: WebGPUDevice,
        compare_enable: bool,
        sampler: WebGPUSampler,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(GPUSampler::new_inherited(
                channel,
                device,
                compare_enable,
                sampler,
                label,
            )),
            global,
            cx,
        )
    }
}

impl GPUSampler {
    pub(crate) fn id(&self) -> WebGPUSampler {
        self.dropppable.sampler
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createsampler>
    pub(crate) fn create(
        cx: &mut JSContext,
        device: &GPUDevice,
        descriptor: &GPUSamplerDescriptor,
    ) -> DomRoot<GPUSampler> {
        let sampler_id = device.global().wgpu_id_hub().create_sampler_id();
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
            compare: descriptor.compare.map(Convert::convert),
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
            &device.global(),
            device.channel(),
            device.id(),
            compare_enable,
            sampler,
            descriptor.parent.label.clone(),
        )
    }
}

impl GPUSamplerMethods<crate::DomTypeHolder> for GPUSampler {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self, no_gc: &NoGC) -> USVString {
        self.label.borrow(no_gc).clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc_mut: &mut NoGC, value: USVString) {
        *self.label.borrow_mut(no_gc_mut) = value;
    }
}
