/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Data and main loop of WebGPU thread.

use std::borrow::Cow;
use std::slice;
use std::sync::{Arc, Mutex};

use log::{info, warn};
use paint_api::{CrossProcessPaintApi, WebRenderExternalImageIdManager, WebRenderImageHandlerType};
use rustc_hash::FxHashMap;
use servo_base::generic_channel::{GenericReceiver, GenericSender, GenericSharedMemory};
use servo_base::id::PipelineId;
use servo_config::pref;
use webgpu_traits::{
    Adapter, DeviceLostReason, Error, ErrorScope, Mapping, Pipeline, PopError, ShaderCompilationInfo, WebGPU, WebGPUAdapter, WebGPUContextId, WebGPUDevice, WebGPUMsg, WebGPUQueue, WebGPURequest, apply_render_bundle_command, apply_render_command,
};
use webrender_api::ExternalImageId;
use wgc::command::{ComputePassDescriptor};
use wgc::device::DeviceDescriptor;
use wgc::id;
use wgc::id::DeviceId;
use wgc::pipeline::ShaderModuleDescriptor;
use wgc::resource::BufferMapOperation;
pub use wgpu_core as wgc;
use wgpu_core::command::{RenderPassDescriptor, TexelCopyTextureInfo};
use wgpu_core::resource::{BufferAccessResult, TextureViewDescriptor};
pub use wgpu_types as wgt;
use wgpu_types::error::WebGpuError;
use wgpu_types::{
    ExperimentalFeatures, Extent3d, ExternalTextureDescriptor, ExternalTextureFormat,
    ExternalTextureTransferFunction, MemoryHints, Origin3d, TexelCopyBufferLayout, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use wgt::InstanceDescriptor;

use crate::canvas_context::WebGpuExternalImageMap;
use crate::poll_thread::Poller;

#[derive(Eq, Hash, PartialEq)]
pub(crate) struct DeviceScope {
    pub device_id: DeviceId,
    pub pipeline_id: PipelineId,
    /// <https://www.w3.org/TR/webgpu/#dom-gpudevice-errorscopestack-slot>
    ///
    /// Is `None` if device is lost
    pub error_scope_stack: Option<Vec<ErrorScope>>,
    // TODO:
    // Queue for this device (to remove transmutes)
    // queue_id: QueueId,
    // Poller for this device
    // poller: Poller,
}

impl DeviceScope {
    pub fn new(device_id: DeviceId, pipeline_id: PipelineId) -> Self {
        Self {
            device_id,
            pipeline_id,
            error_scope_stack: Some(Vec::new()),
        }
    }
}

#[expect(clippy::upper_case_acronyms)] // Name of the library
pub(crate) struct WGPU {
    receiver: GenericReceiver<WebGPURequest>,
    sender: GenericSender<WebGPURequest>,
    pub(crate) script_sender: GenericSender<WebGPUMsg>,
    pub(crate) global: Arc<wgc::global::Global>,
    devices: Arc<Mutex<FxHashMap<DeviceId, DeviceScope>>>,
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
            wgt::Backends::PRIMARY
        } else {
            info!(
                "Selecting backends based on dom.webgpu.wgpu_backend pref: {:?}",
                backend_pref
            );
            wgt::Backends::from_comma_list(&backend_pref)
        };
        let global = Arc::new(wgc::global::Global::new(
            "wgpu-core",
            InstanceDescriptor {
                backends,
                backend_options: wgt::BackendOptions {
                    gl: wgt::GlBackendOptions {
                        gles_minor_version: wgt::Gles3MinorVersion::Automatic,
                        fence_behavior: wgt::GlFenceBehavior::Normal,
                        debug_fns: wgt::GlDebugFns::Auto,
                    },
                    dx12: wgt::Dx12BackendOptions {
                        ..Default::default()
                    },
                    noop: wgt::NoopBackendOptions::default(),
                },

                flags: wgt::InstanceFlags::from_build_config() |
                    wgt::InstanceFlags::AUTOMATIC_TIMESTAMP_NORMALIZATION,
                // TODO(sagudev): firefox actually sets this, but it can cause OOM for us
                // meaning that we are likely leaking something
                memory_budget_thresholds: wgt::MemoryBudgetThresholds {
                    for_resource_creation: Some(95),
                    for_device_loss: Some(99),
                },
                display: None,
            },
            None,
        ));
        WGPU {
            poller: Poller::new(Arc::clone(&global)),
            receiver,
            sender,
            script_sender,
            global,
            devices: Arc::new(Mutex::new(FxHashMap::default())),
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
                        device_id,
                        host_map,
                        offset,
                        size,
                    } => {
                        let glob = Arc::clone(&self.global);
                        let resp_sender = sender.clone();
                        let token = self.poller.token();
                        let callback = Box::from(move |result: BufferAccessResult| {
                            drop(token);
                            let response = result.and_then(|_| {
                                let global = &glob;
                                let (slice_pointer, range_size) =
                                    global.buffer_get_mapped_range(buffer_id, offset, size)?;
                                // SAFETY: guarantee to be safe from wgpu
                                let data = unsafe {
                                    slice::from_raw_parts(
                                        slice_pointer.as_ptr(),
                                        range_size as usize,
                                    )
                                };

                                Ok(Mapping {
                                    data: GenericSharedMemory::from_bytes(data),
                                    range: offset..offset + range_size,
                                    mode: host_map,
                                })
                            });
                            if let Err(e) = resp_sender.send(response) {
                                warn!("Could not send BufferMapAsync Response ({})", e);
                            }
                        });

                        let operation = BufferMapOperation {
                            host: host_map,
                            callback: Some(callback),
                        };
                        let global = &self.global;
                        let result = global.buffer_map_async(buffer_id, offset, size, operation);
                        self.poller.wake();
                        // Per spec we also need to raise validation error here
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::CommandEncoderFinish {
                        command_encoder_id,
                        device_id,
                        desc,
                        command_buffer_id,
                    } => {
                        let global = &self.global;
                        let (_, error) = global.command_encoder_finish(
                            command_encoder_id,
                            &desc,
                            Some(command_buffer_id),
                        );
                        self.maybe_dispatch_wgpu_error(device_id, error.map(|(_, e)| e));
                    },
                    WebGPURequest::CopyBufferToBuffer {
                        device_id,
                        command_encoder_id,
                        source_id,
                        source_offset,
                        destination_id,
                        destination_offset,
                        size,
                    } => {
                        let global = &self.global;
                        let result = global.command_encoder_copy_buffer_to_buffer(
                            command_encoder_id,
                            source_id,
                            source_offset,
                            destination_id,
                            destination_offset,
                            Some(size),
                        );
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::CopyBufferToTexture {
                        device_id,
                        command_encoder_id,
                        source,
                        destination,
                        copy_size,
                    } => {
                        let global = &self.global;
                        let result = global.command_encoder_copy_buffer_to_texture(
                            command_encoder_id,
                            &source,
                            &destination,
                            &copy_size,
                        );
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::CopyTextureToBuffer {
                        device_id,
                        command_encoder_id,
                        source,
                        destination,
                        copy_size,
                    } => {
                        let global = &self.global;
                        let result = global.command_encoder_copy_texture_to_buffer(
                            command_encoder_id,
                            &source,
                            &destination,
                            &copy_size,
                        );
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::CopyTextureToTexture {
                        device_id,
                        command_encoder_id,
                        source,
                        destination,
                        copy_size,
                    } => {
                        let global = &self.global;
                        let result = global.command_encoder_copy_texture_to_texture(
                            command_encoder_id,
                            &source,
                            &destination,
                            &copy_size,
                        );
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::CommandEncoderPushDebugGroup {
                        device_id,
                        command_encoder_id,
                        label,
                    } => {
                        let result = self
                            .global
                            .command_encoder_push_debug_group(command_encoder_id, &label);
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::CommandEncoderPopDebugGroup {
                        device_id,
                        command_encoder_id,
                    } => {
                        let result = self
                            .global
                            .command_encoder_pop_debug_group(command_encoder_id);
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::CommandEncoderInsertDebugMarker {
                        device_id,
                        command_encoder_id,
                        label,
                    } => {
                        let result = self
                            .global
                            .command_encoder_insert_debug_marker(command_encoder_id, &label);
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::CreateBindGroup {
                        device_id,
                        bind_group_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let (_, error) = global.device_create_bind_group(
                            device_id,
                            &descriptor,
                            Some(bind_group_id),
                        );
                        self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::CreateBindGroupLayout {
                        device_id,
                        bind_group_layout_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        if let Some(desc) = descriptor {
                            let (_, error) = global.device_create_bind_group_layout(
                                device_id,
                                &desc,
                                Some(bind_group_layout_id),
                            );

                            self.maybe_dispatch_wgpu_error(device_id, error);
                        }
                    },
                    WebGPURequest::CreateBuffer {
                        device_id,
                        buffer_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let (_, error) =
                            global.device_create_buffer(device_id, &descriptor, Some(buffer_id));

                        self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::CreateCommandEncoder {
                        device_id,
                        command_encoder_id,
                        desc,
                    } => {
                        let global = &self.global;
                        let (_, error) = global.device_create_command_encoder(
                            device_id,
                            &desc,
                            Some(command_encoder_id),
                        );

                        self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::CreateComputePipeline {
                        device_id,
                        compute_pipeline_id,
                        descriptor,
                        async_sender: sender,
                    } => {
                        let global = &self.global;
                        let (_, error) = global.device_create_compute_pipeline(
                            device_id,
                            &descriptor,
                            Some(compute_pipeline_id),
                        );
                        if let Some(sender) = sender {
                            let res = match error.and_then(Error::from_wgpu_error) {
                                // if device is lost we must return pipeline and not raise any error
                                None => Ok(Pipeline {
                                    id: compute_pipeline_id,
                                    label: descriptor.label.unwrap_or_default().to_string(),
                                }),
                                Some(e) => Err(e),
                            };
                            if let Err(e) = sender.send(res) {
                                warn!("Failed sending WebGPUComputePipelineResponse {e:?}");
                            }
                        } else {
                            self.maybe_dispatch_wgpu_error(device_id, error);
                        }
                    },
                    WebGPURequest::CreatePipelineLayout {
                        device_id,
                        pipeline_layout_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let (_, error) = global.device_create_pipeline_layout(
                            device_id,
                            &descriptor,
                            Some(pipeline_layout_id),
                        );
                        self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::CreateRenderPipeline {
                        device_id,
                        render_pipeline_id,
                        descriptor,
                        async_sender: sender,
                    } => {
                        let global = &self.global;
                        let (_, error) = global.device_create_render_pipeline(
                            device_id,
                            &descriptor,
                            Some(render_pipeline_id),
                        );

                        if let Some(sender) = sender {
                            let res = match error.and_then(Error::from_wgpu_error) {
                                // if device is lost we must return pipeline and not raise any error
                                None => Ok(Pipeline {
                                    id: render_pipeline_id,
                                    label: descriptor.label.unwrap_or_default().to_string(),
                                }),
                                Some(e) => Err(e),
                            };
                            if let Err(e) = sender.send(res) {
                                warn!("Failed sending WebGPURenderPipelineResponse {e:?}");
                            }
                        } else {
                            self.maybe_dispatch_wgpu_error(device_id, error);
                        }
                    },
                    WebGPURequest::CreateSampler {
                        device_id,
                        sampler_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let (_, error) =
                            global.device_create_sampler(device_id, &descriptor, Some(sampler_id));
                        self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::CreateShaderModule {
                        device_id,
                        program_id,
                        program,
                        label,
                        callback: sender,
                    } => {
                        let global = &self.global;
                        let source =
                            wgpu_core::pipeline::ShaderModuleSource::Wgsl(Cow::Borrowed(&program));
                        let desc = ShaderModuleDescriptor {
                            label: label.map(|s| s.into()),
                            runtime_checks: wgt::ShaderRuntimeChecks::checked(),
                        };
                        let (_, error) = global.device_create_shader_module(
                            device_id,
                            &desc,
                            source,
                            Some(program_id),
                        );
                        if let Err(e) = sender.send(
                            error
                                .as_ref()
                                .map(|e| ShaderCompilationInfo::from(e, &program)),
                        ) {
                            warn!("Failed to send CompilationInfo {e:?}");
                        }
                        self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::CreateContext {
                        buffer_ids,
                        size,
                        sender,
                    } => {
                        let id = self
                            .webrender_external_image_id_manager
                            .next_id(WebRenderImageHandlerType::WebGpu);
                        let context_id = WebGPUContextId(id.0);

                        if let Err(error) = sender.send(context_id) {
                            warn!("Failed to send ContextId to new context ({error})");
                        };

                        self.create_context(context_id, size, buffer_ids);
                    },
                    WebGPURequest::Present {
                        context_id,
                        pending_texture,
                        size,
                        canvas_epoch,
                    } => {
                        self.present(context_id, pending_texture, size, canvas_epoch);
                    },
                    WebGPURequest::GetImage {
                        context_id,
                        pending_texture,
                        sender,
                    } => self.get_image(context_id, pending_texture, sender),
                    WebGPURequest::ValidateTextureDescriptor {
                        device_id,
                        texture_id,
                        descriptor,
                    } => {
                        // https://gpuweb.github.io/gpuweb/#dom-gpucanvascontext-configure
                        // validating TextureDescriptor by creating dummy texture
                        let global = &self.global;
                        let (_, error) =
                            global.device_create_texture(device_id, &descriptor, Some(texture_id));
                        global.texture_drop(texture_id);
                        self.poller.wake();
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeTexture(texture_id))
                        {
                            warn!("Unable to send FreeTexture({:?}) ({:?})", texture_id, e);
                        };
                        self.maybe_dispatch_wgpu_error(device_id, error);
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
                        let global = &self.global;
                        let (_, error) =
                            global.device_create_texture(device_id, &descriptor, Some(texture_id));
                        self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::CreateTextureView {
                        texture_id,
                        texture_view_id,
                        device_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        if let Some(desc) = descriptor {
                            let (_, error) = global.texture_create_view(
                                texture_id,
                                &desc,
                                Some(texture_view_id),
                            );

                            self.maybe_dispatch_wgpu_error(device_id, error);
                        }
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
                        global.command_encoder_drop(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeCommandEncoder(id)) {
                            warn!("Unable to send FreeCommandEncoder({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropCommandBuffer(id) => {
                        let global = &self.global;
                        global.command_buffer_drop(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeCommandBuffer(id)) {
                            warn!("Unable to send FreeCommandBuffer({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropDevice(device_id) => {
                        let global = &self.global;
                        global.device_drop(device_id);
                        let device_scope = self
                            .devices
                            .lock()
                            .unwrap()
                            .remove(&device_id)
                            .expect("Device should not be dropped by this point");
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeDevice {
                            device_id,
                            pipeline_id: device_scope.pipeline_id,
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
                            .request_adapter(&options, wgt::Backends::all(), Some(adapter_id))
                            .map(|adapter_id| {
                                // TODO: can we do this lazily
                                let adapter_info = global.adapter_get_info(adapter_id);
                                let limits = global.adapter_limits(adapter_id);
                                let features = global.adapter_features(adapter_id);
                                Adapter {
                                    adapter_info,
                                    adapter_id: WebGPUAdapter(adapter_id),
                                    features,
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
                        let mut desc = DeviceDescriptor {
                            label: descriptor.label.as_ref().map(crate::Cow::from),
                            required_features: descriptor.required_features,
                            required_limits: descriptor.required_limits.clone(),
                            memory_hints: MemoryHints::MemoryUsage,
                            trace: wgpu_types::Trace::Off,
                            experimental_features: ExperimentalFeatures::disabled(),
                        };
                        let global = &self.global;
                        // enable external texture support if available
                        let features = global.adapter_features(adapter_id.0);
                        if features.contains(wgpu_types::Features::EXTERNAL_TEXTURE) {
                            desc.required_features |= wgpu_types::Features::EXTERNAL_TEXTURE;
                        }
                        let device = WebGPUDevice(device_id);
                        let queue = WebGPUQueue(queue_id);
                        let result = global
                            .adapter_request_device(
                                adapter_id.0,
                                &desc,
                                Some(device_id),
                                Some(queue_id),
                            )
                            .map(|_| {
                                {
                                    self.devices.lock().unwrap().insert(
                                        device_id,
                                        DeviceScope::new(device_id, pipeline_id),
                                    );
                                }
                                let script_sender = self.script_sender.clone();
                                let devices = Arc::clone(&self.devices);
                                let callback = Box::from(move |reason, msg| {
                                    let reason = match reason {
                                        wgt::DeviceLostReason::Unknown => DeviceLostReason::Unknown,
                                        wgt::DeviceLostReason::Destroyed => {
                                            DeviceLostReason::Destroyed
                                        },
                                    };
                                    // make device lost by removing error scopes stack
                                    let _ = devices
                                        .lock()
                                        .unwrap()
                                        .get_mut(&device_id)
                                        .expect("Device should not be dropped by this point")
                                        .error_scope_stack
                                        .take();
                                    if let Err(e) = script_sender.send(WebGPUMsg::DeviceLost {
                                        device,
                                        pipeline_id,
                                        reason,
                                        msg,
                                    }) {
                                        warn!("Failed to send WebGPUMsg::DeviceLost: {e}");
                                    }
                                });
                                global.device_set_device_lost_closure(device_id, callback);
                                let mut descriptor = descriptor;
                                descriptor.required_limits = global.device_limits(device_id);
                                descriptor.required_features = global.device_features(device_id);
                                descriptor
                            })
                            .map_err(Into::into);

                        if let Err(e) = sender.send((device, queue, result)) {
                            warn!(
                                "Failed to send response to WebGPURequest::RequestDevice ({})",
                                e
                            )
                        }
                    },
                    WebGPURequest::BeginComputePass {
                        command_encoder_id,
                        compute_pass_id,
                        label,
                        timestamp_writes,
                        device_id,
                    } => {
                        let global = &self.global;
                        let (_, error) = global.command_encoder_begin_compute_pass_with_id(
                            command_encoder_id,
                            &ComputePassDescriptor {
                                label,
                                timestamp_writes,
                            },
                            Some(compute_pass_id)
                        );
                        self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::ComputePassSetPipeline {
                        compute_pass_id,
                        pipeline_id,
                        device_id,
                    } => {
                        let result = self.global.compute_pass_set_pipeline_with_id(compute_pass_id, pipeline_id);
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::ComputePassSetBindGroup {
                        compute_pass_id,
                        index,
                        bind_group_id,
                        offsets,
                        device_id,
                    } => {
                        let result = self.global.compute_pass_set_bind_group_with_id(
                            compute_pass_id,
                            index,
                            Some(bind_group_id),
                            &offsets,
                        );
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::ComputePassDispatchWorkgroups {
                        compute_pass_id,
                        x,
                        y,
                        z,
                        device_id,
                    } => {
                        let result = self.global.compute_pass_dispatch_workgroups_with_id(compute_pass_id, x, y, z);
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::ComputePassDispatchWorkgroupsIndirect {
                        compute_pass_id,
                        buffer_id,
                        offset,
                        device_id,
                    } => {
                        let result = self
                            .global
                            .compute_pass_dispatch_workgroups_indirect_with_id(compute_pass_id, buffer_id, offset);
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::ComputePassPushDebugGroup {
                        compute_pass_id,
                        label,
                        device_id,
                    } => {
                        let result = self.global.compute_pass_push_debug_group_with_id(compute_pass_id, &label, 0);
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::ComputePassPopDebugGroup {
                        compute_pass_id,
                        device_id,
                    } => {
                        let result = self.global.compute_pass_pop_debug_group_with_id(compute_pass_id);
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::ComputePassInsertDebugMarker {
                        compute_pass_id,
                        label,
                        device_id,
                    } => {
                        let result = self
                            .global
                            .compute_pass_insert_debug_marker_with_id(compute_pass_id, &label, 0);
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::EndComputePass {
                        compute_pass_id,
                        device_id,
                    } => {
                        // https://www.w3.org/TR/2024/WD-webgpu-20240703/#dom-gpucomputepassencoder-end
                        let result = self.global.compute_pass_end_with_id(compute_pass_id);
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::BeginRenderPass {
                        command_encoder_id,
                        render_pass_id,
                        label,
                        color_attachments,
                        depth_stencil_attachment,
                        timestamp_writes,
                        device_id,
                    } => {
                        let global = &self.global;
                        let desc = &RenderPassDescriptor {
                            label,
                            color_attachments: color_attachments.into(),
                            depth_stencil_attachment,
                            timestamp_writes,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        };
                        let (_, error) =
                            global.command_encoder_begin_render_pass_with_id(command_encoder_id, desc, Some(render_pass_id));
                        self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::RenderPassCommand {
                        render_pass_id,
                        render_command,
                        device_id,
                    } => {
                        let result = apply_render_command(&self.global, render_pass_id, render_command);
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::EndRenderPass {
                        render_pass_id,
                        device_id,
                    } => {
                        // https://www.w3.org/TR/2024/WD-webgpu-20240703/#dom-gpurenderpassencoder-end
                        let result = self.global.render_pass_end_with_id(render_pass_id);
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::Submit {
                        device_id,
                        queue_id,
                        command_buffers,
                    } => {
                        let global = &self.global;
                        let result = {
                            let _guard = self.poller.lock();
                            global.queue_submit(queue_id, &command_buffers)
                        };
                        self.maybe_dispatch_wgpu_error(device_id, result.err().map(|(_, x)| x));
                    },
                    WebGPURequest::UnmapBuffer { buffer_id, mapping } => {
                        let global = &self.global;
                        if let Some(mapping) = mapping &&
                            let Ok((slice_pointer, range_size)) = global.buffer_get_mapped_range(
                                buffer_id,
                                mapping.range.start,
                                Some(mapping.range.end - mapping.range.start),
                            )
                        {
                            unsafe {
                                slice::from_raw_parts_mut(
                                    slice_pointer.as_ptr(),
                                    range_size as usize,
                                )
                            }
                            .copy_from_slice(&mapping.data);
                        }
                        // Ignore result because this operation always succeed from user perspective
                        let _result = global.buffer_unmap(buffer_id);
                    },
                    WebGPURequest::WriteBuffer {
                        device_id,
                        queue_id,
                        buffer_id,
                        buffer_offset,
                        data,
                    } => {
                        let global = &self.global;
                        let result = global.queue_write_buffer(
                            queue_id,
                            buffer_id,
                            buffer_offset as wgt::BufferAddress,
                            &data,
                        );
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::WriteTexture {
                        device_id,
                        queue_id,
                        texture_cv,
                        data_layout,
                        size,
                        data,
                    } => {
                        let global = &self.global;
                        let _guard = self.poller.lock();
                        // TODO: Report result to content process
                        let result = global.queue_write_texture(
                            queue_id,
                            &texture_cv,
                            &data,
                            &data_layout,
                            &size,
                        );
                        drop(_guard);
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
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
                        let global = &self.global;
                        // If any of the following requirements are unmet, generate a validation error and return.
                        // usability must be good.
                        let Some(source) = usable_source else {
                            self.maybe_dispatch_error(
                                device_id,
                                Some(Error::Validation("Source is not usable".to_string())),
                            );
                            continue;
                        };
                        // texture.usage must include both RENDER_ATTACHMENT
                        if !dest_tex_descriptor
                            .usage
                            .contains(TextureUsages::RENDER_ATTACHMENT)
                        {
                            self.maybe_dispatch_error(
                                device_id,
                                Some(Error::Validation(
                                    "Texture usage must include RENDER_ATTACHMENT".to_string(),
                                )),
                            );
                            continue;
                        }
                        // texture.dimension must be "2d".
                        if dest_tex_descriptor.dimension != TextureDimension::D2 {
                            self.maybe_dispatch_error(
                                device_id,
                                Some(Error::Validation(
                                    "Texture dimension must be 2d".to_string(),
                                )),
                            );
                            continue;
                        }
                        // texture.format must be a plain color format supporting RENDER_ATTACHMENT and be a unorm/unorm-srgb or float/ufloat format (not snorm, uint, or sint).
                        // currently to to hard to check
                        // the rest will be checked as part of write texture
                        let _guard = self.poller.lock();
                        let result = global.queue_write_texture(
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
                        drop(_guard);
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
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
                        global.texture_drop(id);
                        self.poller.wake();
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeTexture(id)) {
                            warn!("Unable to send FreeTexture({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropAdapter(id) => {
                        let global = &self.global;
                        global.adapter_drop(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeAdapter(id)) {
                            warn!("Unable to send FreeAdapter({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropBuffer(id) => {
                        let global = &self.global;
                        global.buffer_drop(id);
                        self.poller.wake();
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeBuffer(id)) {
                            warn!("Unable to send FreeBuffer({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropPipelineLayout(id) => {
                        let global = &self.global;
                        global.pipeline_layout_drop(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreePipelineLayout(id)) {
                            warn!("Unable to send FreePipelineLayout({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropComputePipeline(id) => {
                        let global = &self.global;
                        global.compute_pipeline_drop(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeComputePipeline(id))
                        {
                            warn!("Unable to send FreeComputePipeline({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropComputePass(id) => {
                        self.global.compute_pass_drop(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeComputePass(id)) {
                            warn!("Unable to send FreeComputePass({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropRenderPass(id) => {
                        self.global.render_pass_drop(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeRenderPass(id)) {
                            warn!("Unable to send FreeRenderPass({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropRenderPipeline(id) => {
                        let global = &self.global;
                        global.render_pipeline_drop(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeRenderPipeline(id)) {
                            warn!("Unable to send FreeRenderPipeline({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropBindGroup(id) => {
                        let global = &self.global;
                        global.bind_group_drop(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeBindGroup(id)) {
                            warn!("Unable to send FreeBindGroup({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropBindGroupLayout(id) => {
                        let global = &self.global;
                        global.bind_group_layout_drop(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeBindGroupLayout(id))
                        {
                            warn!("Unable to send FreeBindGroupLayout({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropTextureView(id) => {
                        let global = &self.global;
                        global.texture_view_drop(id);
                        self.poller.wake();
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeTextureView(id)) {
                            warn!("Unable to send FreeTextureView({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropSampler(id) => {
                        let global = &self.global;
                        global.sampler_drop(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeSampler(id)) {
                            warn!("Unable to send FreeSampler({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropShaderModule(id) => {
                        let global = &self.global;
                        global.shader_module_drop(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeShaderModule(id)) {
                            warn!("Unable to send FreeShaderModule({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropRenderBundleEncoder(id) => {
                        let global = &self.global;
                        global.render_bundle_encoder_drop(id);
                        if let Err(e) = self
                            .script_sender
                            .send(WebGPUMsg::FreeRenderBundleEncoder(id))
                        {
                            warn!(
                                "Unable to send FreeRenderBundleEncoder({:?}) ({:?})",
                                id, e
                            );
                        };
                    },
                    WebGPURequest::DropRenderBundle(id) => {
                        let global = &self.global;
                        global.render_bundle_drop(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeRenderBundle(id)) {
                            warn!("Unable to send FreeRenderBundle({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropQuerySet(id) => {
                        let global = &self.global;
                        global.query_set_drop(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeQuerySet(id)) {
                            warn!("Unable to send FreeQuerySet({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::PushErrorScope { device_id, filter } => {
                        // <https://www.w3.org/TR/webgpu/#dom-gpudevice-pusherrorscope>
                        let mut devices = self.devices.lock().unwrap();
                        let device_scope = devices
                            .get_mut(&device_id)
                            .expect("Device should not be dropped by this point");
                        if let Some(error_scope_stack) = &mut device_scope.error_scope_stack {
                            error_scope_stack.push(ErrorScope::new(filter));
                        } // else device is lost
                    },
                    WebGPURequest::DispatchError { device_id, error } => {
                        self.dispatch_error(device_id, error);
                    },
                    WebGPURequest::PopErrorScope {
                        device_id,
                        callback: sender,
                    } => {
                        // <https://www.w3.org/TR/webgpu/#dom-gpudevice-poperrorscope>
                        let mut devices = self.devices.lock().unwrap();
                        let device_scope = devices
                            .get_mut(&device_id)
                            .expect("Device should not be dropped by this point");
                        let result =
                            if let Some(error_scope_stack) = &mut device_scope.error_scope_stack {
                                if let Some(error_scope) = error_scope_stack.pop() {
                                    Ok(
                                        // TODO: Do actual selection instead of selecting first error
                                        error_scope.errors.first().cloned(),
                                    )
                                } else {
                                    Err(PopError::Empty)
                                }
                            } else {
                                // This means the device has been lost.
                                Err(PopError::Lost)
                            };
                        if let Err(error) = sender.send(result) {
                            warn!("Error while sending PopErrorScope result: {error}");
                        }
                    },
                    WebGPURequest::ComputeGetBindGroupLayout {
                        device_id,
                        pipeline_id,
                        index,
                        id,
                    } => {
                        let global = &self.global;
                        let (_, error) = global.compute_pipeline_get_bind_group_layout(
                            pipeline_id,
                            index,
                            Some(id),
                        );
                        self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::RenderGetBindGroupLayout {
                        device_id,
                        pipeline_id,
                        index,
                        id,
                    } => {
                        let global = &self.global;
                        let (_, error) = global.render_pipeline_get_bind_group_layout(
                            pipeline_id,
                            index,
                            Some(id),
                        );
                        self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::CreateQuerySet {
                        device_id,
                        query_set_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let (_, error) = global.device_create_query_set(
                            device_id,
                            &descriptor,
                            Some(query_set_id),
                        );
                        self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::ResolveQuerySet {
                        device_id,
                        command_encoder_id,
                        query_set_id,
                        start_query,
                        query_count,
                        destination,
                        destination_offset,
                    } => {
                        let global = &self.global;
                        let result = global.command_encoder_resolve_query_set(
                            command_encoder_id,
                            query_set_id,
                            start_query,
                            query_count,
                            destination,
                            destination_offset,
                        );
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                    WebGPURequest::CreatePlanarTexture {
                        device_id,
                        size,
                        format,
                        texture_id,
                        texture_view_id,
                    } => {
                        let (_, maybe_error) = self.global.device_create_texture(
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
                            },
                            Some(texture_id),
                        );
                        self.maybe_dispatch_error(
                            device_id,
                            maybe_error.map(|error| {
                                Error::Internal(format!(
                                    "Failed to create planar texture: {error:?}"
                                ))
                            }),
                        );
                        let (_, maybe_error) = self.global.texture_create_view(
                            texture_id,
                            &TextureViewDescriptor {
                                ..Default::default()
                            },
                            Some(texture_view_id),
                        );
                        self.maybe_dispatch_error(
                            device_id,
                            maybe_error.map(|error| {
                                Error::Internal(format!(
                                    "Failed to create planar texture view: {error:?}"
                                ))
                            }),
                        );
                    },
                    WebGPURequest::UpdatePlanarTexture {
                        device_id,
                        queue_id,
                        texture_id,
                        snapshot,
                    } => {
                        let result = self.global.queue_write_texture(
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
                        self.maybe_dispatch_error(
                            device_id,
                            result.err().map(|error| {
                                Error::Internal(format!(
                                    "Failed to write planar texture: {error:?}"
                                ))
                            }),
                        );
                    },
                    WebGPURequest::DropPlanarTexture(id, view_id) => {
                        self.global.texture_view_drop(view_id);
                        self.global.texture_drop(id);
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
                            let (_, maybe_error) = self.global.device_create_external_texture(
                                device_id,
                                &desc,
                                &[plane0],
                                Some(external_texture_id),
                            );
                            self.maybe_dispatch_error(
                                device_id,
                                maybe_error.map(|error| {
                                    Error::Internal(format!(
                                        "Failed to import external texture: {error:?}"
                                    ))
                                }),
                            );
                        } else {
                            self.global
                                .create_external_texture_error(Some(external_texture_id), &desc);
                            self.maybe_dispatch_error(
                                device_id,
                                Some(Error::Validation("Usability is not good".to_string())),
                            );
                        }
                    },
                    WebGPURequest::DestroyExternalTexture(id) => {
                        self.global.external_texture_destroy(id);
                    },
                    WebGPURequest::DropExternalTexture(id) => {
                        self.global.external_texture_drop(id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeExternalTexture(id))
                        {
                            warn!("Unable to send FreeExternalTexture({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DestroyQuerySet(query_set_id) => {
                        self.global.query_set_destroy(query_set_id);
                    },
                    WebGPURequest::RenderBundleEncoderFinish {
                        render_bundle_encoder_id,
                        descriptor,
                        render_bundle_id,
                        device_id,
                    } => {
                        let global = &self.global;
                        let (_, error) = global.render_bundle_encoder_finish_with_id(
                            render_bundle_encoder_id,
                            &descriptor,
                            Some(render_bundle_id),
                        );

                        self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::CreateRenderBundleEncoder { device_id, render_bundle_encoder_id, desc } => {
                        let (_, error) = self.global.device_create_render_bundle_encoder_with_id(device_id, &desc, Some(render_bundle_encoder_id));
                        self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::RenderBundleEncoderCommand { render_bundle_encoder_id, render_command, device_id } => {
                        let result = apply_render_bundle_command(&self.global, render_bundle_encoder_id, render_command);
                        self.maybe_dispatch_wgpu_error(device_id, result.err());
                    },
                }
            }
        }
        if let Err(e) = self.script_sender.send(WebGPUMsg::Exit) {
            warn!("Failed to send WebGPUMsg::Exit to script ({})", e);
        }
    }

    #[inline]
    fn maybe_dispatch_wgpu_error<E: WebGpuError>(
        &mut self,
        device_id: id::DeviceId,
        error: Option<E>,
    ) {
        self.maybe_dispatch_error(device_id, error.and_then(Error::from_wgpu_error))
    }

    /// Dispatches error (if there is any)
    fn maybe_dispatch_error(&mut self, device_id: id::DeviceId, error: Option<Error>) {
        if let Some(error) = error {
            self.dispatch_error(device_id, error);
        }
    }

    /// <https://www.w3.org/TR/webgpu/#abstract-opdef-dispatch-error>
    fn dispatch_error(&mut self, device_id: id::DeviceId, error: Error) {
        log::trace!("Dispatching error for device {:?}: {:?}", device_id, error);
        let mut devices = self.devices.lock().unwrap();
        let device_scope = devices
            .get_mut(&device_id)
            .expect("Device should not be dropped by this point");
        if let Some(error_scope_stack) = &mut device_scope.error_scope_stack {
            if let Some(error_scope) = error_scope_stack
                .iter_mut()
                .rev()
                .find(|error_scope| error_scope.filter == error.filter())
            {
                error_scope.errors.push(error);
            } else if self
                .script_sender
                .send(WebGPUMsg::UncapturedError {
                    device: WebGPUDevice(device_id),
                    pipeline_id: device_scope.pipeline_id,
                    error: error.clone(),
                })
                .is_err()
            {
                warn!("Failed to send WebGPUMsg::UncapturedError: {error:?}");
            }
        } // else device is lost
    }
}
