/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use webgpu::wgc::pipeline::ComputePipelineDescriptor;
use webgpu::{WebGPU, WebGPUBindGroupLayout, WebGPUComputePipeline, WebGPURequest, WebGPUResponse};

use crate::conversions::Convert;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUComputePipelineDescriptor, GPUComputePipelineMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webgpu::gpubindgrouplayout::GPUBindGroupLayout;
use crate::dom::webgpu::gpudevice::GPUDevice;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUComputePipeline {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[no_trace]
    compute_pipeline: WebGPUComputePipeline,
    device: Dom<GPUDevice>,
}

impl GPUComputePipeline {
    fn new_inherited(
        compute_pipeline: WebGPUComputePipeline,
        label: USVString,
        device: &GPUDevice,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel: device.channel(),
            label: DomRefCell::new(label),
            compute_pipeline,
            device: Dom::from_ref(device),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        compute_pipeline: WebGPUComputePipeline,
        label: USVString,
        device: &GPUDevice,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUComputePipeline::new_inherited(
                compute_pipeline,
                label,
                device,
            )),
            global,
            CanGc::note(),
        )
    }
}

impl GPUComputePipeline {
    pub(crate) fn id(&self) -> &WebGPUComputePipeline {
        &self.compute_pipeline
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createcomputepipeline>
    pub(crate) fn create(
        device: &GPUDevice,
        descriptor: &GPUComputePipelineDescriptor,
        async_sender: Option<IpcSender<WebGPUResponse>>,
    ) -> WebGPUComputePipeline {
        let compute_pipeline_id = device.global().wgpu_id_hub().create_compute_pipeline_id();

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
                implicit_ids: pipeline_layout.implicit(),
                async_sender,
            })
            .expect("Failed to create WebGPU ComputePipeline");

        WebGPUComputePipeline(compute_pipeline_id)
    }
}

impl GPUComputePipelineMethods<crate::DomTypeHolder> for GPUComputePipeline {
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
            .channel
            .0
            .send(WebGPURequest::ComputeGetBindGroupLayout {
                device_id: self.device.id().0,
                pipeline_id: self.compute_pipeline.0,
                index,
                id,
            })
        {
            warn!("Failed to send WebGPURequest::ComputeGetBindGroupLayout {e:?}");
        }

        Ok(GPUBindGroupLayout::new(
            &self.global(),
            self.channel.clone(),
            WebGPUBindGroupLayout(id),
            USVString::default(),
        ))
    }
}

impl Drop for GPUComputePipeline {
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
