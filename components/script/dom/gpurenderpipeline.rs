/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPURenderPipelineBinding::GPURenderPipelineMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use webgpu::{WebGPUDevice, WebGPURenderPipeline};

#[dom_struct]
pub struct GPURenderPipeline {
    reflector_: Reflector,
    label: DomRefCell<Option<USVString>>,
    render_pipeline: WebGPURenderPipeline,
    device: WebGPUDevice,
}

impl GPURenderPipeline {
    fn new_inherited(
        render_pipeline: WebGPURenderPipeline,
        device: WebGPUDevice,
        label: Option<USVString>,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            render_pipeline,
            device,
        }
    }

    pub fn new(
        global: &GlobalScope,
        render_pipeline: WebGPURenderPipeline,
        device: WebGPUDevice,
        label: Option<USVString>,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPURenderPipeline::new_inherited(
                render_pipeline,
                device,
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
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<USVString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<USVString>) {
        *self.label.borrow_mut() = value;
    }
}
