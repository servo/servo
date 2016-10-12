/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

extern crate gfx;
extern crate ipc_channel;
extern crate msg;
extern crate net_traits;
extern crate profile_traits;
extern crate script_traits;
extern crate url;
extern crate webrender_traits;

// This module contains traits in layout used generically
//   in the rest of Servo.
// The traits are here instead of in layout so
//   that these modules won't have to depend on layout.

use gfx::font_cache_thread::FontCacheThread;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use msg::constellation_msg::PipelineId;
use net_traits::image_cache_thread::ImageCacheThread;
use profile_traits::{mem, time};
use script_traits::{ConstellationControlMsg, LayoutControlMsg};
use script_traits::LayoutMsg as ConstellationMsg;
use std::sync::mpsc::{Receiver, Sender};
use url::Url;

// A static method creating a layout thread
// Here to remove the compositor -> layout dependency
pub trait LayoutThreadFactory {
    type Message;
    fn create(id: PipelineId,
              url: Url,
              is_iframe: bool,
              chan: (Sender<Self::Message>, Receiver<Self::Message>),
              pipeline_port: IpcReceiver<LayoutControlMsg>,
              constellation_chan: IpcSender<ConstellationMsg>,
              script_chan: IpcSender<ConstellationControlMsg>,
              image_cache_thread: ImageCacheThread,
              font_cache_thread: FontCacheThread,
              time_profiler_chan: time::ProfilerChan,
              mem_profiler_chan: mem::ProfilerChan,
              content_process_shutdown_chan: IpcSender<()>,
              webrender_api_sender: webrender_traits::RenderApiSender,
              layout_threads: usize);
}
