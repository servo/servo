/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::GPUCommandBufferBinding::{
    self, GPUCommandBufferMethods,
};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::Dom;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubuffer::GPUBuffer;
use dom_struct::dom_struct;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use webgpu::{WebGPU, WebGPUCommandBuffer};

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
    channel: WebGPU,
    label: DomRefCell<Option<DOMString>>,
    command_buffer: WebGPUCommandBuffer,
    buffers: DomRefCell<HashSet<Dom<GPUBuffer>>>,
}

impl GPUCommandBuffer {
    fn new_inherited(
        channel: WebGPU,
        command_buffer: WebGPUCommandBuffer,
        buffers: HashSet<DomRoot<GPUBuffer>>,
    ) -> GPUCommandBuffer {
        GPUCommandBuffer {
            channel,
            reflector_: Reflector::new(),
            label: DomRefCell::new(None),
            command_buffer,
            buffers: DomRefCell::new(buffers.into_iter().map(|b| Dom::from_ref(&*b)).collect()),
        }
    }

    pub fn new(
        global: &GlobalScope,
        channel: WebGPU,
        command_buffer: WebGPUCommandBuffer,
        buffers: HashSet<DomRoot<GPUBuffer>>,
    ) -> DomRoot<GPUCommandBuffer> {
        reflect_dom_object(
            Box::new(GPUCommandBuffer::new_inherited(
                channel,
                command_buffer,
                buffers,
            )),
            global,
            GPUCommandBufferBinding::Wrap,
        )
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
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<DOMString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<DOMString>) {
        *self.label.borrow_mut() = value;
    }
}
