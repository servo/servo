/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate euclid;
extern crate gfx;
extern crate ipc_channel;
extern crate script_traits;
extern crate msg;
extern crate profile_traits;
extern crate net_traits;
extern crate serde;
extern crate url;
extern crate util;

// This module contains traits in layout used generically
//   in the rest of Servo.
// The traits are here instead of in layout so
//   that these modules won't have to depend on layout.

use gfx::font_cache_task::FontCacheTask;
use gfx::paint_task::LayoutToPaintMsg;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use msg::constellation_msg::{ConstellationChan, Failure, PipelineId};
use net_traits::image_cache_task::ImageCacheTask;
use profile_traits::mem;
use profile_traits::time;
use script_traits::{LayoutControlMsg, ConstellationControlMsg, OpaqueScriptLayoutChannel};
use std::sync::mpsc::Sender;
use url::Url;
use util::ipc::OptionalIpcSender;

/// A channel wrapper for constellation messages
#[derive(Clone, Deserialize, Serialize)]
pub struct LayoutControlChan(pub IpcSender<LayoutControlMsg>);

// A static method creating a layout task
// Here to remove the compositor -> layout dependency
pub trait LayoutTaskFactory {
    // FIXME: use a proper static method
    fn create(_phantom: Option<&mut Self>,
              id: PipelineId,
              url: Url,
              is_iframe: bool,
              chan: OpaqueScriptLayoutChannel,
              pipeline_port: IpcReceiver<LayoutControlMsg>,
              constellation_chan: ConstellationChan,
              failure_msg: Failure,
              script_chan: Sender<ConstellationControlMsg>,
              layout_to_paint_chan: OptionalIpcSender<LayoutToPaintMsg>,
              image_cache_task: ImageCacheTask,
              font_cache_task: FontCacheTask,
              time_profiler_chan: time::ProfilerChan,
              mem_profiler_chan: mem::ProfilerChan,
              shutdown_chan: Sender<()>);
}
