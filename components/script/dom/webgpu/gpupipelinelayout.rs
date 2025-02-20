/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use dom_struct::dom_struct;
use webgpu::wgc::binding_model::PipelineLayoutDescriptor;
use webgpu::{WebGPU, WebGPUBindGroupLayout, WebGPUPipelineLayout, WebGPURequest};

use crate::conversions::Convert;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUPipelineLayoutDescriptor, GPUPipelineLayoutMethods,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webgpu::gpudevice::GPUDevice;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUPipelineLayout {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[no_trace]
    pipeline_layout: WebGPUPipelineLayout,
    #[no_trace]
    bind_group_layouts: Vec<WebGPUBindGroupLayout>,
}

impl GPUPipelineLayout {
    fn new_inherited(
        channel: WebGPU,
        pipeline_layout: WebGPUPipelineLayout,
        label: USVString,
        bgls: Vec<WebGPUBindGroupLayout>,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            label: DomRefCell::new(label),
            pipeline_layout,
            bind_group_layouts: bgls,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        channel: WebGPU,
        pipeline_layout: WebGPUPipelineLayout,
        label: USVString,
        bgls: Vec<WebGPUBindGroupLayout>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUPipelineLayout::new_inherited(
                channel,
                pipeline_layout,
                label,
                bgls,
            )),
            global,
            can_gc,
        )
    }
}

impl GPUPipelineLayout {
    pub(crate) fn id(&self) -> WebGPUPipelineLayout {
        self.pipeline_layout
    }

    pub(crate) fn bind_group_layouts(&self) -> Vec<WebGPUBindGroupLayout> {
        self.bind_group_layouts.clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createpipelinelayout>
    pub(crate) fn create(
        device: &GPUDevice,
        descriptor: &GPUPipelineLayoutDescriptor,
    ) -> DomRoot<GPUPipelineLayout> {
        let bgls = descriptor
            .bindGroupLayouts
            .iter()
            .map(|each| each.id())
            .collect::<Vec<_>>();

        let desc = PipelineLayoutDescriptor {
            label: (&descriptor.parent).convert(),
            bind_group_layouts: Cow::Owned(bgls.iter().map(|l| l.0).collect::<Vec<_>>()),
            push_constant_ranges: Cow::Owned(vec![]),
        };

        let pipeline_layout_id = device.global().wgpu_id_hub().create_pipeline_layout_id();
        device
            .channel()
            .0
            .send(WebGPURequest::CreatePipelineLayout {
                device_id: device.id().0,
                pipeline_layout_id,
                descriptor: desc,
            })
            .expect("Failed to create WebGPU PipelineLayout");

        let pipeline_layout = WebGPUPipelineLayout(pipeline_layout_id);
        GPUPipelineLayout::new(
            &device.global(),
            device.channel().clone(),
            pipeline_layout,
            descriptor.parent.label.clone(),
            bgls,
            CanGc::note(),
        )
    }
}

impl GPUPipelineLayoutMethods<crate::DomTypeHolder> for GPUPipelineLayout {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }
}

impl Drop for GPUPipelineLayout {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropPipelineLayout(self.pipeline_layout.0))
        {
            warn!(
                "Failed to send DropPipelineLayout ({:?}) ({})",
                self.pipeline_layout.0, e
            );
        }
    }
}
