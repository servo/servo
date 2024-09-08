use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use arrayvec::ArrayVec;
use euclid::default::Size2D;
use webrender_api::{ImageData, ImageDescriptor, ImageKey};
use webrender_traits::{WebrenderExternalImageApi, WebrenderImageSource};
use wgpu_core::id;

pub const PRESENTATION_BUFFER_COUNT: usize = 10;

#[derive(Default)]
pub struct WGPUExternalImages {
    pub images: Arc<Mutex<HashMap<u64, PresentationData>>>,
    pub locked_ids: HashMap<u64, Vec<u8>>,
}

impl WebrenderExternalImageApi for WGPUExternalImages {
    fn lock(&mut self, id: u64) -> (WebrenderImageSource, Size2D<i32>) {
        let size;
        let data;
        if let Some(present_data) = self.images.lock().unwrap().get(&id) {
            size = present_data.size;
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
        let _ = self.locked_ids.remove(&id);
    }
}

pub struct PresentationData {
    pub device_id: id::DeviceId,
    pub queue_id: id::QueueId,
    pub data: Vec<u8>,
    pub size: Size2D<i32>,
    pub unassigned_buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    pub available_buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    pub queued_buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    pub buffer_stride: u32,
    pub image_key: ImageKey,
    pub image_desc: ImageDescriptor,
    pub image_data: ImageData,
}
