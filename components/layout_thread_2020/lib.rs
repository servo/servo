/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crossbeam_channel::{select, Receiver, Sender};
use euclid::TypedSize2D;
use gfx::font_cache_thread::FontCacheThread;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use log::{debug, error};
use metrics::{PaintTimeMetrics, ProgressiveWebMetric};
use msg::constellation_msg::{BackgroundHangMonitor, BackgroundHangMonitorRegister};
use msg::constellation_msg::{HangAnnotation, LayoutHangAnnotation};
use msg::constellation_msg::{MonitoredComponentId, MonitoredComponentType};
use msg::constellation_msg::{PipelineId, TopLevelBrowsingContextId};
use net_traits::image_cache::ImageCache;
use profile_traits::{mem, time};
use script_layout_interface::message::Msg;
use script_traits::LayoutMsg as ConstellationMsg;
use script_traits::{ConstellationControlMsg, LayoutControlMsg};
use servo_geometry::DeviceIndependentPixel;
use servo_url::ServoUrl;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use style::thread_state::{self, ThreadState};

pub struct LayoutThread {
    /// The receiver on which we receive messages from the script thread.
    script_receiver: Receiver<Msg>,

    /// The receiver on which we receive messages from the constellation.
    pipeline_receiver: Receiver<LayoutControlMsg>,

    /// Flag that indicates if LayoutThread is busy handling a request.
    busy: Arc<AtomicBool>,

    paint_time_metrics: PaintTimeMetrics,
    background_hang_monitor: Box<dyn BackgroundHangMonitor>,
}

impl layout_traits::LayoutThreadFactory for LayoutThread {
    type Message = Msg;

    fn create(
        id: PipelineId,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        _url: ServoUrl,
        _is_iframe: bool,
        (script_sender, script_receiver): (Sender<Self::Message>, Receiver<Self::Message>),
        pipeline_port: IpcReceiver<LayoutControlMsg>,
        background_hang_monitor_register: Box<dyn BackgroundHangMonitorRegister>,
        _constellation_chan: IpcSender<ConstellationMsg>,
        _script_chan: IpcSender<ConstellationControlMsg>,
        _image_cache: Arc<dyn ImageCache>,
        _font_cache_thread: FontCacheThread,
        _time_profiler_chan: time::ProfilerChan,
        mem_profiler_chan: mem::ProfilerChan,
        content_process_shutdown_chan: Option<IpcSender<()>>,
        _webrender_api_sender: webrender_api::RenderApiSender,
        _webrender_document: webrender_api::DocumentId,
        paint_time_metrics: PaintTimeMetrics,
        busy: Arc<AtomicBool>,
        _load_webfonts_synchronously: bool,
        _initial_window_size: TypedSize2D<u32, DeviceIndependentPixel>,
        _device_pixels_per_px: Option<f32>,
        _dump_display_list: bool,
        _dump_display_list_json: bool,
        _dump_style_tree: bool,
        _dump_rule_tree: bool,
        _relayout_event: bool,
        _nonincremental_layout: bool,
        _trace_layout: bool,
        _dump_flow_tree: bool,
    ) {
        thread::Builder::new()
            .name(format!("LayoutThread {:?}", id))
            .spawn(move || {
                thread_state::initialize(ThreadState::LAYOUT);

                // In order to get accurate crash reports, we install the top-level bc id.
                TopLevelBrowsingContextId::install(top_level_browsing_context_id);

                let background_hang_monitor = background_hang_monitor_register.register_component(
                    MonitoredComponentId(id, MonitoredComponentType::Layout),
                    Duration::from_millis(1000),
                    Duration::from_millis(5000),
                );

                // Proxy IPC messages from the pipeline to the layout thread.
                let pipeline_receiver =
                    ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(pipeline_port);

                let mut layout = LayoutThread {
                    script_receiver,
                    pipeline_receiver,
                    paint_time_metrics,
                    background_hang_monitor,
                    busy,
                };

                let reporter_name = format!("layout-reporter-{}", id);
                mem_profiler_chan.run_with_memory_reporting(
                    || while layout.wait_for_message() {},
                    reporter_name,
                    script_sender,
                    Msg::CollectReports,
                );
                if let Some(content_process_shutdown_chan) = content_process_shutdown_chan {
                    let _ = content_process_shutdown_chan.send(());
                }
            })
            .expect("Thread spawning failed");
    }
}

impl LayoutThread {
    fn wait_for_message(&mut self) -> bool {
        // Notify the background-hang-monitor we are waiting for an event.
        self.background_hang_monitor.notify_wait();

        let continue_ = select! {
            recv(self.pipeline_receiver) -> msg => {
                let msg = msg.unwrap();
                self.busy.store(true, Ordering::Relaxed);
                self.handle_pipeline_message(msg)
            }
            recv(self.script_receiver) -> msg => {
                let msg = msg.unwrap();
                self.busy.store(true, Ordering::Relaxed);
                self.handle_message(msg)
            }
        };
        self.busy.store(false, Ordering::Relaxed);
        continue_
    }

    fn handle_pipeline_message(&mut self, msg: LayoutControlMsg) -> bool {
        match msg {
            LayoutControlMsg::SetScrollStates(new_scroll_states) => {
                self.handle_message(Msg::SetScrollStates(new_scroll_states))
            },
            LayoutControlMsg::TickAnimations => self.handle_message(Msg::TickAnimations),
            LayoutControlMsg::GetCurrentEpoch(sender) => {
                self.handle_message(Msg::GetCurrentEpoch(sender))
            },
            LayoutControlMsg::GetWebFontLoadState(sender) => {
                self.handle_message(Msg::GetWebFontLoadState(sender))
            },
            LayoutControlMsg::ExitNow => self.handle_message(Msg::ExitNow),
            LayoutControlMsg::PaintMetric(epoch, paint_time) => {
                self.paint_time_metrics.maybe_set_metric(epoch, paint_time);
                true
            },
        }
    }

    fn handle_message(&mut self, msg: Msg) -> bool {
        #![allow(unused, unreachable_code)]

        self.notify_activity_to_hang_monitor(&msg);

        match msg {
            Msg::AddStylesheet(stylesheet, before_stylesheet) => {
                error!("unhandled: Msg::AddStylesheet")
            },
            Msg::RemoveStylesheet(stylesheet) => error!("unhandled: Msg::RemoveStylesheet"),
            Msg::SetQuirksMode(mode) => error!("unhandled: Msg::SetQuirksMode"),
            Msg::GetRPC(response_chan) => error!("unhandled: Msg::GetRPC"),
            Msg::Reflow(data) => error!("unhandled: Msg::Reflow"),
            Msg::TickAnimations => error!("unhandled: Msg::TickAnimations"),
            Msg::SetScrollStates(new_scroll_states) => {
                error!("unhandled: Msg::SetScrollStates")
            },
            Msg::UpdateScrollStateFromScript(state) => {
                error!("unhandled: Msg::UpdateScrollStateFromScript")
            },
            Msg::ReapStyleAndLayoutData(dead_data) => {
                error!("unhandled: Msg::ReapStyleAndLayoutData")
            },
            Msg::CollectReports(reports_chan) => error!("unhandled: Msg::CollectReports"),
            Msg::GetCurrentEpoch(sender) => error!("unhandled: Msg::GetCurrentEpoch"),
            Msg::AdvanceClockMs(how_many, do_tick) => error!("unhandled: Msg::AdvanceClockMs"),
            Msg::GetWebFontLoadState(sender) => error!("unhandled: Msg::GetWebFontLoadState"),
            Msg::CreateLayoutThread(info) => error!("unhandled: Msg::CreateLayoutThread"),
            Msg::SetFinalUrl(final_url) => error!("unhandled: Msg::SetFinalUrl"),
            Msg::RegisterPaint(name, properties, painter) => {
                error!("unhandled: Msg::RegisterPaint")
            },
            Msg::PrepareToExit(response_chan) => {
                error!("unhandled: Msg::PrepareToExit");
                return false;
            },
            // Receiving the Exit message at this stage only happens when layout is undergoing a "force exit".
            Msg::ExitNow => {
                debug!("layout: ExitNow received");
                error!("unhandled: Msg::ExitNow");
                return false;
            },
            Msg::SetNavigationStart(time) => self.paint_time_metrics.set_navigation_start(time),
            Msg::GetRunningAnimations(sender) => error!("unhandled: Msg::GetRunningAnimations"),
        }

        true
    }

    fn notify_activity_to_hang_monitor(&self, request: &Msg) {
        let hang_annotation = match request {
            Msg::AddStylesheet(..) => LayoutHangAnnotation::AddStylesheet,
            Msg::RemoveStylesheet(..) => LayoutHangAnnotation::RemoveStylesheet,
            Msg::SetQuirksMode(..) => LayoutHangAnnotation::SetQuirksMode,
            Msg::Reflow(..) => LayoutHangAnnotation::Reflow,
            Msg::GetRPC(..) => LayoutHangAnnotation::GetRPC,
            Msg::TickAnimations => LayoutHangAnnotation::TickAnimations,
            Msg::AdvanceClockMs(..) => LayoutHangAnnotation::AdvanceClockMs,
            Msg::ReapStyleAndLayoutData(..) => LayoutHangAnnotation::ReapStyleAndLayoutData,
            Msg::CollectReports(..) => LayoutHangAnnotation::CollectReports,
            Msg::PrepareToExit(..) => LayoutHangAnnotation::PrepareToExit,
            Msg::ExitNow => LayoutHangAnnotation::ExitNow,
            Msg::GetCurrentEpoch(..) => LayoutHangAnnotation::GetCurrentEpoch,
            Msg::GetWebFontLoadState(..) => LayoutHangAnnotation::GetWebFontLoadState,
            Msg::CreateLayoutThread(..) => LayoutHangAnnotation::CreateLayoutThread,
            Msg::SetFinalUrl(..) => LayoutHangAnnotation::SetFinalUrl,
            Msg::SetScrollStates(..) => LayoutHangAnnotation::SetScrollStates,
            Msg::UpdateScrollStateFromScript(..) => {
                LayoutHangAnnotation::UpdateScrollStateFromScript
            },
            Msg::RegisterPaint(..) => LayoutHangAnnotation::RegisterPaint,
            Msg::SetNavigationStart(..) => LayoutHangAnnotation::SetNavigationStart,
            Msg::GetRunningAnimations(..) => LayoutHangAnnotation::GetRunningAnimations,
        };
        self.background_hang_monitor
            .notify_activity(HangAnnotation::Layout(hang_annotation));
    }
}
