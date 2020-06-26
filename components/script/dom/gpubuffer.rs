/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUBufferBinding::{GPUBufferMethods, GPUSize64};
use crate::dom::bindings::codegen::Bindings::GPUMapModeBinding::GPUMapModeConstants;
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
use std::ops::Range;
use std::ptr;
use std::rc::Rc;
use webgpu::{
    wgpu::device::HostMap, WebGPU, WebGPUBuffer, WebGPUDevice, WebGPURequest, WebGPUResponse,
};

// https://gpuweb.github.io/gpuweb/#buffer-state
#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub enum GPUBufferState {
    Mapped,
    MappedAtCreation,
    MappingPending,
    Unmapped,
    Destroyed,
}

#[dom_struct]
pub struct GPUBuffer {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    channel: WebGPU,
    label: DomRefCell<Option<DOMString>>,
    state: Cell<GPUBufferState>,
    buffer: WebGPUBuffer,
    device: WebGPUDevice,
    valid: Cell<bool>,
    #[ignore_malloc_size_of = "defined in mozjs"]
    mapping: RootedTraceableBox<Heap<*mut JSObject>>,
    mapping_range: DomRefCell<Range<u64>>,
    size: GPUSize64,
    map_mode: Cell<Option<u32>>,
}

impl GPUBuffer {
    fn new_inherited(
        channel: WebGPU,
        buffer: WebGPUBuffer,
        device: WebGPUDevice,
        state: GPUBufferState,
        size: GPUSize64,
        valid: bool,
        mapping: RootedTraceableBox<Heap<*mut JSObject>>,
        mapping_range: Range<u64>,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            label: DomRefCell::new(None),
            state: Cell::new(state),
            valid: Cell::new(valid),
            device,
            buffer,
            mapping,
            size,
            mapping_range: DomRefCell::new(mapping_range),
            map_mode: Cell::new(None),
        }
    }

    #[allow(unsafe_code)]
    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        buffer: WebGPUBuffer,
        device: WebGPUDevice,
        state: GPUBufferState,
        size: GPUSize64,
        valid: bool,
        mapping: RootedTraceableBox<Heap<*mut JSObject>>,
        mapping_range: Range<u64>,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUBuffer::new_inherited(
                channel,
                buffer,
                device,
                state,
                size,
                valid,
                mapping,
                mapping_range,
            )),
            global,
        )
    }
}

impl GPUBuffer {
    pub fn id(&self) -> WebGPUBuffer {
        self.buffer
    }

    pub fn state(&self) -> GPUBufferState {
        self.state.get()
    }

    pub fn is_valid(&self) -> bool {
        self.valid.get()
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
        match self.state.get() {
            GPUBufferState::Unmapped | GPUBufferState::Destroyed => {
                // TODO: Record validation error on the current scope
                return;
            },
            // Step 3
            GPUBufferState::Mapped | GPUBufferState::MappedAtCreation => {
                match ArrayBuffer::from(self.mapping.get()) {
                    Ok(array_buffer) => {
                        // Step 3.2
                        if Some(GPUMapModeConstants::READ) != self.map_mode.get() {
                            self.channel
                                .0
                                .send(WebGPURequest::UnmapBuffer {
                                    device_id: self.device.0,
                                    buffer_id: self.id().0,
                                    array_buffer: array_buffer.to_vec(),
                                    mapped_at_creation: self.map_mode.get() == None,
                                })
                                .unwrap();
                        }
                        // Step 3.3
                        unsafe {
                            DetachArrayBuffer(*cx, self.mapping.handle());
                        }
                    },
                    Err(_) => {
                        warn!(
                            "Could not find ArrayBuffer of Mapped buffer ({:?})",
                            self.buffer.0
                        );
                    },
                };
            },
            // Step 2
            GPUBufferState::MappingPending => unsafe {
                if IsPromiseObject(self.mapping.handle()) {
                    let err = Error::Operation;
                    rooted!(in(*cx) let mut undef = UndefinedValue());
                    err.to_jsval(*cx, &self.global(), undef.handle_mut());
                    RejectPromise(*cx, self.mapping.handle(), undef.handle());
                } else {
                    warn!("No promise object for pending mapping found");
                }
            },
        };
        // Step 3.3
        self.mapping.set(ptr::null_mut());
        // Step 4
        self.state.set(GPUBufferState::Unmapped);
        self.map_mode.set(None);
        *self.mapping_range.borrow_mut() = 0..0;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-destroy
    fn Destroy(&self) {
        let state = self.state.get();
        match state {
            GPUBufferState::Mapped | GPUBufferState::MappedAtCreation => {
                self.Unmap();
            },
            _ => {},
        };
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DestroyBuffer(self.buffer.0))
        {
            warn!(
                "Failed to send WebGPURequest::DestroyBuffer({:?}) ({})",
                self.buffer.0, e
            );
        };
        self.state.set(GPUBufferState::Destroyed);
    }

    #[allow(unsafe_code)]
    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-mapasync-offset-size
    fn MapAsync(&self, mode: u32, offset: u64, size: u64, comp: InRealm) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(&self.global(), comp);
        let map_range = if size == 0 {
            offset..self.size
        } else {
            if offset + size > self.size {
                warn!("Requested mapping size is greated than buffer size");
                promise.reject_error(Error::Abort);
                return promise;
            }
            offset..offset + size
        };
        let host_map = match mode {
            GPUMapModeConstants::READ => HostMap::Read,
            GPUMapModeConstants::WRITE => HostMap::Write,
            _ => {
                promise.reject_error(Error::Abort);
                return promise;
            },
        };
        if self.state.get() != GPUBufferState::Unmapped {
            promise.reject_error(Error::Abort);
            return promise;
        }
        self.mapping.set(*promise.promise_obj());

        let sender = response_async(&promise, self);
        if let Err(e) = self.channel.0.send(WebGPURequest::BufferMapAsync {
            sender,
            buffer_id: self.buffer.0,
            host_map,
            map_range: map_range.clone(),
        }) {
            warn!(
                "Failed to send BufferMapAsync ({:?}) ({})",
                self.buffer.0, e
            );
            promise.reject_error(Error::Operation);
            return promise;
        }

        self.state.set(GPUBufferState::MappingPending);
        self.map_mode.set(Some(mode));
        *self.mapping_range.borrow_mut() = map_range;
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
            WebGPUResponse::BufferMapAsync(bytes) => {
                match unsafe {
                    ArrayBuffer::create(
                        *self.global().get_cx(),
                        CreateWith::Slice(&bytes),
                        MutableHandle::from_raw(self.mapping.handle_mut()),
                    )
                } {
                    Ok(_) => promise.resolve_native(&()),
                    Err(()) => {
                        warn!(
                            "Failed to create ArrayBuffer for buffer({:?})",
                            self.buffer.0
                        );
                        promise.reject_error(Error::Operation);
                    },
                }
                self.state.set(GPUBufferState::Mapped);
            },
            _ => {
                warn!("Wrong WebGPUResponse received");
                promise.reject_error(Error::Operation);
            },
        }
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::BufferMapComplete(self.buffer.0))
        {
            warn!(
                "Failed to send BufferMapComplete({:?}) ({})",
                self.buffer.0, e
            );
        }
    }
}
