/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUPipelineLayoutBinding::GPUPipelineLayoutMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use webgpu::WebGPUPipelineLayout;

#[dom_struct]
pub struct GPUPipelineLayout {
    reflector_: Reflector,
    label: DomRefCell<Option<USVString>>,
    pipeline_layout: WebGPUPipelineLayout,
}

impl GPUPipelineLayout {
    fn new_inherited(pipeline_layout: WebGPUPipelineLayout) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(None),
            pipeline_layout,
        }
    }

    pub fn new(global: &GlobalScope, pipeline_layout: WebGPUPipelineLayout) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUPipelineLayout::new_inherited(pipeline_layout)),
            global,
        )
    }
}

impl GPUPipelineLayout {
    pub fn id(&self) -> WebGPUPipelineLayout {
        self.pipeline_layout
    }
}

impl GPUPipelineLayoutMethods for GPUPipelineLayout {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<USVString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<USVString>) {
        *self.label.borrow_mut() = value;
    }
}
