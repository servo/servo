/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use webgpu_traits::{WebGPU, WebGPUBindGroup, WebGPUDevice, WebGPURequest};
use wgpu_core::binding_model::BindGroupDescriptor;

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUBindGroupDescriptor, GPUBindGroupMethods,
};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpuconvert::convert_bind_group_entry;
use crate::dom::webgpu::gpubindgrouplayout::GPUBindGroupLayout;
use crate::dom::webgpu::gpudevice::GPUDevice;

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPUBindGroup {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    bind_group: WebGPUBindGroup,
}

impl Drop for DroppableGPUBindGroup {
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

#[dom_struct]
pub(crate) struct GPUBindGroup {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    #[no_trace]
    device: WebGPUDevice,
    layout: Dom<GPUBindGroupLayout>,
    droppable: DroppableGPUBindGroup,
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
            label: DomRefCell::new(label),
            device,
            layout: Dom::from_ref(layout),
            droppable: DroppableGPUBindGroup {
                channel,
                bind_group,
            },
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        channel: WebGPU,
        bind_group: WebGPUBindGroup,
        device: WebGPUDevice,
        layout: &GPUBindGroupLayout,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(GPUBindGroup::new_inherited(
                channel, bind_group, device, layout, label,
            )),
            global,
            cx,
        )
    }
}

impl GPUBindGroup {
    pub(crate) fn id(&self) -> &WebGPUBindGroup {
        &self.droppable.bind_group
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbindgroup>
    pub(crate) fn create(
        cx: &mut JSContext,
        device: &GPUDevice,
        descriptor: &GPUBindGroupDescriptor,
    ) -> DomRoot<GPUBindGroup> {
        let entries = descriptor
            .entries
            .iter()
            .map(|bind| convert_bind_group_entry(cx, bind))
            .collect::<Vec<_>>();

        let desc = BindGroupDescriptor {
            label: (&descriptor.parent).convert(),
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
            cx,
            &device.global(),
            device.channel(),
            bind_group,
            device.id(),
            &descriptor.layout,
            descriptor.parent.label.clone(),
        )
    }
}

impl GPUBindGroupMethods<crate::DomTypeHolder> for GPUBindGroup {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }
}
