/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use js::realm::CurrentRealm;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUShaderModuleDescriptor, GPUShaderModuleMethods, GPUShaderModuleWrap,
};
use script_bindings::interfaces::{
    HeapTracedPromiseHelpers, PromiseHelpers, StackRootPromiseHelpers,
};
use script_bindings::reflector::{DomGlobalGeneric, Reflector, reflect_dom_object_with_wrap};
use webgpu_traits::{WebGPU, WebGPURequest, WebGPUShaderModule};

use crate::JSTraceable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::traits::{Equivalence, GPUDeviceTrait, WebGPUGlobalTrait, WebGPUPromiseTrait};

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
pub struct GPUShaderModule<D: DomTypes> {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    compilation_info_promise: <D::Promise as PromiseHelpers<D>>::HeapTraced,
    droppable: DroppableGPUShaderModule,
}

impl<D: Equivalence> GPUShaderModule<D> {
    fn new_inherited(
        channel: WebGPU,
        shader_module: WebGPUShaderModule,
        label: USVString,
        promise: &<D::Promise as PromiseHelpers<D>>::StackRoot,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            compilation_info_promise: promise.to_traced(),
            droppable: DroppableGPUShaderModule {
                channel,
                shader_module,
            },
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        channel: WebGPU,
        shader_module: WebGPUShaderModule,
        label: USVString,
        promise: &<D::Promise as PromiseHelpers<D>>::StackRoot,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPUShaderModule::new_inherited(
                channel,
                shader_module,
                label,
                promise,
            )),
            global,
            cx,
            GPUShaderModuleWrap::<D>,
        )
    }
}

impl<D> GPUShaderModule<D>
where
    D: Equivalence,
    D::Promise: PromiseHelpers<D>,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromiseTrait<D>,
    D::GlobalScope: WebGPUGlobalTrait,
    D::GPUDevice: GPUDeviceTrait<D>,
{
    pub(crate) fn id(&self) -> WebGPUShaderModule {
        self.droppable.shader_module
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpudevice-createshadermodule>
    pub fn create(
        cx: &mut CurrentRealm<'_>,
        device: &D::GPUDevice,
        descriptor: RootedTraceableBox<GPUShaderModuleDescriptor>,
    ) -> DomRoot<GPUShaderModule<D>> {
        let program_id = device
            .global_from_reflector()
            .global_wgpu_id_hub()
            .create_shader_module_id();
        let promise = D::Promise::new_in_realm_rooted(cx);
        let shader_module = GPUShaderModule::new(
            cx,
            &*device.global_from_reflector(),
            device.channel(),
            WebGPUShaderModule(program_id),
            descriptor.parent.label.clone(),
            &promise,
        );
        let callback = promise.callback_promise_gpushadermodule(&*shader_module);
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

impl<D: DomTypes> GPUShaderModuleMethods<D> for GPUShaderModule<D> {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpushadermodule-getcompilationinfo>
    fn GetCompilationInfo(&self) -> <D::Promise as PromiseHelpers<D>>::StackRoot {
        self.compilation_info_promise.root()
    }
}
