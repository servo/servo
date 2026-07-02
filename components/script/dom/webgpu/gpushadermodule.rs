/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::realm::CurrentRealm;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use webgpu_traits::{ShaderCompilationInfo, WebGPU, WebGPURequest, WebGPUShaderModule};

use super::gpucompilationinfo::GPUCompilationInfo;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUShaderModuleDescriptor, GPUShaderModuleMethods,
};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::types::GPUDevice;
use crate::routed_promise::{RoutedPromiseListener, callback_promise};

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPUShaderModule {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    shader_module: WebGPUShaderModule,
}

impl Drop for DroppableGPUShaderModule {
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

#[dom_struct]
pub(crate) struct GPUShaderModule {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    #[ignore_malloc_size_of = "promise"]
    compilation_info_promise: Rc<Promise>,
    droppable: DroppableGPUShaderModule,
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
            label: DomRefCell::new(label),
            compilation_info_promise: promise,
            droppable: DroppableGPUShaderModule {
                channel,
                shader_module,
            },
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        channel: WebGPU,
        shader_module: WebGPUShaderModule,
        label: USVString,
        promise: Rc<Promise>,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(GPUShaderModule::new_inherited(
                channel,
                shader_module,
                label,
                promise,
            )),
            global,
            cx,
        )
    }
}

impl GPUShaderModule {
    pub(crate) fn id(&self) -> WebGPUShaderModule {
        self.droppable.shader_module
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createshadermodule>
    pub(crate) fn create(
        cx: &mut CurrentRealm<'_>,
        device: &GPUDevice,
        descriptor: RootedTraceableBox<GPUShaderModuleDescriptor>,
    ) -> DomRoot<GPUShaderModule> {
        let program_id = device.global().wgpu_id_hub().create_shader_module_id();
        let promise = Promise::new_in_realm(cx);
        let shader_module = GPUShaderModule::new(
            cx,
            &device.global(),
            device.channel(),
            WebGPUShaderModule(program_id),
            descriptor.parent.label.clone(),
            promise.clone(),
        );
        let callback = callback_promise(
            &promise,
            &*shader_module,
            device
                .global()
                .task_manager()
                .dom_manipulation_task_source(),
        );
        device
            .channel()
            .0
            .send(WebGPURequest::CreateShaderModule {
                device_id: device.id().0,
                program_id,
                program: descriptor.code.0.clone(),
                label: None,
                callback,
            })
            .expect("Failed to create WebGPU ShaderModule");
        shader_module
    }
}

impl GPUShaderModuleMethods<crate::DomTypeHolder> for GPUShaderModule {
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

impl RoutedPromiseListener<Option<ShaderCompilationInfo>> for GPUShaderModule {
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        response: Option<ShaderCompilationInfo>,
        promise: &Rc<Promise>,
    ) {
        let info = GPUCompilationInfo::from(cx, &self.global(), response);
        promise.resolve_native(cx, &info);
    }
}
