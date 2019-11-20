/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUBufferBinding::{
    self, GPUBufferMethods, GPUBufferSize,
};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use std::cell::Cell;
use webgpu::{WebGPU, WebGPUBuffer, WebGPUDevice, WebGPURequest};

#[derive(MallocSizeOf)]
pub enum GPUBufferState {
    Mapped,
    Unmapped,
    Destroyed,
}

#[dom_struct]
pub struct GPUBuffer {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    channel: WebGPU,
    label: DomRefCell<Option<DOMString>>,
    size: GPUBufferSize,
    usage: u32,
    state: DomRefCell<GPUBufferState>,
    buffer: WebGPUBuffer,
    device: WebGPUDevice,
    valid: Cell<bool>,
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
    ) -> DomRoot<GPUBuffer> {
        reflect_dom_object(
            Box::new(GPUBuffer::new_inherited(
                channel, buffer, device, state, size, usage, valid,
            )),
            global,
            GPUBufferBinding::Wrap,
        )
    }
}

impl Drop for GPUBuffer {
    fn drop(&mut self) {
        self.Destroy()
    }
}

impl GPUBufferMethods for GPUBuffer {
    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-unmap
    fn Unmap(&self) {
        self.channel
            .0
            .send(WebGPURequest::UnmapBuffer(self.buffer))
            .unwrap();
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpubuffer-destroy
    fn Destroy(&self) {
        match *self.state.borrow() {
            GPUBufferState::Mapped => {
                self.Unmap();
            },
            _ => {},
        };
        self.channel
            .0
            .send(WebGPURequest::DestroyBuffer(self.buffer))
            .unwrap();
        *self.state.borrow_mut() = GPUBufferState::Destroyed;
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
