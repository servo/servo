/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webgpu::wgc::resource::SamplerDescriptor;
use webgpu::{WebGPU, WebGPUDevice, WebGPURequest, WebGPUSampler};

use crate::conversions::Convert;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUSamplerDescriptor, GPUSamplerMethods,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webgpu::gpudevice::GPUDevice;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUSampler {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[no_trace]
    device: WebGPUDevice,
    compare_enable: bool,
    #[no_trace]
    sampler: WebGPUSampler,
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
            channel,
            label: DomRefCell::new(label),
            device,
            sampler,
            compare_enable,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        channel: WebGPU,
        device: WebGPUDevice,
        compare_enable: bool,
        sampler: WebGPUSampler,
        label: USVString,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUSampler::new_inherited(
                channel,
                device,
                compare_enable,
                sampler,
                label,
            )),
            global,
            can_gc,
        )
    }
}

impl GPUSampler {
    pub(crate) fn id(&self) -> WebGPUSampler {
        self.sampler
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createsampler>
    pub(crate) fn create(
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
            &device.global(),
            device.channel().clone(),
            device.id(),
            compare_enable,
            sampler,
            descriptor.parent.label.clone(),
            CanGc::note(),
        )
    }
}

impl GPUSamplerMethods<crate::DomTypeHolder> for GPUSampler {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }
}

impl Drop for GPUSampler {
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
