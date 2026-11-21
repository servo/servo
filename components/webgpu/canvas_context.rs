/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Main process implementation of [GPUCanvasContext](https://www.w3.org/TR/webgpu/#canvas-context)

use std::sync::{Arc, Mutex};

use arrayvec::ArrayVec;
use euclid::default::Size2D;
use log::warn;
use paint_api::{
    CrossProcessPaintApi, ExternalImageSource, SerializableImageData, WebRenderExternalImageApi,
};
use pixels::{SharedSnapshot, Snapshot, SnapshotAlphaMode, SnapshotPixelFormat};
use rustc_hash::FxHashMap;
use servo_base::Epoch;
use servo_base::generic_channel::GenericSender;
use webgpu_traits::{
    CommandBufferDescriptor, CommandEncoderDescriptor, ContextConfiguration, Error, Extent3d,
    HostMap, Origin3d, PRESENTATION_BUFFER_COUNT, PendingTexture, TextureAspect, WebGPUContextId,
};
use webrender_api::units::DeviceIntSize;
use webrender_api::{
    ExternalImageData, ExternalImageId, ExternalImageType, ImageDescriptor, ImageDescriptorFlags,
    ImageFormat, ImageKey,
};
use wgpu_core::command::CommandBuffer;
use wgpu_core::device::Device;
use wgpu_core::device::queue::Queue;
use wgpu_core::resource::{BufferAccessError, BufferMapping, ParentDevice, Texture};
use wgpu_types::{BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT};

use crate::wgpu_thread::WGPU;

pub type WebGpuExternalImageMap = Arc<Mutex<FxHashMap<WebGPUContextId, ContextData>>>;

const fn image_data(context_id: WebGPUContextId) -> ExternalImageData {
    ExternalImageData {
        id: ExternalImageId(context_id.0),
        channel_index: 0,
        image_type: ExternalImageType::Buffer,
        normalized_uvs: false,
    }
}

pub type ResolvedContextConfiguration = ContextConfiguration<Arc<Device>, Arc<Queue>>;
pub type ResolvedPendingTexture = PendingTexture<Arc<Texture>, ResolvedContextConfiguration>;

impl WGPU {
    pub(crate) fn resolve_canvas_config(
        &self,
        config: ContextConfiguration,
    ) -> ResolvedContextConfiguration {
        ContextConfiguration {
            device: self.global.resolve_device_id(config.device),
            queue: self.global.resolve_queue_id(config.queue),
            format: config.format,
            is_opaque: config.is_opaque,
            size: config.size,
        }
    }

    pub(crate) fn resolve_pending_texture(
        &self,
        pending_texture: PendingTexture,
    ) -> ResolvedPendingTexture {
        ResolvedPendingTexture {
            texture: self.global.resolve_texture_id(pending_texture.texture),
            configuration: self.resolve_canvas_config(pending_texture.configuration),
        }
    }
}

/// Allocated buffer on GPU device
#[derive(Clone, Debug)]
struct Buffer(Arc<wgpu_core::resource::Buffer>);

impl Buffer {
    /// Returns true if buffer is compatible with provided configuration
    fn has_compatible_config(&self, config: &ResolvedContextConfiguration) -> bool {
        self.0.same_device(&config.device).is_ok() && self.0.size() == config.buffer_size()
    }
}

/// Mapped GPUBuffer
#[derive(Debug, Clone)]
struct MappedBuffer {
    buffer: Buffer,
    image_size: Size2D<u32>,
    image_format: ImageFormat,
    is_opaque: bool,
}

// Mapped buffer can be shared between safely (it's read-only)
unsafe impl Send for MappedBuffer {}
unsafe impl Sync for MappedBuffer {}

fn stride(image_size: Size2D<u32>, image_format: ImageFormat) -> u32 {
    (image_size.width * image_format.bytes_per_pixel() as u32)
        .next_multiple_of(COPY_BYTES_PER_ROW_ALIGNMENT)
}

impl MappedBuffer {
    /// Obtain a snapshot from this buffer if is mapped or return `None` if it is not mapped.
    fn snapshot(&self) -> Option<Snapshot> {
        let mapping = self.buffer.0.get_mapped_range(0, None).ok()?;
        let padded_byte_width = stride(self.image_size, self.image_format) as usize;
        let bytes_per_pixel = self.image_format.bytes_per_pixel() as usize;
        let unpadded_byte_width = self.image_size.width as usize * bytes_per_pixel;
        let mut data = vec![0; self.image_size.height as usize * unpadded_byte_width];
        let read_slice = mapping.read_slice();
        for row in 0..self.image_size.height {
            let data_start = row as usize * unpadded_byte_width;
            let read_start = row as usize * padded_byte_width;
            data[data_start..data_start + unpadded_byte_width]
                .copy_from_slice(&read_slice[read_start..read_start + unpadded_byte_width]);
        }
        let format = match self.image_format {
            ImageFormat::RGBA8 => SnapshotPixelFormat::RGBA,
            ImageFormat::BGRA8 => SnapshotPixelFormat::BGRA,
            _ => unreachable!("GPUCanvasContext does not support other formats per spec"),
        };
        let alpha_mode = if self.is_opaque {
            SnapshotAlphaMode::AsOpaque {
                premultiplied: false,
            }
        } else {
            SnapshotAlphaMode::Transparent {
                premultiplied: true,
            }
        };
        let mut snapshot = Snapshot::from_vec(self.image_size, format, alpha_mode, data);
        if self.is_opaque {
            snapshot.transform(SnapshotAlphaMode::Opaque, snapshot.format())
        }
        Some(snapshot)
    }
}

/// A staging buffer used for texture to buffer to CPU copy operations.
#[derive(Debug, Default)]
enum StagingBuffer {
    #[default]
    /// The Initial state: the buffer has yet to be created with only an
    /// id reserved for it.
    Unassigned,
    /// The buffer is allocated in the WGPU Device and is ready to be used.
    Available(Buffer),
    /// `mapAsync` is currently running on the buffer.
    Mapping(Buffer),
}

impl StagingBuffer {
    fn new() -> Self {
        Self::Unassigned
    }

    /// Return true if buffer can be used directly with provided config
    /// without any additional work
    fn is_available_and_has_compatible_config(
        &self,
        config: &ResolvedContextConfiguration,
    ) -> bool {
        let StagingBuffer::Available(buffer) = self else {
            return false;
        };
        buffer.has_compatible_config(config)
    }

    /// Return true if buffer is not mapping or being mapped
    const fn needs_assignment(&self) -> bool {
        matches!(
            self,
            StagingBuffer::Unassigned | StagingBuffer::Available(_)
        )
    }

    /// Make buffer available by unmapping / destroying it and then recreating it if needed.
    fn ensure_available(&mut self, config: &ResolvedContextConfiguration) -> Result<(), Error> {
        let recreate = match self {
            StagingBuffer::Unassigned => true,
            StagingBuffer::Available(buffer) | StagingBuffer::Mapping(buffer) => {
                if buffer.has_compatible_config(config) {
                    buffer.0.unmap();
                    false
                } else {
                    true
                }
            },
        };
        if recreate {
            let buffer_size = config.buffer_size();
            config
                .device
                .push_error_scope(webgpu_traits::ErrorFilter::Validation);
            let buffer = config
                .device
                .create_buffer(&wgpu_core::resource::BufferDescriptor {
                    label: None,
                    size: buffer_size,
                    usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

            if let Ok(Some(error)) = config.device.pop_error_scope() {
                return Err(error.into());
            };
            *self = StagingBuffer::Available(Buffer(buffer));
        }
        Ok(())
    }

    /// Makes buffer available and prepares command encoder
    /// that will copy texture to this staging buffer.
    ///
    /// Caller must submit command buffer to queue.
    fn prepare_load_texture_command_buffer(
        &mut self,
        texture: Arc<Texture>,
        config: &ResolvedContextConfiguration,
    ) -> Result<Arc<CommandBuffer>, Box<dyn std::error::Error>> {
        self.ensure_available(config)?;
        let StagingBuffer::Available(buffer) = self else {
            unreachable!("Should be made available by `ensure_available`")
        };
        let command_descriptor = CommandEncoderDescriptor { label: None };
        config
            .device
            .push_error_scope(webgpu_traits::ErrorFilter::Validation);
        let encoder = config.device.create_command_encoder(&command_descriptor);
        let buffer_info = wgpu_types::TexelCopyBufferInfo {
            buffer: buffer.0.clone(),
            layout: wgpu_types::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(config.stride()),
                rows_per_image: None,
            },
        };
        let texture_info = wgpu_types::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        };
        let copy_size = Extent3d {
            width: config.size.width,
            height: config.size.height,
            depth_or_array_layers: 1,
        };
        encoder.copy_texture_to_buffer(&texture_info, &buffer_info, &copy_size);
        let cmd_buffer = encoder.finish(&CommandBufferDescriptor::default());
        if let Ok(Some(error)) = config.device.pop_error_scope() {
            Err(error.into())
        } else {
            Ok(cmd_buffer)
        }
    }
}

pub struct WebGpuExternalImages {
    pub image_map: WebGpuExternalImageMap,
    pub locked_ids: FxHashMap<WebGPUContextId, BufferMapping>,
}

impl WebGpuExternalImages {
    pub fn new(image_map: WebGpuExternalImageMap) -> Self {
        Self {
            image_map,
            locked_ids: Default::default(),
        }
    }
}

impl WebRenderExternalImageApi for WebGpuExternalImages {
    fn lock(&mut self, id: u64) -> (ExternalImageSource<'_>, Size2D<i32>) {
        let id = WebGPUContextId(id);
        let presentation = {
            let mut webgpu_contexts = self.image_map.lock().unwrap();
            webgpu_contexts.get_mut(&id).and_then(|context_data| {
                context_data
                    .presentation
                    .as_ref()
                    .map(|presentation_staging_buffer| {
                        presentation_staging_buffer.mapped_buffer.clone()
                    })
            })
        };
        let Some(presentation) = presentation else {
            return (ExternalImageSource::Invalid, Size2D::zero());
        };
        let Ok(buffer_mapping) = presentation.buffer.0.get_mapped_range(0, None) else {
            return (ExternalImageSource::Invalid, Size2D::zero());
        };
        self.locked_ids.insert(id, buffer_mapping);
        let size = presentation.image_size;
        let buffer_mapping = self.locked_ids.get(&id).unwrap();
        (
            ExternalImageSource::RawData(buffer_mapping.read_slice()),
            size.cast().cast_unit(),
        )
    }

    fn unlock(&mut self, id: u64) {
        let id = WebGPUContextId(id);
        let _ = self.locked_ids.remove(&id);
    }
}

/// Staging buffer currently used for presenting the epoch.
///
/// Users should [`ContextData::replace_presentation`] when done.
pub struct PresentationStagingBuffer {
    epoch: Epoch,
    mapped_buffer: MappedBuffer,
}

impl PresentationStagingBuffer {
    fn new(epoch: Epoch, mapped_buffer: MappedBuffer) -> Self {
        Self {
            epoch,
            mapped_buffer,
        }
    }

    fn destroy(self) -> StagingBuffer {
        let buffer = self.mapped_buffer.buffer;
        buffer.0.unmap();
        StagingBuffer::Available(buffer)
    }
}

/// The embedder process-side representation of what is the `GPUCanvasContext` in script.
pub struct ContextData {
    /// The [`ImageKey`] of the WebRender image associated with this context.
    image_key: Option<ImageKey>,
    /// The current size of this context.
    size: DeviceIntSize,
    /// Staging buffers that are not actively used.
    ///
    /// Staging buffer here are either [`StagingBufferState::Unassigned`] or [`StagingBufferState::Available`].
    /// They are removed from here when they are in process of being mapped or are already mapped.
    inactive_staging_buffers: ArrayVec<StagingBuffer, PRESENTATION_BUFFER_COUNT>,
    /// The [`PresentationStagingBuffer`] of the most recent presentation. This will
    /// be `None` directly after initialization, as clearing is handled completely in
    /// the `ScriptThread`.
    presentation: Option<PresentationStagingBuffer>,
    /// Next epoch to be used
    next_epoch: Epoch,
}

impl ContextData {
    fn new(size: DeviceIntSize) -> Self {
        Self {
            image_key: None,
            size,
            inactive_staging_buffers: (0..PRESENTATION_BUFFER_COUNT)
                .into_iter()
                .map(|_| StagingBuffer::new())
                .collect(),
            presentation: None,
            next_epoch: Epoch(1),
        }
    }

    /// Returns `None` if no staging buffer is unused or failure when making it available
    fn get_or_make_available_buffer(
        &'_ mut self,
        config: &ResolvedContextConfiguration,
    ) -> Option<StagingBuffer> {
        self.inactive_staging_buffers
            .iter()
            // Try to get first preallocated GPUBuffer.
            .position(|staging_buffer| {
                staging_buffer.is_available_and_has_compatible_config(config)
            })
            // Fall back to the first inactive staging buffer.
            .or_else(|| {
                self.inactive_staging_buffers
                    .iter()
                    .position(|staging_buffer| staging_buffer.needs_assignment())
            })
            // Or just the use first one.
            .or_else(|| {
                if self.inactive_staging_buffers.is_empty() {
                    None
                } else {
                    Some(0)
                }
            })
            .and_then(|index| {
                let mut staging_buffer = self.inactive_staging_buffers.remove(index);
                if staging_buffer.ensure_available(config).is_ok() {
                    Some(staging_buffer)
                } else {
                    // If we fail to make it available, return it to the list of inactive staging buffers.
                    self.inactive_staging_buffers.push(staging_buffer);
                    None
                }
            })
    }

    /// Destroy the context that this [`ContextData`] represents,
    /// freeing all of its buffers, and deleting the associated WebRender image.
    fn destroy(mut self, paint_api: &CrossProcessPaintApi) {
        if let Some(image_key) = self.image_key.take() {
            paint_api.delete_image(image_key);
        }
    }

    /// Advance the [`Epoch`] and return the new one.
    fn next_epoch(&mut self) -> Epoch {
        let epoch = self.next_epoch;
        self.next_epoch.next();
        epoch
    }

    /// If the given [`PresentationStagingBuffer`] is for a newer presentation, replace the existing
    /// one. Deallocate the older one by calling [`Self::return_staging_buffer`] on it.
    fn replace_presentation(&mut self, presentation: PresentationStagingBuffer) {
        let stale_presentation = if presentation.epoch >=
            self.presentation
                .as_ref()
                .map(|p| p.epoch)
                .unwrap_or_default()
        {
            self.presentation.replace(presentation)
        } else {
            Some(presentation)
        };
        if let Some(stale_presentation) = stale_presentation {
            self.return_staging_buffer(stale_presentation.destroy());
        }
    }

    fn clear_presentation(&mut self) {
        if let Some(stale_presentation) = self.presentation.take() {
            self.return_staging_buffer(stale_presentation.destroy());
        }
    }

    fn return_staging_buffer(&mut self, staging_buffer: StagingBuffer) {
        self.inactive_staging_buffers.push(staging_buffer)
    }
}

impl crate::WGPU {
    pub(crate) fn create_context(&self, context_id: WebGPUContextId, size: DeviceIntSize) {
        let context_data = ContextData::new(size);
        assert!(
            self.wgpu_image_map
                .lock()
                .unwrap()
                .insert(context_id, context_data)
                .is_none(),
            "Context should be created only once!"
        );
    }

    pub(crate) fn set_image_key(&self, context_id: WebGPUContextId, image_key: ImageKey) {
        let mut webgpu_contexts = self.wgpu_image_map.lock().unwrap();
        let context_data = webgpu_contexts.get_mut(&context_id).unwrap();

        if let Some(old_image_key) = context_data.image_key.replace(image_key) {
            self.paint_api.delete_image(old_image_key);
        }

        self.paint_api.add_image(
            image_key,
            ImageDescriptor {
                format: ImageFormat::BGRA8,
                size: context_data.size,
                stride: None,
                offset: 0,
                flags: ImageDescriptorFlags::empty(),
            },
            SerializableImageData::External(image_data(context_id)),
            false,
        );
    }

    pub(crate) fn get_image(
        &self,
        context_id: WebGPUContextId,
        pending_texture: Option<ResolvedPendingTexture>,
        sender: GenericSender<SharedSnapshot>,
    ) {
        let mut webgpu_contexts = self.wgpu_image_map.lock().unwrap();
        let context_data = webgpu_contexts.get_mut(&context_id).unwrap();
        if let Some(PendingTexture {
            texture,
            configuration,
        }) = pending_texture
        {
            let Some(staging_buffer) = context_data.get_or_make_available_buffer(&configuration)
            else {
                warn!("Failure obtaining available staging buffer");
                sender
                    .send(SharedSnapshot::cleared(configuration.size))
                    .unwrap();
                return;
            };

            let epoch = context_data.next_epoch();
            let wgpu_image_map = self.wgpu_image_map.clone();
            let sender = sender;
            drop(webgpu_contexts);
            self.texture_download(
                texture,
                staging_buffer,
                configuration.clone(),
                move |result| {
                    let mut webgpu_contexts = wgpu_image_map.lock().unwrap();
                    let context_data = webgpu_contexts.get_mut(&context_id).unwrap();
                    sender
                        .send(
                            result
                                .as_ref()
                                .ok()
                                .and_then(|mb| mb.snapshot())
                                .map(|snapshot| snapshot.to_shared())
                                .unwrap_or_else(|| SharedSnapshot::cleared(configuration.size)),
                        )
                        .unwrap();
                    match result {
                        Ok(mapped_buffer) => {
                            context_data.replace_presentation(PresentationStagingBuffer::new(
                                epoch,
                                mapped_buffer,
                            ));
                        },
                        Err(staging_buffer) => {
                            context_data.return_staging_buffer(staging_buffer);
                        },
                    }
                },
            );
        } else {
            sender
                .send(
                    context_data
                        .presentation
                        .as_ref()
                        .and_then(|presentation_staging_buffer| {
                            presentation_staging_buffer.mapped_buffer.snapshot()
                        })
                        .unwrap_or_else(Snapshot::empty)
                        .into_shared(),
                )
                .unwrap();
        }
    }

    /// Read the texture to the staging buffer, map it to CPU memory, and update the
    /// image in WebRender when complete.
    pub(crate) fn present(
        &self,
        context_id: WebGPUContextId,
        pending_texture: Option<ResolvedPendingTexture>,
        size: Size2D<u32>,
        canvas_epoch: Epoch,
    ) {
        let mut webgpu_contexts = self.wgpu_image_map.lock().unwrap();
        let context_data = webgpu_contexts.get_mut(&context_id).unwrap();

        let Some(image_key) = context_data.image_key else {
            return;
        };

        let Some(PendingTexture {
            texture,
            configuration,
        }) = pending_texture
        else {
            context_data.clear_presentation();
            self.paint_api.update_image(
                image_key,
                ImageDescriptor {
                    format: ImageFormat::BGRA8,
                    size: size.cast_unit().cast(),
                    stride: None,
                    offset: 0,
                    flags: ImageDescriptorFlags::empty(),
                },
                SerializableImageData::External(image_data(context_id)),
                Some(canvas_epoch),
            );
            return;
        };
        let Some(staging_buffer) = context_data.get_or_make_available_buffer(&configuration) else {
            warn!("Failure obtaining available staging buffer");
            context_data.clear_presentation();
            self.paint_api.update_image(
                image_key,
                configuration.into(),
                SerializableImageData::External(image_data(context_id)),
                Some(canvas_epoch),
            );
            return;
        };
        let epoch = context_data.next_epoch();
        let wgpu_image_map = self.wgpu_image_map.clone();
        let paint_api = self.paint_api.clone();
        drop(webgpu_contexts);
        self.texture_download(
            texture,
            staging_buffer,
            configuration.clone(),
            move |result| {
                let mut webgpu_contexts = wgpu_image_map.lock().unwrap();
                let context_data = webgpu_contexts.get_mut(&context_id).unwrap();
                match result {
                    Ok(mapped_buffer) => context_data
                        .replace_presentation(PresentationStagingBuffer::new(epoch, mapped_buffer)),
                    Err(staging_buffer) => {
                        context_data.return_staging_buffer(staging_buffer);
                        context_data.clear_presentation();
                    },
                }
                // update image in WR
                paint_api.update_image(
                    image_key,
                    configuration.into(),
                    SerializableImageData::External(image_data(context_id)),
                    Some(canvas_epoch),
                );
            },
        );
    }

    /// Copies data from provided texture using `encoder_id` to the provided [`StagingBuffer`].
    ///
    /// `callback` is guaranteed to be called.
    ///
    /// Returns a [`StagingBuffer`] with the [`StagingBufferState::Mapped`] state
    /// on success or [`StagingBufferState::Available`] on failure.
    fn texture_download(
        &self,
        texture: Arc<Texture>,
        mut staging_buffer: StagingBuffer,
        config: ResolvedContextConfiguration,
        callback: impl FnOnce(Result<MappedBuffer, StagingBuffer>) + Send + 'static,
    ) {
        let Ok(command_buffer) =
            staging_buffer.prepare_load_texture_command_buffer(texture, &config)
        else {
            return callback(Err(staging_buffer));
        };
        let StagingBuffer::Available(buffer) = &staging_buffer else {
            unreachable!("`prepare_load_texture_command_buffer` should make buffer available")
        };
        let buffer = buffer.0.clone();
        let buffer_size = buffer.size();
        {
            config
                .device
                .push_error_scope(webgpu_traits::ErrorFilter::Validation);
            config.queue.submit(&[command_buffer]);
            if let Ok(Some(_)) = config.device.pop_error_scope() {
                return callback(Err(staging_buffer));
            }
        }
        let t = std::mem::take(&mut staging_buffer);
        staging_buffer = match t {
            StagingBuffer::Available(buffer) => StagingBuffer::Mapping(buffer),
            _ => unreachable!("`prepare_load_texture_command_buffer` should make buffer available"),
        };
        let map_callback = {
            let token = self.poller.token();
            Box::new(move |result: Result<HostMap, BufferAccessError>| {
                drop(token);
                let result = match staging_buffer {
                    StagingBuffer::Mapping(buffer) => match result {
                        Ok(_) => Ok(MappedBuffer {
                            buffer,
                            image_size: config.size,
                            image_format: config.format,
                            is_opaque: config.is_opaque,
                        }),
                        Err(_) => Err(StagingBuffer::Available(buffer)),
                    },
                    _ => {
                        unreachable!("Mapping buffer should have StagingBufferState::Mapping state")
                    },
                };
                callback(result);
            })
        };
        {
            config
                .device
                .push_error_scope(webgpu_traits::ErrorFilter::Validation);

            buffer.map_async(HostMap::Read, 0, Some(buffer_size), Some(map_callback));
            self.poller.wake();
            // ignore errors here as they will be reported in the callback
            let _ = config.device.pop_error_scope();
        }
    }

    pub(crate) fn destroy_context(&mut self, context_id: WebGPUContextId) {
        self.wgpu_image_map
            .lock()
            .unwrap()
            .remove(&context_id)
            .unwrap()
            .destroy(&self.paint_api);
    }
}
