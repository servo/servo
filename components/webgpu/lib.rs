/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel::{self, GenericReceiver};
use canvas_context::WebGpuExternalImageMap;
pub use canvas_context::{ContextData, WebGpuExternalImages};
use log::warn;
use webgpu_traits::{WebGPU, WebGPUMsg};
use wgpu_thread::WGPU;
pub use {wgpu_core as wgc, wgpu_types as wgt};

mod poll_thread;
mod wgpu_thread;

use std::borrow::Cow;

use paint_api::{CrossProcessPaintApi, WebRenderExternalImageIdManager};
use servo_config::pref;

pub mod canvas_context;

pub fn start_webgpu_thread(
    paint_api: CrossProcessPaintApi,
    webrender_external_image_id_manager: WebRenderExternalImageIdManager,
    wgpu_image_map: WebGpuExternalImageMap,
) -> Option<(WebGPU, GenericReceiver<WebGPUMsg>)> {
    if !pref!(dom_webgpu_enabled) {
        return None;
    }
    let (sender, receiver) = match generic_channel::channel() {
        Some(sender_and_receiver) => sender_and_receiver,
        None => {
            warn!("Failed to create sender and receiver for WGPU thread",);
            return None;
        },
    };
    let sender_clone = sender.clone();

    let (script_sender, script_recv) = match generic_channel::channel() {
        Some(sender_and_receiver) => sender_and_receiver,
        None => {
            warn!("Failed to create receiver and sender for WGPU thread",);
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
                paint_api,
                webrender_external_image_id_manager,
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
