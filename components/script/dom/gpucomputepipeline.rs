/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUComputePipelineBinding::GPUComputePipelineMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use webgpu::WebGPUComputePipeline;

#[dom_struct]
pub struct GPUComputePipeline {
    reflector_: Reflector,
    label: DomRefCell<Option<USVString>>,
    compute_pipeline: WebGPUComputePipeline,
}

impl GPUComputePipeline {
    fn new_inherited(compute_pipeline: WebGPUComputePipeline, label: Option<USVString>) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            compute_pipeline,
        }
    }

    pub fn new(
        global: &GlobalScope,
        compute_pipeline: WebGPUComputePipeline,
        label: Option<USVString>,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUComputePipeline::new_inherited(compute_pipeline, label)),
            global,
        )
    }
}

impl GPUComputePipeline {
    pub fn id(&self) -> &WebGPUComputePipeline {
        &self.compute_pipeline
    }
}

impl GPUComputePipelineMethods for GPUComputePipeline {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<USVString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<USVString>) {
        *self.label.borrow_mut() = value;
    }
}
