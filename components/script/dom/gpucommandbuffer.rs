/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::GPUCommandBufferBinding::GPUCommandBufferMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::Dom;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubuffer::GPUBuffer;
use dom_struct::dom_struct;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use webgpu::{WebGPU, WebGPUCommandBuffer, WebGPURequest};

impl Eq for DomRoot<GPUBuffer> {}
impl Hash for DomRoot<GPUBuffer> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableField {
    #[ignore_malloc_size_of = "defined in webgpu"]
    channel: WebGPU,
    command_buffer: WebGPUCommandBuffer,
}

impl Drop for DroppableField {
    fn drop(&mut self) {
        if let Err(e) = self.channel.0.send((
            None,
            WebGPURequest::FreeCommandBuffer(self.command_buffer.0),
        )) {
            warn!(
                "Failed to send FreeCommandBuffer({:?}) ({})",
                self.command_buffer.0, e
            );
        }
    }
}

#[dom_struct]
pub struct GPUCommandBuffer {
    reflector_: Reflector,
    label: DomRefCell<Option<USVString>>,
    buffers: DomRefCell<HashSet<Dom<GPUBuffer>>>,
    droppable_field: DroppableField,
}

impl GPUCommandBuffer {
    fn new_inherited(
        channel: WebGPU,
        command_buffer: WebGPUCommandBuffer,
        buffers: HashSet<DomRoot<GPUBuffer>>,
        label: Option<USVString>,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            buffers: DomRefCell::new(buffers.into_iter().map(|b| Dom::from_ref(&*b)).collect()),
            droppable_field: DroppableField {
                channel,
                command_buffer,
            },
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        command_buffer: WebGPUCommandBuffer,
        buffers: HashSet<DomRoot<GPUBuffer>>,
        label: Option<USVString>,
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

impl GPUCommandBuffer {
    pub fn id(&self) -> WebGPUCommandBuffer {
        self.droppable_field.command_buffer
    }

    pub fn buffers(&self) -> Ref<HashSet<Dom<GPUBuffer>>> {
        self.buffers.borrow()
    }
}

impl GPUCommandBufferMethods for GPUCommandBuffer {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<USVString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<USVString>) {
        *self.label.borrow_mut() = value;
    }
}
