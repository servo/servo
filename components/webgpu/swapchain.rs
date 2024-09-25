/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::ptr::NonNull;
use std::slice;
use std::sync::{Arc, Mutex};

use arrayvec::ArrayVec;
use euclid::default::Size2D;
use ipc_channel::ipc::IpcSender;
use log::{error, warn};
use malloc_size_of::MallocSizeOf;
use serde::{Deserialize, Serialize};
use webrender::{RenderApi, Transaction};
use webrender_api::units::DeviceIntSize;
use webrender_api::{
    DirtyRect, DocumentId, ExternalImageData, ExternalImageId, ExternalImageType, ImageData,
    ImageDescriptor, ImageDescriptorFlags, ImageFormat, ImageKey,
};
use webrender_traits::{WebrenderExternalImageApi, WebrenderImageSource};
use wgpu_core::device::HostMap;
use wgpu_core::global::Global;
use wgpu_core::id;
use wgpu_core::resource::{BufferAccessError, BufferMapCallback, BufferMapOperation};

use crate::{wgt, ContextConfiguration, Error, WebGPUMsg};

pub const PRESENTATION_BUFFER_COUNT: usize = 10;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct WebGPUContextId(pub u64);

impl MallocSizeOf for WebGPUContextId {
    fn size_of(&self, _ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        0
    }
}

pub type WGPUImageMap = Arc<Mutex<HashMap<WebGPUContextId, ContextData>>>;

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
        let webgpu_contexts = self.images.lock().unwrap();
        let context_data = webgpu_contexts.get(&id).unwrap();
        let size = context_data.image_desc.size.cast_unit();
        let data = if let Some(present_buffer) = context_data
            .swap_chain
            .as_ref()
            .map(|swap_chain| swap_chain.data.as_ref())
            .flatten()
        {
            present_buffer.slice().to_vec()
        } else {
            context_data.dummy_data()
        };
        self.locked_ids.insert(id, data);
        (
            WebrenderImageSource::Raw(self.locked_ids.get(&id).unwrap().as_slice()),
            size,
        )
    }

    fn unlock(&mut self, id: u64) {
        let id = WebGPUContextId(id);
        self.locked_ids.remove(&id);
    }
}

/// States of presentation buffer
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
enum PresentationBufferState {
    /// Initial state, buffer has yet to be created,
    /// only its id is reserved
    #[default]
    Unassigned,
    /// Buffer is already created and ready to be used immediately
    Available,
    /// Buffer is currently running mapAsync
    Mapping,
    /// Buffer is currently actively mapped to be used by wr
    Mapped,
}

struct SwapChain {
    device_id: id::DeviceId,
    queue_id: id::QueueId,
    data: Option<GPUPresentationBuffer>,
}

pub struct ContextData {
    image_key: ImageKey,
    image_desc: ImageDescriptor,
    image_data: ImageData,
    buffer_ids: ArrayVec<(id::BufferId, PresentationBufferState), PRESENTATION_BUFFER_COUNT>,
    /// If there is no associated swapchain the context is dummy (transparent black)
    swap_chain: Option<SwapChain>,
}

fn stride(size: DeviceIntSize) -> i32 {
    (((size.width as u32 * 4) | (wgt::COPY_BYTES_PER_ROW_ALIGNMENT - 1)) + 1) as i32
}

impl ContextData {
    /// Init ContextData as dummy (transparent black)
    fn new(
        context_id: WebGPUContextId,
        image_key: ImageKey,
        size: DeviceIntSize,
        buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    ) -> Self {
        let image_desc = ImageDescriptor {
            format: ImageFormat::RGBA8,
            size,
            stride: Some(stride(size)),
            offset: 0,
            flags: ImageDescriptorFlags::empty(),
        };

        let image_data = ImageData::External(ExternalImageData {
            id: ExternalImageId(context_id.0),
            channel_index: 0,
            image_type: ExternalImageType::Buffer,
        });

        Self {
            image_key,
            image_desc,
            image_data,
            swap_chain: None,
            buffer_ids: buffer_ids
                .iter()
                .map(|&buffer_id| (buffer_id, PresentationBufferState::Unassigned))
                .collect(),
        }
    }

    /// Returns true if needs image reload
    fn update_image_desc(
        &mut self,
        size: DeviceIntSize,
        format: ImageFormat,
        is_opaque: bool,
    ) -> bool {
        let flags = if is_opaque {
            ImageDescriptorFlags::IS_OPAQUE
        } else {
            ImageDescriptorFlags::empty()
        };

        /// Returns true if property changed
        fn set<T: std::cmp::PartialEq>(t: &mut T, new: T) -> bool {
            if *t != new {
                *t = new;
                true
            } else {
                false
            }
        }

        set(&mut self.image_desc.size, size) ||
            set(&mut self.image_desc.stride, Some(stride(size))) ||
            set(&mut self.image_desc.format, format) ||
            set(&mut self.image_desc.flags, flags)
    }

    fn dummy_data(&self) -> Vec<u8> {
        vec![0; self.buffer_size() as usize]
    }

    fn buffer_stride(&self) -> i32 {
        self.image_desc
            .stride
            .expect("Stride should be set when creating swapchain")
    }

    fn buffer_size(&self) -> wgt::BufferAddress {
        (self.buffer_stride() * self.image_desc.size.height) as wgt::BufferAddress
    }

    /// Returns id of available buffer
    /// and sets state to PresentationBufferState::Mapping
    fn get_available_buffer(&'_ mut self, global: &Arc<Global>) -> Option<id::BufferId> {
        assert!(self.swap_chain.is_some());
        if let Some((buffer_id, buffer_state)) = self
            .buffer_ids
            .iter_mut()
            .find(|(_, state)| *state == PresentationBufferState::Available)
        {
            *buffer_state = PresentationBufferState::Mapping;
            Some(*buffer_id)
        } else if let Some((buffer_id, buffer_state)) = self
            .buffer_ids
            .iter_mut()
            .find(|(_, state)| *state == PresentationBufferState::Unassigned)
        {
            *buffer_state = PresentationBufferState::Mapping;
            let buffer_id = *buffer_id;
            let buffer_desc = wgt::BufferDescriptor {
                label: None,
                size: self.buffer_size(),
                usage: wgt::BufferUsages::MAP_READ | wgt::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            };
            let _ = global.device_create_buffer(
                self.swap_chain.as_ref().unwrap().device_id,
                &buffer_desc,
                Some(buffer_id),
            );
            Some(buffer_id)
        } else {
            None
        }
    }

    fn get_buffer_state(&mut self, buffer_id: id::BufferId) -> &mut PresentationBufferState {
        &mut self
            .buffer_ids
            .iter_mut()
            .find(|(id, _)| *id == buffer_id)
            .expect("Presentation buffer should have associated state")
            .1
    }

    fn unmap_old_buffer(&mut self, presentation_buffer: GPUPresentationBuffer) {
        assert!(self.swap_chain.is_some());
        let buffer_state = self.get_buffer_state(presentation_buffer.buffer_id);
        assert_eq!(*buffer_state, PresentationBufferState::Mapped);
        *buffer_state = PresentationBufferState::Available;
        drop(presentation_buffer);
    }

    /// Returns true if needs image update
    fn destroy_swapchain(&mut self, global: &Arc<Global>) -> bool {
        drop(self.swap_chain.take());
        // free all buffers
        for (buffer_id, buffer_state) in &mut self.buffer_ids {
            match buffer_state {
                PresentationBufferState::Unassigned => {
                    /* These buffer were not yet created in wgpu */
                },
                _ => {
                    global.buffer_drop(*buffer_id);
                },
            }
            *buffer_state = PresentationBufferState::Unassigned;
        }
        // set defaults back
        self.update_image_desc(self.image_desc.size, ImageFormat::RGBA8, false)
    }

    fn destroy(
        mut self,
        global: &Arc<Global>,
        script_sender: &IpcSender<WebGPUMsg>,
        webrender_api: &Arc<Mutex<RenderApi>>,
        webrender_document: DocumentId,
    ) {
        self.destroy_swapchain(global);
        for (buffer_id, _) in self.buffer_ids {
            if let Err(e) = script_sender.send(WebGPUMsg::FreeBuffer(buffer_id)) {
                warn!("Unable to send FreeBuffer({:?}) ({:?})", buffer_id, e);
            };
        }
        let mut txn = Transaction::new();
        txn.delete_image(self.image_key);
        webrender_api
            .lock()
            .unwrap()
            .send_transaction(webrender_document, txn);
    }
}

impl crate::WGPU {
    pub(crate) fn create_context(
        &self,
        context_id: WebGPUContextId,
        image_key: ImageKey,
        size: DeviceIntSize,
        buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    ) {
        let context_data = ContextData::new(context_id, image_key, size, buffer_ids);
        let mut txn = Transaction::new();
        txn.add_image(
            image_key,
            context_data.image_desc,
            context_data.image_data.clone(),
            None,
        );
        self.webrender_api
            .lock()
            .unwrap()
            .send_transaction(self.webrender_document, txn);
        assert!(
            self.wgpu_image_map
                .lock()
                .unwrap()
                .insert(context_id, context_data)
                .is_none(),
            "Context should be created only once!"
        );
    }

    pub(crate) fn update_context(
        &self,
        context_id: WebGPUContextId,
        size: DeviceIntSize,
        config: Option<ContextConfiguration>,
    ) {
        let mut webgpu_contexts = self.wgpu_image_map.lock().unwrap();
        let context_data = webgpu_contexts.get_mut(&context_id).unwrap();

        // If configuration is not provided or presentation format is not valid
        // the context will be dummy until recreation
        let format = config
            .as_ref()
            .map(|config| match config.format {
                wgt::TextureFormat::Rgba8Unorm => Some(ImageFormat::RGBA8),
                wgt::TextureFormat::Bgra8Unorm => Some(ImageFormat::BGRA8),
                _ => None,
            })
            .flatten();

        let needs_image_update = if let Some(format) = format {
            let config = config.expect("Config should exist when valid format is available");
            let needs_rebuild = context_data.swap_chain.is_none() ||
                context_data.image_desc.size != size ||
                context_data.image_desc.is_opaque() != config.is_opaque ||
                context_data.image_desc.format != format;
            if needs_rebuild {
                context_data.destroy_swapchain(&self.global);
                context_data.update_image_desc(size, format, config.is_opaque);
                context_data.swap_chain = Some(SwapChain {
                    device_id: config.device_id,
                    queue_id: config.queue_id,
                    data: None,
                });
                true
            } else {
                false
            }
        } else {
            context_data.destroy_swapchain(&self.global)
        };

        if needs_image_update {
            let mut txn = Transaction::new();
            txn.update_image(
                context_data.image_key,
                context_data.image_desc,
                context_data.image_data.clone(),
                &DirtyRect::All,
            );
            self.webrender_api
                .lock()
                .unwrap()
                .send_transaction(self.webrender_document, txn);
        }
    }

    /// Copies data async from provided texture using encoder_id to available staging presentation buffer
    pub(crate) fn swapchain_present(
        &mut self,
        context_id: WebGPUContextId,
        encoder_id: id::Id<id::markers::CommandEncoder>,
        texture_id: id::Id<id::markers::Texture>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        fn err<T: std::error::Error + 'static>(e: Option<T>) -> Result<(), T> {
            if let Some(error) = e {
                Err(error)
            } else {
                Ok(())
            }
        }

        let global = &self.global;
        let device_id;
        let queue_id;
        let size;
        let buffer_id;
        let buffer_stride;
        let buffer_size;
        let image_desc;
        {
            if let Some(context_data) = self.wgpu_image_map.lock().unwrap().get_mut(&context_id) {
                let Some(swap_chain) = context_data.swap_chain.as_ref() else {
                    return Ok(());
                };
                size = context_data.image_desc.size;
                device_id = swap_chain.device_id;
                queue_id = swap_chain.queue_id;
                buffer_stride = context_data.buffer_stride();
                buffer_size = context_data.buffer_size();
                buffer_id = context_data.get_available_buffer(global).unwrap();
                image_desc = context_data.image_desc.clone();
            } else {
                return Ok(());
            }
        }
        let comm_desc = wgt::CommandEncoderDescriptor { label: None };
        let (encoder_id, error) =
            global.device_create_command_encoder(device_id, &comm_desc, Some(encoder_id));
        err(error)?;
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
        global.command_encoder_copy_texture_to_buffer(
            encoder_id,
            &texture_cv,
            &buffer_cv,
            &copy_size,
        )?;
        let (command_buffer_id, error) =
            global.command_encoder_finish(encoder_id, &wgt::CommandBufferDescriptor::default());
        err(error)?;
        {
            let _guard = self.poller.lock();
            global
                .queue_submit(queue_id, &[command_buffer_id])
                .map_err(Error::from_error)?;
        }
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
                    image_desc,
                );
            }))
        };
        let map_op = BufferMapOperation {
            host: HostMap::Read,
            callback: Some(callback),
        };
        global.buffer_map_async(buffer_id, 0, Some(buffer_size), map_op)?;
        self.poller.wake();
        Ok(())
    }

    pub(crate) fn destroy_context(&mut self, context_id: WebGPUContextId) {
        self.wgpu_image_map
            .lock()
            .unwrap()
            .remove(&context_id)
            .unwrap()
            .destroy(
                &self.global,
                &self.script_sender,
                &self.webrender_api,
                self.webrender_document,
            );
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
    image_desc: ImageDescriptor,
) {
    match result {
        Ok(()) => {
            if let Some(context_data) = wgpu_image_map.lock().unwrap().get_mut(&context_id) {
                let config_changed = image_desc != context_data.image_desc;
                let buffer_state = context_data.get_buffer_state(buffer_id);
                assert_eq!(*buffer_state, PresentationBufferState::Mapping);
                if config_changed {
                    /*
                    This means that while mapasync was running, context got recreated
                    so we need to throw all out work away.

                    It is also possible that we got recreated with same config,
                    so canvas should be cleared, but we handle such case in gpucanvascontext
                    with drawing_buffer.cleared

                    One more case is that we already have newer map async done,
                    so we can replace new image with old image but that should happen very rarely

                    One possible solution to all problems is blocking device timeline
                    (wgpu thread or introduce new timeline/thread for presentation)
                    something like this is also mentioned in spec:

                    2. Ensure that all submitted work items (e.g. queue submissions) have completed writing to the image
                    https://gpuweb.github.io/gpuweb/#abstract-opdef-get-a-copy-of-the-image-contents-of-a-context
                    */
                    let _ = global.buffer_unmap(buffer_id);
                    *buffer_state = PresentationBufferState::Available;
                    return;
                }
                *buffer_state = PresentationBufferState::Mapped;
                let presentation_buffer =
                    GPUPresentationBuffer::new(global, buffer_id, buffer_size);
                let Some(swap_chain) = context_data.swap_chain.as_mut() else {
                    return;
                };
                let old_presentation_buffer = swap_chain.data.replace(presentation_buffer);
                let mut txn = Transaction::new();
                txn.update_image(
                    context_data.image_key,
                    context_data.image_desc,
                    context_data.image_data.clone(),
                    &DirtyRect::All,
                );
                webrender_api
                    .lock()
                    .unwrap()
                    .send_transaction(webrender_document, txn);
                if let Some(old_presentation_buffer) = old_presentation_buffer {
                    context_data.unmap_old_buffer(old_presentation_buffer)
                }
            } else {
                error!("WebGPU Context {:?} is destroyed", context_id);
            }
        },
        _ => error!("Could not map buffer({:?})", buffer_id),
    }
}
