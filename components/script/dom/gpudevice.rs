/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUBufferBinding::GPUBufferDescriptor;
use crate::dom::bindings::codegen::Bindings::GPUDeviceBinding::{self, GPUDeviceMethods};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpuadapter::GPUAdapter;
use crate::dom::gpubuffer::{GPUBuffer, GPUBufferState};
use crate::dom::window::Window;
use crate::script_runtime::JSContext as SafeJSContext;
use dom_struct::dom_struct;
use ipc_channel::ipc;
use js::jsapi::{Heap, JSObject};
use js::jsval::{JSVal, ObjectValue, UndefinedValue};
use js::typedarray::{ArrayBuffer, CreateWith};
use std::ptr::{self, NonNull};
use webgpu::wgpu::resource::{BufferDescriptor, BufferUsage};
use webgpu::{WebGPU, WebGPUBuffer, WebGPUDevice, WebGPURequest};

#[dom_struct]
pub struct GPUDevice {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "channels are hard"]
    channel: WebGPU,
    adapter: Dom<GPUAdapter>,
    #[ignore_malloc_size_of = "mozjs"]
    extensions: Heap<*mut JSObject>,
    #[ignore_malloc_size_of = "mozjs"]
    limits: Heap<*mut JSObject>,
    label: DomRefCell<Option<DOMString>>,
    device: WebGPUDevice,
}

impl GPUDevice {
    fn new_inherited(
        channel: WebGPU,
        adapter: &GPUAdapter,
        extensions: Heap<*mut JSObject>,
        limits: Heap<*mut JSObject>,
        device: WebGPUDevice,
    ) -> GPUDevice {
        Self {
            eventtarget: EventTarget::new_inherited(),
            channel,
            adapter: Dom::from_ref(adapter),
            extensions,
            limits,
            label: DomRefCell::new(None),
            device,
        }
    }

    #[allow(unsafe_code)]
    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        adapter: &GPUAdapter,
        extensions: Heap<*mut JSObject>,
        limits: Heap<*mut JSObject>,
        device: WebGPUDevice,
    ) -> DomRoot<GPUDevice> {
        reflect_dom_object(
            Box::new(GPUDevice::new_inherited(
                channel, adapter, extensions, limits, device,
            )),
            global,
            GPUDeviceBinding::Wrap,
        )
    }
}

impl GPUDevice {
    unsafe fn resolve_create_buffer_mapped(
        &self,
        cx: SafeJSContext,
        gpu_buffer: WebGPUBuffer,
        array_buffer: Vec<u8>,
        descriptor: BufferDescriptor,
        valid: bool,
    ) -> Vec<JSVal> {
        rooted!(in(*cx) let mut js_array_buffer = ptr::null_mut::<JSObject>());
        let mut out = Vec::new();
        assert!(ArrayBuffer::create(
            *cx,
            CreateWith::Slice(array_buffer.as_slice()),
            js_array_buffer.handle_mut(),
        )
        .is_ok());

        let buff = GPUBuffer::new(
            &self.global(),
            self.channel.clone(),
            gpu_buffer,
            self.device,
            GPUBufferState::Mapped,
            descriptor.size,
            descriptor.usage.bits(),
            valid,
        );
        out.push(ObjectValue(buff.reflector().get_jsobject().get()));
        out.push(ObjectValue(js_array_buffer.get()));
        out
    }

    fn validate_buffer_descriptor(
        &self,
        descriptor: &GPUBufferDescriptor,
    ) -> (bool, BufferDescriptor) {
        // TODO: Record a validation error in the current scope if the descriptor is invalid.
        let wgpu_usage = BufferUsage::from_bits(descriptor.usage);
        let valid = wgpu_usage.is_some() && descriptor.size > 0;

        if valid {
            (
                true,
                BufferDescriptor {
                    size: descriptor.size,
                    usage: wgpu_usage.unwrap(),
                },
            )
        } else {
            (
                false,
                BufferDescriptor {
                    size: 0,
                    usage: BufferUsage::STORAGE,
                },
            )
        }
    }
}

impl GPUDeviceMethods for GPUDevice {
    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-adapter
    fn Adapter(&self) -> DomRoot<GPUAdapter> {
        DomRoot::from_ref(&self.adapter)
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-extensions
    fn Extensions(&self, _cx: SafeJSContext) -> NonNull<JSObject> {
        NonNull::new(self.extensions.get()).unwrap()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-limits
    fn Limits(&self, _cx: SafeJSContext) -> NonNull<JSObject> {
        NonNull::new(self.extensions.get()).unwrap()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbuffer
    fn CreateBuffer(&self, descriptor: &GPUBufferDescriptor) -> DomRoot<GPUBuffer> {
        let (valid, wgpu_descriptor) = self.validate_buffer_descriptor(descriptor);
        let (sender, receiver) = ipc::channel().unwrap();
        if let Some(window) = self.global().downcast::<Window>() {
            let id = window.Navigator().create_buffer_id(self.device.0.backend());
            self.channel
                .0
                .send(WebGPURequest::CreateBuffer(
                    sender,
                    self.device,
                    id,
                    wgpu_descriptor,
                ))
                .unwrap();
        } else {
            unimplemented!()
        };

        let buffer = receiver.recv().unwrap();

        GPUBuffer::new(
            &self.global(),
            self.channel.clone(),
            buffer,
            self.device,
            GPUBufferState::Unmapped,
            descriptor.size,
            descriptor.usage,
            valid,
        )
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpudevice-createbuffermapped
    fn CreateBufferMapped(
        &self,
        cx: SafeJSContext,
        descriptor: &GPUBufferDescriptor,
    ) -> Vec<JSVal> {
        let (valid, wgpu_descriptor) = self.validate_buffer_descriptor(descriptor);
        let (sender, receiver) = ipc::channel().unwrap();
        rooted!(in(*cx) let js_val = UndefinedValue());
        if let Some(window) = self.global().downcast::<Window>() {
            let id = window.Navigator().create_buffer_id(self.device.0.backend());
            self.channel
                .0
                .send(WebGPURequest::CreateBufferMapped(
                    sender,
                    self.device,
                    id,
                    wgpu_descriptor.clone(),
                ))
                .unwrap()
        } else {
            return vec![js_val.get()];
        };

        let (buffer, array_buffer) = receiver.recv().unwrap();

        unsafe {
            self.resolve_create_buffer_mapped(cx, buffer, array_buffer, wgpu_descriptor, valid)
        }
    }
}
