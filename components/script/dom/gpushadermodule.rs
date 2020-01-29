/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUShaderModuleBinding::{
    self, GPUShaderModuleMethods,
};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use webgpu::WebGPUShaderModule;

#[dom_struct]
pub struct GPUShaderModule {
    reflector_: Reflector,
    label: DomRefCell<Option<DOMString>>,
    shader_module: WebGPUShaderModule,
}

impl GPUShaderModule {
    fn new_inherited(shader_module: WebGPUShaderModule) -> GPUShaderModule {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(None),
            shader_module,
        }
    }

    pub fn new(
        global: &GlobalScope,
        shader_module: WebGPUShaderModule,
    ) -> DomRoot<GPUShaderModule> {
        reflect_dom_object(
            Box::new(GPUShaderModule::new_inherited(shader_module)),
            global,
            GPUShaderModuleBinding::Wrap,
        )
    }
}

impl GPUShaderModuleMethods for GPUShaderModule {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }
}
