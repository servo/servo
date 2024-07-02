/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webgpu::{WebGPU, WebGPUBindGroupLayout, WebGPURenderPipeline, WebGPURequest};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPURenderPipelineMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubindgrouplayout::GPUBindGroupLayout;

#[dom_struct]
pub struct GPURenderPipeline {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[no_trace]
    render_pipeline: WebGPURenderPipeline,
}

impl GPURenderPipeline {
    fn new_inherited(
        channel: WebGPU,
        render_pipeline: WebGPURenderPipeline,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            label: DomRefCell::new(label),
            render_pipeline,
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        render_pipeline: WebGPURenderPipeline,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPURenderPipeline::new_inherited(
                channel,
                render_pipeline,
                label,
            )),
            global,
        )
    }
}

impl GPURenderPipeline {
    pub fn id(&self) -> WebGPURenderPipeline {
        self.render_pipeline
    }
}

impl GPURenderPipelineMethods for GPURenderPipeline {
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
        let id = self
            .global()
            .wgpu_id_hub()
            .lock()
            .create_bind_group_layout_id(self.render_pipeline.0.backend());

        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::RenderGetBindGroupLayout {
                pipeline_id: self.render_pipeline.0,
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

impl Drop for GPURenderPipeline {
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
