/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use dom_struct::dom_struct;
use webgpu::{WebGPU, WebGPUCommandBuffer, WebGPURequest};

use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUCommandBufferMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubuffer::GPUBuffer;

impl Eq for DomRoot<GPUBuffer> {}
impl Hash for DomRoot<GPUBuffer> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

#[dom_struct]
pub struct GPUCommandBuffer {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[no_trace]
    command_buffer: WebGPUCommandBuffer,
    buffers: DomRefCell<HashSet<Dom<GPUBuffer>>>,
}

impl GPUCommandBuffer {
    fn new_inherited(
        channel: WebGPU,
        command_buffer: WebGPUCommandBuffer,
        buffers: HashSet<DomRoot<GPUBuffer>>,
        label: USVString,
    ) -> Self {
        Self {
            channel,
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            command_buffer,
            buffers: DomRefCell::new(buffers.into_iter().map(|b| Dom::from_ref(&*b)).collect()),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        command_buffer: WebGPUCommandBuffer,
        buffers: HashSet<DomRoot<GPUBuffer>>,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUCommandBuffer::new_inherited(
                channel,
                command_buffer,
                buffers,
                label,
            )),
            global,
        )
    }
}

impl Drop for GPUCommandBuffer {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropCommandBuffer(self.command_buffer.0))
        {
            warn!(
                "Failed to send DropCommandBuffer({:?}) ({})",
                self.command_buffer.0, e
            );
        }
    }
}

impl GPUCommandBuffer {
    pub fn id(&self) -> WebGPUCommandBuffer {
        self.command_buffer
    }

    pub fn buffers(&self) -> Ref<HashSet<Dom<GPUBuffer>>> {
        self.buffers.borrow()
    }
}

impl GPUCommandBufferMethods for GPUCommandBuffer {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }
}
