/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use webgpu::WebGPUShaderModule;

use super::bindings::error::Fallible;
use super::promise::Promise;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUShaderModuleMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct GPUShaderModule {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    #[no_trace]
    shader_module: WebGPUShaderModule,
}

impl GPUShaderModule {
    fn new_inherited(shader_module: WebGPUShaderModule, label: USVString) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            shader_module,
        }
    }

    pub fn new(
        global: &GlobalScope,
        shader_module: WebGPUShaderModule,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUShaderModule::new_inherited(shader_module, label)),
            global,
        )
    }
}

impl GPUShaderModule {
    pub fn id(&self) -> WebGPUShaderModule {
        self.shader_module
    }
}

impl GPUShaderModuleMethods for GPUShaderModule {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpushadermodule-getcompilationinfo
    fn CompilationInfo(&self) -> Fallible<Rc<Promise>> {
        todo!()
    }
}
