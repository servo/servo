/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crossbeam_channel::{Receiver, Sender};
use euclid::Size2D;
use gfx::font_cache_thread::FontCacheThread;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use metrics::PaintTimeMetrics;
use msg::constellation_msg::TopLevelBrowsingContextId;
use msg::constellation_msg::{BackgroundHangMonitorRegister, PipelineId};
use net_traits::image_cache::ImageCache;
use profile_traits::{mem, time};
use script_traits::LayoutMsg as ConstellationMsg;
use script_traits::{ConstellationControlMsg, LayoutControlMsg};
use servo_geometry::DeviceIndependentPixel;
use servo_url::ServoUrl;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct LayoutThread;

impl layout_traits::LayoutThreadFactory for LayoutThread {
    type Message = script_layout_interface::message::Msg;

    #[allow(unused)]
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
        content_process_shutdown_chan: Option<IpcSender<()>>,
        webrender_api_sender: webrender_api::RenderApiSender,
        webrender_document: webrender_api::DocumentId,
        paint_time_metrics: PaintTimeMetrics,
        busy: Arc<AtomicBool>,
        load_webfonts_synchronously: bool,
        initial_window_size: Size2D<u32, DeviceIndependentPixel>,
        device_pixels_per_px: Option<f32>,
        dump_display_list: bool,
        dump_display_list_json: bool,
        dump_style_tree: bool,
        dump_rule_tree: bool,
        relayout_event: bool,
        nonincremental_layout: bool,
        trace_layout: bool,
        dump_flow_tree: bool,
    ) {
    }
}
