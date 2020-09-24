/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUComputePipelineBinding::GPUComputePipelineMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubindgrouplayout::GPUBindGroupLayout;
use crate::dom::gpudevice::GPUDevice;
use dom_struct::dom_struct;
use std::string::String;
use webgpu::{WebGPUBindGroupLayout, WebGPUComputePipeline};

#[dom_struct]
pub struct GPUComputePipeline {
    reflector_: Reflector,
    label: DomRefCell<Option<USVString>>,
    compute_pipeline: WebGPUComputePipeline,
    bind_group_layouts: Vec<WebGPUBindGroupLayout>,
    device: Dom<GPUDevice>,
}

impl GPUComputePipeline {
    fn new_inherited(
        compute_pipeline: WebGPUComputePipeline,
        label: Option<USVString>,
        bgls: Vec<WebGPUBindGroupLayout>,
        device: &GPUDevice,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            compute_pipeline,
            bind_group_layouts: bgls,
            device: Dom::from_ref(device),
        }
    }

    pub fn new(
        global: &GlobalScope,
        compute_pipeline: WebGPUComputePipeline,
        label: Option<USVString>,
        bgls: Vec<WebGPUBindGroupLayout>,
        device: &GPUDevice,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUComputePipeline::new_inherited(
                compute_pipeline,
                label,
                bgls,
                device,
            )),
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

    /// https://gpuweb.github.io/gpuweb/#dom-gpupipelinebase-getbindgrouplayout
    fn GetBindGroupLayout(&self, index: u32) -> Fallible<DomRoot<GPUBindGroupLayout>> {
        if index > self.bind_group_layouts.len() as u32 {
            return Err(Error::Range(String::from("Index out of bounds")));
        }
        Ok(GPUBindGroupLayout::new(
            &self.global(),
            self.bind_group_layouts[index as usize],
            None,
        ))
    }
}
