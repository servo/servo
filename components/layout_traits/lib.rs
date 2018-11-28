/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

// This module contains traits in layout used generically
//   in the rest of Servo.
// The traits are here instead of in layout so
//   that these modules won't have to depend on layout.

use crossbeam_channel::{Receiver, Sender};
use gfx::font_cache_thread::FontCacheThread;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use metrics::PaintTimeMetrics;
use msg::constellation_msg::PipelineId;
use msg::constellation_msg::TopLevelBrowsingContextId;
use net_traits::image_cache::ImageCache;
use profile_traits::mem::ProfilerChan as MemProfilerSender;
use profile_traits::time::ProfilerChan as TimeProfilerSender;
use script_layout_interface::message::Msg;
use script_traits::LayoutMsg as ConstellationMsg;
use script_traits::{ConstellationControlMsg, LayoutControlMsg, LayoutPerThreadInfo};
use servo_url::ServoUrl;
use std::sync::Arc;

// A static method creating a layout thread
// Here to remove the compositor -> layout dependency
pub trait LayoutThreadFactory {
    type Message;
    fn create(thread_info: LayoutPerThreadInfo<Self::Message, PaintTimeMetrics>,
              global_info: LayoutGlobalInfo);
}

#[derive(Clone)]
pub struct LayoutGlobalInfo {
    pub top_level_context_id: TopLevelBrowsingContextId,
    pub font_cache_thread: FontCacheThread,
    pub time_profiler_sender: TimeProfilerSender,
    pub mem_profiler_sender: MemProfilerSender,
    pub webrender_api_sender: webrender_api::RenderApiSender,
    pub webrender_document: webrender_api::DocumentId,
}
