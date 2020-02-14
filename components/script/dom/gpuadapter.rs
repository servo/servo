/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::GPUAdapterBinding::{
    self, GPUAdapterMethods, GPUDeviceDescriptor,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpu::response_async;
use crate::dom::gpu::AsyncWGPUListener;
use crate::dom::gpudevice::GPUDevice;
use crate::dom::promise::Promise;
use crate::realms::InRealm;
use crate::script_runtime::JSContext as SafeJSContext;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use std::ptr::NonNull;
use std::rc::Rc;
use webgpu::{wgpu, WebGPU, WebGPUAdapter, WebGPURequest, WebGPUResponse};

#[dom_struct]
pub struct GPUAdapter {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    channel: WebGPU,
    name: DOMString,
    #[ignore_malloc_size_of = "mozjs"]
    extensions: Heap<*mut JSObject>,
    adapter: WebGPUAdapter,
}

impl GPUAdapter {
    pub fn new_inherited(
        channel: WebGPU,
        name: DOMString,
        extensions: Heap<*mut JSObject>,
        adapter: WebGPUAdapter,
    ) -> GPUAdapter {
        GPUAdapter {
            reflector_: Reflector::new(),
            channel,
            name,
            extensions,
            adapter,
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        name: DOMString,
        extensions: Heap<*mut JSObject>,
        adapter: WebGPUAdapter,
    ) -> DomRoot<GPUAdapter> {
        reflect_dom_object(
            Box::new(GPUAdapter::new_inherited(
                channel, name, extensions, adapter,
            )),
            global,
            GPUAdapterBinding::Wrap,
        )
    }
}

impl GPUAdapterMethods for GPUAdapter {
    // https://gpuweb.github.io/gpuweb/#dom-gpuadapter-name
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    // https://gpuweb.github.io/gpuweb/#dom-gpuadapter-extensions
    fn Extensions(&self, _cx: SafeJSContext) -> NonNull<JSObject> {
        NonNull::new(self.extensions.get()).unwrap()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuadapter-requestdevice
    fn RequestDevice(&self, descriptor: &GPUDeviceDescriptor, comp: InRealm) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(&self.global(), comp);
        let sender = response_async(&promise, self);
        let desc = wgpu::instance::DeviceDescriptor {
            extensions: wgpu::instance::Extensions {
                anisotropic_filtering: descriptor.extensions.anisotropicFiltering,
            },
            limits: wgpu::instance::Limits {
                max_bind_groups: descriptor.limits.maxBindGroups,
            },
        };
        let id = self
            .global()
            .wgpu_create_device_id(self.adapter.0.backend());
        if self
            .channel
            .0
            .send(WebGPURequest::RequestDevice(sender, self.adapter, desc, id))
            .is_err()
        {
            promise.reject_error(Error::Operation);
        }
        promise
    }
}

impl AsyncWGPUListener for GPUAdapter {
    fn handle_response(&self, response: WebGPUResponse, promise: &Rc<Promise>) {
        match response {
            WebGPUResponse::RequestDevice(device_id, queue_id, _descriptor) => {
                let device = GPUDevice::new(
                    &self.global(),
                    self.channel.clone(),
                    &self,
                    Heap::default(),
                    Heap::default(),
                    device_id,
                    queue_id,
                );
                promise.resolve_native(&device);
            },
            _ => promise.reject_error(Error::Operation),
        }
    }
}
