/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#![deny(unsafe_code)]

extern crate gfx;
extern crate ipc_channel;
extern crate msg;
extern crate net_traits;
extern crate profile_traits;
extern crate script_traits;
extern crate serde;
extern crate url;
extern crate util;
extern crate webrender_traits;

// This module contains traits in layout used generically
//   in the rest of Servo.
// The traits are here instead of in layout so
//   that these modules won't have to depend on layout.

use gfx::font_cache_thread::FontCacheThread;
use gfx::paint_thread::LayoutToPaintMsg;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use msg::constellation_msg::{ConstellationChan, Failure, PipelineId};
use net_traits::image_cache_thread::ImageCacheThread;
use profile_traits::{mem, time};
use script_traits::LayoutMsg as ConstellationMsg;
use script_traits::{LayoutControlMsg, ConstellationControlMsg, OpaqueScriptLayoutChannel};
use url::Url;
use util::ipc::OptionalIpcSender;

/// A channel wrapper for constellation messages
#[derive(Clone, Deserialize, Serialize)]
pub struct LayoutControlChan(pub IpcSender<LayoutControlMsg>);

// A static method creating a layout thread
// Here to remove the compositor -> layout dependency
pub trait LayoutThreadFactory {
    // FIXME: use a proper static method
    fn create(_phantom: Option<&mut Self>,
              id: PipelineId,
              url: Url,
              is_iframe: bool,
              chan: OpaqueScriptLayoutChannel,
              pipeline_port: IpcReceiver<LayoutControlMsg>,
              constellation_chan: ConstellationChan<ConstellationMsg>,
              failure_msg: Failure,
              script_chan: IpcSender<ConstellationControlMsg>,
              layout_to_paint_chan: OptionalIpcSender<LayoutToPaintMsg>,
              image_cache_thread: ImageCacheThread,
              font_cache_thread: FontCacheThread,
              time_profiler_chan: time::ProfilerChan,
              mem_profiler_chan: mem::ProfilerChan,
              shutdown_chan: IpcSender<()>,
              content_process_shutdown_chan: IpcSender<()>,
              webrender_api_sender: Option<webrender_traits::RenderApiSender>);
}
