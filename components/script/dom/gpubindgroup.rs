/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use dom_struct::dom_struct;
use webgpu::wgc::binding_model::BindGroupDescriptor;
use webgpu::{WebGPU, WebGPUBindGroup, WebGPUDevice, WebGPURequest};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUBindGroupDescriptor, GPUBindGroupMethods,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubindgrouplayout::GPUBindGroupLayout;
use crate::dom::gpudevice::GPUDevice;

#[dom_struct]
pub struct GPUBindGroup {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[no_trace]
    bind_group: WebGPUBindGroup,
    #[no_trace]
    device: WebGPUDevice,
    layout: Dom<GPUBindGroupLayout>,
}

impl GPUBindGroup {
    fn new_inherited(
        channel: WebGPU,
        bind_group: WebGPUBindGroup,
        device: WebGPUDevice,
        layout: &GPUBindGroupLayout,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            label: DomRefCell::new(label),
            bind_group,
            device,
            layout: Dom::from_ref(layout),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        bind_group: WebGPUBindGroup,
        device: WebGPUDevice,
        layout: &GPUBindGroupLayout,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUBindGroup::new_inherited(
                channel, bind_group, device, layout, label,
            )),
            global,
        )
    }
}

impl GPUBindGroup {
    pub fn id(&self) -> &WebGPUBindGroup {
        &self.bind_group
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbindgroup>
    pub fn create(
        device: &GPUDevice,
        descriptor: &GPUBindGroupDescriptor,
    ) -> DomRoot<GPUBindGroup> {
        let entries = descriptor
            .entries
            .iter()
            .map(|bind| bind.into())
            .collect::<Vec<_>>();

        let desc = BindGroupDescriptor {
            label: (&descriptor.parent).into(),
            layout: descriptor.layout.id().0,
            entries: Cow::Owned(entries),
        };

        let bind_group_id = device.global().wgpu_id_hub().create_bind_group_id();
        device
            .channel()
            .0
            .send(WebGPURequest::CreateBindGroup {
                device_id: device.id().0,
                bind_group_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU BindGroup");

        let bind_group = WebGPUBindGroup(bind_group_id);

        GPUBindGroup::new(
            &device.global(),
            device.channel().clone(),
            bind_group,
            device.id(),
            &descriptor.layout,
            descriptor.parent.label.clone(),
        )
    }
}

impl Drop for GPUBindGroup {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropBindGroup(self.bind_group.0))
        {
            warn!(
                "Failed to send WebGPURequest::DropBindGroup({:?}) ({})",
                self.bind_group.0, e
            );
        };
    }
}

impl GPUBindGroupMethods for GPUBindGroup {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }
}
