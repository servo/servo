/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::ops::ControlFlow;
use std::ptr::NonNull;
use std::slice;
use std::sync::{Arc, Mutex, MutexGuard};

use arrayvec::ArrayVec;
use euclid::default::Size2D;
use log::{error, warn};
use malloc_size_of::MallocSizeOf;
use serde::{Deserialize, Serialize};
use webrender::{RenderApi, Transaction};
use webrender_api::units::DeviceIntSize;
use webrender_api::{
    DirtyRect, ExternalImageData, ExternalImageId, ExternalImageType, ImageData, ImageDescriptor,
    ImageDescriptorFlags, ImageFormat, ImageKey,
};
use webrender_traits::{WebrenderExternalImageApi, WebrenderImageSource};
use wgpu_core::device::HostMap;
use wgpu_core::global::Global;
use wgpu_core::id;
use wgpu_core::resource::{BufferAccessError, BufferMapCallback, BufferMapOperation};

use crate::{wgt, WebGPUMsg};

pub const PRESENTATION_BUFFER_COUNT: usize = 10;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct WebGPUContextId(pub u64);

impl MallocSizeOf for WebGPUContextId {
    fn size_of(&self, _ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        0
    }
}

pub type WGPUImageMap = Arc<Mutex<HashMap<WebGPUContextId, PresentationData>>>;

struct GPUPresentationBuffer {
    global: Arc<Global>,
    buffer_id: id::BufferId,
    data: NonNull<u8>,
    size: usize,
}

// This is safe because `GPUPresentationBuffer` holds exclusive access to ptr
unsafe impl Send for GPUPresentationBuffer {}
unsafe impl Sync for GPUPresentationBuffer {}

impl GPUPresentationBuffer {
    fn new(global: Arc<Global>, buffer_id: id::BufferId, buffer_size: u64) -> Self {
        let (data, size) = global
            .buffer_get_mapped_range(buffer_id, 0, Some(buffer_size))
            .unwrap();
        GPUPresentationBuffer {
            global,
            buffer_id,
            data,
            size: size as usize,
        }
    }

    fn slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data.as_ptr(), self.size) }
    }
}

impl Drop for GPUPresentationBuffer {
    fn drop(&mut self) {
        let _ = self.global.buffer_unmap(self.buffer_id);
    }
}

#[derive(Default)]
pub struct WGPUExternalImages {
    pub images: WGPUImageMap,
    pub locked_ids: HashMap<WebGPUContextId, Vec<u8>>,
}

impl WebrenderExternalImageApi for WGPUExternalImages {
    fn lock(&mut self, id: u64) -> (WebrenderImageSource, Size2D<i32>) {
        let id = WebGPUContextId(id);
        let size;
        let data;
        if let Some(present_data) = self.images.lock().unwrap().get(&id) {
            size = present_data.image_desc.size.cast_unit();
            data = if let Some(present_data) = &present_data.data {
                present_data.slice().to_vec()
            } else {
                present_data.dummy_data()
            };
        } else {
            size = Size2D::new(0, 0);
            data = Vec::new();
        }
        let _ = self.locked_ids.insert(id, data);
        (
            WebrenderImageSource::Raw(self.locked_ids.get(&id).unwrap().as_slice()),
            size,
        )
    }

    fn unlock(&mut self, id: u64) {
        let id = WebGPUContextId(id);
        let _ = self.locked_ids.remove(&id);
    }
}

pub struct PresentationData {
    device_id: id::DeviceId,
    queue_id: id::QueueId,
    data: Option<GPUPresentationBuffer>,
    unassigned_buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    available_buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    queued_buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    image_key: ImageKey,
    image_desc: ImageDescriptor,
    image_data: ImageData,
}

impl PresentationData {
    pub fn new(
        device_id: id::DeviceId,
        queue_id: id::QueueId,
        buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
        image_key: ImageKey,
        image_desc: ImageDescriptor,
        image_data: ImageData,
    ) -> Self {
        Self {
            device_id,
            queue_id,
            data: None,
            unassigned_buffer_ids: buffer_ids,
            available_buffer_ids: ArrayVec::<id::BufferId, PRESENTATION_BUFFER_COUNT>::new(),
            queued_buffer_ids: ArrayVec::<id::BufferId, PRESENTATION_BUFFER_COUNT>::new(),
            image_key,
            image_desc,
            image_data,
        }
    }

    fn dummy_data(&self) -> Vec<u8> {
        let size = (self
            .image_desc
            .stride
            .expect("Stride should be set when creating swapchain") *
            self.image_desc.size.height) as usize;
        vec![0; size]
    }

    fn unmap_old_buffer(&mut self, presentation_buffer: GPUPresentationBuffer) {
        self.queued_buffer_ids
            .retain(|b_id| *b_id != presentation_buffer.buffer_id);
        self.available_buffer_ids
            .push(presentation_buffer.buffer_id);
        drop(presentation_buffer);
    }
}

impl crate::WGPU {
    pub(crate) fn create_swapchain(
        &self,
        device_id: id::DeviceId,
        queue_id: id::QueueId,
        buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
        context_id: WebGPUContextId,
        format: ImageFormat,
        size: DeviceIntSize,
        image_key: ImageKey,
        mut wr: MutexGuard<RenderApi>,
    ) {
        let image_desc = ImageDescriptor {
            format,
            size,
            stride: Some(
                (((size.width as u32 * 4) | (wgt::COPY_BYTES_PER_ROW_ALIGNMENT - 1)) + 1) as i32,
            ),
            offset: 0,
            flags: ImageDescriptorFlags::IS_OPAQUE,
        };

        let image_data = ImageData::External(ExternalImageData {
            id: ExternalImageId(context_id.0),
            channel_index: 0,
            image_type: ExternalImageType::Buffer,
        });
        let _ = self.wgpu_image_map.lock().unwrap().insert(
            context_id,
            PresentationData::new(
                device_id,
                queue_id,
                buffer_ids,
                image_key,
                image_desc,
                image_data.clone(),
            ),
        );

        let mut txn = Transaction::new();
        txn.add_image(image_key, image_desc, image_data, None);
        wr.send_transaction(self.webrender_document, txn);
    }

    pub(crate) fn swapchain_present(
        &mut self,
        context_id: WebGPUContextId,
        encoder_id: id::Id<id::markers::CommandEncoder>,
        texture_id: id::Id<id::markers::Texture>,
    ) -> ControlFlow<()> {
        let global = &self.global;
        let device_id;
        let queue_id;
        let size;
        let buffer_id;
        let buffer_stride;
        {
            if let Some(present_data) = self.wgpu_image_map.lock().unwrap().get_mut(&context_id) {
                size = present_data.image_desc.size;
                device_id = present_data.device_id;
                queue_id = present_data.queue_id;
                buffer_stride = present_data
                    .image_desc
                    .stride
                    .expect("Stride should be set when creating swapchain");
                buffer_id = if let Some(b_id) = present_data.available_buffer_ids.pop() {
                    b_id
                } else if let Some(b_id) = present_data.unassigned_buffer_ids.pop() {
                    let buffer_size = (buffer_stride * size.height) as wgt::BufferAddress;
                    let buffer_desc = wgt::BufferDescriptor {
                        label: None,
                        size: buffer_size,
                        usage: wgt::BufferUsages::MAP_READ | wgt::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    };
                    let _ = global.device_create_buffer(device_id, &buffer_desc, Some(b_id));
                    b_id
                } else {
                    error!("No staging buffer available for {:?}", context_id);
                    return ControlFlow::Break(());
                };
                present_data.queued_buffer_ids.push(buffer_id);
            } else {
                error!("Data not found for {:?}", context_id);
                return ControlFlow::Break(());
            }
        }
        let buffer_size = (size.height * buffer_stride) as wgt::BufferAddress;
        let comm_desc = wgt::CommandEncoderDescriptor { label: None };
        let _ = global.device_create_command_encoder(device_id, &comm_desc, Some(encoder_id));
        let buffer_cv = wgt::ImageCopyBuffer {
            buffer: buffer_id,
            layout: wgt::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(buffer_stride as u32),
                rows_per_image: None,
            },
        };
        let texture_cv = wgt::ImageCopyTexture {
            texture: texture_id,
            mip_level: 0,
            origin: wgt::Origin3d::ZERO,
            aspect: wgt::TextureAspect::All,
        };
        let copy_size = wgt::Extent3d {
            width: size.width as u32,
            height: size.height as u32,
            depth_or_array_layers: 1,
        };
        let _ = global.command_encoder_copy_texture_to_buffer(
            encoder_id,
            &texture_cv,
            &buffer_cv,
            &copy_size,
        );
        let _ = global.command_encoder_finish(encoder_id, &wgt::CommandBufferDescriptor::default());
        let _ = global.queue_submit(queue_id, &[encoder_id.into_command_buffer_id()]);
        let callback = {
            let global = Arc::clone(&self.global);
            let wgpu_image_map = Arc::clone(&self.wgpu_image_map);
            let webrender_api = Arc::clone(&self.webrender_api);
            let webrender_document = self.webrender_document;
            let token = self.poller.token();
            BufferMapCallback::from_rust(Box::from(move |result| {
                drop(token);
                update_wr_image(
                    result,
                    global,
                    buffer_id,
                    buffer_size,
                    wgpu_image_map,
                    context_id,
                    webrender_api,
                    webrender_document,
                );
            }))
        };
        let map_op = BufferMapOperation {
            host: HostMap::Read,
            callback: Some(callback),
        };
        let _ = global.buffer_map_async(buffer_id, 0, Some(buffer_size), map_op);
        self.poller.wake();

        ControlFlow::Continue(())
    }

    pub(crate) fn destroy_swapchain(
        &mut self,
        context_id: WebGPUContextId,
        image_key: webrender_api::ImageKey,
    ) {
        let data = self
            .wgpu_image_map
            .lock()
            .unwrap()
            .remove(&context_id)
            .unwrap();
        let global = &self.global;
        for b_id in data.available_buffer_ids.iter() {
            global.buffer_drop(*b_id);
        }
        for b_id in data.queued_buffer_ids.iter() {
            global.buffer_drop(*b_id);
        }
        for b_id in data.unassigned_buffer_ids.iter() {
            if let Err(e) = self.script_sender.send(WebGPUMsg::FreeBuffer(*b_id)) {
                warn!("Unable to send FreeBuffer({:?}) ({:?})", *b_id, e);
            };
        }
        let mut txn = Transaction::new();
        txn.delete_image(image_key);
        self.webrender_api
            .lock()
            .unwrap()
            .send_transaction(self.webrender_document, txn);
    }
}

fn update_wr_image(
    result: Result<(), BufferAccessError>,
    global: Arc<Global>,
    buffer_id: id::BufferId,
    buffer_size: u64,
    wgpu_image_map: WGPUImageMap,
    context_id: WebGPUContextId,
    webrender_api: Arc<Mutex<RenderApi>>,
    webrender_document: webrender_api::DocumentId,
) {
    match result {
        Ok(()) => {
            if let Some(present_data) = wgpu_image_map.lock().unwrap().get_mut(&context_id) {
                let presentation_buffer =
                    GPUPresentationBuffer::new(global, buffer_id, buffer_size);
                let old_presentation_buffer = present_data.data.replace(presentation_buffer);
                let mut txn = Transaction::new();
                txn.update_image(
                    present_data.image_key,
                    present_data.image_desc,
                    present_data.image_data.clone(),
                    &DirtyRect::All,
                );
                webrender_api
                    .lock()
                    .unwrap()
                    .send_transaction(webrender_document, txn);
                if let Some(old_presentation_buffer) = old_presentation_buffer {
                    present_data.unmap_old_buffer(old_presentation_buffer)
                }
            } else {
                error!("Data not found for {:?}", context_id);
            }
        },
        _ => error!("Could not map buffer({:?})", buffer_id),
    }
}
