/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use log::warn;
use webrender::RenderApiSender;
use wgpu_thread::WGPU;
pub use {wgpu_core as wgc, wgpu_types as wgt};

pub mod identity;
mod poll_thread;
mod wgpu_thread;

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use arrayvec::ArrayVec;
use euclid::default::Size2D;
pub use gpu_error::{Error, ErrorFilter, PopError};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
pub use render_commands::RenderCommand;
use serde::{Deserialize, Serialize};
use servo_config::pref;
use webrender_api::{DocumentId, ImageData, ImageDescriptor, ImageKey};
use webrender_traits::{
    WebrenderExternalImageApi, WebrenderExternalImageRegistry, WebrenderImageSource,
};
use wgc::id;

mod gpu_error;
mod ipc_messages;
mod render_commands;
pub use identity::*;
pub use ipc_messages::recv::*;
pub use ipc_messages::to_dom::*;
pub use ipc_messages::to_script::*;
pub use wgpu_thread::PRESENTATION_BUFFER_COUNT;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebGPU(pub IpcSender<WebGPURequest>);

impl WebGPU {
    pub fn new(
        webrender_api_sender: RenderApiSender,
        webrender_document: DocumentId,
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
            warn!("Failed to spawn WGPU thread ({})", e);
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

#[derive(Default)]
pub struct WGPUExternalImages {
    pub images: Arc<Mutex<HashMap<u64, PresentationData>>>,
    pub locked_ids: HashMap<u64, Vec<u8>>,
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
    unassigned_buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    available_buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    queued_buffer_ids: ArrayVec<id::BufferId, PRESENTATION_BUFFER_COUNT>,
    buffer_stride: u32,
    image_key: ImageKey,
    image_desc: ImageDescriptor,
    image_data: ImageData,
}

pub trait Transmute<U: id::Marker> {
    fn transmute(self) -> id::Id<U>;
}

impl Transmute<id::markers::Queue> for id::Id<id::markers::Device> {
    fn transmute(self) -> id::Id<id::markers::Queue> {
        // if this is removed next one should be removed too.
        self.into_queue_id()
    }
}

impl Transmute<id::markers::Device> for id::Id<id::markers::Queue> {
    fn transmute(self) -> id::Id<id::markers::Device> {
        // SAFETY: This is safe because queue_id = device_id in wgpu
        unsafe { id::Id::from_raw(self.into_raw()) }
    }
}
