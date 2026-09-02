/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use jstraceable_derive::JSTraceable;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPURenderPipelineMethods, GPURenderPipelineWrap,
};
use script_bindings::reflector::{DomGlobalGeneric, Reflector, reflect_dom_object_with_wrap};
use servo_base::generic_channel::GenericCallback;
use webgpu_traits::{
    WebGPU, WebGPUBindGroupLayout, WebGPURenderPipeline, WebGPURenderPipelineResponse,
    WebGPURequest,
};
use wgpu_core::pipeline::RenderPipelineDescriptor;

use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::gpubindgrouplayout::GPUBindGroupLayout;
use crate::traits::{Equivalence, GPUDeviceTrait, WebGPUGlobalTrait};

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPURenderPipeline {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    render_pipeline: WebGPURenderPipeline,
}

impl Drop for DroppableGPURenderPipeline {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropRenderPipeline(self.render_pipeline.0))
        {
            warn!(
                "Failed to send WebGPURequest::DropRenderPipeline({:?}) ({})",
                self.render_pipeline.0, e
            );
        };
    }
}

#[dom_struct]
pub struct GPURenderPipeline<D: DomTypes> {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    device: Dom<D::GPUDevice>,
    droppable: DroppableGPURenderPipeline,
}

impl<D> GPURenderPipeline<D>
where
    D: Equivalence,
    D::GPUDevice: GPUDeviceTrait<D>,
{
    fn new_inherited(
        render_pipeline: WebGPURenderPipeline,
        label: USVString,
        device: &D::GPUDevice,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            device: Dom::from_ref(device),
            droppable: DroppableGPURenderPipeline {
                channel: device.channel(),
                render_pipeline,
            },
        }
    }

    pub fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        render_pipeline: WebGPURenderPipeline,
        label: USVString,
        device: &D::GPUDevice,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPURenderPipeline::new_inherited(
                render_pipeline,
                label,
                device,
            )),
            global,
            cx,
            GPURenderPipelineWrap::<D>,
        )
    }
}

impl<D> GPURenderPipeline<D>
where
    D: Equivalence,
    D::GPUDevice: GPUDeviceTrait<D>,
    D::GlobalScope: WebGPUGlobalTrait,
    Self: DomGlobalGeneric<D>,
{
    pub(crate) fn id(&self) -> WebGPURenderPipeline {
        self.droppable.render_pipeline
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderpipeline>
    pub fn create(
        device: &D::GPUDevice,
        descriptor: RenderPipelineDescriptor<'static>,
        async_sender: Option<GenericCallback<WebGPURenderPipelineResponse>>,
    ) -> Fallible<WebGPURenderPipeline> {
        let render_pipeline_id = device
            .global_from_reflector()
            .global_wgpu_id_hub()
            .create_render_pipeline_id();

        device
            .channel()
            .0
            .send(WebGPURequest::CreateRenderPipeline {
                device_id: device.id().0,
                render_pipeline_id,
                descriptor,
                async_sender,
            })
            .expect("Failed to create WebGPU render pipeline");

        Ok(WebGPURenderPipeline(render_pipeline_id))
    }
}

impl<D> GPURenderPipelineMethods<D> for GPURenderPipeline<D>
where
    D: Equivalence,
    D::GlobalScope: DomGlobalGeneric<D> + WebGPUGlobalTrait,
    D::GPUDevice: GPUDeviceTrait<D>,
    Self: DomGlobalGeneric<D>,
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
            .send(WebGPURequest::RenderGetBindGroupLayout {
                device_id: self.device.id().0,
                pipeline_id: self.id().0,
                index,
                id,
            })
        {
            warn!("Failed to send WebGPURequest::RenderGetBindGroupLayout {e:?}");
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
