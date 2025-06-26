/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::ptr::NonNull;
use std::slice;
use std::sync::{Arc, Mutex};

use arrayvec::ArrayVec;
use compositing_traits::{
    CrossProcessCompositorApi, SerializableImageData, WebrenderExternalImageApi,
    WebrenderImageSource,
};
use euclid::default::Size2D;
use ipc_channel::ipc::IpcSender;
use log::{error, warn};
use pixels::{IpcSnapshot, Snapshot, SnapshotAlphaMode, SnapshotPixelFormat};
use serde::{Deserialize, Serialize};
use webgpu_traits::{
    ContextConfiguration, Error, PRESENTATION_BUFFER_COUNT, WebGPUContextId, WebGPUMsg,
};
use webrender_api::units::DeviceIntSize;
use webrender_api::{
    ExternalImageData, ExternalImageId, ExternalImageType, ImageDescriptor, ImageDescriptorFlags,
    ImageFormat, ImageKey,
};
use wgpu_core::device::HostMap;
use wgpu_core::global::Global;
use wgpu_core::id;
use wgpu_core::resource::{BufferAccessError, BufferMapOperation};

use crate::wgt;

const DEFAULT_IMAGE_FORMAT: ImageFormat = ImageFormat::RGBA8;

pub type WGPUImageMap = Arc<Mutex<HashMap<WebGPUContextId, ContextData>>>;

/// Presentation id encodes current configuration and current image
/// so that async presentation does not update context with older data
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
struct PresentationId(u64);

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
        let size = context_data.image_desc.size().cast_unit();
        let data = if let Some(present_buffer) = context_data
            .swap_chain
            .as_ref()
            .and_then(|swap_chain| swap_chain.data.as_ref())
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WebGPUImageDescriptor(pub ImageDescriptor);

impl WebGPUImageDescriptor {
    fn new(format: ImageFormat, size: DeviceIntSize, is_opaque: bool) -> Self {
        let stride = ((size.width * format.bytes_per_pixel()) |
            (wgt::COPY_BYTES_PER_ROW_ALIGNMENT as i32 - 1)) +
            1;
        Self(ImageDescriptor {
            format,
            size,
            stride: Some(stride),
            offset: 0,
            flags: if is_opaque {
                ImageDescriptorFlags::IS_OPAQUE
            } else {
                ImageDescriptorFlags::empty()
            },
        })
    }

    fn default(size: DeviceIntSize) -> Self {
        Self::new(DEFAULT_IMAGE_FORMAT, size, false)
    }

    /// Returns true if needs image update (if it's changed)
    fn update(&mut self, new: Self) -> bool {
        if self.0 != new.0 {
            self.0 = new.0;
            true
        } else {
            false
        }
    }

    fn buffer_stride(&self) -> i32 {
        self.0
            .stride
            .expect("Stride should be set by WebGPUImageDescriptor")
    }

    fn buffer_size(&self) -> wgt::BufferAddress {
        (self.buffer_stride() * self.0.size.height) as wgt::BufferAddress
    }

    fn size(&self) -> DeviceIntSize {
        self.0.size
    }
}

pub struct ContextData {
    image_key: ImageKey,
    image_desc: WebGPUImageDescriptor,
    image_data: ExternalImageData,
    buffer_ids: ArrayVec<(id::BufferId, PresentationBufferState), PRESENTATION_BUFFER_COUNT>,
    /// If there is no associated swapchain the context is dummy (transparent black)
    swap_chain: Option<SwapChain>,
    /// Next presentation id to be returned
    next_presentation_id: PresentationId,
    /// Current id that is presented/configured
    ///
    /// This value only grows
    current_presentation_id: PresentationId,
}

impl ContextData {
    /// Init ContextData as dummy (transparent black)
    fn new(
        context_id: WebGPUContextId,
        image_key: ImageKey,
        size: DeviceIntSize,
        buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    ) -> Self {
        let image_data = ExternalImageData {
            id: ExternalImageId(context_id.0),
            channel_index: 0,
            image_type: ExternalImageType::Buffer,
            normalized_uvs: false,
        };

        Self {
            image_key,
            image_desc: WebGPUImageDescriptor::default(size),
            image_data,
            swap_chain: None,
            buffer_ids: buffer_ids
                .iter()
                .map(|&buffer_id| (buffer_id, PresentationBufferState::Unassigned))
                .collect(),
            current_presentation_id: PresentationId(0),
            next_presentation_id: PresentationId(1),
        }
    }

    fn dummy_data(&self) -> Vec<u8> {
        vec![0; self.image_desc.buffer_size() as usize]
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
                size: self.image_desc.buffer_size(),
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
            error!("No available presentation buffer: {:?}", self.buffer_ids);
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

    fn destroy_swapchain(&mut self, global: &Arc<Global>) {
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
    }

    fn destroy(
        mut self,
        global: &Arc<Global>,
        script_sender: &IpcSender<WebGPUMsg>,
        compositor_api: &CrossProcessCompositorApi,
    ) {
        self.destroy_swapchain(global);
        for (buffer_id, _) in self.buffer_ids {
            if let Err(e) = script_sender.send(WebGPUMsg::FreeBuffer(buffer_id)) {
                warn!("Unable to send FreeBuffer({:?}) ({:?})", buffer_id, e);
            };
        }
        compositor_api.delete_image(self.image_key);
    }

    /// Returns true if presentation id was updated (was newer)
    fn check_and_update_presentation_id(&mut self, presentation_id: PresentationId) -> bool {
        if presentation_id > self.current_presentation_id {
            self.current_presentation_id = presentation_id;
            true
        } else {
            false
        }
    }

    /// Returns new presentation id
    fn next_presentation_id(&mut self) -> PresentationId {
        let res = PresentationId(self.next_presentation_id.0);
        self.next_presentation_id.0 += 1;
        res
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
        self.compositor_api.add_image(
            image_key,
            context_data.image_desc.0,
            SerializableImageData::External(context_data.image_data),
        );
        assert!(
            self.wgpu_image_map
                .lock()
                .unwrap()
                .insert(context_id, context_data)
                .is_none(),
            "Context should be created only once!"
        );
    }

    pub(crate) fn get_image(&self, context_id: WebGPUContextId) -> IpcSnapshot {
        let webgpu_contexts = self.wgpu_image_map.lock().unwrap();
        let context_data = webgpu_contexts.get(&context_id).unwrap();
        let size = context_data.image_desc.size().cast().cast_unit();
        let data = if let Some(present_buffer) = context_data
            .swap_chain
            .as_ref()
            .and_then(|swap_chain| swap_chain.data.as_ref())
        {
            let format = match context_data.image_desc.0.format {
                ImageFormat::RGBA8 => SnapshotPixelFormat::RGBA,
                ImageFormat::BGRA8 => SnapshotPixelFormat::BGRA,
                _ => unimplemented!(),
            };
            let alpha_mode = if context_data.image_desc.0.is_opaque() {
                SnapshotAlphaMode::AsOpaque {
                    premultiplied: false,
                }
            } else {
                SnapshotAlphaMode::Transparent {
                    premultiplied: true,
                }
            };
            Snapshot::from_vec(size, format, alpha_mode, present_buffer.slice().to_vec())
        } else {
            Snapshot::cleared(size)
        };
        data.as_ipc()
    }

    pub(crate) fn update_context(
        &self,
        context_id: WebGPUContextId,
        size: DeviceIntSize,
        config: Option<ContextConfiguration>,
    ) {
        let mut webgpu_contexts = self.wgpu_image_map.lock().unwrap();
        let context_data = webgpu_contexts.get_mut(&context_id).unwrap();

        let presentation_id = context_data.next_presentation_id();
        context_data.check_and_update_presentation_id(presentation_id);

        // If configuration is not provided
        // the context will be dummy/empty until recreation
        let needs_image_update = if let Some(config) = config {
            let new_image_desc =
                WebGPUImageDescriptor::new(config.format(), size, config.is_opaque);
            let needs_swapchain_rebuild = context_data.swap_chain.is_none() ||
                new_image_desc.buffer_size() != context_data.image_desc.buffer_size();
            if needs_swapchain_rebuild {
                context_data.destroy_swapchain(&self.global);
                context_data.swap_chain = Some(SwapChain {
                    device_id: config.device_id,
                    queue_id: config.queue_id,
                    data: None,
                });
            }
            context_data.image_desc.update(new_image_desc)
        } else {
            context_data.destroy_swapchain(&self.global);
            context_data
                .image_desc
                .update(WebGPUImageDescriptor::default(size))
        };

        if needs_image_update {
            self.compositor_api.update_image(
                context_data.image_key,
                context_data.image_desc.0,
                SerializableImageData::External(context_data.image_data),
            );
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
        let buffer_id;
        let image_desc;
        let presentation_id;
        {
            if let Some(context_data) = self.wgpu_image_map.lock().unwrap().get_mut(&context_id) {
                let Some(swap_chain) = context_data.swap_chain.as_ref() else {
                    return Ok(());
                };
                device_id = swap_chain.device_id;
                queue_id = swap_chain.queue_id;
                buffer_id = context_data.get_available_buffer(global).unwrap();
                image_desc = context_data.image_desc;
                presentation_id = context_data.next_presentation_id();
            } else {
                return Ok(());
            }
        }
        let comm_desc = wgt::CommandEncoderDescriptor { label: None };
        let (encoder_id, error) =
            global.device_create_command_encoder(device_id, &comm_desc, Some(encoder_id));
        err(error)?;
        let buffer_cv = wgt::TexelCopyBufferInfo {
            buffer: buffer_id,
            layout: wgt::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image_desc.buffer_stride() as u32),
                rows_per_image: None,
            },
        };
        let texture_cv = wgt::TexelCopyTextureInfo {
            texture: texture_id,
            mip_level: 0,
            origin: wgt::Origin3d::ZERO,
            aspect: wgt::TextureAspect::All,
        };
        let copy_size = wgt::Extent3d {
            width: image_desc.size().width as u32,
            height: image_desc.size().height as u32,
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
                .map_err(|(_, error)| Error::from_error(error))?;
        }
        let callback = {
            let global = Arc::clone(&self.global);
            let wgpu_image_map = Arc::clone(&self.wgpu_image_map);
            let compositor_api = self.compositor_api.clone();
            let token = self.poller.token();
            Box::new(move |result| {
                drop(token);
                update_wr_image(
                    result,
                    global,
                    buffer_id,
                    wgpu_image_map,
                    context_id,
                    compositor_api,
                    image_desc,
                    presentation_id,
                );
            })
        };
        let map_op = BufferMapOperation {
            host: HostMap::Read,
            callback: Some(callback),
        };
        global.buffer_map_async(buffer_id, 0, Some(image_desc.buffer_size()), map_op)?;
        self.poller.wake();
        Ok(())
    }

    pub(crate) fn destroy_context(&mut self, context_id: WebGPUContextId) {
        self.wgpu_image_map
            .lock()
            .unwrap()
            .remove(&context_id)
            .unwrap()
            .destroy(&self.global, &self.script_sender, &self.compositor_api);
    }
}

#[allow(clippy::too_many_arguments)]
fn update_wr_image(
    result: Result<(), BufferAccessError>,
    global: Arc<Global>,
    buffer_id: id::BufferId,
    wgpu_image_map: WGPUImageMap,
    context_id: WebGPUContextId,
    compositor_api: CrossProcessCompositorApi,
    image_desc: WebGPUImageDescriptor,
    presentation_id: PresentationId,
) {
    match result {
        Ok(()) => {
            if let Some(context_data) = wgpu_image_map.lock().unwrap().get_mut(&context_id) {
                if !context_data.check_and_update_presentation_id(presentation_id) {
                    let buffer_state = context_data.get_buffer_state(buffer_id);
                    if *buffer_state == PresentationBufferState::Mapping {
                        let _ = global.buffer_unmap(buffer_id);
                        *buffer_state = PresentationBufferState::Available;
                    }
                    // throw away all work, because we are too old
                    return;
                }
                assert_eq!(image_desc, context_data.image_desc);
                let buffer_state = context_data.get_buffer_state(buffer_id);
                assert_eq!(*buffer_state, PresentationBufferState::Mapping);
                *buffer_state = PresentationBufferState::Mapped;
                let presentation_buffer =
                    GPUPresentationBuffer::new(global, buffer_id, image_desc.buffer_size());
                let Some(swap_chain) = context_data.swap_chain.as_mut() else {
                    return;
                };
                let old_presentation_buffer = swap_chain.data.replace(presentation_buffer);
                compositor_api.update_image(
                    context_data.image_key,
                    context_data.image_desc.0,
                    SerializableImageData::External(context_data.image_data),
                );
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
