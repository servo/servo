/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use webgpu::{WebGPU, WebGPURequest, WebGPUResponse, WebGPUShaderModule};

use super::gpu::AsyncWGPUListener;
use super::gpucompilationinfo::GPUCompilationInfo;
use super::promise::Promise;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUShaderModuleMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct GPUShaderModule {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[no_trace]
    shader_module: WebGPUShaderModule,
    #[ignore_malloc_size_of = "promise"]
    compilation_info_promise: Rc<Promise>,
}

impl GPUShaderModule {
    fn new_inherited(
        channel: WebGPU,
        shader_module: WebGPUShaderModule,
        label: USVString,
        promise: Rc<Promise>,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            label: DomRefCell::new(label),
            shader_module,
            compilation_info_promise: promise,
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        shader_module: WebGPUShaderModule,
        label: USVString,
        promise: Rc<Promise>,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUShaderModule::new_inherited(
                channel,
                shader_module,
                label,
                promise,
            )),
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
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpushadermodule-getcompilationinfo>
    fn GetCompilationInfo(&self) -> Rc<Promise> {
        self.compilation_info_promise.clone()
    }
}

impl AsyncWGPUListener for GPUShaderModule {
    fn handle_response(&self, response: WebGPUResponse, promise: &Rc<Promise>) {
        match response {
            WebGPUResponse::CompilationInfo(info) => {
                let info = GPUCompilationInfo::from(&self.global(), info);
                promise.resolve_native(&info);
            },
            _ => unreachable!("Wrong response received on AsyncWGPUListener for GPUShaderModule"),
        }
    }
}

impl Drop for GPUShaderModule {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropShaderModule(self.shader_module.0))
        {
            warn!(
                "Failed to send DropShaderModule ({:?}) ({})",
                self.shader_module.0, e
            );
        }
    }
}
