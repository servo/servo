/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Data and main loop of WebGPU thread.

use std::cell::RefCell;
use std::collections::HashMap;
use std::slice;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use arrayvec::ArrayVec;
use euclid::default::Size2D;
use ipc_channel::ipc::{IpcReceiver, IpcSender, IpcSharedMemory};
use log::{error, warn};
use msg::constellation_msg::PipelineId;
use webrender::{RenderApi, RenderApiSender, Transaction};
use webrender_api::{DirtyRect, DocumentId};
use webrender_traits::{WebrenderExternalImageRegistry, WebrenderImageHandlerType};
use wgc::command::{ImageCopyBuffer, ImageCopyTexture};
use wgc::device::queue::SubmittedWorkDoneClosure;
use wgc::device::{DeviceDescriptor, HostMap, ImplicitPipelineIds};
use wgc::pipeline::ShaderModuleDescriptor;
use wgc::resource::{BufferMapCallback, BufferMapOperation};
use wgc::{gfx_select, id};
use wgt::InstanceDescriptor;
pub use {wgpu_core as wgc, wgpu_types as wgt};

use crate::{
    ErrorScopeId, PresentationData, Transmute, WebGPU, WebGPUAdapter, WebGPUDevice, WebGPUMsg,
    WebGPUOpResult, WebGPUQueue, WebGPURequest, WebGPUResponse,
};

const DEVICE_POLL_INTERVAL: Duration = Duration::from_millis(50);
pub const PRESENTATION_BUFFER_COUNT: usize = 10;

#[allow(clippy::upper_case_acronyms)] // Name of the library
pub(crate) struct WGPU {
    receiver: IpcReceiver<(Option<ErrorScopeId>, WebGPURequest)>,
    sender: IpcSender<(Option<ErrorScopeId>, WebGPURequest)>,
    script_sender: IpcSender<WebGPUMsg>,
    global: Arc<wgc::global::Global>,
    adapters: Vec<WebGPUAdapter>,
    devices: HashMap<WebGPUDevice, PipelineId>,
    // Track invalid adapters https://gpuweb.github.io/gpuweb/#invalid
    _invalid_adapters: Vec<WebGPUAdapter>,
    //TODO: Remove this (https://github.com/gfx-rs/wgpu/issues/867)
    error_command_encoders: RefCell<HashMap<id::CommandEncoderId, String>>,
    webrender_api: Arc<Mutex<RenderApi>>,
    webrender_document: DocumentId,
    external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
    wgpu_image_map: Arc<Mutex<HashMap<u64, PresentationData>>>,
    last_poll: Instant,
}

impl WGPU {
    pub(crate) fn new(
        receiver: IpcReceiver<(Option<ErrorScopeId>, WebGPURequest)>,
        sender: IpcSender<(Option<ErrorScopeId>, WebGPURequest)>,
        script_sender: IpcSender<WebGPUMsg>,
        webrender_api_sender: RenderApiSender,
        webrender_document: DocumentId,
        external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
        wgpu_image_map: Arc<Mutex<HashMap<u64, PresentationData>>>,
    ) -> Self {
        WGPU {
            receiver,
            sender,
            script_sender,
            global: Arc::new(wgc::global::Global::new(
                "wgpu-core",
                InstanceDescriptor {
                    backends: wgt::Backends::PRIMARY,
                    ..Default::default()
                },
            )),
            adapters: Vec::new(),
            devices: HashMap::new(),
            _invalid_adapters: Vec::new(),
            error_command_encoders: RefCell::new(HashMap::new()),
            webrender_api: Arc::new(Mutex::new(webrender_api_sender.create_api())),
            webrender_document,
            external_images,
            wgpu_image_map,
            last_poll: Instant::now(),
        }
    }

    pub(crate) fn run(&mut self) {
        loop {
            let diff = DEVICE_POLL_INTERVAL.checked_sub(self.last_poll.elapsed());
            if diff.is_none() {
                let _ = self.global.poll_all_devices(false);
                self.last_poll = Instant::now();
            }
            if let Ok((scope_id, msg)) = self
                .receiver
                .try_recv_timeout(diff.unwrap_or(DEVICE_POLL_INTERVAL))
            {
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
                        let callback = BufferMapCallback::from_rust(Box::from(move |result| {
                            match result {
                                Ok(()) => {
                                    let global = &glob;
                                    let (slice_pointer, range_size) = gfx_select!(buffer_id =>
                                                global.buffer_get_mapped_range(buffer_id, 0, None))
                                    .unwrap();
                                    // SAFETY: guarantee to be safe from wgpu
                                    let data = unsafe {
                                        slice::from_raw_parts(slice_pointer, range_size as usize)
                                    };

                                    if let Err(e) =
                                        resp_sender.send(Some(Ok(WebGPUResponse::BufferMapAsync(
                                            IpcSharedMemory::from_bytes(data),
                                        ))))
                                    {
                                        warn!("Could not send BufferMapAsync Response ({})", e);
                                    }
                                },
                                Err(_) => {
                                    warn!("Could not map buffer({:?})", buffer_id);
                                    if let Err(e) = resp_sender
                                        .send(Some(Err(String::from("Failed to map Buffer"))))
                                    {
                                        warn!("Could not send BufferMapAsync Response ({})", e);
                                    }
                                },
                            };
                        }));

                        let operation = BufferMapOperation {
                            host: host_map,
                            callback: Some(callback),
                        };
                        let global = &self.global;
                        let result = gfx_select!(buffer_id => global.buffer_map_async(
                            buffer_id,
                            offset,
                            size,
                            operation
                        ));
                        if let Err(ref e) = result {
                            if let Err(w) = sender.send(Some(Err(format!("{:?}", e)))) {
                                warn!("Failed to send BufferMapAsync Response ({:?})", w);
                            }
                        }
                        self.send_result(device_id, scope_id, result);
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
                            tuple_to_result(
                                gfx_select!(command_encoder_id => global.command_encoder_finish(
                                    command_encoder_id,
                                    &wgt::CommandBufferDescriptor::default()
                                )),
                            )
                            .map_err(|e| format!("{:?}", e))
                        };
                        self.encoder_record_error(command_encoder_id, &result);
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
                        self.encoder_record_error(command_encoder_id, &result);
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
                        self.encoder_record_error(command_encoder_id, &result);
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
                        self.encoder_record_error(command_encoder_id, &result);
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
                        self.encoder_record_error(command_encoder_id, &result);
                    },
                    WebGPURequest::CreateBindGroup {
                        device_id,
                        bind_group_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let result = tuple_to_result(gfx_select!(bind_group_id =>
                            global.device_create_bind_group(device_id, &descriptor, Some(bind_group_id))));
                        self.send_result(device_id, scope_id, result);
                    },
                    WebGPURequest::CreateBindGroupLayout {
                        device_id,
                        bind_group_layout_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        if let Some(desc) = descriptor {
                            let result = tuple_to_result(gfx_select!(bind_group_layout_id =>
                                global.device_create_bind_group_layout(device_id, &desc, Some(bind_group_layout_id))));

                            self.send_result(device_id, scope_id, result);
                        }
                    },
                    WebGPURequest::CreateBuffer {
                        device_id,
                        buffer_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        if let Some(desc) = descriptor {
                            let result = tuple_to_result(gfx_select!(buffer_id =>
                                global.device_create_buffer(device_id, &desc, Some(buffer_id))));

                            self.send_result(device_id, scope_id, result);
                        }
                    },
                    WebGPURequest::CreateCommandEncoder {
                        device_id,
                        command_encoder_id,
                        label,
                    } => {
                        let global = &self.global;
                        let desc = wgt::CommandEncoderDescriptor { label };
                        let result = tuple_to_result(gfx_select!(command_encoder_id =>
                            global.device_create_command_encoder(device_id, &desc, Some(command_encoder_id))));

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
                            .map_or(Vec::with_capacity(0), |(_, bgls)| {
                                bgls.iter().map(|x| Some(x.to_owned())).collect()
                            });
                        let implicit =
                            implicit_ids
                                .as_ref()
                                .map(|(layout, _)| ImplicitPipelineIds {
                                    root_id: Some(*layout),
                                    group_ids: bgls.as_slice(),
                                });
                        let result = tuple_to_result(
                            gfx_select!(compute_pipeline_id => global.device_create_compute_pipeline(
                                device_id,
                                &descriptor,
                                Some(compute_pipeline_id),
                                implicit
                            )),
                        );
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
                        let result = tuple_to_result(gfx_select!(pipeline_layout_id =>
                            global.device_create_pipeline_layout(device_id, &descriptor, Some(pipeline_layout_id))));
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
                            .map_or(Vec::with_capacity(0), |(_, bgls)| {
                                bgls.iter().map(|x| Some(x.to_owned())).collect()
                            });
                        let implicit =
                            implicit_ids
                                .as_ref()
                                .map(|(layout, _)| ImplicitPipelineIds {
                                    root_id: Some(*layout),
                                    group_ids: bgls.as_slice(),
                                });
                        if let Some(desc) = descriptor {
                            let result = tuple_to_result(gfx_select!(render_pipeline_id =>
                            global.device_create_render_pipeline(
                                device_id,
                                &desc,
                                Some(render_pipeline_id),
                                implicit)
                            ));
                            self.send_result(device_id, scope_id, result);
                        }
                    },
                    WebGPURequest::CreateSampler {
                        device_id,
                        sampler_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        let result = tuple_to_result(
                            gfx_select!(sampler_id => global.device_create_sampler(
                                device_id,
                                &descriptor,
                                Some(sampler_id)
                            )),
                        );
                        self.send_result(device_id, scope_id, result);
                    },
                    WebGPURequest::CreateShaderModule {
                        device_id,
                        program_id,
                        program,
                        label,
                    } => {
                        let global = &self.global;
                        let source = wgpu_core::pipeline::ShaderModuleSource::Wgsl(
                            crate::Cow::Owned(program),
                        );
                        let desc = ShaderModuleDescriptor {
                            label: label.map(|s| s.into()),
                            shader_bound_checks: wgt::ShaderBoundChecks::default(),
                        };
                        let result = tuple_to_result(gfx_select!(program_id =>
                            global.device_create_shader_module(device_id, &desc, source, Some(program_id))));
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
                        let mut wr = self.webrender_api.lock().unwrap();
                        let image_key = wr.generate_image_key();
                        if let Err(e) = sender.send(image_key) {
                            warn!("Failed to send ImageKey ({})", e);
                        }
                        let _ = self.wgpu_image_map.lock().unwrap().insert(
                            external_id,
                            PresentationData {
                                device_id,
                                queue_id: device_id.transmute(),
                                data: vec![255; (buffer_stride * height as u32) as usize],
                                size: Size2D::new(width, height),
                                unassigned_buffer_ids: buffer_ids,
                                available_buffer_ids: ArrayVec::<
                                    id::BufferId,
                                    PRESENTATION_BUFFER_COUNT,
                                >::new(),
                                queued_buffer_ids: ArrayVec::<
                                    id::BufferId,
                                    PRESENTATION_BUFFER_COUNT,
                                >::new(),
                                buffer_stride,
                                image_key,
                                image_desc,
                                image_data: image_data.clone(),
                            },
                        );

                        let mut txn = Transaction::new();
                        txn.add_image(image_key, image_desc, image_data, None);
                        wr.send_transaction(self.webrender_document, txn);
                    },
                    WebGPURequest::CreateTexture {
                        device_id,
                        texture_id,
                        descriptor,
                    } => {
                        let global = &self.global;
                        if let Some(desc) = descriptor {
                            let result = tuple_to_result(
                                gfx_select!(texture_id => global.device_create_texture(
                                    device_id,
                                    &desc,
                                    Some(texture_id)
                                )),
                            );
                            self.send_result(device_id, scope_id, result);
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
                            let result = tuple_to_result(
                                gfx_select!(texture_view_id => global.texture_create_view(
                                    texture_id,
                                    &desc,
                                    Some(texture_view_id)
                                )),
                            );

                            self.send_result(device_id, scope_id, result);
                        }
                    },
                    WebGPURequest::DestroyBuffer(buffer) => {
                        let global = &self.global;
                        let _result = gfx_select!(buffer => global.buffer_destroy(buffer));
                    },
                    WebGPURequest::DestroyDevice(device) => {
                        let global = &self.global;
                        gfx_select!(device => global.device_destroy(device));
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
                        let mut txn = Transaction::new();
                        txn.delete_image(image_key);
                        self.webrender_api
                            .lock()
                            .unwrap()
                            .send_transaction(self.webrender_document, txn);
                    },
                    WebGPURequest::DestroyTexture {
                        device_id,
                        texture_id,
                    } => {
                        let global = &self.global;
                        let result = gfx_select!(texture_id => global.texture_destroy(texture_id));
                        self.send_result(device_id, scope_id, result);
                    },
                    WebGPURequest::Exit(sender) => {
                        if let Err(e) = sender.send(()) {
                            warn!("Failed to send response to WebGPURequest::Exit ({})", e)
                        }
                        break;
                    },
                    WebGPURequest::DropCommandBuffer(id) => {
                        self.error_command_encoders
                            .borrow_mut()
                            .remove(&id.into_command_encoder_id());
                        let global = &self.global;
                        gfx_select!(id => global.command_buffer_drop(id));
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeCommandBuffer(id)) {
                            warn!("Unable to send FreeCommandBuffer({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropDevice(device_id) => {
                        let device = WebGPUDevice(device_id);
                        let pipeline_id = self.devices.remove(&device).unwrap();
                        if let Err(e) = self.script_sender.send(WebGPUMsg::CleanDevice {
                            device,
                            pipeline_id,
                        }) {
                            warn!("Unable to send CleanDevice({:?}) ({:?})", device_id, e);
                        }
                        let global = &self.global;
                        gfx_select!(device_id => global.device_drop(device_id));
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeDevice(device_id)) {
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
                        let result = tuple_to_result(
                            gfx_select!(render_bundle_id => global.render_bundle_encoder_finish(
                                render_bundle_encoder,
                                &descriptor,
                                Some(render_bundle_id)
                            )),
                        );

                        self.send_result(device_id, scope_id, result);
                    },
                    WebGPURequest::RequestAdapter {
                        sender,
                        options,
                        ids,
                    } => {
                        let adapter_id = match self
                            .global
                            .request_adapter(&options, wgc::instance::AdapterInputs::IdSet(&ids))
                        {
                            Ok(id) => id,
                            Err(w) => {
                                if let Err(e) = sender.send(Some(Err(format!("{:?}", w)))) {
                                    warn!(
                                    "Failed to send response to WebGPURequest::RequestAdapter ({})",
                                    e
                                )
                                }
                                break;
                            },
                        };
                        let adapter = WebGPUAdapter(adapter_id);
                        self.adapters.push(adapter);
                        let global = &self.global;
                        // TODO: can we do this lazily
                        let info =
                            gfx_select!(adapter_id => global.adapter_get_info(adapter_id)).unwrap();
                        let limits =
                            gfx_select!(adapter_id => global.adapter_limits(adapter_id)).unwrap();
                        let features =
                            gfx_select!(adapter_id => global.adapter_features(adapter_id)).unwrap();
                        if let Err(e) = sender.send(Some(Ok(WebGPUResponse::RequestAdapter {
                            adapter_info: info,
                            adapter_id: adapter,
                            features,
                            limits,
                            channel: WebGPU(self.sender.clone()),
                        }))) {
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
                    } => {
                        let desc = DeviceDescriptor {
                            label: descriptor.label.as_ref().map(crate::Cow::from),
                            required_features: descriptor.required_features,
                            required_limits: descriptor.required_limits.clone(),
                        };
                        let global = &self.global;
                        let (device_id, queue_id) = match gfx_select!(device_id => global.adapter_request_device(
                            adapter_id.0,
                            &desc,
                            None,
                            Some(device_id),
                            Some(device_id.transmute()),
                        )) {
                            (_, _, Some(e)) => {
                                if let Err(w) = sender.send(Some(Err(format!("{:?}", e)))) {
                                    warn!(
                                    "Failed to send response to WebGPURequest::RequestDevice ({})",
                                    w
                                )
                                }
                                break;
                            },
                            (device_id, queue_id, None) => (device_id, queue_id),
                        };
                        let device = WebGPUDevice(device_id);
                        let queue = WebGPUQueue(queue_id);
                        self.devices.insert(device, pipeline_id);
                        if let Err(e) = sender.send(Some(Ok(WebGPUResponse::RequestDevice {
                            device_id: device,
                            queue_id: queue,
                            descriptor,
                        }))) {
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
                        self.encoder_record_error(command_encoder_id, &result);
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
                        self.encoder_record_error(command_encoder_id, &result);
                    },
                    WebGPURequest::Submit {
                        queue_id,
                        command_buffers,
                    } => {
                        let global = &self.global;
                        let cmd_id = command_buffers.iter().find(|id| {
                            self.error_command_encoders
                                .borrow()
                                .contains_key(&id.into_command_encoder_id())
                        });
                        let result = if cmd_id.is_some() {
                            Err(String::from("Invalid command buffer submitted"))
                        } else {
                            gfx_select!(queue_id => global.queue_submit(queue_id, &command_buffers))
                                .map_err(|e| format!("{:?}", e))
                        };
                        self.send_result(queue_id.transmute(), scope_id, result);
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
                                        usage: wgt::BufferUsages::MAP_READ |
                                            wgt::BufferUsages::COPY_DST,
                                        mapped_at_creation: false,
                                    };
                                    let _ = gfx_select!(b_id => global.device_create_buffer(
                                        device_id,
                                        &buffer_desc,
                                        Some(b_id)
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
                            Some(encoder_id)
                        ));

                        let buffer_cv = ImageCopyBuffer {
                            buffer: buffer_id,
                            layout: wgt::ImageDataLayout {
                                offset: 0,
                                bytes_per_row: Some(buffer_stride),
                                rows_per_image: None,
                            },
                        };
                        let texture_cv = ImageCopyTexture {
                            texture: texture_id,
                            mip_level: 0,
                            origin: wgt::Origin3d::ZERO,
                            aspect: wgt::TextureAspect::All,
                        };
                        let copy_size = wgt::Extent3d {
                            width: size.width as u32,
                            height: size.height as u32,
                            depth_or_array_layers: 1,
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
                            &[encoder_id.into_command_buffer_id()]
                        ));

                        let glob = Arc::clone(&self.global);
                        let wgpu_image_map = Arc::clone(&self.wgpu_image_map);
                        let webrender_api = Arc::clone(&self.webrender_api);
                        let webrender_document = self.webrender_document;
                        let callback = BufferMapCallback::from_rust(Box::from(move |result| {
                            match result {
                                Ok(()) => {
                                    let global = &glob;
                                    let (slice_pointer, range_size) = gfx_select!(buffer_id =>
                                        global.buffer_get_mapped_range(buffer_id, 0, Some(buffer_size as u64)))
                                    .unwrap();
                                    let data = unsafe {
                                        slice::from_raw_parts(slice_pointer, range_size as usize)
                                    }
                                    .to_vec();
                                    if let Some(present_data) =
                                        wgpu_image_map.lock().unwrap().get_mut(&external_id)
                                    {
                                        present_data.data = data;
                                        let mut txn = Transaction::new();
                                        txn.update_image(
                                            present_data.image_key,
                                            present_data.image_desc,
                                            present_data.image_data.clone(),
                                            &DirtyRect::All,
                                        );
                                        webrender_api
                                            .lock()
                                            .unwrap()
                                            .send_transaction(webrender_document, txn);
                                        present_data
                                            .queued_buffer_ids
                                            .retain(|b_id| *b_id != buffer_id);
                                        present_data.available_buffer_ids.push(buffer_id);
                                    } else {
                                        warn!(
                                            "Data not found for ExternalImageId({:?})",
                                            external_id
                                        );
                                    }
                                    let _ =
                                        gfx_select!(buffer_id => global.buffer_unmap(buffer_id));
                                },
                                _ => error!("Could not map buffer({:?})", buffer_id),
                            }
                        }));
                        let map_op = BufferMapOperation {
                            host: HostMap::Read,
                            callback: Some(callback),
                        };
                        let _ = gfx_select!(buffer_id
                            => global.buffer_map_async(buffer_id, 0, Some(buffer_size), map_op));
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
                            let (slice_pointer, range_size) =
                                gfx_select!(buffer_id => global.buffer_get_mapped_range(
                                    buffer_id,
                                    offset,
                                    Some(size)
                                ))
                                .unwrap();
                            unsafe {
                                slice::from_raw_parts_mut(slice_pointer, range_size as usize)
                            }
                            .copy_from_slice(&array_buffer);
                        }
                        let result = gfx_select!(buffer_id => global.buffer_unmap(buffer_id));
                        self.send_result(device_id, scope_id, result);
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
                        self.send_result(queue_id.transmute(), scope_id, result);
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
                        self.send_result(queue_id.transmute(), scope_id, result);
                    },
                    WebGPURequest::QueueOnSubmittedWorkDone { sender, queue_id } => {
                        let global = &self.global;

                        let callback = SubmittedWorkDoneClosure::from_rust(Box::from(move || {
                            if let Err(e) = sender.send(Some(Ok(WebGPUResponse::SubmittedWorkDone)))
                            {
                                warn!("Could not send SubmittedWorkDone Response ({})", e);
                            }
                        }));
                        let result = gfx_select!(queue_id => global.queue_on_submitted_work_done(queue_id, callback));
                        self.send_result(queue_id.transmute(), scope_id, result);
                    },
                    WebGPURequest::DropTexture(id) => {
                        let global = &self.global;
                        gfx_select!(id => global.texture_drop(id, true));
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeTexture(id)) {
                            warn!("Unable to send FreeTexture({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropAdapter(id) => {
                        let global = &self.global;
                        gfx_select!(id => global.adapter_drop(id));
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeAdapter(id)) {
                            warn!("Unable to send FreeAdapter({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropBuffer(id) => {
                        let global = &self.global;
                        gfx_select!(id => global.buffer_drop(id, true));
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeBuffer(id)) {
                            warn!("Unable to send FreeBuffer({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropPipelineLayout(id) => {
                        let global = &self.global;
                        gfx_select!(id => global.pipeline_layout_drop(id));
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreePipelineLayout(id)) {
                            warn!("Unable to send FreePipelineLayout({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropComputePipeline(id) => {
                        let global = &self.global;
                        gfx_select!(id => global.compute_pipeline_drop(id));
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeComputePipeline(id))
                        {
                            warn!("Unable to send FreeComputePipeline({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropRenderPipeline(id) => {
                        let global = &self.global;
                        gfx_select!(id => global.render_pipeline_drop(id));
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeRenderPipeline(id)) {
                            warn!("Unable to send FreeRenderPipeline({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropBindGroup(id) => {
                        let global = &self.global;
                        gfx_select!(id => global.bind_group_drop(id));
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeBindGroup(id)) {
                            warn!("Unable to send FreeBindGroup({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropBindGroupLayout(id) => {
                        let global = &self.global;
                        gfx_select!(id => global.bind_group_layout_drop(id));
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeBindGroupLayout(id))
                        {
                            warn!("Unable to send FreeBindGroupLayout({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropTextureView(id) => {
                        let global = &self.global;
                        let _result = gfx_select!(id => global.texture_view_drop(id, true));
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeTextureView(id)) {
                            warn!("Unable to send FreeTextureView({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropSampler(id) => {
                        let global = &self.global;
                        gfx_select!(id => global.sampler_drop(id));
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeSampler(id)) {
                            warn!("Unable to send FreeSampler({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropShaderModule(id) => {
                        let global = &self.global;
                        gfx_select!(id => global.shader_module_drop(id));
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeShaderModule(id)) {
                            warn!("Unable to send FreeShaderModule({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropRenderBundle(id) => {
                        let global = &self.global;
                        gfx_select!(id => global.render_bundle_drop(id));
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeRenderBundle(id)) {
                            warn!("Unable to send FreeRenderBundle({:?}) ({:?})", id, e);
                        };
                    },
                    WebGPURequest::DropQuerySet(id) => {
                        let global = &self.global;
                        gfx_select!(id => global.query_set_drop(id));
                        if let Err(e) = self.script_sender.send(WebGPUMsg::FreeQuerySet(id)) {
                            warn!("Unable to send FreeQuerySet({:?}) ({:?})", id, e);
                        };
                    },
                }
            }
        }
        if let Err(e) = self.script_sender.send(WebGPUMsg::Exit) {
            warn!("Failed to send WebGPUMsg::Exit to script ({})", e);
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
        result: &Result<U, T>,
    ) {
        if let Err(ref e) = result {
            self.error_command_encoders
                .borrow_mut()
                .entry(encoder_id)
                .or_insert_with(|| format!("{:?}", e));
        }
    }
}

fn tuple_to_result<T, E>(res: (T, Option<E>)) -> Result<T, E> {
    if let Some(err) = res.1 {
        Err(err)
    } else {
        Ok(res.0)
    }
}
