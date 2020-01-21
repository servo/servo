/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
pub extern crate wgpu_core as wgpu;

use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use servo_config::pref;
use smallvec::SmallVec;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebGPUResponse {
    RequestAdapter(String, WebGPUAdapter, WebGPU),
    RequestDevice(WebGPUDevice, wgpu::instance::DeviceDescriptor),
}

pub type WebGPUResponseResult = Result<WebGPUResponse, String>;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebGPURequest {
    RequestAdapter(
        IpcSender<WebGPUResponseResult>,
        wgpu::instance::RequestAdapterOptions,
        SmallVec<[wgpu::id::AdapterId; 4]>,
    ),
    RequestDevice(
        IpcSender<WebGPUResponseResult>,
        WebGPUAdapter,
        wgpu::instance::DeviceDescriptor,
        wgpu::id::DeviceId,
    ),
    Exit(IpcSender<()>),
    CreateBuffer(
        IpcSender<WebGPUBuffer>,
        WebGPUDevice,
        wgpu::id::BufferId,
        wgpu::resource::BufferDescriptor,
    ),
    CreateBufferMapped(
        IpcSender<(WebGPUBuffer, Vec<u8>)>,
        WebGPUDevice,
        wgpu::id::BufferId,
        wgpu::resource::BufferDescriptor,
    ),
    CreateBindGroupLayout(
        IpcSender<WebGPUBindGroupLayout>,
        WebGPUDevice,
        wgpu::id::BindGroupLayoutId,
        Vec<wgpu::binding_model::BindGroupLayoutBinding>,
    ),
    CreatePipelineLayout(
        IpcSender<WebGPUPipelineLayout>,
        WebGPUDevice,
        wgpu::id::PipelineLayoutId,
        Vec<wgpu::id::BindGroupLayoutId>,
    ),
    UnmapBuffer(WebGPUBuffer),
    DestroyBuffer(WebGPUBuffer),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebGPU(pub IpcSender<WebGPURequest>);

impl WebGPU {
    pub fn new() -> Option<Self> {
        if !pref!(dom.webgpu.enabled) {
            return None;
        }
        let (sender, receiver) = match ipc::channel() {
            Ok(sender_and_receiver) => sender_and_receiver,
            Err(e) => {
                warn!(
                    "Failed to create sender and receiciver for WGPU thread ({})",
                    e
                );
                return None;
            },
        };
        let sender_clone = sender.clone();

        if let Err(e) = std::thread::Builder::new()
            .name("WGPU".to_owned())
            .spawn(move || {
                WGPU::new(receiver, sender_clone).run();
            })
        {
            warn!("Failed to spwan WGPU thread ({})", e);
            return None;
        }
        Some(WebGPU(sender))
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
    global: wgpu::hub::Global<()>,
    adapters: Vec<WebGPUAdapter>,
    devices: Vec<WebGPUDevice>,
    // Track invalid adapters https://gpuweb.github.io/gpuweb/#invalid
    _invalid_adapters: Vec<WebGPUAdapter>,
}

impl WGPU {
    fn new(receiver: IpcReceiver<WebGPURequest>, sender: IpcSender<WebGPURequest>) -> Self {
        WGPU {
            receiver,
            sender,
            global: wgpu::hub::Global::new("wgpu-core"),
            adapters: Vec::new(),
            devices: Vec::new(),
            _invalid_adapters: Vec::new(),
        }
    }

    fn deinit(self) {
        self.global.delete()
    }

    fn run(mut self) {
        while let Ok(msg) = self.receiver.recv() {
            match msg {
                WebGPURequest::RequestAdapter(sender, options, ids) => {
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
                    if let Err(e) = sender.send(Ok(WebGPUResponse::RequestAdapter(
                        info.name,
                        adapter,
                        WebGPU(self.sender.clone()),
                    ))) {
                        warn!(
                            "Failed to send response to WebGPURequest::RequestAdapter ({})",
                            e
                        )
                    }
                },
                WebGPURequest::RequestDevice(sender, adapter, descriptor, id) => {
                    let global = &self.global;
                    let id = gfx_select!(id => global.adapter_request_device(
                        adapter.0,
                        &descriptor,
                        id
                    ));
                    let device = WebGPUDevice(id);
                    self.devices.push(device);
                    if let Err(e) =
                        sender.send(Ok(WebGPUResponse::RequestDevice(device, descriptor)))
                    {
                        warn!(
                            "Failed to send response to WebGPURequest::RequestDevice ({})",
                            e
                        )
                    }
                },
                WebGPURequest::CreateBuffer(sender, device, id, descriptor) => {
                    let global = &self.global;
                    let buffer_id =
                        gfx_select!(id => global.device_create_buffer(device.0, &descriptor, id));
                    let buffer = WebGPUBuffer(buffer_id);
                    if let Err(e) = sender.send(buffer) {
                        warn!(
                            "Failed to send response to WebGPURequest::CreateBuffer ({})",
                            e
                        )
                    }
                },
                WebGPURequest::CreateBufferMapped(sender, device, id, descriptor) => {
                    let global = &self.global;
                    let buffer_size = descriptor.size as usize;

                    let (buffer_id, arr_buff_ptr) = gfx_select!(id =>
                        global.device_create_buffer_mapped(device.0, &descriptor, id));
                    let buffer = WebGPUBuffer(buffer_id);

                    let mut array_buffer = Vec::with_capacity(buffer_size);
                    unsafe {
                        array_buffer.set_len(buffer_size);
                        std::ptr::copy(arr_buff_ptr, array_buffer.as_mut_ptr(), buffer_size);
                    };
                    if let Err(e) = sender.send((buffer, array_buffer)) {
                        warn!(
                            "Failed to send response to WebGPURequest::CreateBufferMapped ({})",
                            e
                        )
                    }
                },
                WebGPURequest::UnmapBuffer(buffer) => {
                    let global = &self.global;
                    gfx_select!(buffer.0 => global.buffer_unmap(buffer.0));
                },
                WebGPURequest::DestroyBuffer(buffer) => {
                    let global = &self.global;
                    gfx_select!(buffer.0 => global.buffer_destroy(buffer.0));
                },
                WebGPURequest::CreateBindGroupLayout(sender, device, id, bindings) => {
                    let global = &self.global;
                    let descriptor = wgpu_core::binding_model::BindGroupLayoutDescriptor {
                        bindings: bindings.as_ptr(),
                        bindings_length: bindings.len(),
                    };
                    let bgl_id = gfx_select!(id => global.device_create_bind_group_layout(device.0, &descriptor, id));
                    let bgl = WebGPUBindGroupLayout(bgl_id);

                    if let Err(e) = sender.send(bgl) {
                        warn!(
                            "Failed to send response to WebGPURequest::CreateBindGroupLayout ({})",
                            e
                        )
                    }
                },
                WebGPURequest::CreatePipelineLayout(sender, device, id, bind_group_layouts) => {
                    let global = &self.global;
                    let descriptor = wgpu_core::binding_model::PipelineLayoutDescriptor {
                        bind_group_layouts: bind_group_layouts.as_ptr(),
                        bind_group_layouts_length: bind_group_layouts.len(),
                    };
                    let pl_id = gfx_select!(id => global.device_create_pipeline_layout(device.0, &descriptor, id));
                    let pipeline_layout = WebGPUPipelineLayout(pl_id);

                    if let Err(e) = sender.send(pipeline_layout) {
                        warn!(
                            "Failed to send response to WebGPURequest::CreatePipelineLayout ({})",
                            e
                        )
                    }
                },
                WebGPURequest::Exit(sender) => {
                    self.deinit();
                    if let Err(e) = sender.send(()) {
                        warn!("Failed to send response to WebGPURequest::Exit ({})", e)
                    }
                    return;
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

webgpu_resource!(WebGPUAdapter, wgpu::id::AdapterId);
webgpu_resource!(WebGPUDevice, wgpu::id::DeviceId);
webgpu_resource!(WebGPUBuffer, wgpu::id::BufferId);
webgpu_resource!(WebGPUBindGroupLayout, wgpu::id::BindGroupLayoutId);
webgpu_resource!(WebGPUPipelineLayout, wgpu::id::PipelineLayoutId);
