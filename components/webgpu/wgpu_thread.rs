/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Data and main loop of WebGPU thread.

use std::sync::{Arc, Mutex};

use log::{info, warn};
use paint_api::{CrossProcessPaintApi, WebRenderExternalImageIdManager, WebRenderImageHandlerType};
use rustc_hash::FxHashMap;
use servo_base::generic_channel::{GenericReceiver, GenericSender, GenericSharedMemory};
use servo_base::id::PipelineId;
use servo_config::pref;
use webgpu_traits::id::DeviceId;
use webgpu_traits::{
    Adapter, BufferAddress, BufferUpdate, CompilationInfo, Error, Extent3d, HostMap, Mapping,
    Origin3d, Pipeline, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
    WebGPU, WebGPUAdapter, WebGPUContextId, WebGPUDevice, WebGPUMsg, WebGPUQueue, WebGPURequest,
    id,
};
use webrender_api::ExternalImageId;
use wgpu_core::LabelHelpers;
use wgpu_core::resource::BufferAccessResult;
use wgpu_core_remote::global::Global;
use wgpu_core_remote::map_buffer_access_error;
use wgpu_types::{
    CompilationMessage, ExternalTextureDescriptor, ExternalTextureFormat,
    ExternalTextureTransferFunction, InstanceDescriptor,
};

use crate::canvas_context::WebGpuExternalImageMap;
use crate::poll_thread::Poller;

#[expect(clippy::upper_case_acronyms)] // Name of the library
pub(crate) struct WGPU {
    receiver: GenericReceiver<WebGPURequest>,
    sender: GenericSender<WebGPURequest>,
    pub(crate) script_sender: GenericSender<WebGPUMsg>,
    pub(crate) global: Global,
    devices: FxHashMap<DeviceId, PipelineId>,
    buffers: Arc<Mutex<FxHashMap<id::BufferId, GenericSharedMemory>>>,
    pub(crate) paint_api: CrossProcessPaintApi,
    pub(crate) webrender_external_image_id_manager: WebRenderExternalImageIdManager,
    pub(crate) wgpu_image_map: WebGpuExternalImageMap,
    /// Provides access to poller thread
    pub(crate) poller: Poller,
}

impl WGPU {
    pub(crate) fn new(
        receiver: GenericReceiver<WebGPURequest>,
        sender: GenericSender<WebGPURequest>,
        script_sender: GenericSender<WebGPUMsg>,
        paint_api: CrossProcessPaintApi,
        webrender_external_image_id_manager: WebRenderExternalImageIdManager,
        wgpu_image_map: WebGpuExternalImageMap,
    ) -> Self {
        let backend_pref = pref!(dom_webgpu_wgpu_backend);
        let backends = if backend_pref.is_empty() {
            wgpu_types::Backends::PRIMARY
        } else {
            info!(
                "Selecting backends based on dom.webgpu.wgpu_backend pref: {:?}",
                backend_pref
            );
            wgpu_types::Backends::from_comma_list(&backend_pref)
        };
        let global = Global::new(
            "wgpu-core",
            InstanceDescriptor {
                backends,
                backend_options: wgpu_types::BackendOptions {
                    gl: wgpu_types::GlBackendOptions {
                        gles_minor_version: wgpu_types::Gles3MinorVersion::Automatic,
                        fence_behavior: wgpu_types::GlFenceBehavior::Normal,
                        debug_fns: wgpu_types::GlDebugFns::Auto,
                    },
                    dx12: wgpu_types::Dx12BackendOptions {
                        ..Default::default()
                    },
                    noop: wgpu_types::NoopBackendOptions::default(),
                },

                flags: wgpu_types::InstanceFlags::from_build_config() |
                    wgpu_types::InstanceFlags::AUTOMATIC_TIMESTAMP_NORMALIZATION |
                    wgpu_types::InstanceFlags::STRICT_WEBGPU_COMPLIANCE,
                // TODO(sagudev): firefox actually sets this, but it can cause OOM for us
                // meaning that we are likely leaking something
                memory_budget_thresholds: wgpu_types::MemoryBudgetThresholds {
                    for_resource_creation: Some(95),
                    for_device_loss: Some(99),
                },
                display: None,
            },
            None,
        );
        WGPU {
            poller: Poller::new(global.instance().clone()),
            receiver,
            sender,
            script_sender,
            global,
            devices: FxHashMap::default(),
            buffers: Arc::new(Mutex::new(FxHashMap::default())),
            paint_api,
            webrender_external_image_id_manager,
            wgpu_image_map,
        }
    }

    pub(crate) fn run(&mut self) {
        loop {
            if let Ok(msg) = self.receiver.recv() {
                log::trace!("recv: {msg:?}");
                match msg {
                    WebGPURequest::SetImageKey {
                        context_id,
                        image_key,
                    } => self.set_image_key(context_id, image_key),
                    WebGPURequest::BufferMapAsync {
                        callback: sender,
                        buffer_id,
                        device_id: _,
                        mode,
                        offset,
                        size,
                        buffer_size,
                    } => {
                        let buffer = self.global.resolve_buffer_id(buffer_id);
                        let buffers = Arc::clone(&self.buffers);
                        let resp_sender = sender.clone();
                        let token = self.poller.token();
                        let callback = Box::from(move |result: BufferAccessResult| {
                            drop(token);
                            let response = result.and_then(|mode| {
                                let mut data =
                                    buffers.lock().unwrap().remove(&buffer_id).unwrap_or_else(
                                        || GenericSharedMemory::from_byte(0, buffer_size as usize),
                                    );
                                if mode == HostMap::Read {
                                    let mapping = buffer.get_mapped_range(offset, size)?;
                                    let data = unsafe { data.deref_mut() };
                                    data[offset as usize..(offset + mapping.len()) as usize]
                                        .copy_from_slice(mapping.read_slice());
                                }

                                Ok(Mapping {
                                    data,
                                    range: offset..size.map(|s| offset + s).unwrap_or(buffer_size),
                                    mode,
                                })
                            });
                            if let Err(e) =
                                resp_sender.send(response.map_err(map_buffer_access_error))
                            {
                                warn!("Could not send BufferMapAsync Response ({})", e);
                            }
                        });

                        self.global
                            .buffer_map_async(buffer_id, mode, offset, size, Some(callback));
                        self.poller.wake();
                    },
                    WebGPURequest::CommandEncoderCommand {
                        command_encoder_id,
                        command,
                        device_id: _,
                    } => {
                        self.global
                            .handle_command_encoder_command(command_encoder_id, command);
                    },
                    WebGPURequest::CreateBindGroup {
                        device_id,
                        bind_group_id,
                        descriptor,
                    } => {
                        self.global
                            .device_create_bind_group(device_id, &descriptor, bind_group_id);
                    },
                    WebGPURequest::CreateBindGroupLayout {
                        device_id,
                        bind_group_layout_id,
                        descriptor,
                    } => {
                        self.global.device_create_bind_group_layout(
                            device_id,
                            &descriptor,
                            bind_group_layout_id,
                        );
                    },
                    WebGPURequest::CreateBuffer {
                        device_id,
                        buffer_id,
                        descriptor,
                    } => {
                        self.global
                            .device_create_buffer(device_id, &descriptor, buffer_id);
                    },
                    WebGPURequest::CreateCommandEncoder {
                        device_id,
                        command_encoder_id,
                        desc,
                    } => {
                        self.global.device_create_command_encoder(
                            device_id,
                            &desc,
                            command_encoder_id,
                        );
                    },
                    WebGPURequest::CreateComputePipeline {
                        device_id,
                        compute_pipeline_id,
                        descriptor,
                        async_sender: sender,
                    } => {
                        if let Some(sender) = sender {
                            if let Err(error) = sender.send(
                                self.global
                                    .device_create_compute_pipeline_or_error(
                                        device_id,
                                        &descriptor,
                                        compute_pipeline_id,
                                    )
                                    .map(|()| Pipeline {
                                        id: compute_pipeline_id,
                                        label: descriptor.label.to_string(),
                                    }),
                            ) {
                                log::warn!(
                                    "Failed to send compute pipeline creation result: {:?}",
                                    error
                                );
                            }
                        } else {
                            self.global.device_create_compute_pipeline(
                                device_id,
                                &descriptor,
                                compute_pipeline_id,
                            );
                        }
                    },
                    WebGPURequest::CreatePipelineLayout {
                        device_id,
                        pipeline_layout_id,
                        descriptor,
                    } => {
                        self.global.device_create_pipeline_layout(
                            device_id,
                            &descriptor,
                            pipeline_layout_id,
                        );
                    },
                    WebGPURequest::CreateRenderPipeline {
                        device_id,
                        render_pipeline_id,
                        descriptor,
                        async_sender: sender,
                    } => {
                        if let Some(sender) = sender {
                            if let Err(error) = sender.send(
                                self.global
                                    .device_create_render_pipeline_or_error(
                                        device_id,
                                        &descriptor,
                                        render_pipeline_id,
                                    )
                                    .map(|()| Pipeline {
                                        id: render_pipeline_id,
                                        label: descriptor.label.to_string(),
                                    }),
                            ) {
                                log::warn!(
                                    "Failed to send render pipeline creation result: {:?}",
                                    error
                                );
                            }
                        } else {
                            self.global.device_create_render_pipeline(
                                device_id,
                                &descriptor,
                                render_pipeline_id,
                            );
                        }
                    },
                    WebGPURequest::CreateSampler {
                        device_id,
                        sampler_id,
                        descriptor,
                    } => {
                        self.global
                            .device_create_sampler(device_id, &descriptor, sampler_id);
                    },
                    WebGPURequest::CreateShaderModule {
                        device_id,
                        program_id,
                        descriptor,
                        callback,
                    } => {
                        self.global
                            .device_create_shader_module(device_id, &descriptor, program_id);
                        let compilation_info =
                            self.global.shader_module_compilation_info(program_id);
                        let compilation_info = CompilationInfo {
                            messages: compilation_info
                                .messages
                                .into_iter()
                                .map(
                                    |CompilationMessage {
                                         message,
                                         message_type,
                                         location,
                                     }| CompilationMessage {
                                        message,
                                        message_type,
                                        location: location.map(|l| l.to_utf16(&descriptor.code)),
                                    },
                                )
                                .collect(),
                        };
                        if let Err(error) = callback.send(compilation_info) {
                            log::warn!(
                                "Failed to send shader module compilation info: {:?}",
                                error
                            );
                        }
                    },
                    WebGPURequest::CreateContext { size, sender } => {
                        let id = self
                            .webrender_external_image_id_manager
                            .next_id(WebRenderImageHandlerType::WebGpu);
                        let context_id = WebGPUContextId(id.0);

                        if let Err(error) = sender.send(context_id) {
                            warn!("Failed to send ContextId to new context ({error})");
                        };

                        self.create_context(context_id, size);
                    },
                    WebGPURequest::Present {
                        context_id,
                        pending_texture,
                        size,
                        canvas_epoch,
                    } => {
                        self.present(
                            context_id,
                            pending_texture.map(|pt| self.resolve_pending_texture(pt)),
                            size,
                            canvas_epoch,
                        );
                    },
                    WebGPURequest::GetImage {
                        context_id,
                        pending_texture,
                        sender,
                    } => self.get_image(
                        context_id,
                        pending_texture.map(|pt| self.resolve_pending_texture(pt)),
                        sender,
                    ),
                    WebGPURequest::ValidateTextureDescriptor {
                        device_id,
                        descriptor,
                    } => {
                        // https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-configure
                        let error = self
                            .global
                            .device_validate_texture_descriptor(device_id, &descriptor);
                        if let Some(error) = error {
                            self.global.device_handle_error(
                                device_id,
                                error,
                                None,
                                "GPUCanvasContext.configure",
                            );
                        }
                    },
                    WebGPURequest::DestroyContext { context_id } => {
                        self.destroy_context(context_id);
                        self.webrender_external_image_id_manager
                            .remove(&ExternalImageId(context_id.0));
                    },
                    WebGPURequest::CreateTexture {
                        device_id,
                        texture_id,
                        descriptor,
                    } => {
                        self.global
                            .device_create_texture(device_id, &descriptor, texture_id);
                    },
                    WebGPURequest::CreateTextureView {
                        texture_id,
                        texture_view_id,
                        device_id: _,
                        descriptor,
                    } => {
                        self.global
                            .texture_create_view(texture_id, &descriptor, texture_view_id);
                    },
                    WebGPURequest::DestroyBuffer(buffer) => {
                        let global = &self.global;
                        global.buffer_destroy(buffer);
                    },
                    WebGPURequest::DestroyDevice(device) => {
                        let global = &self.global;
                        global.device_destroy(device);
                        // Wake poller thread to trigger DeviceLostClosure
                        self.poller.wake();
                    },
                    WebGPURequest::DestroyTexture(texture_id) => {
                        let global = &self.global;
                        global.texture_destroy(texture_id);
                    },
                    WebGPURequest::Exit(sender) => {
                        if let Err(e) = sender.send(()) {
                            warn!("Failed to send response to WebGPURequest::Exit ({})", e)
                        }
                        break;
                    },
                    WebGPURequest::DropCommandEncoder(id) => {
                        let global = &self.global;
                        global.command_encoder_remove(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeCommandEncoder(id)) {
                            warn!("Unable to send FreeCommandEncoder({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropCommandBuffer(id) => {
                        let global = &self.global;
                        global.command_buffer_remove(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeCommandBuffer(id)) {
                            warn!("Unable to send FreeCommandBuffer({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropDevice(device_id) => {
                        self.global.device_remove(device_id);
                        let pipeline_id = self.devices.remove(&device_id).unwrap();
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeDevice {
                            device_id,
                            pipeline_id,
                        }) {
                            warn!("Unable to send FreeDevice({:?}) ({:?})", device_id, e);
                        };
                    },
                    WebGPURequest::RequestAdapter {
                        sender,
                        options,
                        adapter_id,
                    } => {
                        let global = &self.global;
                        let response = self
                            .global
                            .request_adapter(
                                &options,
                                false,
                                wgpu_types::Backends::all(),
                                adapter_id,
                            )
                            .map(|adapter_id| {
                                // TODO: can we do this lazily
                                let adapter_info = global.adapter_get_info(adapter_id);
                                let limits = global.adapter_limits(adapter_id);
                                let features = global.adapter_features(adapter_id);
                                Adapter {
                                    adapter_info,
                                    adapter_id: WebGPUAdapter(adapter_id),
                                    features: features.features_webgpu,
                                    limits,
                                    channel: WebGPU(self.sender.clone()),
                                }
                            })
                            .map_err(|err| err.to_string());

                        if let Err(e) = sender.send(Some(response)) {
                            warn!(
                                "Failed to send response to WebGPURequest::RequestAdapter ({})",
                                e
                            )
                        }
                    },
                    WebGPURequest::RequestDevice {
                        sender,
                        adapter_id,
                        descriptor,
                        device_id,
                        queue_id,
                        pipeline_id,
                    } => {
                        // enable external texture support if available
                        let adapter_features = self.global.adapter_features(adapter_id.0);
                        let additional_features =
                            if adapter_features.contains(wgpu_types::Features::EXTERNAL_TEXTURE) {
                                wgpu_types::Features::EXTERNAL_TEXTURE
                            } else {
                                wgpu_types::Features::empty()
                            };
                        let device = WebGPUDevice(device_id);
                        let queue = WebGPUQueue(queue_id);
                        let result = self
                            .global
                            .adapter_request_device(
                                adapter_id.0,
                                &descriptor,
                                wgpu_types::Trace::Off,
                                additional_features,
                                device_id,
                                queue_id,
                            )
                            .map(|_| {
                                {
                                    self.devices.insert(device_id, pipeline_id);
                                }
                                let script_sender = self.script_sender.clone();
                                let callback = Box::from(move |reason, msg| {
                                    if let Err(e) = script_sender.send(WebGPUMsg::DeviceLost {
                                        device,
                                        pipeline_id,
                                        reason,
                                        msg,
                                    }) {
                                        warn!("Failed to send WebGPUMsg::DeviceLost: {e}");
                                    }
                                });
                                self.global
                                    .device_set_device_lost_closure(device_id, callback);
                                let sender = self.script_sender.clone();
                                self.global.device_on_uncaptured_error(
                                    device_id,
                                    Arc::new(move |error| {
                                        sender
                                            .send(WebGPUMsg::UncapturedError {
                                                device,
                                                pipeline_id,
                                                error: error.into(),
                                            })
                                            .unwrap();
                                    }),
                                );
                                let mut descriptor = descriptor;
                                descriptor.required_limits = self.global.device_limits(device_id);
                                descriptor.required_features =
                                    self.global.device_features(device_id).features_webgpu;
                                (
                                    descriptor,
                                    additional_features
                                        .contains(wgpu_types::Features::EXTERNAL_TEXTURE),
                                )
                            });

                        if let Err(e) = sender.send((device, queue, result)) {
                            warn!(
                                "Failed to send response to WebGPURequest::RequestDevice ({})",
                                e
                            )
                        }
                    },
                    WebGPURequest::ComputePassCommand {
                        compute_pass_id,
                        compute_command,
                        device_id: _,
                    } => {
                        self.global
                            .handle_compute_pass_command(compute_pass_id, compute_command);
                    },
                    WebGPURequest::RenderPassCommand {
                        render_pass_id,
                        render_command,
                        device_id: _,
                    } => {
                        self.global
                            .handle_render_pass_command(render_pass_id, render_command);
                    },
                    WebGPURequest::Submit {
                        device_id: _,
                        queue_id,
                        command_buffers,
                    } => {
                        self.global.queue_submit(queue_id, &command_buffers);
                    },
                    WebGPURequest::UnmapBuffer {
                        buffer_id,
                        buffer_update,
                    } => {
                        if let BufferUpdate::Write(data, range) = &buffer_update &&
                            let Ok(mapping) = self.global.buffer_get_mapped_range(
                                buffer_id,
                                range.start,
                                Some(range.end - range.start),
                            )
                        {
                            mapping.write_slice().copy_from_slice(
                                &data.as_ref()[range.start as usize..range.end as usize],
                            );
                        }
                        let data = match buffer_update {
                            BufferUpdate::Read(generic_shared_memory) => generic_shared_memory,
                            BufferUpdate::Write(generic_shared_memory, _) => generic_shared_memory,
                        };
                        self.buffers.lock().unwrap().insert(buffer_id, data);

                        self.global.buffer_unmap(buffer_id);
                    },
                    WebGPURequest::WriteBuffer {
                        device_id: _,
                        queue_id,
                        buffer_id,
                        buffer_offset,
                        data,
                    } => {
                        self.global.queue_write_buffer(
                            queue_id,
                            buffer_id,
                            buffer_offset as BufferAddress,
                            &data,
                        );
                    },
                    WebGPURequest::WriteTexture {
                        device_id: _,
                        queue_id,
                        texture_cv,
                        data_layout,
                        size,
                        data,
                    } => {
                        self.global.queue_write_texture(
                            queue_id,
                            &texture_cv,
                            &data,
                            &data_layout,
                            &size,
                        );
                    },
                    WebGPURequest::CopyExternalImageToTexture {
                        device_id,
                        queue_id,
                        usable_source,
                        destination,
                        dest_tex_descriptor,
                        copy_size,
                    } => {
                        // device and queue timeline of https://www.w3.org/TR/webgpu/#dom-gpuqueue-copyexternalimagetotexture
                        // If any of the following requirements are unmet, generate a validation error and return.
                        // usability must be good.
                        let Some(source) = usable_source else {
                            self.global.device_handle_error(
                                device_id,
                                Error::Validation("Source is not usable".to_string()),
                                None,
                                "GPUQueue.copyExternalImageToTexture",
                            );
                            continue;
                        };
                        // texture.usage must include both RENDER_ATTACHMENT
                        if !dest_tex_descriptor
                            .usage
                            .contains(TextureUsages::RENDER_ATTACHMENT)
                        {
                            self.global.device_handle_error(
                                device_id,
                                Error::Validation(
                                    "Texture usage must include RENDER_ATTACHMENT".to_string(),
                                ),
                                None,
                                "GPUQueue.copyExternalImageToTexture",
                            );
                            continue;
                        }
                        // texture.dimension must be "2d".
                        if dest_tex_descriptor.dimension != TextureDimension::D2 {
                            self.global.device_handle_error(
                                device_id,
                                Error::Validation("Texture dimension must be 2d".to_string()),
                                None,
                                "GPUQueue.copyExternalImageToTexture",
                            );
                            continue;
                        }
                        // texture.format must be a plain color format supporting RENDER_ATTACHMENT and be a unorm/unorm-srgb or float/ufloat format (not snorm, uint, or sint).
                        // currently to to hard to check
                        // the rest will be checked as part of write texture
                        self.global.queue_write_texture(
                            queue_id,
                            &destination,
                            source.data(),
                            &TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(source.size().width * 4),
                                rows_per_image: None,
                            },
                            &copy_size,
                        );
                    },
                    WebGPURequest::QueueOnSubmittedWorkDone { sender, queue_id } => {
                        let global = &self.global;
                        let token = self.poller.token();
                        let callback = Box::from(move || {
                            drop(token);
                            if let Err(e) = sender.send(()) {
                                warn!("Could not send SubmittedWorkDone Response ({})", e);
                            }
                        });
                        global.queue_on_submitted_work_done(queue_id, callback);
                        self.poller.wake();
                    },
                    WebGPURequest::DropTexture(id) => {
                        let global = &self.global;
                        global.texture_remove(id);
                        self.poller.wake();
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeTexture(id)) {
                            warn!("Unable to send FreeTexture({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropAdapter(id) => {
                        let global = &self.global;
                        global.adapter_remove(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeAdapter(id)) {
                            warn!("Unable to send FreeAdapter({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropBuffer(id) => {
                        let global = &self.global;
                        global.buffer_remove(id);
                        self.buffers.lock().unwrap().remove(&id);
                        self.poller.wake();
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeBuffer(id)) {
                            warn!("Unable to send FreeBuffer({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropPipelineLayout(id) => {
                        let global = &self.global;
                        global.pipeline_layout_remove(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreePipelineLayout(id)) {
                            warn!("Unable to send FreePipelineLayout({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropComputePipeline(id) => {
                        let global = &self.global;
                        global.compute_pipeline_remove(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeComputePipeline(id))
                        {
                            warn!("Unable to send FreeComputePipeline({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropComputePass(id) => {
                        self.global.compute_pass_remove(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeComputePass(id)) {
                            warn!("Unable to send FreeComputePass({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropRenderPass(id) => {
                        self.global.render_pass_remove(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeRenderPass(id)) {
                            warn!("Unable to send FreeRenderPass({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropRenderPipeline(id) => {
                        let global = &self.global;
                        global.render_pipeline_remove(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeRenderPipeline(id)) {
                            warn!("Unable to send FreeRenderPipeline({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropBindGroup(id) => {
                        let global = &self.global;
                        global.bind_group_remove(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeBindGroup(id)) {
                            warn!("Unable to send FreeBindGroup({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropBindGroupLayout(id) => {
                        let global = &self.global;
                        global.bind_group_layout_remove(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeBindGroupLayout(id))
                        {
                            warn!("Unable to send FreeBindGroupLayout({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropTextureView(id) => {
                        let global = &self.global;
                        global.texture_view_remove(id);
                        self.poller.wake();
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeTextureView(id)) {
                            warn!("Unable to send FreeTextureView({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropSampler(id) => {
                        let global = &self.global;
                        global.sampler_remove(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeSampler(id)) {
                            warn!("Unable to send FreeSampler({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropShaderModule(id) => {
                        let global = &self.global;
                        global.shader_module_remove(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeShaderModule(id)) {
                            warn!("Unable to send FreeShaderModule({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropRenderBundleEncoder(id) => {
                        let global = &self.global;
                        global.render_bundle_encoder_remove(id);
                        if let Err(e) = self
                            .script_sender
                            .send(WebGPUMsg::FreeRenderBundleEncoder(id))
                        {
                            warn!("Unable to send FreeRenderBundleEncoder({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropRenderBundle(id) => {
                        let global = &self.global;
                        global.render_bundle_remove(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeRenderBundle(id)) {
                            warn!("Unable to send FreeRenderBundle({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropQuerySet(id) => {
                        let global = &self.global;
                        global.query_set_remove(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeQuerySet(id)) {
                            warn!("Unable to send FreeQuerySet({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::PushErrorScope { device_id, filter } => {
                        self.global.device_push_error_scope(device_id, filter);
                    },
                    WebGPURequest::DispatchError { device_id, error } => {
                        self.global.device_handle_error(device_id, error, None, "");
                    },
                    WebGPURequest::PopErrorScope {
                        device_id,
                        callback: sender,
                    } => {
                        let result = self.global.device_pop_error_scope(device_id);
                        let result = result
                            .map(|error| error.map(|error| error.into()))
                            .map_err(|_| ());
                        if let Err(error) = sender.send(result) {
                            warn!("Error while sending PopErrorScope result: {error}");
                        }
                    },
                    WebGPURequest::ComputeGetBindGroupLayout {
                        device_id: _,
                        pipeline_id,
                        index,
                        id,
                    } => {
                        self.global
                            .compute_pipeline_get_bind_group_layout(pipeline_id, index, id);
                    },
                    WebGPURequest::RenderGetBindGroupLayout {
                        device_id: _,
                        pipeline_id,
                        index,
                        id,
                    } => {
                        self.global
                            .render_pipeline_get_bind_group_layout(pipeline_id, index, id);
                    },
                    WebGPURequest::CreateQuerySet {
                        device_id,
                        query_set_id,
                        descriptor,
                    } => {
                        self.global
                            .device_create_query_set(device_id, &descriptor, query_set_id);
                    },
                    WebGPURequest::CreatePlanarTexture {
                        device_id,
                        size,
                        format,
                        texture_id,
                        texture_view_id,
                    } => {
                        self.global.device_create_texture(
                            device_id,
                            &TextureDescriptor {
                                label: None,
                                size: Extent3d {
                                    width: size.width,
                                    height: size.height,
                                    depth_or_array_layers: 1,
                                },
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: TextureDimension::D2,
                                format: match format {
                                    pixels::SnapshotPixelFormat::RGBA => TextureFormat::Rgba8Unorm,
                                    pixels::SnapshotPixelFormat::BGRA => TextureFormat::Bgra8Unorm,
                                },
                                usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
                                view_formats: Vec::new(),
                                texture_binding_view_dimension: None,
                            },
                            texture_id,
                        );
                        self.global.texture_create_view(
                            texture_id,
                            &TextureViewDescriptor {
                                ..Default::default()
                            },
                            texture_view_id,
                        );
                    },
                    WebGPURequest::UpdatePlanarTexture {
                        device_id: _,
                        queue_id,
                        texture_id,
                        snapshot,
                    } => {
                        self.global.queue_write_texture(
                            queue_id,
                            &TexelCopyTextureInfo {
                                texture: texture_id,
                                mip_level: 0,
                                origin: Origin3d::ZERO,
                                aspect: TextureAspect::All,
                            },
                            snapshot.data(),
                            &TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(snapshot.size().width * 4),
                                rows_per_image: None,
                            },
                            &Extent3d {
                                width: snapshot.size().width,
                                height: snapshot.size().height,
                                depth_or_array_layers: 1,
                            },
                        );
                    },
                    WebGPURequest::DropPlanarTexture(id, view_id) => {
                        self.global.texture_view_remove(view_id);
                        self.global.texture_remove(id);
                        self.poller.wake();
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeTextureView(view_id))
                        {
                            warn!("Unable to send FreeTextureView({:?}) ({:?})", view_id, e);
                        };
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeTexture(id)) {
                            warn!("Unable to send FreeTexture({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::ImportExternalTexture {
                        device_id,
                        external_texture_id,
                        size,
                        label,
                        plane0,
                    } => {
                        let desc = ExternalTextureDescriptor {
                            label: Some(label.into()),
                            width: size.width,
                            height: size.height,
                            format: ExternalTextureFormat::Rgba,
                            yuv_conversion_matrix: [0.; 16],
                            gamut_conversion_matrix: [
                                1., 0., 0., //
                                0., 1., 0., //
                                0., 0., 1., //
                            ],
                            src_transfer_function: ExternalTextureTransferFunction::default(),
                            dst_transfer_function: ExternalTextureTransferFunction::default(),
                            sample_transform: [
                                1., 0., //
                                0., 1., //
                                0., 0., //
                            ],
                            load_transform: [
                                1., 0., //
                                0., 1., //
                                0., 0., //
                            ],
                        };
                        if let Some(plane0) = plane0 {
                            self.global.device_create_external_texture(
                                device_id,
                                &desc,
                                &[plane0],
                                external_texture_id,
                            );
                        } else {
                            self.global.create_external_texture_error(
                                device_id,
                                external_texture_id,
                                &desc,
                            );
                        }
                    },
                    WebGPURequest::DestroyExternalTexture(id) => {
                        self.global.external_texture_destroy(id);
                    },
                    WebGPURequest::DropExternalTexture(id) => {
                        self.global.external_texture_remove(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeExternalTexture(id))
                        {
                            warn!("Unable to send FreeExternalTexture({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DestroyQuerySet(query_set_id) => {
                        self.global.query_set_destroy(query_set_id);
                    },
                    WebGPURequest::CreateRenderBundleEncoder {
                        device_id,
                        render_bundle_encoder_id,
                        desc,
                    } => {
                        self.global.device_create_render_bundle_encoder(
                            device_id,
                            &desc,
                            render_bundle_encoder_id,
                        );
                    },
                    WebGPURequest::RenderBundleEncoderCommand {
                        render_bundle_encoder_id,
                        render_command,
                        device_id: _,
                    } => {
                        self.global.handle_render_bundle_encoder_command(
                            render_bundle_encoder_id,
                            render_command,
                        );
                    },
                }
            }
        }
        if let Err(e) = self.script_sender.send(WebGPUMsg::Exit) {
            warn!("Failed to send WebGPUMsg::Exit to script ({})", e);
        }
    }
}
