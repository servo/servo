/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_base::generic_channel::GenericCallback;
use webgpu_traits::{
    WebGPU, WebGPUBindGroupLayout, WebGPURenderPipeline, WebGPURenderPipelineResponse,
    WebGPURequest,
};
use wgpu_core::pipeline::RenderPipelineDescriptor;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPURenderPipelineMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webgpu::gpubindgrouplayout::GPUBindGroupLayout;
use crate::dom::webgpu::gpudevice::{GPUDevice, PipelineLayout};
use crate::script_runtime::CanGc;

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
pub(crate) struct GPURenderPipeline {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    device: Dom<GPUDevice>,
    droppable: DroppableGPURenderPipeline,
}

impl GPURenderPipeline {
    fn new_inherited(
        render_pipeline: WebGPURenderPipeline,
        label: USVString,
        device: &GPUDevice,
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

    pub(crate) fn new(
        global: &GlobalScope,
        render_pipeline: WebGPURenderPipeline,
        label: USVString,
        device: &GPUDevice,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPURenderPipeline::new_inherited(
                render_pipeline,
                label,
                device,
            )),
            global,
            can_gc,
        )
    }
}

impl GPURenderPipeline {
    pub(crate) fn id(&self) -> WebGPURenderPipeline {
        self.droppable.render_pipeline
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createrenderpipeline>
    pub(crate) fn create(
        device: &GPUDevice,
        pipeline_layout: PipelineLayout,
        descriptor: RenderPipelineDescriptor<'static>,
        async_sender: Option<GenericCallback<WebGPURenderPipelineResponse>>,
    ) -> Fallible<WebGPURenderPipeline> {
        let render_pipeline_id = device.global().wgpu_id_hub().create_render_pipeline_id();

        device
            .channel()
            .0
            .send(WebGPURequest::CreateRenderPipeline {
                device_id: device.id().0,
                render_pipeline_id,
                descriptor,
                implicit_ids: pipeline_layout.implicit(),
                async_sender,
            })
            .expect("Failed to create WebGPU render pipeline");

        Ok(WebGPURenderPipeline(render_pipeline_id))
    }
}

impl GPURenderPipelineMethods<crate::DomTypeHolder> for GPURenderPipeline {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpupipelinebase-getbindgrouplayout>
    fn GetBindGroupLayout(&self, index: u32) -> Fallible<DomRoot<GPUBindGroupLayout>> {
        let id = self.global().wgpu_id_hub().create_bind_group_layout_id();

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
            &self.global(),
            self.droppable.channel.clone(),
            WebGPUBindGroupLayout(id),
            USVString::default(),
            CanGc::deprecated_note(),
        ))
    }
}
