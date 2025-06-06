/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Data and main loop of WebGPU thread.

use std::borrow::Cow;
use std::collections::HashMap;
use std::slice;
use std::sync::{Arc, Mutex};

use base::id::PipelineId;
use compositing_traits::{
    CrossProcessCompositorApi, WebrenderExternalImageRegistry, WebrenderImageHandlerType,
};
use ipc_channel::ipc::{IpcReceiver, IpcSender, IpcSharedMemory};
use log::{info, warn};
use servo_config::pref;
use webgpu_traits::{
    Adapter, ComputePassId, DeviceLostReason, Error, ErrorScope, Mapping, Pipeline, PopError,
    RenderPassId, ShaderCompilationInfo, WebGPU, WebGPUAdapter, WebGPUContextId, WebGPUDevice,
    WebGPUMsg, WebGPUQueue, WebGPURequest, apply_render_command,
};
use webrender_api::ExternalImageId;
use wgc::command::{ComputePass, ComputePassDescriptor, RenderPass};
use wgc::device::{DeviceDescriptor, ImplicitPipelineIds};
use wgc::id;
use wgc::id::DeviceId;
use wgc::pipeline::ShaderModuleDescriptor;
use wgc::resource::BufferMapOperation;
use wgpu_core::command::RenderPassDescriptor;
use wgpu_core::device::DeviceError;
use wgpu_core::pipeline::{CreateComputePipelineError, CreateRenderPipelineError};
use wgpu_core::resource::BufferAccessResult;
use wgpu_types::MemoryHints;
use wgt::InstanceDescriptor;
pub use {wgpu_core as wgc, wgpu_types as wgt};

use crate::poll_thread::Poller;
use crate::swapchain::WGPUImageMap;

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

/// This roughly matches <https://www.w3.org/TR/2024/WD-webgpu-20240703/#encoder-state>
#[derive(Debug, Default, Eq, PartialEq)]
enum Pass<P> {
    /// Pass is open (not ended)
    Open {
        /// Actual pass
        pass: P,
        /// we need to store valid field
        /// because wgpu does not invalidate pass on error
        valid: bool,
    },
    /// When pass is ended we need to drop it so we replace it with this
    #[default]
    Ended,
}

impl<P> Pass<P> {
    /// Creates new open pass
    fn new(pass: P, valid: bool) -> Self {
        Self::Open { pass, valid }
    }

    /// Replaces pass with ended
    fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

#[allow(clippy::upper_case_acronyms)] // Name of the library
pub(crate) struct WGPU {
    receiver: IpcReceiver<WebGPURequest>,
    sender: IpcSender<WebGPURequest>,
    pub(crate) script_sender: IpcSender<WebGPUMsg>,
    pub(crate) global: Arc<wgc::global::Global>,
    devices: Arc<Mutex<HashMap<DeviceId, DeviceScope>>>,
    // TODO: Remove this (https://github.com/gfx-rs/wgpu/issues/867)
    /// This stores first error on command encoder,
    /// because wgpu does not invalidate command encoder object
    /// (this is also reused for invalidation of command buffers)
    error_command_encoders: HashMap<id::CommandEncoderId, String>,
    pub(crate) compositor_api: CrossProcessCompositorApi,
    pub(crate) external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
    pub(crate) wgpu_image_map: WGPUImageMap,
    /// Provides access to poller thread
    pub(crate) poller: Poller,
    /// Store compute passes
    compute_passes: HashMap<ComputePassId, Pass<ComputePass>>,
    /// Store render passes
    render_passes: HashMap<RenderPassId, Pass<RenderPass>>,
}

impl WGPU {
    pub(crate) fn new(
        receiver: IpcReceiver<WebGPURequest>,
        sender: IpcSender<WebGPURequest>,
        script_sender: IpcSender<WebGPUMsg>,
        compositor_api: CrossProcessCompositorApi,
        external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
        wgpu_image_map: WGPUImageMap,
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
            &InstanceDescriptor {
                backends,
                ..Default::default()
            },
        ));
        WGPU {
            poller: Poller::new(Arc::clone(&global)),
            receiver,
            sender,
            script_sender,
            global,
            devices: Arc::new(Mutex::new(HashMap::new())),
            error_command_encoders: HashMap::new(),
            compositor_api,
            external_images,
            wgpu_image_map,
            compute_passes: HashMap::new(),
            render_passes: HashMap::new(),
        }
    }

    pub(crate) fn run(&mut self) {
        loop {
            if let Ok(msg) = self.receiver.recv() {
                log::trace!("recv: {msg:?}");
                match msg {
                    WebGPURequest::BufferMapAsync {
                        sender,
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
                                    data: IpcSharedMemory::from_bytes(data),
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
                    } => {
                        let global = &self.global;
                        let result = if let Some(err) =
                            self.error_command_encoders.get(&command_encoder_id)
                        {
                            Err(Error::Validation(err.clone()))
                        } else if let Some(error) =
                            global.command_encoder_finish(command_encoder_id, &desc).1
                        {
                            Err(Error::from_error(error))
                        } else {
                            Ok(())
                        };

                        // invalidate command buffer too
                        self.encoder_record_error(command_encoder_id, &result);
                        // dispatch validation error
                        self.maybe_dispatch_error(device_id, result.err());
                    },
                    WebGPURequest::CopyBufferToBuffer {
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
                            size,
                        );
                        self.encoder_record_error(command_encoder_id, &result);
                    },
                    WebGPURequest::CopyBufferToTexture {
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
                        self.encoder_record_error(command_encoder_id, &result);
                    },
                    WebGPURequest::CopyTextureToBuffer {
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
                        self.encoder_record_error(command_encoder_id, &result);
                    },
                    WebGPURequest::CopyTextureToTexture {
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
                        self.encoder_record_error(command_encoder_id, &result);
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
                        implicit_ids,
                        async_sender: sender,
                    } => {
                        let global = &self.global;
                        let bgls = implicit_ids
                            .as_ref()
                            .map_or(Vec::with_capacity(0), |(_, bgls)| {
                                bgls.iter().map(|x| x.to_owned()).collect()
                            });
                        let implicit =
                            implicit_ids
                                .as_ref()
                                .map(|(layout, _)| ImplicitPipelineIds {
                                    root_id: *layout,
                                    group_ids: bgls.as_slice(),
                                });
                        let (_, error) = global.device_create_compute_pipeline(
                            device_id,
                            &descriptor,
                            Some(compute_pipeline_id),
                            implicit,
                        );
                        if let Some(sender) = sender {
                            let res = match error {
                                // if device is lost we must return pipeline and not raise any error
                                Some(CreateComputePipelineError::Device(
                                    DeviceError::Lost | DeviceError::Invalid(_),
                                )) |
                                None => Ok(Pipeline {
                                    id: compute_pipeline_id,
                                    label: descriptor.label.unwrap_or_default().to_string(),
                                }),
                                Some(e) => Err(Error::from_error(e)),
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
                        implicit_ids,
                        async_sender: sender,
                    } => {
                        let global = &self.global;
                        let bgls = implicit_ids
                            .as_ref()
                            .map_or(Vec::with_capacity(0), |(_, bgls)| {
                                bgls.iter().map(|x| x.to_owned()).collect()
                            });
                        let implicit =
                            implicit_ids
                                .as_ref()
                                .map(|(layout, _)| ImplicitPipelineIds {
                                    root_id: *layout,
                                    group_ids: bgls.as_slice(),
                                });
                        let (_, error) = global.device_create_render_pipeline(
                            device_id,
                            &descriptor,
                            Some(render_pipeline_id),
                            implicit,
                        );

                        if let Some(sender) = sender {
                            let res = match error {
                                // if device is lost we must return pipeline and not raise any error
                                Some(CreateRenderPipelineError::Device(
                                    DeviceError::Lost | DeviceError::Invalid(_),
                                )) |
                                None => Ok(Pipeline {
                                    id: render_pipeline_id,
                                    label: descriptor.label.unwrap_or_default().to_string(),
                                }),
                                Some(e) => Err(Error::from_error(e)),
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
                        sender,
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
                            .external_images
                            .lock()
                            .expect("Lock poisoned?")
                            .next_id(WebrenderImageHandlerType::WebGPU);
                        let image_key = self.compositor_api.generate_image_key_blocking().unwrap();
                        let context_id = WebGPUContextId(id.0);
                        if let Err(e) = sender.send((context_id, image_key)) {
                            warn!("Failed to send ExternalImageId to new context ({})", e);
                        };
                        self.create_context(context_id, image_key, size, buffer_ids);
                    },
                    WebGPURequest::UpdateContext {
                        context_id,
                        size,
                        configuration,
                    } => {
                        self.update_context(context_id, size, configuration);
                    },
                    WebGPURequest::SwapChainPresent {
                        context_id,
                        texture_id,
                        encoder_id,
                    } => {
                        let result = self.swapchain_present(context_id, encoder_id, texture_id);
                        if let Err(e) = result {
                            log::error!("Error occured in SwapChainPresent: {e:?}");
                        }
                    },
                    WebGPURequest::GetImage { context_id, sender } => {
                        sender.send(self.get_image(context_id)).unwrap()
                    },
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
                        self.external_images
                            .lock()
                            .expect("Lock poisoned?")
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
                        let _result = global.buffer_destroy(buffer);
                    },
                    WebGPURequest::DestroyDevice(device) => {
                        let global = &self.global;
                        global.device_destroy(device);
                        // Wake poller thread to trigger DeviceLostClosure
                        self.poller.wake();
                    },
                    WebGPURequest::DestroyTexture(texture_id) => {
                        let global = &self.global;
                        let _ = global.texture_destroy(texture_id);
                    },
                    WebGPURequest::Exit(sender) => {
                        if let Err(e) = sender.send(()) {
                            warn!("Failed to send response to WebGPURequest::Exit ({})", e)
                        }
                        break;
                    },
                    WebGPURequest::DropCommandBuffer(id) => {
                        self.error_command_encoders
                            .remove(&id.into_command_encoder_id());
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
                    WebGPURequest::RenderBundleEncoderFinish {
                        render_bundle_encoder,
                        descriptor,
                        render_bundle_id,
                        device_id,
                    } => {
                        let global = &self.global;
                        let (_, error) = global.render_bundle_encoder_finish(
                            render_bundle_encoder,
                            &descriptor,
                            Some(render_bundle_id),
                        );

                        self.maybe_dispatch_wgpu_error(device_id, error);
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
                        let desc = DeviceDescriptor {
                            label: descriptor.label.as_ref().map(crate::Cow::from),
                            required_features: descriptor.required_features,
                            required_limits: descriptor.required_limits.clone(),
                            memory_hints: MemoryHints::MemoryUsage,
                            trace: wgpu_types::Trace::Off,
                        };
                        let global = &self.global;
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
                        device_id: _device_id,
                    } => {
                        let global = &self.global;
                        let (pass, error) = global.command_encoder_begin_compute_pass(
                            command_encoder_id,
                            &ComputePassDescriptor {
                                label,
                                timestamp_writes: None,
                            },
                        );
                        assert!(
                            self.compute_passes
                                .insert(compute_pass_id, Pass::new(pass, error.is_none()))
                                .is_none(),
                            "ComputePass should not exist yet."
                        );
                        // TODO: Command encoder state errors
                        // self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::ComputePassSetPipeline {
                        compute_pass_id,
                        pipeline_id,
                        device_id,
                    } => {
                        let pass = self
                            .compute_passes
                            .get_mut(&compute_pass_id)
                            .expect("ComputePass should exists");
                        if let Pass::Open { pass, valid } = pass {
                            *valid &= self
                                .global
                                .compute_pass_set_pipeline(pass, pipeline_id)
                                .is_ok();
                        } else {
                            self.maybe_dispatch_error(
                                device_id,
                                Some(Error::Validation("pass already ended".to_string())),
                            );
                        };
                    },
                    WebGPURequest::ComputePassSetBindGroup {
                        compute_pass_id,
                        index,
                        bind_group_id,
                        offsets,
                        device_id,
                    } => {
                        let pass = self
                            .compute_passes
                            .get_mut(&compute_pass_id)
                            .expect("ComputePass should exists");
                        if let Pass::Open { pass, valid } = pass {
                            *valid &= self
                                .global
                                .compute_pass_set_bind_group(
                                    pass,
                                    index,
                                    Some(bind_group_id),
                                    &offsets,
                                )
                                .is_ok();
                        } else {
                            self.maybe_dispatch_error(
                                device_id,
                                Some(Error::Validation("pass already ended".to_string())),
                            );
                        };
                    },
                    WebGPURequest::ComputePassDispatchWorkgroups {
                        compute_pass_id,
                        x,
                        y,
                        z,
                        device_id,
                    } => {
                        let pass = self
                            .compute_passes
                            .get_mut(&compute_pass_id)
                            .expect("ComputePass should exists");
                        if let Pass::Open { pass, valid } = pass {
                            *valid &= self
                                .global
                                .compute_pass_dispatch_workgroups(pass, x, y, z)
                                .is_ok();
                        } else {
                            self.maybe_dispatch_error(
                                device_id,
                                Some(Error::Validation("pass already ended".to_string())),
                            );
                        };
                    },
                    WebGPURequest::ComputePassDispatchWorkgroupsIndirect {
                        compute_pass_id,
                        buffer_id,
                        offset,
                        device_id,
                    } => {
                        let pass = self
                            .compute_passes
                            .get_mut(&compute_pass_id)
                            .expect("ComputePass should exists");
                        if let Pass::Open { pass, valid } = pass {
                            *valid &= self
                                .global
                                .compute_pass_dispatch_workgroups_indirect(pass, buffer_id, offset)
                                .is_ok();
                        } else {
                            self.maybe_dispatch_error(
                                device_id,
                                Some(Error::Validation("pass already ended".to_string())),
                            );
                        };
                    },
                    WebGPURequest::EndComputePass {
                        compute_pass_id,
                        device_id,
                        command_encoder_id,
                    } => {
                        // https://www.w3.org/TR/2024/WD-webgpu-20240703/#dom-gpucomputepassencoder-end
                        let pass = self
                            .compute_passes
                            .get_mut(&compute_pass_id)
                            .expect("ComputePass should exists");
                        // TODO: Command encoder state error
                        if let Pass::Open { mut pass, valid } = pass.take() {
                            // `pass.end` does step 1-4
                            // and if it returns ok we check the validity of the pass at step 5
                            if self.global.compute_pass_end(&mut pass).is_ok() && !valid {
                                self.encoder_record_error(
                                    command_encoder_id,
                                    &Err::<(), _>("Pass is invalid".to_string()),
                                );
                            }
                        } else {
                            self.dispatch_error(
                                device_id,
                                Error::Validation("pass already ended".to_string()),
                            );
                        };
                    },
                    WebGPURequest::BeginRenderPass {
                        command_encoder_id,
                        render_pass_id,
                        label,
                        color_attachments,
                        depth_stencil_attachment,
                        device_id: _device_id,
                    } => {
                        let global = &self.global;
                        let desc = &RenderPassDescriptor {
                            label,
                            color_attachments: color_attachments.into(),
                            depth_stencil_attachment: depth_stencil_attachment.as_ref(),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        };
                        let (pass, error) =
                            global.command_encoder_begin_render_pass(command_encoder_id, desc);
                        assert!(
                            self.render_passes
                                .insert(render_pass_id, Pass::new(pass, error.is_none()))
                                .is_none(),
                            "RenderPass should not exist yet."
                        );
                        // TODO: Command encoder state errors
                        // self.maybe_dispatch_wgpu_error(device_id, error);
                    },
                    WebGPURequest::RenderPassCommand {
                        render_pass_id,
                        render_command,
                        device_id,
                    } => {
                        let pass = self
                            .render_passes
                            .get_mut(&render_pass_id)
                            .expect("RenderPass should exists");
                        if let Pass::Open { pass, valid } = pass {
                            *valid &=
                                apply_render_command(&self.global, pass, render_command).is_ok();
                        } else {
                            self.maybe_dispatch_error(
                                device_id,
                                Some(Error::Validation("pass already ended".to_string())),
                            );
                        };
                    },
                    WebGPURequest::EndRenderPass {
                        render_pass_id,
                        device_id,
                        command_encoder_id,
                    } => {
                        // https://www.w3.org/TR/2024/WD-webgpu-20240703/#dom-gpurenderpassencoder-end
                        let pass = self
                            .render_passes
                            .get_mut(&render_pass_id)
                            .expect("RenderPass should exists");
                        // TODO: Command encoder state error
                        if let Pass::Open { mut pass, valid } = pass.take() {
                            // `pass.end` does step 1-4
                            // and if it returns ok we check the validity of the pass at step 5
                            if self.global.render_pass_end(&mut pass).is_ok() && !valid {
                                self.encoder_record_error(
                                    command_encoder_id,
                                    &Err::<(), _>("Pass is invalid".to_string()),
                                );
                            }
                        } else {
                            self.dispatch_error(
                                device_id,
                                Error::Validation("Pass already ended".to_string()),
                            );
                        };
                    },
                    WebGPURequest::Submit {
                        device_id,
                        queue_id,
                        command_buffers,
                    } => {
                        let global = &self.global;
                        let cmd_id = command_buffers.iter().find(|id| {
                            self.error_command_encoders
                                .contains_key(&id.into_command_encoder_id())
                        });
                        let result = if cmd_id.is_some() {
                            Err(Error::Validation(String::from(
                                "Invalid command buffer submitted",
                            )))
                        } else {
                            let _guard = self.poller.lock();
                            global
                                .queue_submit(queue_id, &command_buffers)
                                .map_err(|(_, error)| Error::from_error(error))
                        };
                        self.maybe_dispatch_error(device_id, result.err());
                    },
                    WebGPURequest::UnmapBuffer { buffer_id, mapping } => {
                        let global = &self.global;
                        if let Some(mapping) = mapping {
                            if let Ok((slice_pointer, range_size)) = global.buffer_get_mapped_range(
                                buffer_id,
                                mapping.range.start,
                                Some(mapping.range.end - mapping.range.start),
                            ) {
                                unsafe {
                                    slice::from_raw_parts_mut(
                                        slice_pointer.as_ptr(),
                                        range_size as usize,
                                    )
                                }
                                .copy_from_slice(&mapping.data);
                            }
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
                        //TODO: Report result to content process
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
                        // Pass might have already ended.
                        self.compute_passes.remove(&id);
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeComputePass(id)) {
                            warn!("Unable to send FreeComputePass({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropRenderPass(id) => {
                        self.render_passes
                            .remove(&id)
                            .expect("RenderPass should exists");
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
                        let _result = global.texture_view_drop(id);
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
                    WebGPURequest::PopErrorScope { device_id, sender } => {
                        // <https://www.w3.org/TR/webgpu/#dom-gpudevice-poperrorscope>
                        let mut devices = self.devices.lock().unwrap();
                        let device_scope = devices
                            .get_mut(&device_id)
                            .expect("Device should not be dropped by this point");
                        if let Some(error_scope_stack) = &mut device_scope.error_scope_stack {
                            if let Some(error_scope) = error_scope_stack.pop() {
                                if let Err(e) = sender.send(Ok(
                                    // TODO: Do actual selection instead of selecting first error
                                    error_scope.errors.first().cloned(),
                                )) {
                                    warn!(
                                        "Unable to send {:?} to poperrorscope: {e:?}",
                                        error_scope.errors
                                    );
                                }
                            } else if let Err(e) = sender.send(Err(PopError::Empty)) {
                                warn!("Unable to send PopError::Empty: {e:?}");
                            }
                        } else {
                            // device lost
                            if let Err(e) = sender.send(Err(PopError::Lost)) {
                                warn!("Unable to send PopError::Lost due {e:?}");
                            }
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
                }
            }
        }
        if let Err(e) = self.script_sender.send(WebGPUMsg::Exit) {
            warn!("Failed to send WebGPUMsg::Exit to script ({})", e);
        }
    }

    fn maybe_dispatch_wgpu_error<E: std::error::Error + 'static>(
        &mut self,
        device_id: id::DeviceId,
        error: Option<E>,
    ) {
        self.maybe_dispatch_error(device_id, error.map(|e| Error::from_error(e)))
    }

    /// Dispatches error (if there is any)
    fn maybe_dispatch_error(&mut self, device_id: id::DeviceId, error: Option<Error>) {
        if let Some(error) = error {
            self.dispatch_error(device_id, error);
        }
    }

    /// <https://www.w3.org/TR/webgpu/#abstract-opdef-dispatch-error>
    fn dispatch_error(&mut self, device_id: id::DeviceId, error: Error) {
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

    fn encoder_record_error<U, T: std::fmt::Debug>(
        &mut self,
        encoder_id: id::CommandEncoderId,
        result: &Result<U, T>,
    ) {
        if let Err(e) = result {
            self.error_command_encoders
                .entry(encoder_id)
                .or_insert_with(|| format!("{:?}", e));
        }
    }
}
