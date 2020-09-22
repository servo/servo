/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GPUBufferBinding::GPUSize64;
use crate::dom::bindings::codegen::Bindings::GPUCommandEncoderBinding::{
    GPUTextureCopyView, GPUTextureDataLayout,
};
use crate::dom::bindings::codegen::Bindings::GPUQueueBinding::GPUQueueMethods;
use crate::dom::bindings::codegen::Bindings::GPUTextureBinding::GPUExtent3D;
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer as BufferSource;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpubuffer::{GPUBuffer, GPUBufferState};
use crate::dom::gpucommandbuffer::GPUCommandBuffer;
use crate::dom::gpucommandencoder::{convert_texture_cv, convert_texture_data_layout};
use crate::dom::gpudevice::{convert_texture_size_to_dict, convert_texture_size_to_wgt, GPUDevice};
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSharedMemory;
use webgpu::{identity::WebGPUOpResult, wgt, WebGPU, WebGPUQueue, WebGPURequest};

#[dom_struct]
pub struct GPUQueue {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webgpu"]
    channel: WebGPU,
    device: DomRefCell<Option<Dom<GPUDevice>>>,
    label: DomRefCell<Option<USVString>>,
    queue: WebGPUQueue,
}

impl GPUQueue {
    fn new_inherited(channel: WebGPU, queue: WebGPUQueue) -> Self {
        GPUQueue {
            channel,
            reflector_: Reflector::new(),
            device: DomRefCell::new(None),
            label: DomRefCell::new(None),
            queue,
        }
    }

    pub fn new(global: &GlobalScope, channel: WebGPU, queue: WebGPUQueue) -> DomRoot<Self> {
        reflect_dom_object(Box::new(GPUQueue::new_inherited(channel, queue)), global)
    }
}

impl GPUQueue {
    pub fn set_device(&self, device: &GPUDevice) {
        *self.device.borrow_mut() = Some(Dom::from_ref(device));
    }
}

impl GPUQueueMethods for GPUQueue {
    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn GetLabel(&self) -> Option<USVString> {
        self.label.borrow().clone()
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuobjectbase-label
    fn SetLabel(&self, value: Option<USVString>) {
        *self.label.borrow_mut() = value;
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuqueue-submit
    fn Submit(&self, command_buffers: Vec<DomRoot<GPUCommandBuffer>>) {
        let valid = command_buffers.iter().all(|cb| {
            cb.buffers().iter().all(|b| match b.state() {
                GPUBufferState::Unmapped => true,
                _ => false,
            })
        });
        let scope_id = self.device.borrow().as_ref().unwrap().use_current_scope();
        if !valid {
            self.device.borrow().as_ref().unwrap().handle_server_msg(
                scope_id,
                WebGPUOpResult::ValidationError(String::from(
                    "Referenced GPUBuffer(s) are not Unmapped",
                )),
            );
            return;
        }
        let command_buffers = command_buffers.iter().map(|cb| cb.id().0).collect();
        self.channel
            .0
            .send((
                scope_id,
                WebGPURequest::Submit {
                    queue_id: self.queue.0,
                    command_buffers,
                },
            ))
            .unwrap();
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuqueue-writebuffer
    #[allow(unsafe_code)]
    fn WriteBuffer(
        &self,
        buffer: &GPUBuffer,
        buffer_offset: GPUSize64,
        data: BufferSource,
        data_offset: GPUSize64,
        size: Option<GPUSize64>,
    ) -> Fallible<()> {
        let bytes = match data {
            BufferSource::ArrayBufferView(d) => d.to_vec(),
            BufferSource::ArrayBuffer(d) => d.to_vec(),
        };
        let content_size = if let Some(s) = size {
            s
        } else {
            bytes.len() as GPUSize64 - data_offset
        };
        let valid = data_offset + content_size <= bytes.len() as u64 &&
            buffer.state() == GPUBufferState::Unmapped &&
            content_size % wgt::COPY_BUFFER_ALIGNMENT == 0 &&
            buffer_offset % wgt::COPY_BUFFER_ALIGNMENT == 0;

        if !valid {
            return Err(Error::Operation);
        }

        let final_data = IpcSharedMemory::from_bytes(
            &bytes[data_offset as usize..(data_offset + content_size) as usize],
        );
        if let Err(e) = self.channel.0.send((
            self.device.borrow().as_ref().unwrap().use_current_scope(),
            WebGPURequest::WriteBuffer {
                queue_id: self.queue.0,
                buffer_id: buffer.id().0,
                buffer_offset,
                data: final_data,
            },
        )) {
            warn!("Failed to send WriteBuffer({:?}) ({})", buffer.id(), e);
            return Err(Error::Operation);
        }

        Ok(())
    }

    /// https://gpuweb.github.io/gpuweb/#dom-gpuqueue-writetexture
    fn WriteTexture(
        &self,
        destination: &GPUTextureCopyView,
        data: BufferSource,
        data_layout: &GPUTextureDataLayout,
        size: GPUExtent3D,
    ) -> Fallible<()> {
        let (bytes, len) = match data {
            BufferSource::ArrayBufferView(d) => (d.to_vec(), d.len() as u64),
            BufferSource::ArrayBuffer(d) => (d.to_vec(), d.len() as u64),
        };
        let valid = data_layout.offset <= len;

        if !valid {
            return Err(Error::Operation);
        }

        let texture_cv = convert_texture_cv(destination);
        let texture_layout = convert_texture_data_layout(data_layout);
        let write_size = convert_texture_size_to_wgt(&convert_texture_size_to_dict(&size));
        let final_data = IpcSharedMemory::from_bytes(&bytes);

        if let Err(e) = self.channel.0.send((
            self.device.borrow().as_ref().unwrap().use_current_scope(),
            WebGPURequest::WriteTexture {
                queue_id: self.queue.0,
                texture_cv,
                data_layout: texture_layout,
                size: write_size,
                data: final_data,
            },
        )) {
            warn!(
                "Failed to send WriteTexture({:?}) ({})",
                destination.texture.id().0,
                e
            );
            return Err(Error::Operation);
        }

        Ok(())
    }
}
