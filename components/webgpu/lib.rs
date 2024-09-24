/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use log::warn;
use swapchain::WGPUImageMap;
pub use swapchain::{ContextData, WGPUExternalImages};
use webrender::RenderApiSender;
use wgpu_thread::WGPU;
pub use {wgpu_core as wgc, wgpu_types as wgt};

pub mod identity;
mod poll_thread;
mod wgpu_thread;

use std::borrow::Cow;
use std::sync::{Arc, Mutex};

pub use gpu_error::{Error, ErrorFilter, PopError};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
pub use render_commands::RenderCommand;
use serde::{Deserialize, Serialize};
use servo_config::pref;
use webrender_api::DocumentId;
use webrender_traits::WebrenderExternalImageRegistry;

mod gpu_error;
mod ipc_messages;
mod render_commands;
pub mod swapchain;
pub use identity::*;
pub use ipc_messages::recv::*;
pub use ipc_messages::to_dom::*;
pub use ipc_messages::to_script::*;
pub use swapchain::PRESENTATION_BUFFER_COUNT;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebGPU(pub IpcSender<WebGPURequest>);

impl WebGPU {
    pub fn new(
        webrender_api_sender: RenderApiSender,
        webrender_document: DocumentId,
        external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
        wgpu_image_map: WGPUImageMap,
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
