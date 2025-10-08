/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Main process implementation of [GPUCanvasContext](https://www.w3.org/TR/webgpu/#canvas-context)

use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

use arrayvec::ArrayVec;
use base::Epoch;
use compositing_traits::{
    CrossProcessCompositorApi, ExternalImageSource, SerializableImageData,
    WebrenderExternalImageApi,
};
use euclid::default::Size2D;
use ipc_channel::ipc::IpcSender;
use log::warn;
use pixels::{IpcSnapshot, Snapshot, SnapshotAlphaMode, SnapshotPixelFormat};
use rustc_hash::FxHashMap;
use webgpu_traits::{
    ContextConfiguration, PRESENTATION_BUFFER_COUNT, PendingTexture, WebGPUContextId, WebGPUMsg,
};
use webrender_api::units::DeviceIntSize;
use webrender_api::{
    ExternalImageData, ExternalImageId, ExternalImageType, ImageDescriptor, ImageDescriptorFlags,
    ImageFormat, ImageKey,
};
use wgpu_core::device::HostMap;
use wgpu_core::global::Global;
use wgpu_core::id::{
    self, BufferId, CommandBufferId, CommandEncoderId, DeviceId, QueueId, TextureId,
};
use wgpu_core::resource::{
    BufferAccessError, BufferDescriptor, BufferMapOperation, CreateBufferError,
};
use wgpu_types::{
    BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT, CommandBufferDescriptor, CommandEncoderDescriptor,
    Extent3d, Origin3d, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo,
    TextureAspect,
};

pub type WGPUImageMap = Arc<Mutex<FxHashMap<WebGPUContextId, ContextData>>>;

const fn image_data(context_id: WebGPUContextId) -> ExternalImageData {
    ExternalImageData {
        id: ExternalImageId(context_id.0),
        channel_index: 0,
        image_type: ExternalImageType::Buffer,
        normalized_uvs: false,
    }
}

/// Allocated buffer on GPU device
#[derive(Clone, Copy, Debug)]
struct Buffer {
    device_id: DeviceId,
    queue_id: QueueId,
    size: u64,
}

impl Buffer {
    /// Returns true if buffer is compatible with provided configuration
    fn has_compatible_config(&self, config: &ContextConfiguration) -> bool {
        config.device_id == self.device_id && self.size == config.buffer_size()
    }
}

/// Mapped GPUBuffer
#[derive(Debug)]
struct MappedBuffer {
    buffer: Buffer,
    data: NonNull<u8>,
    len: u64,
    image_size: Size2D<u32>,
    image_format: ImageFormat,
    is_opaque: bool,
}

// Mapped buffer can be shared between safely (it's read-only)
unsafe impl Send for MappedBuffer {}

impl MappedBuffer {
    const fn slice(&'_ self) -> &'_ [u8] {
        // Safety: Pointer is from wgpu, and we only use it here
        unsafe { std::slice::from_raw_parts(self.data.as_ptr(), self.len as usize) }
    }

    fn stride(&self) -> u32 {
        (self.image_size.width * self.image_format.bytes_per_pixel() as u32)
            .next_multiple_of(COPY_BYTES_PER_ROW_ALIGNMENT)
    }
}

#[derive(Debug)]
enum StagingBufferState {
    /// The Initial state: the buffer has yet to be created with only an
    /// id reserved for it.
    Unassigned,
    /// The buffer is allocated in the WGPU Device and is ready to be used.
    Available(Buffer),
    /// `mapAsync` is currently running on the buffer.
    Mapping(Buffer),
    /// The buffer is currently mapped.
    Mapped(MappedBuffer),
}

/// A staging buffer used for texture to buffer to CPU copy operations.
#[derive(Debug)]
struct StagingBuffer {
    global: Arc<Global>,
    buffer_id: BufferId,
    state: StagingBufferState,
}

// [`StagingBuffer`] only used for reading (never for writing)
// so it is safe to share between threads.
unsafe impl Sync for StagingBuffer {}

impl StagingBuffer {
    fn new(global: Arc<Global>, buffer_id: BufferId) -> Self {
        Self {
            global,
            buffer_id,
            state: StagingBufferState::Unassigned,
        }
    }

    const fn is_mapped(&self) -> bool {
        matches!(self.state, StagingBufferState::Mapped(..))
    }

    /// Return true if buffer can be used directly with provided config
    /// without any additional work
    fn is_available_and_has_compatible_config(&self, config: &ContextConfiguration) -> bool {
        let StagingBufferState::Available(buffer) = &self.state else {
            return false;
        };
        buffer.has_compatible_config(config)
    }

    /// Return true if buffer is not mapping or being mapped
    const fn needs_assignment(&self) -> bool {
        matches!(
            self.state,
            StagingBufferState::Unassigned | StagingBufferState::Available(_)
        )
    }

    /// Make buffer available by unmapping / destroying it and then recreating it if needed.
    fn ensure_available(&mut self, config: &ContextConfiguration) -> Result<(), CreateBufferError> {
        let recreate = match &self.state {
            StagingBufferState::Unassigned => true,
            StagingBufferState::Available(buffer) |
            StagingBufferState::Mapping(buffer) |
            StagingBufferState::Mapped(MappedBuffer { buffer, .. }) => {
                if buffer.has_compatible_config(config) {
                    let _ = self.global.buffer_unmap(self.buffer_id);
                    false
                } else {
                    self.global.buffer_drop(self.buffer_id);
                    true
                }
            },
        };
        if recreate {
            let buffer_size = config.buffer_size();
            let (_, error) = self.global.device_create_buffer(
                config.device_id,
                &BufferDescriptor {
                    label: None,
                    size: buffer_size,
                    usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
                Some(self.buffer_id),
            );
            if let Some(error) = error {
                return Err(error);
            };
            self.state = StagingBufferState::Available(Buffer {
                device_id: config.device_id,
                queue_id: config.queue_id,
                size: buffer_size,
            });
        }
        Ok(())
    }

    /// Makes buffer available and prepares command encoder
    /// that will copy texture to this staging buffer.
    ///
    /// Caller must submit command buffer to queue.
    fn prepare_load_texture_command_buffer(
        &mut self,
        texture_id: TextureId,
        encoder_id: CommandEncoderId,
        config: &ContextConfiguration,
    ) -> Result<CommandBufferId, Box<dyn std::error::Error>> {
        self.ensure_available(config)?;
        let StagingBufferState::Available(buffer) = &self.state else {
            unreachable!("Should be made available by `ensure_available`")
        };
        let device_id = buffer.device_id;
        let command_descriptor = CommandEncoderDescriptor { label: None };
        let (encoder_id, error) = self.global.device_create_command_encoder(
            device_id,
            &command_descriptor,
            Some(encoder_id),
        );
        if let Some(error) = error {
            return Err(error.into());
        };
        let buffer_info = TexelCopyBufferInfo {
            buffer: self.buffer_id,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(config.stride()),
                rows_per_image: None,
            },
        };
        let texture_info = TexelCopyTextureInfo {
            texture: texture_id,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        };
        let copy_size = Extent3d {
            width: config.size.width,
            height: config.size.height,
            depth_or_array_layers: 1,
        };
        self.global.command_encoder_copy_texture_to_buffer(
            encoder_id,
            &texture_info,
            &buffer_info,
            &copy_size,
        )?;
        let (command_buffer_id, error) = self
            .global
            .command_encoder_finish(encoder_id, &CommandBufferDescriptor::default());
        if let Some(error) = error {
            return Err(error.into());
        };
        Ok(command_buffer_id)
    }

    /// Unmaps the buffer or cancels a mapping operation if one is in progress.
    fn unmap(&mut self) {
        match self.state {
            StagingBufferState::Unassigned | StagingBufferState::Available(_) => {},
            StagingBufferState::Mapping(buffer) |
            StagingBufferState::Mapped(MappedBuffer { buffer, .. }) => {
                let _ = self.global.buffer_unmap(self.buffer_id);
                self.state = StagingBufferState::Available(buffer)
            },
        }
    }

    /// Obtain a snapshot from this buffer if is mapped or return `None` if it is not mapped.
    fn snapshot(&self) -> Option<Snapshot> {
        let StagingBufferState::Mapped(mapped) = &self.state else {
            return None;
        };
        let format = match mapped.image_format {
            ImageFormat::RGBA8 => SnapshotPixelFormat::RGBA,
            ImageFormat::BGRA8 => SnapshotPixelFormat::BGRA,
            _ => unreachable!("GPUCanvasContext does not support other formats per spec"),
        };
        let alpha_mode = if mapped.is_opaque {
            SnapshotAlphaMode::AsOpaque {
                premultiplied: false,
            }
        } else {
            SnapshotAlphaMode::Transparent {
                premultiplied: true,
            }
        };
        let padded_byte_width = mapped.stride();
        let data = mapped.slice();
        let bytes_per_pixel = mapped.image_format.bytes_per_pixel() as usize;
        let mut result_unpadded =
            Vec::<u8>::with_capacity(mapped.image_size.area() as usize * bytes_per_pixel);
        for row in 0..mapped.image_size.height {
            let start = (row * padded_byte_width).try_into().ok()?;
            result_unpadded
                .extend(&data[start..start + mapped.image_size.width as usize * bytes_per_pixel]);
        }
        let mut snapshot =
            Snapshot::from_vec(mapped.image_size, format, alpha_mode, result_unpadded);
        if mapped.is_opaque {
            snapshot.transform(SnapshotAlphaMode::Opaque, snapshot.format())
        }
        Some(snapshot)
    }
}

impl Drop for StagingBuffer {
    fn drop(&mut self) {
        match self.state {
            StagingBufferState::Unassigned => {},
            StagingBufferState::Available(_) |
            StagingBufferState::Mapping(_) |
            StagingBufferState::Mapped(_) => {
                self.global.buffer_drop(self.buffer_id);
            },
        }
    }
}

#[derive(Default)]
pub struct WGPUExternalImages {
    pub images: WGPUImageMap,
    pub locked_ids: FxHashMap<WebGPUContextId, PresentationStagingBuffer>,
}

impl WebrenderExternalImageApi for WGPUExternalImages {
    fn lock(&mut self, id: u64) -> (ExternalImageSource<'_>, Size2D<i32>) {
        let id = WebGPUContextId(id);
        let presentation = {
            let mut webgpu_contexts = self.images.lock().unwrap();
            webgpu_contexts
                .get_mut(&id)
                .and_then(|context_data| context_data.presentation.clone())
        };
        let Some(presentation) = presentation else {
            return (ExternalImageSource::Invalid, Size2D::zero());
        };
        self.locked_ids.insert(id, presentation);
        let presentation = self.locked_ids.get(&id).unwrap();
        let StagingBufferState::Mapped(mapped_buffer) = &presentation.staging_buffer.state else {
            unreachable!("Presentation staging buffer should be mapped")
        };
        let size = mapped_buffer.image_size;
        (
            ExternalImageSource::RawData(mapped_buffer.slice()),
            size.cast().cast_unit(),
        )
    }

    fn unlock(&mut self, id: u64) {
        let id = WebGPUContextId(id);
        let Some(presentation) = self.locked_ids.remove(&id) else {
            return;
        };
        let mut webgpu_contexts = self.images.lock().unwrap();
        if let Some(context_data) = webgpu_contexts.get_mut(&id) {
            // We use this to return staging buffer if a newer one exists.
            presentation.maybe_destroy(context_data);
        } else {
            // This will not free this buffer id in script,
            // but that's okay because we still have many free ids.
            drop(presentation);
        }
    }
}

/// Staging buffer currently used for presenting the epoch.
///
/// Users should [`ContextData::replace_presentation`] when done.
#[derive(Clone)]
pub struct PresentationStagingBuffer {
    epoch: Epoch,
    staging_buffer: Arc<StagingBuffer>,
}

impl PresentationStagingBuffer {
    fn new(epoch: Epoch, staging_buffer: StagingBuffer) -> Self {
        Self {
            epoch,
            staging_buffer: Arc::new(staging_buffer),
        }
    }

    /// If the internal staging buffer is not shared,
    /// unmap it and call [`ContextData::return_staging_buffer`] with it.
    fn maybe_destroy(self, context_data: &mut ContextData) {
        if let Some(mut staging_buffer) = Arc::into_inner(self.staging_buffer) {
            staging_buffer.unmap();
            context_data.return_staging_buffer(staging_buffer);
        }
    }
}

/// The embedder process-side representation of what is the `GPUCanvasContext` in script.
pub struct ContextData {
    /// The [`ImageKey`] of the WebRender image associated with this context.
    image_key: ImageKey,
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
    fn new(
        image_key: ImageKey,
        global: &Arc<Global>,
        buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    ) -> Self {
        Self {
            image_key,
            inactive_staging_buffers: buffer_ids
                .iter()
                .map(|buffer_id| StagingBuffer::new(global.clone(), *buffer_id))
                .collect(),
            presentation: None,
            next_epoch: Epoch(1),
        }
    }

    /// Returns `None` if no staging buffer is unused or failure when making it available
    fn get_or_make_available_buffer(
        &'_ mut self,
        config: &ContextConfiguration,
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
    fn destroy(
        self,
        script_sender: &IpcSender<WebGPUMsg>,
        compositor_api: &CrossProcessCompositorApi,
    ) {
        // This frees the id in the `ScriptThread`.
        for staging_buffer in self.inactive_staging_buffers {
            if let Err(error) = script_sender.send(WebGPUMsg::FreeBuffer(staging_buffer.buffer_id))
            {
                warn!(
                    "Unable to send FreeBuffer({:?}) ({error})",
                    staging_buffer.buffer_id
                );
            };
        }
        compositor_api.delete_image(self.image_key);
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
            stale_presentation.maybe_destroy(self);
        }
    }

    fn clear_presentation(&mut self) {
        if let Some(stale_presentation) = self.presentation.take() {
            stale_presentation.maybe_destroy(self);
        }
    }

    fn return_staging_buffer(&mut self, staging_buffer: StagingBuffer) {
        self.inactive_staging_buffers.push(staging_buffer)
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
        let context_data = ContextData::new(image_key, &self.global, buffer_ids);
        self.compositor_api.add_image(
            image_key,
            ImageDescriptor {
                format: ImageFormat::BGRA8,
                size,
                stride: None,
                offset: 0,
                flags: ImageDescriptorFlags::empty(),
            },
            SerializableImageData::External(image_data(context_id)),
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

    pub(crate) fn get_image(
        &self,
        context_id: WebGPUContextId,
        pending_texture: Option<PendingTexture>,
        sender: IpcSender<IpcSnapshot>,
    ) {
        let mut webgpu_contexts = self.wgpu_image_map.lock().unwrap();
        let context_data = webgpu_contexts.get_mut(&context_id).unwrap();
        if let Some(PendingTexture {
            texture_id,
            encoder_id,
            configuration,
        }) = pending_texture
        {
            let Some(staging_buffer) = context_data.get_or_make_available_buffer(&configuration)
            else {
                warn!("Failure obtaining available staging buffer");
                sender
                    .send(Snapshot::cleared(configuration.size).as_ipc())
                    .unwrap();
                return;
            };

            let epoch = context_data.next_epoch();
            let wgpu_image_map = self.wgpu_image_map.clone();
            let sender = sender.clone();
            drop(webgpu_contexts);
            self.texture_download(
                texture_id,
                encoder_id,
                staging_buffer,
                configuration,
                move |staging_buffer| {
                    let mut webgpu_contexts = wgpu_image_map.lock().unwrap();
                    let context_data = webgpu_contexts.get_mut(&context_id).unwrap();
                    sender
                        .send(
                            staging_buffer
                                .snapshot()
                                .unwrap_or_else(|| Snapshot::cleared(configuration.size))
                                .as_ipc(),
                        )
                        .unwrap();
                    if staging_buffer.is_mapped() {
                        context_data.replace_presentation(PresentationStagingBuffer::new(
                            epoch,
                            staging_buffer,
                        ));
                    } else {
                        // failure
                        context_data.return_staging_buffer(staging_buffer);
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
                            presentation_staging_buffer.staging_buffer.snapshot()
                        })
                        .unwrap_or_else(Snapshot::empty)
                        .as_ipc(),
                )
                .unwrap();
        }
    }

    /// Read the texture to the staging buffer, map it to CPU memory, and update the
    /// image in WebRender when complete.
    pub(crate) fn present(
        &self,
        context_id: WebGPUContextId,
        pending_texture: Option<PendingTexture>,
        size: Size2D<u32>,
        canvas_epoch: Epoch,
    ) {
        let mut webgpu_contexts = self.wgpu_image_map.lock().unwrap();
        let context_data = webgpu_contexts.get_mut(&context_id).unwrap();
        let image_key = context_data.image_key;
        let Some(PendingTexture {
            texture_id,
            encoder_id,
            configuration,
        }) = pending_texture
        else {
            context_data.clear_presentation();
            self.compositor_api.update_image(
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
            self.compositor_api.update_image(
                image_key,
                configuration.into(),
                SerializableImageData::External(image_data(context_id)),
                Some(canvas_epoch),
            );
            return;
        };
        let epoch = context_data.next_epoch();
        let wgpu_image_map = self.wgpu_image_map.clone();
        let compositor_api = self.compositor_api.clone();
        drop(webgpu_contexts);
        self.texture_download(
            texture_id,
            encoder_id,
            staging_buffer,
            configuration,
            move |staging_buffer| {
                let mut webgpu_contexts = wgpu_image_map.lock().unwrap();
                let context_data = webgpu_contexts.get_mut(&context_id).unwrap();
                if staging_buffer.is_mapped() {
                    context_data.replace_presentation(PresentationStagingBuffer::new(
                        epoch,
                        staging_buffer,
                    ));
                } else {
                    context_data.return_staging_buffer(staging_buffer);
                    context_data.clear_presentation();
                }
                // update image in WR
                compositor_api.update_image(
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
        texture_id: TextureId,
        encoder_id: CommandEncoderId,
        mut staging_buffer: StagingBuffer,
        config: ContextConfiguration,
        callback: impl FnOnce(StagingBuffer) + Send + 'static,
    ) {
        let Ok(command_buffer_id) =
            staging_buffer.prepare_load_texture_command_buffer(texture_id, encoder_id, &config)
        else {
            return callback(staging_buffer);
        };
        let StagingBufferState::Available(buffer) = &staging_buffer.state else {
            unreachable!("`prepare_load_texture_command_buffer` should make buffer available")
        };
        let buffer_id = staging_buffer.buffer_id;
        let buffer_size = buffer.size;
        {
            let _guard = self.poller.lock();
            let result = self
                .global
                .queue_submit(buffer.queue_id, &[command_buffer_id]);
            if result.is_err() {
                return callback(staging_buffer);
            }
        }
        staging_buffer.state = match staging_buffer.state {
            StagingBufferState::Available(buffer) => StagingBufferState::Mapping(buffer),
            _ => unreachable!("`prepare_load_texture_command_buffer` should make buffer available"),
        };
        let map_callback = {
            let token = self.poller.token();
            Box::new(move |result: Result<(), BufferAccessError>| {
                drop(token);
                staging_buffer.state = match staging_buffer.state {
                    StagingBufferState::Mapping(buffer) => {
                        if let Ok((data, len)) = result.and_then(|_| {
                            staging_buffer.global.buffer_get_mapped_range(
                                staging_buffer.buffer_id,
                                0,
                                Some(buffer.size),
                            )
                        }) {
                            StagingBufferState::Mapped(MappedBuffer {
                                buffer,
                                data,
                                len,
                                image_size: config.size,
                                image_format: config.format,
                                is_opaque: config.is_opaque,
                            })
                        } else {
                            StagingBufferState::Available(buffer)
                        }
                    },
                    _ => {
                        unreachable!("Mapping buffer should have StagingBufferState::Mapping state")
                    },
                };
                callback(staging_buffer);
            })
        };
        let map_op = BufferMapOperation {
            host: HostMap::Read,
            callback: Some(map_callback),
        };
        // error is handled by map_callback
        let _ = self
            .global
            .buffer_map_async(buffer_id, 0, Some(buffer_size), map_op);
        self.poller.wake();
    }

    pub(crate) fn destroy_context(&mut self, context_id: WebGPUContextId) {
        self.wgpu_image_map
            .lock()
            .unwrap()
            .remove(&context_id)
            .unwrap()
            .destroy(&self.script_sender, &self.compositor_api);
    }
}
