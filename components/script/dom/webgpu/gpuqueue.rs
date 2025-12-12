/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use base::generic_channel::GenericSharedMemory;
use dom_struct::dom_struct;
use webgpu_traits::{WebGPU, WebGPUQueue, WebGPURequest};

use crate::conversions::{Convert, TryConvert};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUExtent3D, GPUImageCopyTexture, GPUImageDataLayout, GPUQueueMethods, GPUSize64,
};
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer as BufferSource;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::webgpu::gpubuffer::GPUBuffer;
use crate::dom::webgpu::gpucommandbuffer::GPUCommandBuffer;
use crate::dom::webgpu::gpudevice::GPUDevice;
use crate::routed_promise::{RoutedPromiseListener, route_promise};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUQueue {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    #[no_trace]
    channel: WebGPU,
    device: DomRefCell<Option<Dom<GPUDevice>>>,
    label: DomRefCell<USVString>,
    #[no_trace]
    queue: WebGPUQueue,
}

impl GPUQueue {
    fn new_inherited(channel: WebGPU, queue: WebGPUQueue) -> Self {
        GPUQueue {
            channel,
            reflector_: Reflector::new(),
            device: DomRefCell::new(None),
            label: DomRefCell::new(USVString::default()),
            queue,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        channel: WebGPU,
        queue: WebGPUQueue,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(GPUQueue::new_inherited(channel, queue)),
            global,
            can_gc,
        )
    }
}

impl GPUQueue {
    pub(crate) fn set_device(&self, device: &GPUDevice) {
        *self.device.borrow_mut() = Some(Dom::from_ref(device));
    }

    pub(crate) fn id(&self) -> WebGPUQueue {
        self.queue
    }
}

impl GPUQueueMethods<crate::DomTypeHolder> for GPUQueue {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn Label(&self) -> USVString {
        self.label.borrow().clone()
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label>
    fn SetLabel(&self, value: USVString) {
        *self.label.borrow_mut() = value;
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuqueue-submit>
    fn Submit(&self, command_buffers: Vec<DomRoot<GPUCommandBuffer>>) {
        let command_buffers = command_buffers.iter().map(|cb| cb.id().0).collect();
        self.channel
            .0
            .send(WebGPURequest::Submit {
                device_id: self.device.borrow().as_ref().unwrap().id().0,
                queue_id: self.queue.0,
                command_buffers,
            })
            .unwrap();
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuqueue-writebuffer>
    fn WriteBuffer(
        &self,
        buffer: &GPUBuffer,
        buffer_offset: GPUSize64,
        data: BufferSource,
        data_offset: GPUSize64,
        size: Option<GPUSize64>,
    ) -> Fallible<()> {
        // Step 1
        let sizeof_element: usize = match data {
            BufferSource::ArrayBufferView(ref d) => d.get_array_type().byte_size().unwrap_or(1),
            BufferSource::ArrayBuffer(_) => 1,
        };
        let data = match data {
            BufferSource::ArrayBufferView(d) => d.to_vec(),
            BufferSource::ArrayBuffer(d) => d.to_vec(),
        };
        // Step 2
        let data_size: usize = data.len() / sizeof_element;
        debug_assert_eq!(data.len() % sizeof_element, 0);
        // Step 3
        let content_size = if let Some(s) = size {
            s
        } else {
            (data_size as GPUSize64)
                .checked_sub(data_offset)
                .ok_or(Error::Operation(None))?
        };

        // Step 4
        let valid = data_offset + content_size <= data_size as u64 &&
            content_size * sizeof_element as u64 % wgpu_types::COPY_BUFFER_ALIGNMENT == 0;
        if !valid {
            return Err(Error::Operation(None));
        }

        // Step 5&6
        let contents = GenericSharedMemory::from_bytes(
            &data[(data_offset as usize) * sizeof_element..
                ((data_offset + content_size) as usize) * sizeof_element],
        );
        if let Err(e) = self.channel.0.send(WebGPURequest::WriteBuffer {
            device_id: self.device.borrow().as_ref().unwrap().id().0,
            queue_id: self.queue.0,
            buffer_id: buffer.id().0,
            buffer_offset,
            data: contents,
        }) {
            warn!("Failed to send WriteBuffer({:?}) ({})", buffer.id(), e);
            return Err(Error::Operation(None));
        }

        Ok(())
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuqueue-writetexture>
    fn WriteTexture(
        &self,
        destination: &GPUImageCopyTexture,
        data: BufferSource,
        data_layout: &GPUImageDataLayout,
        size: GPUExtent3D,
    ) -> Fallible<()> {
        let (bytes, len) = match data {
            BufferSource::ArrayBufferView(d) => (d.to_vec(), d.len() as u64),
            BufferSource::ArrayBuffer(d) => (d.to_vec(), d.len() as u64),
        };
        let valid = data_layout.offset <= len;

        if !valid {
            return Err(Error::Operation(None));
        }

        let texture_cv = destination.try_convert()?;
        let texture_layout = data_layout.convert();
        let write_size = (&size).try_convert()?;
        let final_data = GenericSharedMemory::from_bytes(&bytes);

        if let Err(e) = self.channel.0.send(WebGPURequest::WriteTexture {
            device_id: self.device.borrow().as_ref().unwrap().id().0,
            queue_id: self.queue.0,
            texture_cv,
            data_layout: texture_layout,
            size: write_size,
            data: final_data,
        }) {
            warn!(
                "Failed to send WriteTexture({:?}) ({})",
                destination.texture.id().0,
                e
            );
            return Err(Error::Operation(None));
        }

        Ok(())
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuqueue-onsubmittedworkdone>
    fn OnSubmittedWorkDone(&self, can_gc: CanGc) -> Rc<Promise> {
        let global = self.global();
        let promise = Promise::new(&global, can_gc);
        let task_source = global.task_manager().dom_manipulation_task_source();
        let sender = route_promise(&promise, self, task_source);

        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::QueueOnSubmittedWorkDone {
                sender,
                queue_id: self.queue.0,
            })
        {
            warn!("QueueOnSubmittedWorkDone failed with {e}")
        }
        promise
    }
}

impl RoutedPromiseListener<()> for GPUQueue {
    fn handle_response(&self, _response: (), promise: &Rc<Promise>, can_gc: CanGc) {
        promise.resolve_native(&(), can_gc);
    }
}
