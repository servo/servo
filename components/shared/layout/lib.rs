/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

// This module contains traits in layout used generically
//   in the rest of Servo.
// The traits are here instead of in layout so
//   that these modules won't have to depend on layout.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use gfx::font_cache_thread::FontCacheThread;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use metrics::PaintTimeMetrics;
use msg::constellation_msg::{
    BackgroundHangMonitorRegister, PipelineId, TopLevelBrowsingContextId,
};
use net_traits::image_cache::ImageCache;
use profile_traits::{mem, time};
use script_traits::{
    ConstellationControlMsg, LayoutControlMsg, LayoutMsg as ConstellationMsg, WebrenderIpcSender,
    WindowSizeData,
};
use servo_url::ServoUrl;

// A static method creating a layout thread
// Here to remove the compositor -> layout dependency
pub trait LayoutThreadFactory {
    type Message;
    fn create(
        id: PipelineId,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        url: ServoUrl,
        is_iframe: bool,
        chan: (Sender<Self::Message>, Receiver<Self::Message>),
        pipeline_port: IpcReceiver<LayoutControlMsg>,
        background_hang_monitor: Box<dyn BackgroundHangMonitorRegister>,
        constellation_chan: IpcSender<ConstellationMsg>,
        script_chan: IpcSender<ConstellationControlMsg>,
        image_cache: Arc<dyn ImageCache>,
        font_cache_thread: FontCacheThread,
        time_profiler_chan: time::ProfilerChan,
        mem_profiler_chan: mem::ProfilerChan,
        webrender_api_sender: WebrenderIpcSender,
        paint_time_metrics: PaintTimeMetrics,
        busy: Arc<AtomicBool>,
        window_size: WindowSizeData,
    );
}
