/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use arrayvec::ArrayVec;
use euclid::default::Size2D;
use malloc_size_of::MallocSizeOf;
use serde::{Deserialize, Serialize};
use webrender::{RenderApi, Transaction};
use webrender_api::units::DeviceIntSize;
use webrender_api::{
    ExternalImageData, ExternalImageId, ExternalImageType, ImageData, ImageDescriptor,
    ImageDescriptorFlags, ImageFormat, ImageKey,
};
use webrender_traits::{WebrenderExternalImageApi, WebrenderImageSource};
use wgpu_core::id;

use crate::wgt;

pub const PRESENTATION_BUFFER_COUNT: usize = 10;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct WebGPUContextId(pub u64);

impl MallocSizeOf for WebGPUContextId {
    fn size_of(&self, _ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        0
    }
}

pub type WGPUImageMap = Arc<Mutex<HashMap<WebGPUContextId, PresentationData>>>;

#[derive(Default)]
pub struct WGPUExternalImages {
    pub images: Arc<Mutex<HashMap<WebGPUContextId, PresentationData>>>,
    pub locked_ids: HashMap<WebGPUContextId, Vec<u8>>,
}

impl WebrenderExternalImageApi for WGPUExternalImages {
    fn lock(&mut self, id: u64) -> (WebrenderImageSource, Size2D<i32>) {
        let id = WebGPUContextId(id);
        let size;
        let data;
        if let Some(present_data) = self.images.lock().unwrap().get(&id) {
            size = present_data.image_desc.size.cast_unit();
            data = present_data.data.clone();
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
    pub device_id: id::DeviceId,
    pub queue_id: id::QueueId,
    pub data: Vec<u8>,
    pub unassigned_buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    pub available_buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    pub queued_buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    pub image_key: ImageKey,
    pub image_desc: ImageDescriptor,
    pub image_data: ImageData,
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
        let height = image_desc.size.height;
        Self {
            device_id,
            queue_id,
            // TODO: transparent black image
            data: vec![
                255;
                (image_desc
                    .stride
                    .expect("Stride should be set when creating swapchain") *
                    height) as usize
            ],
            unassigned_buffer_ids: buffer_ids,
            available_buffer_ids: ArrayVec::<id::BufferId, PRESENTATION_BUFFER_COUNT>::new(),
            queued_buffer_ids: ArrayVec::<id::BufferId, PRESENTATION_BUFFER_COUNT>::new(),
            image_key,
            image_desc,
            image_data,
        }
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
}
