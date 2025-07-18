/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use log::warn;
use swapchain::WGPUImageMap;
pub use swapchain::{ContextData, WGPUExternalImages};
use webgpu_traits::{WebGPU, WebGPUMsg};
use wgpu_thread::WGPU;
pub use {wgpu_core as wgc, wgpu_types as wgt};

mod poll_thread;
mod wgpu_thread;

use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use compositing_traits::{CrossProcessCompositorApi, WebrenderExternalImageRegistry};
use ipc_channel::ipc::{self, IpcReceiver};
use servo_config::pref;

pub mod swapchain;

pub fn start_webgpu_thread(
    compositor_api: CrossProcessCompositorApi,
    external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
    wgpu_image_map: WGPUImageMap,
) -> Option<(WebGPU, IpcReceiver<WebGPUMsg>)> {
    if !pref!(dom_webgpu_enabled) {
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
                compositor_api,
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
