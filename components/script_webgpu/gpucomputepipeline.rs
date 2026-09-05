/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUComputePipelineDescriptor, GPUComputePipelineMethods, GPUComputePipelineWrap,
};
use script_bindings::interfaces::{GlobalScopeHelpers, PromiseHelpers};
use script_bindings::reflector::{DomGlobalGeneric, Reflector, reflect_dom_object_with_wrap};
use servo_base::generic_channel::GenericCallback;
use webgpu_traits::{
    WebGPU, WebGPUBindGroupLayout, WebGPUComputePipeline, WebGPUComputePipelineResponse,
    WebGPURequest,
};
use wgpu_core::pipeline::ComputePipelineDescriptor;

use crate::JSTraceable;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::gpubindgrouplayout::GPUBindGroupLayout;
use crate::gpuconvert::WebGPUConvert;
use crate::traits::{Equivalence, GPUDeviceTrait, WebGPUGlobalTrait, WebGPUPromiseTrait};

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPUComputePipeline {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    compute_pipeline: WebGPUComputePipeline,
}

impl Drop for DroppableGPUComputePipeline {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropComputePipeline(self.compute_pipeline.0))
        {
            warn!(
                "Failed to send WebGPURequest::DropComputePipeline({:?}) ({})",
                self.compute_pipeline.0, e
            );
        };
    }
}

#[dom_struct]
pub struct GPUComputePipeline<D: DomTypes> {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    device: Dom<D::GPUDevice>,
    droppable: DroppableGPUComputePipeline,
}

impl<D> GPUComputePipeline<D>
where
    D: Equivalence,
    D::GPUDevice: GPUDeviceTrait<D>,
{
    fn new_inherited(
        compute_pipeline: WebGPUComputePipeline,
        label: USVString,
        device: &D::GPUDevice,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            device: Dom::from_ref(device),
            droppable: DroppableGPUComputePipeline {
                channel: device.channel(),
                compute_pipeline,
            },
        }
    }

    pub fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        compute_pipeline: WebGPUComputePipeline,
        label: USVString,
        device: &D::GPUDevice,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPUComputePipeline::new_inherited(
                compute_pipeline,
                label,
                device,
            )),
            global,
            cx,
            GPUComputePipelineWrap::<D>,
        )
    }
}

impl<D> GPUComputePipeline<D>
where
    D: Equivalence,
    D::Promise: PromiseHelpers<D>,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromiseTrait<D>,
    D::GlobalScope: WebGPUGlobalTrait + GlobalScopeHelpers<D>,
    D::GPUDevice: GPUDeviceTrait<D>,
{
    pub(crate) fn id(&self) -> &WebGPUComputePipeline {
        &self.droppable.compute_pipeline
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcomputepipeline>
    pub fn create(
        device: &D::GPUDevice,
        descriptor: &GPUComputePipelineDescriptor<D>,
        async_sender: Option<GenericCallback<WebGPUComputePipelineResponse>>,
    ) -> WebGPUComputePipeline {
        let compute_pipeline_id = device
            .global_from_reflector()
            .global_wgpu_id_hub()
            .create_compute_pipeline_id();

        let pipeline_layout = device.get_pipeline_layout_data(&descriptor.parent.layout);

        let desc = ComputePipelineDescriptor {
            label: (&descriptor.parent.parent).convert(),
            layout: pipeline_layout.explicit(),
            stage: (&descriptor.compute).convert(),
            cache: None,
        };

        device
            .channel()
            .0
            .send(WebGPURequest::CreateComputePipeline {
                device_id: device.id().0,
                compute_pipeline_id,
                descriptor: desc,
                async_sender,
            })
            .expect("Failed to create WebGPU ComputePipeline");

        WebGPUComputePipeline(compute_pipeline_id)
    }
}

impl<D> GPUComputePipelineMethods<D> for GPUComputePipeline<D>
where
    D: Equivalence,
    D::GlobalScope: WebGPUGlobalTrait,
    D::GPUDevice: GPUDeviceTrait<D>,
    Self: DomGlobalGeneric<D>,
    D::Promise: PromiseHelpers<D>,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromiseTrait<D>,
{
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpupipelinebase-getbindgrouplayout>
    fn GetBindGroupLayout(
        &self,
        cx: &mut JSContext,
        index: u32,
    ) -> Fallible<DomRoot<GPUBindGroupLayout<D>>> {
        let id = self
            .global_from_reflector()
            .global_wgpu_id_hub()
            .create_bind_group_layout_id();

        if let Err(e) = self
            .droppable
            .channel
            .0
            .send(WebGPURequest::ComputeGetBindGroupLayout {
                device_id: self.device.id().0,
                pipeline_id: self.id().0,
                index,
                id,
            })
        {
            warn!("Failed to send WebGPURequest::ComputeGetBindGroupLayout {e:?}");
        }

        Ok(GPUBindGroupLayout::new(
            cx,
            &*self.global_from_reflector(),
            self.droppable.channel.clone(),
            WebGPUBindGroupLayout(id),
            USVString::default(),
        ))
    }
}
