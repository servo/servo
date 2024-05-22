/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webgpu::{WebGPU, WebGPUBindGroupLayout, WebGPUPipelineLayout, WebGPURequest};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUPipelineLayoutMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct GPUPipelineLayout {
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

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        pipeline_layout: WebGPUPipelineLayout,
        label: USVString,
        bgls: Vec<WebGPUBindGroupLayout>,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUPipelineLayout::new_inherited(
                channel,
                pipeline_layout,
                label,
                bgls,
            )),
            global,
        )
    }
}

impl GPUPipelineLayout {
    pub fn id(&self) -> WebGPUPipelineLayout {
        self.pipeline_layout
    }

    pub fn bind_group_layouts(&self) -> Vec<WebGPUBindGroupLayout> {
        self.bind_group_layouts.clone()
    }
}

impl GPUPipelineLayoutMethods for GPUPipelineLayout {
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
