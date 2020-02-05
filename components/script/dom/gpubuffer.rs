/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::GPUBufferBinding::{
    self, GPUBufferMethods, GPUBufferSize,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpu::{response_async, AsyncWGPUListener};
use crate::dom::promise::Promise;
use crate::realms::InRealm;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use js::jsval::UndefinedValue;
use js::rust::jsapi_wrapped::{DetachArrayBuffer, IsPromiseObject, RejectPromise};
use js::rust::MutableHandle;
use js::typedarray::{ArrayBuffer, CreateWith};
use std::cell::Cell;
use std::ptr;
use std::rc::Rc;
use webgpu::{
    wgpu::resource::BufferUsage, WebGPU, WebGPUBuffer, WebGPUDevice, WebGPURequest, WebGPUResponse,
};

// https://gpuweb.github.io/gpuweb/#buffer-state
#[derive(Clone, MallocSizeOf)]
pub enum GPUBufferState {
    MappedForReading,
    MappedForWriting,
    MappedPendingForReading,
    MappedPendingForWriting,
    Unmapped,
    Destroyed,
}

#[dom_struct]
pub struct GPUBuffer {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    channel: WebGPU,
    label: DomRefCell<Option<DOMString>>,
    size: GPUBufferSize,
    usage: u32,
    state: DomRefCell<GPUBufferState>,
    buffer: WebGPUBuffer,
    device: WebGPUDevice,
    valid: Cell<bool>,
    #[ignore_malloc_size_of = "defined in mozjs"]
    mapping: RootedTraceableBox<Heap<*mut JSObject>>,
}

impl GPUBuffer {
    fn new_inherited(
        channel: WebGPU,
        buffer: WebGPUBuffer,
        device: WebGPUDevice,
        state: GPUBufferState,
        size: GPUBufferSize,
        usage: u32,
        valid: bool,
        mapping: RootedTraceableBox<Heap<*mut JSObject>>,
    ) -> GPUBuffer {
        Self {
            reflector_: Reflector::new(),
            channel,
            label: DomRefCell::new(None),
            state: DomRefCell::new(state),
            size: size,
            usage: usage,
            valid: Cell::new(valid),
            device,
            buffer,
            mapping,
        }
    }

    #[allow(unsafe_code)]
    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        buffer: WebGPUBuffer,
        device: WebGPUDevice,
        state: GPUBufferState,
        size: GPUBufferSize,
        usage: u32,
        valid: bool,
        mapping: RootedTraceableBox<Heap<*mut JSObject>>,
    ) -> DomRoot<GPUBuffer> {
        reflect_dom_object(
            Box::new(GPUBuffer::new_inherited(
                channel, buffer, device, state, size, usage, valid, mapping,
            )),
            global,
            GPUBufferBinding::Wrap,
        )
    }
}

impl GPUBuffer {
    pub fn id(&self) -> WebGPUBuffer {
        self.buffer
    }

    pub fn size(&self) -> GPUBufferSize {
        self.size
    }

    pub fn usage(&self) -> u32 {
        self.usage
    }

    pub fn state(&self) -> Ref<GPUBufferState> {
        self.state.borrow()
    }
}

impl Drop for GPUBuffer {
    fn drop(&mut self) {
        self.Destroy()
    }
}

impl GPUBufferMethods for GPUBuffer {
    #[allow(unsafe_code)]
    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-unmap
    fn Unmap(&self) {
        let cx = self.global().get_cx();
        // Step 1
        match *self.state.borrow() {
            GPUBufferState::Unmapped | GPUBufferState::Destroyed => {
                // TODO: Record validation error on the current scope
                return;
            },
            GPUBufferState::MappedForWriting => {
                // Step 3.1
                match ArrayBuffer::from(self.mapping.get()) {
                    Ok(array_buffer) => {
                        self.channel
                            .0
                            .send(WebGPURequest::UnmapBuffer {
                                device_id: self.device.0,
                                buffer_id: self.id().0,
                                array_buffer: array_buffer.to_vec(),
                            })
                            .unwrap();
                        // Step 3.2
                        unsafe {
                            DetachArrayBuffer(*cx, self.mapping.handle());
                        }
                    },
                    _ => {
                        // Step 2
                        unsafe {
                            if IsPromiseObject(self.mapping.handle()) {
                                let err = Error::Abort;
                                rooted!(in(*cx) let mut undef = UndefinedValue());
                                err.to_jsval(*cx, &self.global(), undef.handle_mut());
                                RejectPromise(*cx, self.mapping.handle(), undef.handle());
                            };
                        }
                    },
                };
            },
            _ => {},
        };
        // Step 3.3
        self.mapping.set(ptr::null_mut());
        // Step 4
        *self.state.borrow_mut() = GPUBufferState::Unmapped;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-destroy
    fn Destroy(&self) {
        let state = self.state.borrow().clone();
        match state {
            GPUBufferState::MappedForReading | GPUBufferState::MappedForWriting => {
                self.Unmap();
            },
            _ => {},
        };
        self.channel
            .0
            .send(WebGPURequest::DestroyBuffer(self.buffer.0))
            .unwrap();
        *self.state.borrow_mut() = GPUBufferState::Destroyed;
    }

    #[allow(unsafe_code)]
    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-mapreadasync
    fn MapReadAsync(&self, comp: InRealm) -> Rc<Promise> {
        // Step 1 & 2
        let promise = Promise::new_in_current_realm(&self.global(), comp);
        match *self.state.borrow() {
            GPUBufferState::Unmapped => {
                match BufferUsage::from_bits(self.usage) {
                    Some(usage) => {
                        if !usage.contains(BufferUsage::MAP_READ) {
                            // TODO: Record validation error on the current scope
                            promise.reject_error(Error::Abort);
                            return promise;
                        };
                    },
                    None => {
                        promise.reject_error(Error::Abort);
                        return promise;
                    },
                }
            },
            _ => {
                promise.reject_error(Error::Abort);
                return promise;
            },
        }
        // Step 3
        self.mapping.set(*promise.promise_obj());
        // Step 4
        *self.state.borrow_mut() = GPUBufferState::MappedPendingForReading;

        // Step 5.1
        if unsafe {
            ArrayBuffer::create(
                *self.global().get_cx(),
                CreateWith::Length(self.size as u32),
                MutableHandle::from_raw(self.mapping.handle_mut()),
            )
        }
        .is_err()
        {
            promise.reject_error(Error::Operation);
            return promise;
        }

        let sender = response_async(&promise, self);
        if self
            .channel
            .0
            .send(WebGPURequest::MapReadAsync {
                sender,
                buffer_id: self.buffer.0,
                device_id: self.device.0,
                usage: self.usage,
                size: self.size,
            })
            .is_err()
        {
            promise.reject_error(Error::Operation);
            return promise;
        }

        // Step 6
        promise
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }
}

impl AsyncWGPUListener for GPUBuffer {
    #[allow(unsafe_code)]
    fn handle_response(&self, response: WebGPUResponse, promise: &Rc<Promise>) {
        match response {
            WebGPUResponse::MapReadAsync(bytes) => unsafe {
                match ArrayBuffer::from(self.mapping.get()) {
                    Ok(mut array_buffer) => {
                        // Step 5.2
                        array_buffer.update(&bytes);
                        // Step 5.3
                        *self.state.borrow_mut() = GPUBufferState::MappedForReading;
                        // Step 5.4
                        promise.resolve_native(&array_buffer);
                    },
                    _ => promise.reject_error(Error::Operation),
                };
            },
            _ => promise.reject_error(Error::Operation),
        }
    }
}
