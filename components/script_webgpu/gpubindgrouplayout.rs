/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::marker::PhantomData;

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use jstraceable_derive::JSTraceable;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUBindGroupLayoutDescriptor, GPUBindGroupLayoutMethods, GPUBindGroupLayoutWrap,
};
use script_bindings::interfaces::GlobalScopeHelpers;
use script_bindings::reflector::{DomGlobalGeneric, Reflector, reflect_dom_object_with_wrap};
use webgpu_traits::{WebGPU, WebGPUBindGroupLayout, WebGPURequest};
use wgpu_core::binding_model::BindGroupLayoutDescriptor;

use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::gpuconvert::{WebGPUConvert, convert_bind_group_layout_entry};
use crate::traits::{GPUDeviceTrait, WebGPUGlobalTrait};

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPUBindGroupLayout {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    bind_group_layout: WebGPUBindGroupLayout,
}

impl Drop for DroppableGPUBindGroupLayout {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropBindGroupLayout(self.bind_group_layout.0))
        {
            warn!(
                "Failed to send WebGPURequest::DropBindGroupLayout({:?}) ({})",
                self.bind_group_layout.0, e
            );
        };
    }
}

#[dom_struct]
pub struct GPUBindGroupLayout<D: DomTypes> {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    droppable: DroppableGPUBindGroupLayout,
    #[no_trace = "PhantomData does not exist"]
    phantom: PhantomData<D>,
}

impl<D> GPUBindGroupLayout<D>
where
    D: DomTypes<GPUBindGroupLayout = GPUBindGroupLayout<D>>,
{
    fn new_inherited(
        channel: WebGPU,
        bind_group_layout: WebGPUBindGroupLayout,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            droppable: DroppableGPUBindGroupLayout {
                channel,
                bind_group_layout,
            },
            phantom: PhantomData,
        }
    }

    pub fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        channel: WebGPU,
        bind_group_layout: WebGPUBindGroupLayout,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPUBindGroupLayout::new_inherited(
                channel,
                bind_group_layout,
                label,
            )),
            global,
            cx,
            GPUBindGroupLayoutWrap::<D>,
        )
    }
}

impl<D> GPUBindGroupLayout<D>
where
    D: DomTypes<GPUBindGroupLayout = GPUBindGroupLayout<D>>,
    D::GPUDevice: GPUDeviceTrait<D>,
    D::GlobalScope: WebGPUGlobalTrait + GlobalScopeHelpers<D>,
{
    pub fn id(&self) -> WebGPUBindGroupLayout {
        self.droppable.bind_group_layout
    }

    /// <https://gpuweb.github.io/gpuweb/#GPUDevice-createBindGroupLayout>
    pub fn create(
        cx: &mut JSContext,
        device: &D::GPUDevice,
        descriptor: &GPUBindGroupLayoutDescriptor,
    ) -> Fallible<DomRoot<GPUBindGroupLayout<D>>> {
        let entries = descriptor
            .entries
            .iter()
            .map(|bgle| convert_bind_group_layout_entry::<D>(bgle, device))
            .collect::<Fallible<Result<Vec<_>, _>>>()?;

        let desc = match entries {
            Ok(entries) => Some(BindGroupLayoutDescriptor {
                label: (&descriptor.parent).convert(),
                entries: Cow::Owned(entries),
            }),
            Err(error) => {
                device.dispatch_error(error);
                None
            },
        };

        let global = <D::GPUDevice as DomGlobalGeneric<D>>::global_from_reflector(device);
        let bind_group_layout_id = global.global_wgpu_id_hub().create_bind_group_layout_id();
        device
            .channel()
            .0
            .send(WebGPURequest::CreateBindGroupLayout {
                device_id: device.id().0,
                bind_group_layout_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU BindGroupLayout");

        let bgl = WebGPUBindGroupLayout(bind_group_layout_id);

        Ok(GPUBindGroupLayout::new(
            cx,
            &*global,
            device.channel(),
            bgl,
            descriptor.parent.label.clone(),
        ))
    }
}

impl<D: DomTypes> GPUBindGroupLayoutMethods<D> for GPUBindGroupLayout<D> {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
    }
}
