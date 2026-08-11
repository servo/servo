/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use jstraceable_derive::JSTraceable;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUBindGroupDescriptor, GPUBindGroupMethods, GPUBindGroupWrap,
};
use script_bindings::interfaces::PromiseHelpers;
use script_bindings::reflector::{DomGlobalGeneric, Reflector, reflect_dom_object_with_wrap};
use webgpu_traits::{WebGPU, WebGPUBindGroup, WebGPUDevice, WebGPURequest};
use wgpu_core::binding_model::BindGroupDescriptor;

use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::gpubindgrouplayout::GPUBindGroupLayout;
use crate::gpubuffer::GPUBuffer;
use crate::gpuconvert::{WebGPUConvert, convert_bind_group_entry};
use crate::traits::{
    GPUDeviceTrait, GPUExternalTextureTrait, GPUSamplerTrait, GPUTextureTrait, GPUTextureViewTrait,
    WebGPUGlobalTrait,
};

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
pub struct GPUBindGroup<D: DomTypes> {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    #[no_trace]
    device: WebGPUDevice,
    layout: Dom<GPUBindGroupLayout<D>>,
    droppable: DroppableGPUBindGroup,
}

impl<D> GPUBindGroup<D>
where
    D: DomTypes<GPUBindGroup = GPUBindGroup<D>>,
{
    fn new_inherited(
        channel: WebGPU,
        bind_group: WebGPUBindGroup,
        device: WebGPUDevice,
        layout: &GPUBindGroupLayout<D>,
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
        global: &D::GlobalScope,
        channel: WebGPU,
        bind_group: WebGPUBindGroup,
        device: WebGPUDevice,
        layout: &GPUBindGroupLayout<D>,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPUBindGroup::new_inherited(
                channel, bind_group, device, layout, label,
            )),
            global,
            cx,
            GPUBindGroupWrap::<D>,
        )
    }
}

impl<D> GPUBindGroup<D>
where
    D: DomTypes<
            GPUBuffer = GPUBuffer<D>,
            GPUBindGroup = GPUBindGroup<D>,
            GPUBindGroupLayout = GPUBindGroupLayout<D>,
        >,
    D::GPUDevice: DomGlobalGeneric<D> + GPUDeviceTrait<D>,
    D::GlobalScope: WebGPUGlobalTrait,
    D::GPUExternalTexture: GPUExternalTextureTrait,
    D::GPUSampler: GPUSamplerTrait,
    D::GPUTexture: GPUTextureTrait,
    D::GPUTextureView: GPUTextureViewTrait,
    D::Promise: PromiseHelpers<D>,
{
    pub fn id(&self) -> &WebGPUBindGroup {
        &self.droppable.bind_group
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbindgroup>
    pub fn create(
        cx: &mut JSContext,
        device: &D::GPUDevice,
        descriptor: &GPUBindGroupDescriptor<D>,
    ) -> DomRoot<GPUBindGroup<D>> {
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

        let bind_group_id = <D::GPUDevice as DomGlobalGeneric<D>>::global_from_reflector(device)
            .global_wgpu_id_hub()
            .create_bind_group_id();
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

        let global = <D::GPUDevice as DomGlobalGeneric<D>>::global_from_reflector(device);
        GPUBindGroup::new(
            cx,
            &*global,
            device.channel(),
            bind_group,
            device.id(),
            &descriptor.layout,
            descriptor.parent.label.clone(),
        )
    }
}

impl<D: DomTypes> GPUBindGroupMethods<D> for GPUBindGroup<D> {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
    }
}
