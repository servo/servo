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
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;
use std::rc::Rc;
use std::slice;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use webrender_traits::{
    WebrenderExternalImageApi, WebrenderExternalImageRegistry, WebrenderImageHandlerType,
    WebrenderImageSource,
};
use wgpu::{
    binding_model::BindGroupDescriptor,
    command::{BufferCopyView, ComputePass, RenderBundleEncoder, RenderPass, TextureCopyView},
    device::HostMap,
    id,
    instance::RequestAdapterOptions,
    pipeline::{ComputePipelineDescriptor, RenderPipelineDescriptor},
    resource::{BufferMapAsyncStatus, BufferMapOperation},
};

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
        host_map: HostMap,
        map_range: std::ops::Range<u64>,
    },
    BufferMapComplete(id::BufferId),
    CommandEncoderFinish {
        command_encoder_id: id::CommandEncoderId,
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
        // TODO: Consider using NonZeroU64 to reduce enum size
        scope_id: Option<u64>,
        bind_group_id: id::BindGroupId,
        descriptor: BindGroupDescriptor<'static>,
    },
    CreateBindGroupLayout {
        device_id: id::DeviceId,
        scope_id: Option<u64>,
        bind_group_layout_id: id::BindGroupLayoutId,
        descriptor: wgt::BindGroupLayoutDescriptor<'static>,
    },
    CreateBuffer {
        device_id: id::DeviceId,
        buffer_id: id::BufferId,
        descriptor: wgt::BufferDescriptor<Option<String>>,
    },
    CreateCommandEncoder {
        device_id: id::DeviceId,
        // TODO(zakorgy): Serialize CommandEncoderDescriptor in wgpu-core
        // wgpu::command::CommandEncoderDescriptor,
        command_encoder_id: id::CommandEncoderId,
        label: Option<String>,
    },
    CreateComputePipeline {
        device_id: id::DeviceId,
        scope_id: Option<u64>,
        compute_pipeline_id: id::ComputePipelineId,
        descriptor: ComputePipelineDescriptor<'static>,
    },
    CreateContext(IpcSender<webrender_api::ExternalImageId>),
    CreatePipelineLayout {
        device_id: id::DeviceId,
        scope_id: Option<u64>,
        pipeline_layout_id: id::PipelineLayoutId,
        descriptor: wgt::PipelineLayoutDescriptor<'static, id::BindGroupLayoutId>,
    },
    CreateRenderPipeline {
        device_id: id::DeviceId,
        scope_id: Option<u64>,
        render_pipeline_id: id::RenderPipelineId,
        descriptor: RenderPipelineDescriptor<'static>,
    },
    CreateSampler {
        device_id: id::DeviceId,
        sampler_id: id::SamplerId,
        descriptor: wgt::SamplerDescriptor<Option<String>>,
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
        descriptor: wgt::TextureDescriptor<Option<String>>,
    },
    CreateTextureView {
        texture_id: id::TextureId,
        texture_view_id: id::TextureViewId,
        descriptor: wgt::TextureViewDescriptor<Option<String>>,
    },
    DestroyBuffer(id::BufferId),
    DestroySwapChain {
        external_id: u64,
        image_key: webrender_api::ImageKey,
    },
    DestroyTexture(id::TextureId),
    Exit(IpcSender<()>),
    FreeDevice(id::DeviceId),
    RenderBundleEncoderFinish {
        render_bundle_encoder: RenderBundleEncoder,
        descriptor: wgt::RenderBundleDescriptor<Option<String>>,
        render_bundle_id: id::RenderBundleId,
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
        compute_pass: ComputePass,
    },
    RunRenderPass {
        command_encoder_id: id::CommandEncoderId,
        render_pass: RenderPass,
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
pub struct WebGPU(pub IpcSender<WebGPURequest>);

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
            .send(WebGPURequest::Exit(sender))
            .map_err(|_| "Failed to send Exit message")
    }
}

struct WGPU<'a> {
    receiver: IpcReceiver<WebGPURequest>,
    sender: IpcSender<WebGPURequest>,
    script_sender: IpcSender<WebGPUMsg>,
    global: wgpu::hub::Global<IdentityRecyclerFactory>,
    adapters: Vec<WebGPUAdapter>,
    devices: HashMap<WebGPUDevice, PipelineId>,
    // Track invalid adapters https://gpuweb.github.io/gpuweb/#invalid
    _invalid_adapters: Vec<WebGPUAdapter>,
    // Buffers with pending mapping
    buffer_maps: HashMap<id::BufferId, Rc<BufferMapInfo<'a, WebGPUResponseResult>>>,
    // Presentation Buffers with pending mapping
    present_buffer_maps: HashMap<id::BufferId, Rc<BufferMapInfo<'a, WebGPURequest>>>,
    webrender_api: webrender_api::RenderApi,
    webrender_document: webrender_api::DocumentId,
    external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
    wgpu_image_map: Arc<Mutex<HashMap<u64, PresentationData>>>,
    last_poll: Instant,
}

impl<'a> WGPU<'a> {
    fn new(
        receiver: IpcReceiver<WebGPURequest>,
        sender: IpcSender<WebGPURequest>,
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
            if let Ok(msg) = self.receiver.try_recv() {
                match msg {
                    WebGPURequest::BufferMapAsync {
                        sender,
                        buffer_id,
                        host_map,
                        map_range,
                    } => {
                        let map_info = BufferMapInfo {
                            buffer_id,
                            sender,
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
                            match status {
                                BufferMapAsyncStatus::Success => {
                                    let global = &info.global;
                                    let data_pt = gfx_select!(info.buffer_id =>
                                        global.buffer_get_mapped_range(info.buffer_id, 0, None))
                                    .unwrap();
                                    let data = slice::from_raw_parts(data_pt, info.size);
                                    if let Err(e) =
                                        info.sender.send(Ok(WebGPUResponse::BufferMapAsync(
                                            IpcSharedMemory::from_bytes(data),
                                        )))
                                    {
                                        warn!("Could not send BufferMapAsync Response ({})", e);
                                    }
                                },
                                _ => error!("Could not map buffer({:?})", info.buffer_id),
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
                        let _ = gfx_select!(buffer_id => global.buffer_map_async(buffer_id, map_range, operation));
                    },
                    WebGPURequest::BufferMapComplete(buffer_id) => {
                        self.buffer_maps.remove(&buffer_id);
                    },
                    WebGPURequest::CommandEncoderFinish { command_encoder_id } => {
                        let global = &self.global;
                        let _ = gfx_select!(command_encoder_id => global.command_encoder_finish(
                            command_encoder_id,
                            &wgt::CommandBufferDescriptor::default()
                        ));
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
                        let _ = gfx_select!(command_encoder_id => global.command_encoder_copy_buffer_to_buffer(
                            command_encoder_id,
                            source_id,
                            source_offset,
                            destination_id,
                            destination_offset,
                            size
                        ));
                    },
                    WebGPURequest::CopyBufferToTexture {
                        command_encoder_id,
                        source,
                        destination,
                        copy_size,
                    } => {
                        let global = &self.global;
                        let _ = gfx_select!(command_encoder_id => global.command_encoder_copy_buffer_to_texture(
                            command_encoder_id,
                            &source,
                            &destination,
                            &copy_size
                        ));
                    },
                    WebGPURequest::CopyTextureToBuffer {
                        command_encoder_id,
                        source,
                        destination,
                        copy_size,
                    } => {
                        let global = &self.global;
                        let _ = gfx_select!(command_encoder_id => global.command_encoder_copy_texture_to_buffer(
                            command_encoder_id,
                            &source,
                            &destination,
                            &copy_size
                        ));
                    },
                    WebGPURequest::CopyTextureToTexture {
                        command_encoder_id,
                        source,
                        destination,
                        copy_size,
                    } => {
                        let global = &self.global;
                        let _ = gfx_select!(command_encoder_id => global.command_encoder_copy_texture_to_texture(
                            command_encoder_id,
                            &source,
                            &destination,
                            &copy_size
                        ));
                    },
                    WebGPURequest::CreateBindGroup {
                        device_id,
                        scope_id,
                        bind_group_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let result = gfx_select!(bind_group_id =>
                            global.device_create_bind_group(device_id, &descriptor, bind_group_id));
                        if let Some(s_id) = scope_id {
                            self.send_result(device_id, s_id, result);
                        }
                    },
                    WebGPURequest::CreateBindGroupLayout {
                        device_id,
                        scope_id,
                        bind_group_layout_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let result = gfx_select!(bind_group_layout_id =>
                            global.device_create_bind_group_layout(device_id, &descriptor, bind_group_layout_id));
                        if let Some(s_id) = scope_id {
                            self.send_result(device_id, s_id, result);
                        }
                    },
                    WebGPURequest::CreateBuffer {
                        device_id,
                        buffer_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let st;
                        let label = match descriptor.label {
                            Some(ref s) => {
                                st = CString::new(s.as_bytes()).unwrap();
                                st.as_ptr()
                            },
                            None => ptr::null(),
                        };
                        let _ = gfx_select!(buffer_id =>
                            global.device_create_buffer(device_id, &descriptor.map_label(|_| label), buffer_id));
                    },
                    WebGPURequest::CreateCommandEncoder {
                        device_id,
                        command_encoder_id,
                        label,
                    } => {
                        let global = &self.global;
                        let st;
                        let label = match label {
                            Some(ref s) => {
                                st = CString::new(s.as_bytes()).unwrap();
                                st.as_ptr()
                            },
                            None => ptr::null(),
                        };
                        let desc = wgt::CommandEncoderDescriptor { label };
                        let _ = gfx_select!(command_encoder_id =>
                            global.device_create_command_encoder(device_id, &desc, command_encoder_id));
                    },
                    WebGPURequest::CreateComputePipeline {
                        device_id,
                        scope_id,
                        compute_pipeline_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let result = gfx_select!(compute_pipeline_id =>
                            global.device_create_compute_pipeline(device_id, &descriptor, compute_pipeline_id));
                        if let Some(s_id) = scope_id {
                            self.send_result(device_id, s_id, result);
                        }
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
                        scope_id,
                        pipeline_layout_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let result = gfx_select!(pipeline_layout_id =>
                            global.device_create_pipeline_layout(device_id, &descriptor, pipeline_layout_id));
                        if let Some(s_id) = scope_id {
                            self.send_result(device_id, s_id, result);
                        }
                    },
                    //TODO: consider https://github.com/gfx-rs/wgpu/issues/684
                    WebGPURequest::CreateRenderPipeline {
                        device_id,
                        scope_id,
                        render_pipeline_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let result = gfx_select!(render_pipeline_id =>
                            global.device_create_render_pipeline(device_id, &descriptor, render_pipeline_id));
                        if let Some(s_id) = scope_id {
                            self.send_result(device_id, s_id, result);
                        }
                    },
                    WebGPURequest::CreateSampler {
                        device_id,
                        sampler_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let st;
                        let label = match descriptor.label {
                            Some(ref s) => {
                                st = CString::new(s.as_bytes()).unwrap();
                                st.as_ptr()
                            },
                            None => ptr::null(),
                        };
                        let _ = gfx_select!(sampler_id => global.device_create_sampler(
                            device_id,
                            &descriptor.map_label(|_| label),
                            sampler_id
                        ));
                    },
                    WebGPURequest::CreateShaderModule {
                        device_id,
                        program_id,
                        program,
                    } => {
                        let global = &self.global;
                        let source =
                            wgpu_core::pipeline::ShaderModuleSource::SpirV(Cow::Owned(program));
                        let _ = gfx_select!(program_id =>
                            global.device_create_shader_module(device_id, source, program_id));
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
                        let st;
                        let label = match descriptor.label {
                            Some(ref s) => {
                                st = CString::new(s.as_bytes()).unwrap();
                                st.as_ptr()
                            },
                            None => ptr::null(),
                        };
                        let _ = gfx_select!(texture_id => global.device_create_texture(
                            device_id,
                            &descriptor.map_label(|_| label),
                            texture_id
                        ));
                    },
                    WebGPURequest::CreateTextureView {
                        texture_id,
                        texture_view_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let st;
                        let label = match descriptor.label {
                            Some(ref s) => {
                                st = CString::new(s.as_bytes()).unwrap();
                                st.as_ptr()
                            },
                            None => ptr::null(),
                        };
                        let _ = gfx_select!(texture_view_id => global.texture_create_view(
                            texture_id,
                            Some(&descriptor.map_label(|_| label)),
                            texture_view_id
                        ));
                    },
                    WebGPURequest::DestroyBuffer(buffer) => {
                        let global = &self.global;
                        gfx_select!(buffer => global.buffer_destroy(buffer));
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
                            gfx_select!(b_id => global.buffer_destroy(*b_id));
                        }
                        for b_id in data.queued_buffer_ids.iter() {
                            gfx_select!(b_id => global.buffer_destroy(*b_id));
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
                        gfx_select!(texture => global.texture_destroy(texture));
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
                    } => {
                        let global = &self.global;
                        let st;
                        let label = match descriptor.label {
                            Some(ref s) => {
                                st = CString::new(s.as_bytes()).unwrap();
                                st.as_ptr()
                            },
                            None => ptr::null(),
                        };
                        let _ = gfx_select!(render_bundle_id => global.render_bundle_encoder_finish(
                            render_bundle_encoder,
                            &descriptor.map_label(|_| label),
                            render_bundle_id
                        ));
                    },
                    WebGPURequest::RequestAdapter {
                        sender,
                        options,
                        ids,
                    } => {
                        let adapter_id = match self.global.pick_adapter(
                            &options,
                            wgpu::instance::AdapterInputs::IdSet(&ids, |id| id.backend()),
                        ) {
                            Some(id) => id,
                            None => {
                                if let Err(e) =
                                    sender.send(Err("Failed to get webgpu adapter".to_string()))
                                {
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
                        let info = gfx_select!(adapter_id => global.adapter_get_info(adapter_id));
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
                        let result = gfx_select!(device_id => global.adapter_request_device(
                            adapter_id.0,
                            &descriptor,
                            None,
                            device_id
                        ));
                        // TODO: Handle error gracefully acc. to spec.
                        let id = result.unwrap();
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
                        let _ = gfx_select!(command_encoder_id => global.command_encoder_run_compute_pass(
                            command_encoder_id,
                            &compute_pass
                        ));
                    },
                    WebGPURequest::RunRenderPass {
                        command_encoder_id,
                        render_pass,
                    } => {
                        let global = &self.global;
                        let _ = gfx_select!(command_encoder_id => global.command_encoder_run_render_pass(
                            command_encoder_id,
                            &render_pass
                        ));
                    },
                    WebGPURequest::Submit {
                        queue_id,
                        command_buffers,
                    } => {
                        let global = &self.global;
                        let _ = gfx_select!(queue_id => global.queue_submit(queue_id, &command_buffers));
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
                                        label: ptr::null(),
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
                        let comm_desc = wgt::CommandEncoderDescriptor { label: ptr::null() };
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
                            let info =
                                Rc::from_raw(userdata as *const BufferMapInfo<WebGPURequest>);
                            match status {
                                BufferMapAsyncStatus::Success => {
                                    if let Err(e) =
                                        info.sender.send(WebGPURequest::UpdateWebRenderData {
                                            buffer_id: info.buffer_id,
                                            buffer_size: info.size,
                                            external_id: info.external_id.unwrap(),
                                        })
                                    {
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
                        let _ = gfx_select!(buffer_id => global.buffer_unmap(buffer_id));
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
                        gfx_select!(queue_id => global.queue_write_buffer(
                            queue_id,
                            buffer_id,
                            buffer_offset as wgt::BufferAddress,
                            &data
                        ));
                    },
                    WebGPURequest::WriteTexture {
                        queue_id,
                        texture_cv,
                        data_layout,
                        size,
                        data,
                    } => {
                        let global = &self.global;
                        gfx_select!(queue_id => global.queue_write_texture(
                            queue_id,
                            &texture_cv,
                            &data,
                            &data_layout,
                            &size
                        ));
                    },
                }
            }
        }
    }

    fn send_result<U: id::TypedId, T: std::fmt::Debug>(
        &self,
        device_id: id::DeviceId,
        scope_id: u64,
        result: Result<U, T>,
    ) {
        let &pipeline_id = self.devices.get(&WebGPUDevice(device_id)).unwrap();
        if let Err(w) = self.script_sender.send(WebGPUMsg::WebGPUOpResult {
            device: WebGPUDevice(device_id),
            scope_id,
            pipeline_id,
            result: if let Err(e) = result {
                WebGPUOpResult::ValidationError(format!("{:?}", e))
            } else {
                WebGPUOpResult::Success
            },
        }) {
            warn!("Failed to send WebGPUOpResult ({})", w);
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
