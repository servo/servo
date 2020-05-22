/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate log;
#[macro_use]
pub extern crate wgpu_core as wgpu;
pub extern crate wgpu_types as wgt;

pub mod identity;

use identity::{IdentityRecyclerFactory, WebGPUMsg};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use serde::{Deserialize, Serialize};
use servo_config::pref;
use smallvec::SmallVec;
use std::ptr;
use wgpu::{
    binding_model::{
        BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    },
    id::{
        AdapterId, BindGroupId, BindGroupLayoutId, BufferId, CommandBufferId, CommandEncoderId,
        ComputePipelineId, DeviceId, PipelineLayoutId, QueueId, ShaderModuleId,
    },
    instance::RequestAdapterOptions,
};
use wgt::{BufferAddress, BufferDescriptor, CommandBufferDescriptor, DeviceDescriptor};

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
        _descriptor: DeviceDescriptor,
    },
}

pub type WebGPUResponseResult = Result<WebGPUResponse, String>;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebGPURequest {
    CommandEncoderFinish {
        sender: IpcSender<WebGPUCommandBuffer>,
        command_encoder_id: CommandEncoderId,
        // TODO(zakorgy): Serialize CommandBufferDescriptor in wgpu-core
        // wgpu::command::CommandBufferDescriptor,
    },
    CopyBufferToBuffer {
        command_encoder_id: CommandEncoderId,
        source_id: BufferId,
        source_offset: BufferAddress,
        destination_id: BufferId,
        destination_offset: BufferAddress,
        size: BufferAddress,
    },
    CreateBindGroup {
        sender: IpcSender<WebGPUBindGroup>,
        device_id: DeviceId,
        bind_group_id: BindGroupId,
        bind_group_layout_id: BindGroupLayoutId,
        bindings: Vec<BindGroupEntry>,
    },
    CreateBindGroupLayout {
        sender: IpcSender<WebGPUBindGroupLayout>,
        device_id: DeviceId,
        bind_group_layout_id: BindGroupLayoutId,
        bindings: Vec<BindGroupLayoutEntry>,
    },
    CreateBuffer {
        sender: IpcSender<WebGPUBuffer>,
        device_id: DeviceId,
        buffer_id: BufferId,
        descriptor: BufferDescriptor<String>,
    },
    CreateBufferMapped {
        sender: IpcSender<WebGPUBuffer>,
        device_id: DeviceId,
        buffer_id: BufferId,
        descriptor: BufferDescriptor<String>,
    },
    CreateCommandEncoder {
        sender: IpcSender<WebGPUCommandEncoder>,
        device_id: DeviceId,
        // TODO(zakorgy): Serialize CommandEncoderDescriptor in wgpu-core
        // wgpu::command::CommandEncoderDescriptor,
        command_encoder_id: CommandEncoderId,
    },
    CreateComputePipeline {
        sender: IpcSender<WebGPUComputePipeline>,
        device_id: DeviceId,
        compute_pipeline_id: ComputePipelineId,
        pipeline_layout_id: PipelineLayoutId,
        program_id: ShaderModuleId,
        entry_point: String,
    },
    CreatePipelineLayout {
        sender: IpcSender<WebGPUPipelineLayout>,
        device_id: DeviceId,
        pipeline_layout_id: PipelineLayoutId,
        bind_group_layouts: Vec<BindGroupLayoutId>,
    },
    CreateShaderModule {
        sender: IpcSender<WebGPUShaderModule>,
        device_id: DeviceId,
        program_id: ShaderModuleId,
        program: Vec<u32>,
    },
    DestroyBuffer(BufferId),
    Exit(IpcSender<()>),
    RequestAdapter {
        sender: IpcSender<WebGPUResponseResult>,
        options: RequestAdapterOptions,
        ids: SmallVec<[AdapterId; 4]>,
    },
    RequestDevice {
        sender: IpcSender<WebGPUResponseResult>,
        adapter_id: WebGPUAdapter,
        descriptor: DeviceDescriptor,
        device_id: DeviceId,
    },
    RunComputePass {
        command_encoder_id: CommandEncoderId,
        pass_data: Vec<u8>,
    },
    Submit {
        queue_id: QueueId,
        command_buffers: Vec<CommandBufferId>,
    },
    UnmapBuffer {
        device_id: DeviceId,
        buffer_id: BufferId,
        array_buffer: Vec<u8>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebGPU(pub IpcSender<WebGPURequest>);

impl WebGPU {
    pub fn new() -> Option<(Self, IpcReceiver<WebGPUMsg>)> {
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
                WGPU::new(receiver, sender_clone, script_sender).run();
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

struct WGPU {
    receiver: IpcReceiver<WebGPURequest>,
    sender: IpcSender<WebGPURequest>,
    global: wgpu::hub::Global<IdentityRecyclerFactory>,
    adapters: Vec<WebGPUAdapter>,
    devices: Vec<WebGPUDevice>,
    // Track invalid adapters https://gpuweb.github.io/gpuweb/#invalid
    _invalid_adapters: Vec<WebGPUAdapter>,
}

impl WGPU {
    fn new(
        receiver: IpcReceiver<WebGPURequest>,
        sender: IpcSender<WebGPURequest>,
        script_sender: IpcSender<WebGPUMsg>,
    ) -> Self {
        let factory = IdentityRecyclerFactory {
            sender: script_sender,
        };
        WGPU {
            receiver,
            sender,
            global: wgpu::hub::Global::new("wgpu-core", factory),
            adapters: Vec::new(),
            devices: Vec::new(),
            _invalid_adapters: Vec::new(),
        }
    }

    fn run(mut self) {
        while let Ok(msg) = self.receiver.recv() {
            match msg {
                WebGPURequest::CommandEncoderFinish {
                    sender,
                    command_encoder_id,
                } => {
                    let global = &self.global;
                    let command_buffer_id = gfx_select!(command_encoder_id => global.command_encoder_finish(
                        command_encoder_id,
                        &CommandBufferDescriptor::default()
                    ));
                    if let Err(e) = sender.send(WebGPUCommandBuffer(command_buffer_id)) {
                        warn!(
                            "Failed to send response to WebGPURequest::CommandEncoderFinish ({})",
                            e
                        )
                    }
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
                WebGPURequest::CreateBindGroup {
                    sender,
                    device_id,
                    bind_group_id,
                    bind_group_layout_id,
                    bindings,
                } => {
                    let global = &self.global;
                    let descriptor = BindGroupDescriptor {
                        layout: bind_group_layout_id,
                        entries: bindings.as_ptr(),
                        entries_length: bindings.len(),
                        label: ptr::null(),
                    };
                    let bg_id = gfx_select!(bind_group_id =>
                        global.device_create_bind_group(device_id, &descriptor, bind_group_id));
                    let bind_group = WebGPUBindGroup(bg_id);

                    if let Err(e) = sender.send(bind_group) {
                        warn!(
                            "Failed to send response to WebGPURequest::CreateBindGroup ({})",
                            e
                        )
                    }
                },
                WebGPURequest::CreateBindGroupLayout {
                    sender,
                    device_id,
                    bind_group_layout_id,
                    bindings,
                } => {
                    let global = &self.global;
                    let descriptor = BindGroupLayoutDescriptor {
                        entries: bindings.as_ptr(),
                        entries_length: bindings.len(),
                        label: ptr::null(),
                    };
                    let bgl_id = gfx_select!(bind_group_layout_id =>
                        global.device_create_bind_group_layout(device_id, &descriptor, bind_group_layout_id));
                    let bgl = WebGPUBindGroupLayout(bgl_id);

                    if let Err(e) = sender.send(bgl) {
                        warn!(
                            "Failed to send response to WebGPURequest::CreateBindGroupLayout ({})",
                            e
                        )
                    }
                },
                WebGPURequest::CreateBuffer {
                    sender,
                    device_id,
                    buffer_id,
                    descriptor,
                } => {
                    let global = &self.global;
                    let desc = BufferDescriptor {
                        size: descriptor.size,
                        usage: descriptor.usage,
                        label: ptr::null(),
                    };
                    let id = gfx_select!(buffer_id => global.device_create_buffer(device_id, &desc, buffer_id));
                    let buffer = WebGPUBuffer(id);
                    if let Err(e) = sender.send(buffer) {
                        warn!(
                            "Failed to send response to WebGPURequest::CreateBuffer ({})",
                            e
                        )
                    }
                },
                WebGPURequest::CreateBufferMapped {
                    sender,
                    device_id,
                    buffer_id,
                    descriptor,
                } => {
                    let global = &self.global;
                    let desc = BufferDescriptor {
                        size: descriptor.size,
                        usage: descriptor.usage,
                        label: ptr::null(),
                    };
                    let (buffer_id, _arr_buff_ptr) = gfx_select!(buffer_id =>
                        global.device_create_buffer_mapped(device_id, &desc, buffer_id));
                    let buffer = WebGPUBuffer(buffer_id);

                    if let Err(e) = sender.send(buffer) {
                        warn!(
                            "Failed to send response to WebGPURequest::CreateBufferMapped ({})",
                            e
                        )
                    }
                },
                WebGPURequest::CreateCommandEncoder {
                    sender,
                    device_id,
                    command_encoder_id,
                } => {
                    let global = &self.global;
                    let id = gfx_select!(command_encoder_id =>
                        global.device_create_command_encoder(device_id, &Default::default(), command_encoder_id));
                    if let Err(e) = sender.send(WebGPUCommandEncoder(id)) {
                        warn!(
                            "Failed to send response to WebGPURequest::CreateCommandEncoder ({})",
                            e
                        )
                    }
                },
                WebGPURequest::CreateComputePipeline {
                    sender,
                    device_id,
                    compute_pipeline_id,
                    pipeline_layout_id,
                    program_id,
                    entry_point,
                } => {
                    let global = &self.global;
                    let entry_point = std::ffi::CString::new(entry_point).unwrap();
                    let descriptor = wgpu_core::pipeline::ComputePipelineDescriptor {
                        layout: pipeline_layout_id,
                        compute_stage: wgpu_core::pipeline::ProgrammableStageDescriptor {
                            module: program_id,
                            entry_point: entry_point.as_ptr(),
                        },
                    };
                    let cp_id = gfx_select!(compute_pipeline_id =>
                        global.device_create_compute_pipeline(device_id, &descriptor, compute_pipeline_id));
                    let compute_pipeline = WebGPUComputePipeline(cp_id);

                    if let Err(e) = sender.send(compute_pipeline) {
                        warn!(
                            "Failed to send response to WebGPURequest::CreateComputePipeline ({})",
                            e
                        )
                    }
                },
                WebGPURequest::CreatePipelineLayout {
                    sender,
                    device_id,
                    pipeline_layout_id,
                    bind_group_layouts,
                } => {
                    let global = &self.global;
                    let descriptor = wgpu_core::binding_model::PipelineLayoutDescriptor {
                        bind_group_layouts: bind_group_layouts.as_ptr(),
                        bind_group_layouts_length: bind_group_layouts.len(),
                    };
                    let pl_id = gfx_select!(pipeline_layout_id =>
                        global.device_create_pipeline_layout(device_id, &descriptor, pipeline_layout_id));
                    let pipeline_layout = WebGPUPipelineLayout(pl_id);

                    if let Err(e) = sender.send(pipeline_layout) {
                        warn!(
                            "Failed to send response to WebGPURequest::CreatePipelineLayout ({})",
                            e
                        )
                    }
                },
                WebGPURequest::CreateShaderModule {
                    sender,
                    device_id,
                    program_id,
                    program,
                } => {
                    let global = &self.global;
                    let descriptor = wgpu_core::pipeline::ShaderModuleDescriptor {
                        code: wgpu_core::U32Array {
                            bytes: program.as_ptr(),
                            length: program.len(),
                        },
                    };
                    let sm_id = gfx_select!(program_id =>
                        global.device_create_shader_module(device_id, &descriptor, program_id));
                    let shader_module = WebGPUShaderModule(sm_id);

                    if let Err(e) = sender.send(shader_module) {
                        warn!(
                            "Failed to send response to WebGPURequest::CreateShaderModule ({})",
                            e
                        )
                    }
                },
                WebGPURequest::DestroyBuffer(buffer) => {
                    let global = &self.global;
                    gfx_select!(buffer => global.buffer_destroy(buffer));
                },
                WebGPURequest::Exit(sender) => {
                    drop(self.global);
                    if let Err(e) = sender.send(()) {
                        warn!("Failed to send response to WebGPURequest::Exit ({})", e)
                    }
                    return;
                },
                WebGPURequest::RequestAdapter {
                    sender,
                    options,
                    ids,
                } => {
                    let adapter_id = if let Some(pos) = self
                        .adapters
                        .iter()
                        .position(|adapter| ids.contains(&adapter.0))
                    {
                        self.adapters[pos].0
                    } else {
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
                        adapter_id
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
                } => {
                    let global = &self.global;
                    let id = gfx_select!(device_id => global.adapter_request_device(
                        adapter_id.0,
                        &descriptor,
                        None,
                        device_id
                    ));

                    let device = WebGPUDevice(id);
                    // Note: (zakorgy) Note sure if sending the queue is needed at all,
                    // since wgpu-core uses the same id for the device and the queue
                    let queue = WebGPUQueue(id);
                    self.devices.push(device);
                    if let Err(e) = sender.send(Ok(WebGPUResponse::RequestDevice {
                        device_id: device,
                        queue_id: queue,
                        _descriptor: descriptor,
                    })) {
                        warn!(
                            "Failed to send response to WebGPURequest::RequestDevice ({})",
                            e
                        )
                    }
                },
                WebGPURequest::RunComputePass {
                    command_encoder_id,
                    pass_data,
                } => {
                    let global = &self.global;
                    gfx_select!(command_encoder_id => global.command_encoder_run_compute_pass(
                        command_encoder_id,
                        &pass_data
                    ));
                },
                WebGPURequest::Submit {
                    queue_id,
                    command_buffers,
                } => {
                    let global = &self.global;
                    let _ = gfx_select!(queue_id => global.queue_submit(
                        queue_id,
                        &command_buffers
                    ));
                },
                WebGPURequest::UnmapBuffer {
                    device_id,
                    buffer_id,
                    array_buffer,
                } => {
                    let global = &self.global;

                    gfx_select!(buffer_id => global.device_set_buffer_sub_data(
                        device_id,
                        buffer_id,
                        0,
                        array_buffer.as_slice()
                    ));
                },
            }
        }
    }
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

webgpu_resource!(WebGPUAdapter, AdapterId);
webgpu_resource!(WebGPUBindGroup, BindGroupId);
webgpu_resource!(WebGPUBindGroupLayout, BindGroupLayoutId);
webgpu_resource!(WebGPUBuffer, BufferId);
webgpu_resource!(WebGPUCommandBuffer, CommandBufferId);
webgpu_resource!(WebGPUCommandEncoder, CommandEncoderId);
webgpu_resource!(WebGPUComputePipeline, ComputePipelineId);
webgpu_resource!(WebGPUDevice, DeviceId);
webgpu_resource!(WebGPUPipelineLayout, PipelineLayoutId);
webgpu_resource!(WebGPUQueue, QueueId);
webgpu_resource!(WebGPUShaderModule, ShaderModuleId);
