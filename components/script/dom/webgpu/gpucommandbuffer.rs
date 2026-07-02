/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use webgpu_traits::{WebGPU, WebGPUCommandBuffer, WebGPURequest};

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::GPUCommandBufferMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPUCommandBuffer {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    command_buffer: WebGPUCommandBuffer,
}

impl Drop for DroppableGPUCommandBuffer {
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

#[dom_struct]
pub(crate) struct GPUCommandBuffer {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    droppable: DroppableGPUCommandBuffer,
}

impl GPUCommandBuffer {
    fn new_inherited(
        channel: WebGPU,
        command_buffer: WebGPUCommandBuffer,
        label: USVString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            label: DomRefCell::new(label),
            droppable: DroppableGPUCommandBuffer {
                channel,
                command_buffer,
            },
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        channel: WebGPU,
        command_buffer: WebGPUCommandBuffer,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(GPUCommandBuffer::new_inherited(
                channel,
                command_buffer,
                label,
            )),
            global,
            cx,
        )
    }
}

impl GPUCommandBuffer {
    pub(crate) fn id(&self) -> WebGPUCommandBuffer {
        self.droppable.command_buffer
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
