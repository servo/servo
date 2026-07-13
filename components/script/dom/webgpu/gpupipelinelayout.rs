/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use js::cell::JSCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use webgpu_traits::{WebGPU, WebGPUBindGroupLayout, WebGPUPipelineLayout, WebGPURequest};
use wgpu_core::binding_model::PipelineLayoutDescriptor;

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUPipelineLayoutDescriptor, GPUPipelineLayoutMethods,
};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webgpu::gpudevice::GPUDevice;

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPUPipelineLayout {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    pipeline_layout: WebGPUPipelineLayout,
}

impl Drop for DroppableGPUPipelineLayout {
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

#[dom_struct]
pub(crate) struct GPUPipelineLayout {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "JSCell is hard to measure"]
    label: JSCell<USVString>,
    #[no_trace]
    bind_group_layouts: Vec<WebGPUBindGroupLayout>,
    droppable: DroppableGPUPipelineLayout,
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
            label: JSCell::new(label),
            bind_group_layouts: bgls,
            droppable: DroppableGPUPipelineLayout {
                channel,
                pipeline_layout,
            },
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        channel: WebGPU,
        pipeline_layout: WebGPUPipelineLayout,
        label: USVString,
        bgls: Vec<WebGPUBindGroupLayout>,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(GPUPipelineLayout::new_inherited(
                channel,
                pipeline_layout,
                label,
                bgls,
            )),
            global,
            cx,
        )
    }
}

impl GPUPipelineLayout {
    pub(crate) fn id(&self) -> WebGPUPipelineLayout {
        self.droppable.pipeline_layout
    }

    pub(crate) fn bind_group_layouts(&self) -> Vec<WebGPUBindGroupLayout> {
        self.bind_group_layouts.clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createpipelinelayout>
    pub(crate) fn create(
        cx: &mut JSContext,
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
            // TODO(sagudev): this needs webidl sync
            bind_group_layouts: Cow::Owned(bgls.iter().map(|l| Some(l.0)).collect::<Vec<_>>()),
            immediate_size: 0,
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
            cx,
            &device.global(),
            device.channel(),
            pipeline_layout,
            descriptor.parent.label.clone(),
            bgls,
        )
    }
}

impl GPUPipelineLayoutMethods<crate::DomTypeHolder> for GPUPipelineLayout {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self, no_gc: &NoGC) -> USVString {
        self.label.borrow(no_gc).clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc_mut: &mut NoGC, value: USVString) {
        *self.label.borrow_mut(no_gc_mut) = value;
    }
}
