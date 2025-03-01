/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webgpu::{WebGPU, WebGPUCommandBuffer, WebGPURequest};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUCommandBufferMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUCommandBuffer {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    label: DomRefCell<USVString>,
    #[no_trace]
    command_buffer: WebGPUCommandBuffer,
}

impl GPUCommandBuffer {
    fn new_inherited(
        channel: WebGPU,
        command_buffer: WebGPUCommandBuffer,
        label: USVString,
    ) -> Self {
        Self {
            channel,
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            command_buffer,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        channel: WebGPU,
        command_buffer: WebGPUCommandBuffer,
        label: USVString,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUCommandBuffer::new_inherited(
                channel,
                command_buffer,
                label,
            )),
            global,
            can_gc,
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
    pub(crate) fn id(&self) -> WebGPUCommandBuffer {
        self.command_buffer
    }
}

impl GPUCommandBufferMethods<crate::DomTypeHolder> for GPUCommandBuffer {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }
}
