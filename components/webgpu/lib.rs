/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate log;
#[macro_use]
pub extern crate wgpu_core as wgpu;
pub extern crate wgpu_types as wgt;

pub mod identity;

use arrayvec::ArrayVec;
use euclid::default::Size2D;
use identity::{IdentityRecyclerFactory, WebGPUMsg, WebGPUOpResult};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender, IpcSharedMemory};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use msg::constellation_msg::PipelineId;
use serde::{Deserialize, Serialize};
use servo_config::pref;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::num::NonZeroU64;
use std::rc::Rc;
use std::slice;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use webrender_traits::{
    WebrenderExternalImageApi, WebrenderExternalImageRegistry, WebrenderImageHandlerType,
    WebrenderImageSource,
};
use wgpu::{
    binding_model::{BindGroupDescriptor, BindGroupLayoutDescriptor, PipelineLayoutDescriptor},
    command::{
        BufferCopyView, ComputePass, RenderBundleDescriptor, RenderBundleEncoder, RenderPass,
        TextureCopyView,
    },
    device::{HostMap, ImplicitPipelineIds},
    id,
    instance::RequestAdapterOptions,
    pipeline::{ComputePipelineDescriptor, RenderPipelineDescriptor},
    resource::{
        BufferDescriptor, BufferMapAsyncStatus, BufferMapOperation, SamplerDescriptor,
        TextureDescriptor, TextureViewDescriptor,
    },
};

pub type ErrorScopeId = NonZeroU64;
const DEVICE_POLL_INTERVAL: u64 = 100;
pub const PRESENTATION_BUFFER_COUNT: usize = 10;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebGPUResponse {
    RequestAdapter {
        adapter_name: String,
        adapter_id: WebGPUAdapter,
        channel: WebGPU,
    },
    RequestDevice {
        device_id: WebGPUDevice,
        queue_id: WebGPUQueue,
        _descriptor: wgt::DeviceDescriptor,
        label: Option<String>,
    },
    BufferMapAsync(IpcSharedMemory),
}

pub type WebGPUResponseResult = Result<WebGPUResponse, String>;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebGPURequest {
    BufferMapAsync {
        sender: IpcSender<WebGPUResponseResult>,
        buffer_id: id::BufferId,
        device_id: id::DeviceId,
        host_map: HostMap,
        map_range: std::ops::Range<u64>,
    },
    BufferMapComplete(id::BufferId),
    CommandEncoderFinish {
        command_encoder_id: id::CommandEncoderId,
        device_id: id::DeviceId,
        is_error: bool,
        // TODO(zakorgy): Serialize CommandBufferDescriptor in wgpu-core
        // wgpu::command::CommandBufferDescriptor,
    },
    CopyBufferToBuffer {
        command_encoder_id: id::CommandEncoderId,
        source_id: id::BufferId,
        source_offset: wgt::BufferAddress,
        destination_id: id::BufferId,
        destination_offset: wgt::BufferAddress,
        size: wgt::BufferAddress,
    },
    CopyBufferToTexture {
        command_encoder_id: id::CommandEncoderId,
        source: BufferCopyView,
        destination: TextureCopyView,
        copy_size: wgt::Extent3d,
    },
    CopyTextureToBuffer {
        command_encoder_id: id::CommandEncoderId,
        source: TextureCopyView,
        destination: BufferCopyView,
        copy_size: wgt::Extent3d,
    },
    CopyTextureToTexture {
        command_encoder_id: id::CommandEncoderId,
        source: TextureCopyView,
        destination: TextureCopyView,
        copy_size: wgt::Extent3d,
    },
    CreateBindGroup {
        device_id: id::DeviceId,
        bind_group_id: id::BindGroupId,
        descriptor: BindGroupDescriptor<'static>,
    },
    CreateBindGroupLayout {
        device_id: id::DeviceId,
        bind_group_layout_id: id::BindGroupLayoutId,
        descriptor: Option<BindGroupLayoutDescriptor<'static>>,
    },
    CreateBuffer {
        device_id: id::DeviceId,
        buffer_id: id::BufferId,
        descriptor: Option<BufferDescriptor<'static>>,
    },
    CreateCommandEncoder {
        device_id: id::DeviceId,
        // TODO(zakorgy): Serialize CommandEncoderDescriptor in wgpu-core
        // wgpu::command::CommandEncoderDescriptor,
        command_encoder_id: id::CommandEncoderId,
        label: Option<Cow<'static, str>>,
    },
    CreateComputePipeline {
        device_id: id::DeviceId,
        compute_pipeline_id: id::ComputePipelineId,
        descriptor: ComputePipelineDescriptor<'static>,
        implicit_ids: Option<(id::PipelineLayoutId, Vec<id::BindGroupLayoutId>)>,
    },
    CreateContext(IpcSender<webrender_api::ExternalImageId>),
    CreatePipelineLayout {
        device_id: id::DeviceId,
        pipeline_layout_id: id::PipelineLayoutId,
        descriptor: PipelineLayoutDescriptor<'static>,
    },
    CreateRenderPipeline {
        device_id: id::DeviceId,
        render_pipeline_id: id::RenderPipelineId,
        descriptor: Option<RenderPipelineDescriptor<'static>>,
        implicit_ids: Option<(id::PipelineLayoutId, Vec<id::BindGroupLayoutId>)>,
    },
    CreateSampler {
        device_id: id::DeviceId,
        sampler_id: id::SamplerId,
        descriptor: SamplerDescriptor<'static>,
    },
    CreateShaderModule {
        device_id: id::DeviceId,
        program_id: id::ShaderModuleId,
        program: Vec<u32>,
    },
    CreateSwapChain {
        device_id: id::DeviceId,
        buffer_ids: ArrayVec<[id::BufferId; PRESENTATION_BUFFER_COUNT]>,
        external_id: u64,
        sender: IpcSender<webrender_api::ImageKey>,
        image_desc: webrender_api::ImageDescriptor,
        image_data: webrender_api::ImageData,
    },
    CreateTexture {
        device_id: id::DeviceId,
        texture_id: id::TextureId,
        descriptor: Option<TextureDescriptor<'static>>,
    },
    CreateTextureView {
        texture_id: id::TextureId,
        texture_view_id: id::TextureViewId,
        device_id: id::DeviceId,
        descriptor: Option<TextureViewDescriptor<'static>>,
    },
    DestroyBuffer(id::BufferId),
    DestroySwapChain {
        external_id: u64,
        image_key: webrender_api::ImageKey,
    },
    DestroyTexture(id::TextureId),
    Exit(IpcSender<()>),
    FreeCommandBuffer(id::CommandBufferId),
    FreeDevice(id::DeviceId),
    RenderBundleEncoderFinish {
        render_bundle_encoder: RenderBundleEncoder,
        descriptor: RenderBundleDescriptor<'static>,
        render_bundle_id: id::RenderBundleId,
        device_id: id::DeviceId,
    },
    RequestAdapter {
        sender: IpcSender<WebGPUResponseResult>,
        options: RequestAdapterOptions,
        ids: SmallVec<[id::AdapterId; 4]>,
    },
    RequestDevice {
        sender: IpcSender<WebGPUResponseResult>,
        adapter_id: WebGPUAdapter,
        descriptor: wgt::DeviceDescriptor,
        device_id: id::DeviceId,
        pipeline_id: PipelineId,
        label: Option<String>,
    },
    RunComputePass {
        command_encoder_id: id::CommandEncoderId,
        compute_pass: Option<ComputePass>,
    },
    RunRenderPass {
        command_encoder_id: id::CommandEncoderId,
        render_pass: Option<RenderPass>,
    },
    Submit {
        queue_id: id::QueueId,
        command_buffers: Vec<id::CommandBufferId>,
    },
    SwapChainPresent {
        external_id: u64,
        texture_id: id::TextureId,
        encoder_id: id::CommandEncoderId,
    },
    UnmapBuffer {
        buffer_id: id::BufferId,
        device_id: id::DeviceId,
        array_buffer: IpcSharedMemory,
        is_map_read: bool,
        offset: u64,
        size: u64,
    },
    UpdateWebRenderData {
        buffer_id: id::BufferId,
        external_id: u64,
        buffer_size: usize,
    },
    WriteBuffer {
        queue_id: id::QueueId,
        buffer_id: id::BufferId,
        buffer_offset: u64,
        data: IpcSharedMemory,
    },
    WriteTexture {
        queue_id: id::QueueId,
        texture_cv: TextureCopyView,
        data_layout: wgt::TextureDataLayout,
        size: wgt::Extent3d,
        data: IpcSharedMemory,
    },
}

struct BufferMapInfo<'a, T> {
    buffer_id: id::BufferId,
    sender: IpcSender<T>,
    global: &'a wgpu::hub::Global<IdentityRecyclerFactory>,
    size: usize,
    external_id: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebGPU(pub IpcSender<(Option<ErrorScopeId>, WebGPURequest)>);

impl WebGPU {
    pub fn new(
        webrender_api_sender: webrender_api::RenderApiSender,
        webrender_document: webrender_api::DocumentId,
        external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
        wgpu_image_map: Arc<Mutex<HashMap<u64, PresentationData>>>,
    ) -> Option<(Self, IpcReceiver<WebGPUMsg>)> {
        if !pref!(dom.webgpu.enabled) {
            return None;
        }
        let (sender, receiver) = match ipc::channel() {
            Ok(sender_and_receiver) => sender_and_receiver,
            Err(e) => {
                warn!(
                    "Failed to create sender and receiver for WGPU thread ({})",
                    e
                );
                return None;
            },
        };
        let sender_clone = sender.clone();

        let (script_sender, script_recv) = match ipc::channel() {
            Ok(sender_and_receiver) => sender_and_receiver,
            Err(e) => {
                warn!(
                    "Failed to create receiver and sender for WGPU thread ({})",
                    e
                );
                return None;
            },
        };

        if let Err(e) = std::thread::Builder::new()
            .name("WGPU".to_owned())
            .spawn(move || {
                WGPU::new(
                    receiver,
                    sender_clone,
                    script_sender,
                    webrender_api_sender,
                    webrender_document,
                    external_images,
                    wgpu_image_map,
                )
                .run();
            })
        {
            warn!("Failed to spwan WGPU thread ({})", e);
            return None;
        }
        Some((WebGPU(sender), script_recv))
    }

    pub fn exit(&self, sender: IpcSender<()>) -> Result<(), &'static str> {
        self.0
            .send((None, WebGPURequest::Exit(sender)))
            .map_err(|_| "Failed to send Exit message")
    }
}

struct WGPU<'a> {
    receiver: IpcReceiver<(Option<ErrorScopeId>, WebGPURequest)>,
    sender: IpcSender<(Option<ErrorScopeId>, WebGPURequest)>,
    script_sender: IpcSender<WebGPUMsg>,
    global: wgpu::hub::Global<IdentityRecyclerFactory>,
    adapters: Vec<WebGPUAdapter>,
    devices: HashMap<WebGPUDevice, PipelineId>,
    // Track invalid adapters https://gpuweb.github.io/gpuweb/#invalid
    _invalid_adapters: Vec<WebGPUAdapter>,
    // Buffers with pending mapping
    buffer_maps: HashMap<id::BufferId, Rc<BufferMapInfo<'a, WebGPUResponseResult>>>,
    // Presentation Buffers with pending mapping
    present_buffer_maps:
        HashMap<id::BufferId, Rc<BufferMapInfo<'a, (Option<ErrorScopeId>, WebGPURequest)>>>,
    //TODO: Remove this (https://github.com/gfx-rs/wgpu/issues/867)
    error_command_encoders: RefCell<HashMap<id::CommandEncoderId, String>>,
    webrender_api: webrender_api::RenderApi,
    webrender_document: webrender_api::DocumentId,
    external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
    wgpu_image_map: Arc<Mutex<HashMap<u64, PresentationData>>>,
    last_poll: Instant,
}

impl<'a> WGPU<'a> {
    fn new(
        receiver: IpcReceiver<(Option<ErrorScopeId>, WebGPURequest)>,
        sender: IpcSender<(Option<ErrorScopeId>, WebGPURequest)>,
        script_sender: IpcSender<WebGPUMsg>,
        webrender_api_sender: webrender_api::RenderApiSender,
        webrender_document: webrender_api::DocumentId,
        external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
        wgpu_image_map: Arc<Mutex<HashMap<u64, PresentationData>>>,
    ) -> Self {
        let factory = IdentityRecyclerFactory {
            sender: script_sender.clone(),
            self_sender: sender.clone(),
        };
        WGPU {
            receiver,
            sender,
            script_sender,
            global: wgpu::hub::Global::new("wgpu-core", factory, wgt::BackendBit::PRIMARY),
            adapters: Vec::new(),
            devices: HashMap::new(),
            _invalid_adapters: Vec::new(),
            buffer_maps: HashMap::new(),
            present_buffer_maps: HashMap::new(),
            error_command_encoders: RefCell::new(HashMap::new()),
            webrender_api: webrender_api_sender.create_api(),
            webrender_document,
            external_images,
            wgpu_image_map,
            last_poll: Instant::now(),
        }
    }

    fn run(&'a mut self) {
        loop {
            if self.last_poll.elapsed() >= Duration::from_millis(DEVICE_POLL_INTERVAL) {
                let _ = self.global.poll_all_devices(false);
                self.last_poll = Instant::now();
            }
            if let Ok((scope_id, msg)) = self.receiver.try_recv() {
                match msg {
                    WebGPURequest::BufferMapAsync {
                        sender,
                        buffer_id,
                        device_id,
                        host_map,
                        map_range,
                    } => {
                        let map_info = BufferMapInfo {
                            buffer_id,
                            sender: sender.clone(),
                            global: &self.global,
                            size: (map_range.end - map_range.start) as usize,
                            external_id: None,
                        };
                        self.buffer_maps.insert(buffer_id, Rc::new(map_info));

                        unsafe extern "C" fn callback(
                            status: BufferMapAsyncStatus,
                            userdata: *mut u8,
                        ) {
                            let info = Rc::from_raw(
                                userdata as *const BufferMapInfo<WebGPUResponseResult>,
                            );
                            let msg = match status {
                                BufferMapAsyncStatus::Success => {
                                    let global = &info.global;
                                    let data_pt = gfx_select!(info.buffer_id =>
                                        global.buffer_get_mapped_range(info.buffer_id, 0, None))
                                    .unwrap();
                                    let data = slice::from_raw_parts(data_pt, info.size);
                                    Ok(WebGPUResponse::BufferMapAsync(IpcSharedMemory::from_bytes(
                                        data,
                                    )))
                                },
                                _ => {
                                    warn!("Could not map buffer({:?})", info.buffer_id);
                                    Err(String::from("Failed to map Buffer"))
                                },
                            };
                            if let Err(e) = info.sender.send(msg) {
                                warn!("Could not send BufferMapAsync Response ({})", e);
                            }
                        }

                        let operation = BufferMapOperation {
                            host: host_map,
                            callback,
                            user_data: convert_to_pointer(
                                self.buffer_maps.get(&buffer_id).unwrap().clone(),
                            ),
                        };
                        let global = &self.global;
                        let result = gfx_select!(buffer_id => global.buffer_map_async(buffer_id, map_range, operation));
                        if let Err(ref e) = result {
                            if let Err(w) = sender.send(Err(format!("{:?}", e))) {
                                warn!("Failed to send BufferMapAsync Response ({:?})", w);
                            }
                        }
                        self.send_result(device_id, scope_id, result);
                    },
                    WebGPURequest::BufferMapComplete(buffer_id) => {
                        self.buffer_maps.remove(&buffer_id);
                    },
                    WebGPURequest::CommandEncoderFinish {
                        command_encoder_id,
                        device_id,
                        is_error,
                    } => {
                        let global = &self.global;
                        let result = if is_error {
                            Err(String::from("Invalid GPUCommandEncoder"))
                        } else if let Some(err) = self
                            .error_command_encoders
                            .borrow()
                            .get(&command_encoder_id)
                        {
                            Err(err.clone())
                        } else {
                            gfx_select!(command_encoder_id => global.command_encoder_finish(
                                command_encoder_id,
                                &wgt::CommandBufferDescriptor::default()
                            ))
                            .map_err(|e| format!("{:?}", e))
                        };
                        self.encoder_record_error(command_encoder_id, result.clone());
                        self.send_result(device_id, scope_id, result);
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
                        let result = gfx_select!(command_encoder_id => global.command_encoder_copy_buffer_to_buffer(
                            command_encoder_id,
                            source_id,
                            source_offset,
                            destination_id,
                            destination_offset,
                            size
                        ));
                        self.encoder_record_error(command_encoder_id, result);
                    },
                    WebGPURequest::CopyBufferToTexture {
                        command_encoder_id,
                        source,
                        destination,
                        copy_size,
                    } => {
                        let global = &self.global;
                        let result = gfx_select!(command_encoder_id => global.command_encoder_copy_buffer_to_texture(
                            command_encoder_id,
                            &source,
                            &destination,
                            &copy_size
                        ));
                        self.encoder_record_error(command_encoder_id, result);
                    },
                    WebGPURequest::CopyTextureToBuffer {
                        command_encoder_id,
                        source,
                        destination,
                        copy_size,
                    } => {
                        let global = &self.global;
                        let result = gfx_select!(command_encoder_id => global.command_encoder_copy_texture_to_buffer(
                            command_encoder_id,
                            &source,
                            &destination,
                            &copy_size
                        ));
                        self.encoder_record_error(command_encoder_id, result);
                    },
                    WebGPURequest::CopyTextureToTexture {
                        command_encoder_id,
                        source,
                        destination,
                        copy_size,
                    } => {
                        let global = &self.global;
                        let result = gfx_select!(command_encoder_id => global.command_encoder_copy_texture_to_texture(
                            command_encoder_id,
                            &source,
                            &destination,
                            &copy_size
                        ));
                        self.encoder_record_error(command_encoder_id, result);
                    },
                    WebGPURequest::CreateBindGroup {
                        device_id,
                        bind_group_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let result = gfx_select!(bind_group_id =>
                            global.device_create_bind_group(device_id, &descriptor, bind_group_id));
                        if result.is_err() {
                            let _ = gfx_select!(bind_group_id => global.bind_group_error(bind_group_id));
                        }
                        self.send_result(device_id, scope_id, result);
                    },
                    WebGPURequest::CreateBindGroupLayout {
                        device_id,
                        bind_group_layout_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        if let Some(desc) = descriptor {
                            let result = gfx_select!(bind_group_layout_id =>
                                global.device_create_bind_group_layout(device_id, &desc, bind_group_layout_id));
                            if result.is_err() {
                                let _ = gfx_select!(bind_group_layout_id =>
                                    global.bind_group_layout_error(bind_group_layout_id));
                            }
                            self.send_result(device_id, scope_id, result);
                        } else {
                            let _ = gfx_select!(bind_group_layout_id =>
                                global.bind_group_layout_error(bind_group_layout_id));
                        }
                    },
                    WebGPURequest::CreateBuffer {
                        device_id,
                        buffer_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        if let Some(desc) = descriptor {
                            let result = gfx_select!(buffer_id =>
                                global.device_create_buffer(device_id, &desc, buffer_id));
                            if result.is_err() {
                                let _ = gfx_select!(buffer_id => global.buffer_error(buffer_id));
                            }
                            self.send_result(device_id, scope_id, result);
                        } else {
                            let _ = gfx_select!(buffer_id => global.buffer_error(buffer_id));
                        }
                    },
                    WebGPURequest::CreateCommandEncoder {
                        device_id,
                        command_encoder_id,
                        label,
                    } => {
                        let global = &self.global;
                        let desc = wgt::CommandEncoderDescriptor { label };
                        let result = gfx_select!(command_encoder_id =>
                            global.device_create_command_encoder(device_id, &desc, command_encoder_id));
                        if result.is_err() {
                            let _ = gfx_select!(command_encoder_id => global.command_encoder_error(command_encoder_id));
                        }
                        self.send_result(device_id, scope_id, result);
                    },
                    WebGPURequest::CreateComputePipeline {
                        device_id,
                        compute_pipeline_id,
                        descriptor,
                        implicit_ids,
                    } => {
                        let global = &self.global;
                        let bgls = implicit_ids
                            .as_ref()
                            .map_or(Vec::with_capacity(0), |(_, bgls)| bgls.clone());
                        let implicit =
                            implicit_ids
                                .as_ref()
                                .map(|(layout, _)| ImplicitPipelineIds {
                                    root_id: *layout,
                                    group_ids: bgls.as_slice(),
                                });
                        let result = gfx_select!(compute_pipeline_id => global.device_create_compute_pipeline(
                            device_id,
                            &descriptor,
                            compute_pipeline_id,
                            implicit
                        ));
                        if result.is_err() {
                            let _ = gfx_select!(compute_pipeline_id =>
                                global.compute_pipeline_error(compute_pipeline_id));
                            if let Some((layout, bgls)) = implicit_ids {
                                let _ = gfx_select!(layout => global.pipeline_layout_error(layout));
                                bgls.iter().for_each(|&bgl| {
                                    let _ = gfx_select!(bgl => global.bind_group_layout_error(bgl));
                                });
                            }
                        }
                        self.send_result(device_id, scope_id, result);
                    },
                    WebGPURequest::CreateContext(sender) => {
                        let id = self
                            .external_images
                            .lock()
                            .expect("Lock poisoned?")
                            .next_id(WebrenderImageHandlerType::WebGPU);
                        if let Err(e) = sender.send(id) {
                            warn!("Failed to send ExternalImageId to new context ({})", e);
                        };
                    },
                    WebGPURequest::CreatePipelineLayout {
                        device_id,
                        pipeline_layout_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let result = gfx_select!(pipeline_layout_id =>
                            global.device_create_pipeline_layout(device_id, &descriptor, pipeline_layout_id));
                        if result.is_err() {
                            let _ = gfx_select!(pipeline_layout_id => global.pipeline_layout_error(pipeline_layout_id));
                        }
                        self.send_result(device_id, scope_id, result);
                    },
                    WebGPURequest::CreateRenderPipeline {
                        device_id,
                        render_pipeline_id,
                        descriptor,
                        implicit_ids,
                    } => {
                        let global = &self.global;
                        let bgls = implicit_ids
                            .as_ref()
                            .map_or(Vec::with_capacity(0), |(_, bgls)| bgls.clone());
                        let implicit =
                            implicit_ids
                                .as_ref()
                                .map(|(layout, _)| ImplicitPipelineIds {
                                    root_id: *layout,
                                    group_ids: bgls.as_slice(),
                                });
                        if let Some(desc) = descriptor {
                            let result = gfx_select!(render_pipeline_id =>
                            global.device_create_render_pipeline(device_id, &desc, render_pipeline_id, implicit));
                            if result.is_err() {
                                let _ = gfx_select!(render_pipeline_id =>
                                    global.render_pipeline_error(render_pipeline_id));
                                if let Some((layout, bgls)) = implicit_ids {
                                    let _ =
                                        gfx_select!(layout => global.pipeline_layout_error(layout));
                                    bgls.iter().for_each(|&bgl| {
                                        let _ = gfx_select!(bgl => global.bind_group_layout_error(bgl));
                                    });
                                }
                            }
                            self.send_result(device_id, scope_id, result);
                        } else {
                            let _ = gfx_select!(render_pipeline_id => global.render_pipeline_error(render_pipeline_id));
                        }
                    },
                    WebGPURequest::CreateSampler {
                        device_id,
                        sampler_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let result = gfx_select!(sampler_id => global.device_create_sampler(
                            device_id,
                            &descriptor,
                            sampler_id
                        ));
                        if result.is_err() {
                            let _ = gfx_select!(sampler_id => global.sampler_error(sampler_id));
                        }
                        self.send_result(device_id, scope_id, result);
                    },
                    WebGPURequest::CreateShaderModule {
                        device_id,
                        program_id,
                        program,
                    } => {
                        let global = &self.global;
                        let source =
                            wgpu_core::pipeline::ShaderModuleSource::SpirV(Cow::Owned(program));
                        let result = gfx_select!(program_id =>
                            global.device_create_shader_module(device_id, source, program_id));
                        if result.is_err() {
                            let _ =
                                gfx_select!(program_id => global.shader_module_error(program_id));
                        }
                        self.send_result(device_id, scope_id, result);
                    },
                    WebGPURequest::CreateSwapChain {
                        device_id,
                        buffer_ids,
                        external_id,
                        sender,
                        image_desc,
                        image_data,
                    } => {
                        let height = image_desc.size.height;
                        let width = image_desc.size.width;
                        let buffer_stride =
                            ((width * 4) as u32 | (wgt::COPY_BYTES_PER_ROW_ALIGNMENT - 1)) + 1;
                        let image_key = self.webrender_api.generate_image_key();
                        if let Err(e) = sender.send(image_key) {
                            warn!("Failed to send ImageKey ({})", e);
                        }
                        let _ = self.wgpu_image_map.lock().unwrap().insert(
                            external_id,
                            PresentationData {
                                device_id,
                                queue_id: device_id,
                                data: vec![255; (buffer_stride * height as u32) as usize],
                                size: Size2D::new(width, height),
                                unassigned_buffer_ids: buffer_ids,
                                available_buffer_ids: ArrayVec::<
                                    [id::BufferId; PRESENTATION_BUFFER_COUNT],
                                >::new(),
                                queued_buffer_ids: ArrayVec::<
                                    [id::BufferId; PRESENTATION_BUFFER_COUNT],
                                >::new(),
                                buffer_stride,
                                image_key,
                                image_desc,
                                image_data: image_data.clone(),
                            },
                        );

                        let mut txn = webrender_api::Transaction::new();
                        txn.add_image(image_key, image_desc, image_data, None);
                        self.webrender_api
                            .send_transaction(self.webrender_document, txn);
                    },
                    WebGPURequest::CreateTexture {
                        device_id,
                        texture_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        if let Some(desc) = descriptor {
                            let result = gfx_select!(texture_id => global.device_create_texture(
                                device_id,
                                &desc,
                                texture_id
                            ));
                            if result.is_err() {
                                let _ = gfx_select!(texture_id => global.texture_error(texture_id));
                            }
                            self.send_result(device_id, scope_id, result);
                        } else {
                            let _ = gfx_select!(texture_id => global.texture_error(texture_id));
                        }
                    },
                    WebGPURequest::CreateTextureView {
                        texture_id,
                        texture_view_id,
                        device_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        if let Some(desc) = descriptor {
                            let result = gfx_select!(texture_view_id => global.texture_create_view(
                                texture_id,
                                &desc,
                                texture_view_id
                            ));
                            if result.is_err() {
                                let _ = gfx_select!(texture_view_id => global.texture_view_error(texture_view_id));
                            }
                            self.send_result(device_id, scope_id, result);
                        } else {
                            let _ = gfx_select!(texture_view_id => global.texture_view_error(texture_view_id));
                        }
                    },
                    WebGPURequest::DestroyBuffer(buffer) => {
                        let global = &self.global;
                        gfx_select!(buffer => global.buffer_drop(buffer, false));
                    },
                    WebGPURequest::DestroySwapChain {
                        external_id,
                        image_key,
                    } => {
                        let data = self
                            .wgpu_image_map
                            .lock()
                            .unwrap()
                            .remove(&external_id)
                            .unwrap();
                        let global = &self.global;
                        for b_id in data.available_buffer_ids.iter() {
                            gfx_select!(b_id => global.buffer_drop(*b_id, false));
                        }
                        for b_id in data.queued_buffer_ids.iter() {
                            gfx_select!(b_id => global.buffer_drop(*b_id, false));
                        }
                        for b_id in data.unassigned_buffer_ids.iter() {
                            if let Err(e) = self.script_sender.send(WebGPUMsg::FreeBuffer(*b_id)) {
                                warn!("Unable to send FreeBuffer({:?}) ({:?})", *b_id, e);
                            };
                        }
                        let mut txn = webrender_api::Transaction::new();
                        txn.delete_image(image_key);
                        self.webrender_api
                            .send_transaction(self.webrender_document, txn);
                    },
                    WebGPURequest::DestroyTexture(texture) => {
                        let global = &self.global;
                        gfx_select!(texture => global.texture_drop(texture));
                    },
                    WebGPURequest::Exit(sender) => {
                        if let Err(e) = self.script_sender.send(WebGPUMsg::Exit) {
                            warn!("Failed to send WebGPUMsg::Exit to script ({})", e);
                        }
                        if let Err(e) = sender.send(()) {
                            warn!("Failed to send response to WebGPURequest::Exit ({})", e)
                        }
                        return;
                    },
                    WebGPURequest::FreeCommandBuffer(command_buffer_id) => {
                        self.error_command_encoders
                            .borrow_mut()
                            .remove(&command_buffer_id);
                    },
                    WebGPURequest::FreeDevice(device_id) => {
                        let device = WebGPUDevice(device_id);
                        let pipeline_id = self.devices.remove(&device).unwrap();
                        if let Err(e) = self.script_sender.send(WebGPUMsg::CleanDevice {
                            device,
                            pipeline_id,
                        }) {
                            warn!("Unable to send CleanDevice({:?}) ({:?})", device_id, e);
                        }
                    },
                    WebGPURequest::RenderBundleEncoderFinish {
                        render_bundle_encoder,
                        descriptor,
                        render_bundle_id,
                        device_id,
                    } => {
                        let global = &self.global;
                        let result = gfx_select!(render_bundle_id => global.render_bundle_encoder_finish(
                            render_bundle_encoder,
                            &descriptor,
                            render_bundle_id
                        ));
                        if result.is_err() {
                            let _ = gfx_select!(render_bundle_id => global.render_bundle_error(render_bundle_id));
                        }
                        self.send_result(device_id, scope_id, result);
                    },
                    WebGPURequest::RequestAdapter {
                        sender,
                        options,
                        ids,
                    } => {
                        let adapter_id = match self.global.request_adapter(
                            &options,
                            wgpu::instance::AdapterInputs::IdSet(&ids, |id| id.backend()),
                        ) {
                            Ok(id) => id,
                            Err(w) => {
                                if let Err(e) = sender.send(Err(format!("{:?}", w))) {
                                    warn!(
                                    "Failed to send response to WebGPURequest::RequestAdapter ({})",
                                    e
                                )
                                }
                                return;
                            },
                        };
                        let adapter = WebGPUAdapter(adapter_id);
                        self.adapters.push(adapter);
                        let global = &self.global;
                        let info =
                            gfx_select!(adapter_id => global.adapter_get_info(adapter_id)).unwrap();
                        if let Err(e) = sender.send(Ok(WebGPUResponse::RequestAdapter {
                            adapter_name: info.name,
                            adapter_id: adapter,
                            channel: WebGPU(self.sender.clone()),
                        })) {
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
                        pipeline_id,
                        label,
                    } => {
                        let global = &self.global;
                        let id = match gfx_select!(device_id => global.adapter_request_device(
                            adapter_id.0,
                            &descriptor,
                            None,
                            device_id
                        )) {
                            Ok(id) => id,
                            Err(e) => {
                                let _ = gfx_select!(device_id => global.device_error(device_id));
                                if let Err(w) = sender.send(Err(format!("{:?}", e))) {
                                    warn!(
                                    "Failed to send response to WebGPURequest::RequestDevice ({})",
                                    w
                                )
                                }
                                return;
                            },
                        };
                        let device = WebGPUDevice(id);
                        // Note: (zakorgy) Note sure if sending the queue is needed at all,
                        // since wgpu-core uses the same id for the device and the queue
                        let queue = WebGPUQueue(id);
                        self.devices.insert(device, pipeline_id);
                        if let Err(e) = sender.send(Ok(WebGPUResponse::RequestDevice {
                            device_id: device,
                            queue_id: queue,
                            _descriptor: descriptor,
                            label,
                        })) {
                            warn!(
                                "Failed to send response to WebGPURequest::RequestDevice ({})",
                                e
                            )
                        }
                    },
                    WebGPURequest::RunComputePass {
                        command_encoder_id,
                        compute_pass,
                    } => {
                        let global = &self.global;
                        let result = if let Some(pass) = compute_pass {
                            gfx_select!(command_encoder_id => global.command_encoder_run_compute_pass(
                                command_encoder_id,
                                &pass
                            )).map_err(|e| format!("{:?}", e))
                        } else {
                            Err(String::from("Invalid ComputePass"))
                        };
                        self.encoder_record_error(command_encoder_id, result);
                    },
                    WebGPURequest::RunRenderPass {
                        command_encoder_id,
                        render_pass,
                    } => {
                        let global = &self.global;
                        let result = if let Some(pass) = render_pass {
                            gfx_select!(command_encoder_id => global.command_encoder_run_render_pass(
                                command_encoder_id,
                                &pass
                            )).map_err(|e| format!("{:?}", e))
                        } else {
                            Err(String::from("Invalid RenderPass"))
                        };
                        self.encoder_record_error(command_encoder_id, result);
                    },
                    WebGPURequest::Submit {
                        queue_id,
                        command_buffers,
                    } => {
                        let global = &self.global;
                        let cmd_id = command_buffers
                            .iter()
                            .find(|id| self.error_command_encoders.borrow().contains_key(id));
                        let result = if cmd_id.is_some() {
                            Err(String::from("Invalid command buffer submitted"))
                        } else {
                            gfx_select!(queue_id => global.queue_submit(queue_id, &command_buffers))
                                .map_err(|e| format!("{:?}", e))
                        };
                        self.send_result(queue_id, scope_id, result);
                    },
                    WebGPURequest::SwapChainPresent {
                        external_id,
                        texture_id,
                        encoder_id,
                    } => {
                        let global = &self.global;
                        let device_id;
                        let queue_id;
                        let size;
                        let buffer_id;
                        let buffer_stride;
                        {
                            if let Some(present_data) =
                                self.wgpu_image_map.lock().unwrap().get_mut(&external_id)
                            {
                                size = present_data.size;
                                device_id = present_data.device_id;
                                queue_id = present_data.queue_id;
                                buffer_stride = present_data.buffer_stride;
                                buffer_id = if let Some(b_id) =
                                    present_data.available_buffer_ids.pop()
                                {
                                    b_id
                                } else if let Some(b_id) = present_data.unassigned_buffer_ids.pop()
                                {
                                    let buffer_size =
                                        (buffer_stride * size.height as u32) as wgt::BufferAddress;
                                    let buffer_desc = wgt::BufferDescriptor {
                                        label: None,
                                        size: buffer_size,
                                        usage: wgt::BufferUsage::MAP_READ |
                                            wgt::BufferUsage::COPY_DST,
                                        mapped_at_creation: false,
                                    };
                                    let _ = gfx_select!(b_id => global.device_create_buffer(
                                        device_id,
                                        &buffer_desc,
                                        b_id
                                    ));
                                    b_id
                                } else {
                                    warn!(
                                        "No staging buffer available for ExternalImageId({:?})",
                                        external_id
                                    );
                                    continue;
                                };
                                present_data.queued_buffer_ids.push(buffer_id);
                            } else {
                                warn!("Data not found for ExternalImageId({:?})", external_id);
                                continue;
                            }
                        }

                        let buffer_size =
                            (size.height as u32 * buffer_stride) as wgt::BufferAddress;
                        let comm_desc = wgt::CommandEncoderDescriptor { label: None };
                        let _ = gfx_select!(encoder_id => global.device_create_command_encoder(
                            device_id,
                            &comm_desc,
                            encoder_id
                        ));

                        let buffer_cv = BufferCopyView {
                            buffer: buffer_id,
                            layout: wgt::TextureDataLayout {
                                offset: 0,
                                bytes_per_row: buffer_stride,
                                rows_per_image: 0,
                            },
                        };
                        let texture_cv = TextureCopyView {
                            texture: texture_id,
                            mip_level: 0,
                            origin: wgt::Origin3d::ZERO,
                        };
                        let copy_size = wgt::Extent3d {
                            width: size.width as u32,
                            height: size.height as u32,
                            depth: 1,
                        };
                        let _ = gfx_select!(encoder_id => global.command_encoder_copy_texture_to_buffer(
                            encoder_id,
                            &texture_cv,
                            &buffer_cv,
                            &copy_size
                        ));
                        let _ = gfx_select!(encoder_id => global.command_encoder_finish(
                            encoder_id,
                            &wgt::CommandBufferDescriptor::default()
                        ));
                        let _ = gfx_select!(queue_id => global.queue_submit(
                            queue_id,
                            &[encoder_id]
                        ));

                        let map_info = BufferMapInfo {
                            buffer_id,
                            sender: self.sender.clone(),
                            global: &self.global,
                            size: buffer_size as usize,
                            external_id: Some(external_id),
                        };
                        self.present_buffer_maps
                            .insert(buffer_id, Rc::new(map_info));
                        unsafe extern "C" fn callback(
                            status: BufferMapAsyncStatus,
                            userdata: *mut u8,
                        ) {
                            let info = Rc::from_raw(
                                userdata
                                    as *const BufferMapInfo<(Option<ErrorScopeId>, WebGPURequest)>,
                            );
                            match status {
                                BufferMapAsyncStatus::Success => {
                                    if let Err(e) = info.sender.send((
                                        None,
                                        WebGPURequest::UpdateWebRenderData {
                                            buffer_id: info.buffer_id,
                                            buffer_size: info.size,
                                            external_id: info.external_id.unwrap(),
                                        },
                                    )) {
                                        warn!("Could not send UpdateWebRenderData ({})", e);
                                    }
                                },
                                _ => error!("Could not map buffer({:?})", info.buffer_id),
                            }
                        }
                        let map_op = BufferMapOperation {
                            host: HostMap::Read,
                            callback,
                            user_data: convert_to_pointer(
                                self.present_buffer_maps.get(&buffer_id).unwrap().clone(),
                            ),
                        };
                        let _ = gfx_select!(buffer_id => global.buffer_map_async(buffer_id, 0..buffer_size, map_op));
                    },
                    WebGPURequest::UnmapBuffer {
                        buffer_id,
                        device_id,
                        array_buffer,
                        is_map_read,
                        offset,
                        size,
                    } => {
                        let global = &self.global;
                        if !is_map_read {
                            let map_ptr = gfx_select!(buffer_id => global.buffer_get_mapped_range(
                                buffer_id,
                                offset,
                                wgt::BufferSize::new(size)
                            ))
                            .unwrap();
                            unsafe { slice::from_raw_parts_mut(map_ptr, size as usize) }
                                .copy_from_slice(&array_buffer);
                        }
                        let result = gfx_select!(buffer_id => global.buffer_unmap(buffer_id));
                        self.send_result(device_id, scope_id, result);
                    },
                    WebGPURequest::UpdateWebRenderData {
                        buffer_id,
                        buffer_size,
                        external_id,
                    } => {
                        let global = &self.global;
                        let data_pt = gfx_select!(buffer_id =>
                            global.buffer_get_mapped_range(buffer_id, 0, None))
                        .unwrap();
                        let data = unsafe { slice::from_raw_parts(data_pt, buffer_size) }.to_vec();
                        if let Some(present_data) =
                            self.wgpu_image_map.lock().unwrap().get_mut(&external_id)
                        {
                            present_data.data = data;
                            let mut txn = webrender_api::Transaction::new();
                            txn.update_image(
                                present_data.image_key,
                                present_data.image_desc,
                                present_data.image_data.clone(),
                                &webrender_api::DirtyRect::All,
                            );
                            self.webrender_api
                                .send_transaction(self.webrender_document, txn);
                            present_data
                                .queued_buffer_ids
                                .retain(|b_id| *b_id != buffer_id);
                            present_data.available_buffer_ids.push(buffer_id);
                        } else {
                            warn!("Data not found for ExternalImageId({:?})", external_id);
                        }
                        let _ = gfx_select!(buffer_id => global.buffer_unmap(buffer_id));
                        self.present_buffer_maps.remove(&buffer_id);
                    },
                    WebGPURequest::WriteBuffer {
                        queue_id,
                        buffer_id,
                        buffer_offset,
                        data,
                    } => {
                        let global = &self.global;
                        //TODO: Report result to content process
                        let result = gfx_select!(queue_id => global.queue_write_buffer(
                            queue_id,
                            buffer_id,
                            buffer_offset as wgt::BufferAddress,
                            &data
                        ));
                        self.send_result(queue_id, scope_id, result);
                    },
                    WebGPURequest::WriteTexture {
                        queue_id,
                        texture_cv,
                        data_layout,
                        size,
                        data,
                    } => {
                        let global = &self.global;
                        //TODO: Report result to content process
                        let result = gfx_select!(queue_id => global.queue_write_texture(
                            queue_id,
                            &texture_cv,
                            &data,
                            &data_layout,
                            &size
                        ));
                        self.send_result(queue_id, scope_id, result);
                    },
                }
            }
        }
    }

    fn send_result<U, T: std::fmt::Debug>(
        &self,
        device_id: id::DeviceId,
        scope_id: Option<ErrorScopeId>,
        result: Result<U, T>,
    ) {
        let &pipeline_id = self.devices.get(&WebGPUDevice(device_id)).unwrap();
        if let Err(w) = self.script_sender.send(WebGPUMsg::WebGPUOpResult {
            device: WebGPUDevice(device_id),
            scope_id,
            pipeline_id,
            result: if let Err(e) = result {
                let err = format!("{:?}", e);
                if err.contains("OutOfMemory") {
                    WebGPUOpResult::OutOfMemoryError
                } else {
                    WebGPUOpResult::ValidationError(err)
                }
            } else {
                WebGPUOpResult::Success
            },
        }) {
            warn!("Failed to send WebGPUOpResult ({})", w);
        }
    }

    fn encoder_record_error<U, T: std::fmt::Debug>(
        &self,
        encoder_id: id::CommandEncoderId,
        result: Result<U, T>,
    ) {
        if let Err(e) = result {
            self.error_command_encoders
                .borrow_mut()
                .entry(encoder_id)
                .or_insert_with(|| format!("{:?}", e));
        }
    }
}

fn convert_to_pointer<T: Sized>(obj: Rc<T>) -> *mut u8 {
    Rc::into_raw(obj) as *mut u8
}

macro_rules! webgpu_resource {
    ($name:ident, $id:ty) => {
        #[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
        pub struct $name(pub $id);

        impl MallocSizeOf for $name {
            fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
                0
            }
        }

        impl Eq for $name {}
    };
}

webgpu_resource!(WebGPUAdapter, id::AdapterId);
webgpu_resource!(WebGPUBindGroup, id::BindGroupId);
webgpu_resource!(WebGPUBindGroupLayout, id::BindGroupLayoutId);
webgpu_resource!(WebGPUBuffer, id::BufferId);
webgpu_resource!(WebGPUCommandBuffer, id::CommandBufferId);
webgpu_resource!(WebGPUCommandEncoder, id::CommandEncoderId);
webgpu_resource!(WebGPUComputePipeline, id::ComputePipelineId);
webgpu_resource!(WebGPUDevice, id::DeviceId);
webgpu_resource!(WebGPUPipelineLayout, id::PipelineLayoutId);
webgpu_resource!(WebGPUQueue, id::QueueId);
webgpu_resource!(WebGPURenderBundle, id::RenderBundleId);
webgpu_resource!(WebGPURenderPipeline, id::RenderPipelineId);
webgpu_resource!(WebGPUSampler, id::SamplerId);
webgpu_resource!(WebGPUShaderModule, id::ShaderModuleId);
webgpu_resource!(WebGPUSwapChain, id::SwapChainId);
webgpu_resource!(WebGPUTexture, id::TextureId);
webgpu_resource!(WebGPUTextureView, id::TextureViewId);

pub struct WGPUExternalImages {
    pub images: Arc<Mutex<HashMap<u64, PresentationData>>>,
    pub locked_ids: HashMap<u64, Vec<u8>>,
}

impl WGPUExternalImages {
    pub fn new() -> Self {
        Self {
            images: Arc::new(Mutex::new(HashMap::new())),
            locked_ids: HashMap::new(),
        }
    }
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
    device_id: id::DeviceId,
    queue_id: id::QueueId,
    pub data: Vec<u8>,
    pub size: Size2D<i32>,
    unassigned_buffer_ids: ArrayVec<[id::BufferId; PRESENTATION_BUFFER_COUNT]>,
    available_buffer_ids: ArrayVec<[id::BufferId; PRESENTATION_BUFFER_COUNT]>,
    queued_buffer_ids: ArrayVec<[id::BufferId; PRESENTATION_BUFFER_COUNT]>,
    buffer_stride: u32,
    image_key: webrender_api::ImageKey,
    image_desc: webrender_api::ImageDescriptor,
    image_data: webrender_api::ImageData,
}
