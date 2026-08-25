/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use jstraceable_derive::JSTraceable;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUCommandBufferMethods, GPUCommandBufferWrap,
};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_wrap};
use webgpu_traits::{WebGPU, WebGPUCommandBuffer, WebGPURequest};

use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;

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
pub struct GPUCommandBuffer<D: DomTypes> {
    reflector_: Reflector,
    label: DomRefCell<USVString>,
    droppable: DroppableGPUCommandBuffer,
    #[no_trace = "PhantomData does not exist"]
    phantom: PhantomData<D>,
}

impl<D> GPUCommandBuffer<D>
where
    D: DomTypes<GPUCommandBuffer = GPUCommandBuffer<D>>,
{
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
            phantom: PhantomData,
        }
    }

    pub fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        channel: WebGPU,
        command_buffer: WebGPUCommandBuffer,
        label: USVString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPUCommandBuffer::new_inherited(
                channel,
                command_buffer,
                label,
            )),
            global,
            cx,
            GPUCommandBufferWrap::<D>,
        )
    }
}

impl<D: DomTypes> GPUCommandBuffer<D> {
    pub fn id(&self) -> WebGPUCommandBuffer {
        self.droppable.command_buffer
    }
}

impl<D: DomTypes> GPUCommandBufferMethods<D> for GPUCommandBuffer<D> {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, no_gc: &NoGC, value: USVString) {
        *self.label.safe_borrow_mut(no_gc) = value;
    }
}
