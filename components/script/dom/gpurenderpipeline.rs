/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::string::String;

use dom_struct::dom_struct;
use webgpu::{WebGPU, WebGPUBindGroupLayout, WebGPURenderPipeline, WebGPURequest};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPURenderPipelineMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubindgrouplayout::GPUBindGroupLayout;
use crate::dom::gpudevice::GPUDevice;

#[dom_struct]
pub struct GPURenderPipeline {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[no_trace]
    render_pipeline: WebGPURenderPipeline,
    #[no_trace]
    bind_group_layouts: Vec<WebGPUBindGroupLayout>,
    device: Dom<GPUDevice>,
}

impl GPURenderPipeline {
    fn new_inherited(
        render_pipeline: WebGPURenderPipeline,
        label: USVString,
        bgls: Vec<WebGPUBindGroupLayout>,
        device: &GPUDevice,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel: device.channel(),
            label: DomRefCell::new(label),
            render_pipeline,
            bind_group_layouts: bgls,
            device: Dom::from_ref(device),
        }
    }

    pub fn new(
        global: &GlobalScope,
        render_pipeline: WebGPURenderPipeline,
        label: USVString,
        bgls: Vec<WebGPUBindGroupLayout>,
        device: &GPUDevice,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPURenderPipeline::new_inherited(
                render_pipeline,
                label,
                bgls,
                device,
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
        if index > self.bind_group_layouts.len() as u32 {
            return Err(Error::Range(String::from("Index out of bounds")));
        }
        Ok(GPUBindGroupLayout::new(
            &self.global(),
            self.channel.clone(),
            self.bind_group_layouts[index as usize],
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
