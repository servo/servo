/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate gfx;
extern crate gfx_traits;
extern crate ipc_channel;
#[macro_use]
extern crate log;
extern crate malloc_size_of;
#[macro_use]
extern crate malloc_size_of_derive;
extern crate msg;
extern crate profile_traits;
extern crate script_traits;
extern crate servo_config;
extern crate time;

use gfx::display_list::{DisplayItem, DisplayList};
use gfx_traits::Epoch;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use profile_traits::time::{ProfilerChan, ProfilerCategory, send_profile_data};
use profile_traits::time::TimerMetadata;
use script_traits::{ConstellationControlMsg, LayoutMsg, ProgressiveWebMetricType};
use servo_config::opts;
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::HashMap;
use time::precise_time_ns;

pub trait ProfilerMetadataFactory {
    fn new_metadata(&self) -> Option<TimerMetadata>;
}

pub trait ProgressiveWebMetric {
    fn get_navigation_start(&self) -> Option<f64>;
    fn set_navigation_start(&mut self, time: f64);
    fn get_time_profiler_chan(&self) -> &ProfilerChan;
    fn send_queued_constellation_msg(&self, name: ProgressiveWebMetricType, time: f64);
}

/// maximum task time is 50ms (in ns)
pub const MAX_TASK_NS: u64 = 50000000;
/// 10 second window (in ns)
const INTERACTIVE_WINDOW_SECONDS_IN_NS: u64 = 10000000000;


fn set_metric<U: ProgressiveWebMetric>(
    pwm: &U,
    metadata: Option<TimerMetadata>,
    metric_type: ProgressiveWebMetricType,
    category: ProfilerCategory,
    attr: &Cell<Option<f64>>,
    metric_time: Option<f64>)
{
    let navigation_start = match pwm.get_navigation_start() {
        Some(time) => time,
        None => {
            warn!("Trying to set metric before navigation start");
            return;
        }
    };
    let now = match metric_time {
        Some(time) => time,
        None => precise_time_ns() as f64,
    };
    let time = now - navigation_start;
    attr.set(Some(time));

    // Queue performance observer notification.
    pwm.send_queued_constellation_msg(metric_type, time);

    // Send the metric to the time profiler.
    send_profile_data(
        category,
        metadata,
        &pwm.get_time_profiler_chan(),
        time as u64,
        time as u64,
        0,
        0,
    );

    // Print the metric to console if the print-pwm option was given.
    if opts::get().print_pwm {
        println!("{:?} {:?}", metric_type, time);
    }

}

// https://github.com/GoogleChrome/lighthouse/issues/27
// we can look at three different metrics here:
// navigation start -> visually ready (dom content loaded)
// navigation start -> thread ready (main thread available)
// visually ready -> thread ready
#[derive(MallocSizeOf)]
pub struct InteractiveMetrics {
    /// when we navigated to the page
    navigation_start: Option<f64>,
    /// indicates if the page is visually ready
    dom_content_loaded: Cell<Option<f64>>,
    /// main thread is available -- there's been a 10s window with no tasks longer than 50ms
    main_thread_available: Cell<Option<f64>>,
    // max(main_thread_available, dom_content_loaded)
    time_to_interactive: Cell<Option<f64>>,
    #[ignore_malloc_size_of = "can't measure channels"]
    time_profiler_chan: ProfilerChan,
}

#[derive(Clone, Copy, Debug, MallocSizeOf)]
pub struct InteractiveWindow {
    start: u64,
}


impl InteractiveWindow {
    pub fn new() -> InteractiveWindow {
        InteractiveWindow {
            start: precise_time_ns(),
        }
    }

    // We need to either start or restart the 10s window
    //   start: we've added a new document
    //   restart: there was a task > 50ms
    //   not all documents are interactive
    pub fn start_window(&mut self) {
        self.start = precise_time_ns();
    }

    /// check if 10s has elapsed since start
    pub fn needs_check(&self) -> bool {
        precise_time_ns() - self.start >= INTERACTIVE_WINDOW_SECONDS_IN_NS
    }

    pub fn get_start(&self) -> u64 {
        self.start
    }
}

#[derive(Debug)]
pub enum InteractiveFlag {
    DOMContentLoaded,
    TimeToInteractive(f64),
}

impl InteractiveMetrics {
    pub fn new(time_profiler_chan: ProfilerChan) -> InteractiveMetrics {
        InteractiveMetrics {
            navigation_start: None,
            dom_content_loaded: Cell::new(None),
            main_thread_available: Cell::new(None),
            time_to_interactive: Cell::new(None),
            time_profiler_chan: time_profiler_chan,
        }
    }

    pub fn set_dom_content_loaded(&self) {
        if self.dom_content_loaded.get().is_none() {
            self.dom_content_loaded.set(Some(precise_time_ns() as f64));
        }
    }

    pub fn set_main_thread_available(&self, time: f64) {
        if self.main_thread_available.get().is_none() {
            self.main_thread_available.set(Some(time));
        }
    }

    pub fn get_dom_content_loaded(&self) -> Option<f64> {
        self.dom_content_loaded.get()
    }

    pub fn get_main_thread_available(&self) -> Option<f64> {
        self.main_thread_available.get()
    }

    // can set either dlc or tti first, but both must be set to actually calc metric
    // when the second is set, set_tti is called with appropriate time
    pub fn maybe_set_tti<T>(
        &self,
        profiler_metadata_factory: &T,
        metric: InteractiveFlag,
    ) where
        T: ProfilerMetadataFactory,
    {
        if self.get_tti().is_some() {
            return;
        }
        match metric {
            InteractiveFlag::DOMContentLoaded => self.set_dom_content_loaded(),
            InteractiveFlag::TimeToInteractive(time) => self.set_main_thread_available(time),
        }

        let dcl = self.dom_content_loaded.get();
        let mta = self.main_thread_available.get();
        let (dcl, mta) = match (dcl, mta) {
        (Some(dcl), Some(mta)) => (dcl, mta),
            _ => return,
        };
        let metric_time = match dcl.partial_cmp(&mta) {
            Some(Ordering::Less) => mta,
            Some(_) => dcl,
            None => panic!("no ordering possible. something bad happened"),
        };
        set_metric(
            self,
            profiler_metadata_factory.new_metadata(),
            ProgressiveWebMetricType::TimeToInteractive,
            ProfilerCategory::TimeToInteractive,
            &self.time_to_interactive,
            Some(metric_time));
    }

    pub fn get_tti(&self) -> Option<f64> {
        self.time_to_interactive.get()
    }
}

impl ProgressiveWebMetric for InteractiveMetrics {
    fn get_navigation_start(&self) -> Option<f64> {
        self.navigation_start
    }

    fn set_navigation_start(&mut self, time: f64) {
        self.navigation_start = Some(time);
    }

    fn send_queued_constellation_msg(&self, _name: ProgressiveWebMetricType, _time: f64) { }

    fn get_time_profiler_chan(&self) -> &ProfilerChan {
        &self.time_profiler_chan
    }
}

pub struct PaintTimeMetrics {
    pending_metrics: RefCell<HashMap<Epoch, (Option<TimerMetadata>, bool)>>,
    navigation_start: Option<f64>,
    first_paint: Cell<Option<f64>>,
    first_contentful_paint: Cell<Option<f64>>,
    pipeline_id: PipelineId,
    time_profiler_chan: ProfilerChan,
    constellation_chan: IpcSender<LayoutMsg>,
    script_chan: IpcSender<ConstellationControlMsg>,
}

impl PaintTimeMetrics {
    pub fn new(
        pipeline_id: PipelineId,
        time_profiler_chan: ProfilerChan,
        constellation_chan: IpcSender<LayoutMsg>,
        script_chan: IpcSender<ConstellationControlMsg>)
            -> PaintTimeMetrics {
        PaintTimeMetrics {
            pending_metrics: RefCell::new(HashMap::new()),
            navigation_start: None,
            first_paint: Cell::new(None),
            first_contentful_paint: Cell::new(None),
            pipeline_id,
            time_profiler_chan,
            constellation_chan,
            script_chan,
        }
    }

    pub fn maybe_set_first_paint<T>(&self, profiler_metadata_factory: &T)
    where
        T: ProfilerMetadataFactory,
    {
        if self.first_paint.get().is_some() {
            return;
        }

        set_metric(
            self,
            profiler_metadata_factory.new_metadata(),
            ProgressiveWebMetricType::FirstPaint,
            ProfilerCategory::TimeToFirstPaint,
            &self.first_paint,
            None,
        );
    }

    pub fn maybe_observe_paint_time<T>(
        &self,
        profiler_metadata_factory: &T,
        epoch: Epoch,
        display_list: &DisplayList)
    where T: ProfilerMetadataFactory {
        if self.first_paint.get().is_some() && self.first_contentful_paint.get().is_some() {
            // If we already set all paint metrics, we just bail out.
            return;
        }

        let mut is_contentful = false;
        // Analyze the display list to figure out if this may be the first
        // contentful paint (i.e. the display list contains items of type text,
        // image, non-white canvas or SVG).
        for item in &display_list.list {
            match item {
                &DisplayItem::Text(_) |
                &DisplayItem::Image(_) => {
                    is_contentful = true;
                    break;
                }
                _ => (),
            }
        }

        self.pending_metrics.borrow_mut().insert(epoch, (
            profiler_metadata_factory.new_metadata(),
            is_contentful,
        ));

        // Send the pending metric information to the compositor thread.
        // The compositor will record the current time after painting the
        // frame with the given ID and will send the metric back to us.
        let msg = LayoutMsg::PendingPaintMetric(self.pipeline_id, epoch);
        if let Err(e) = self.constellation_chan.send(msg) {
            warn!("Failed to send PendingPaintMetric {:?}", e);
        }
    }

    pub fn maybe_set_metric(&self, epoch: Epoch, paint_time: f64) {
        if self.first_paint.get().is_some() && self.first_contentful_paint.get().is_some() ||
            self.navigation_start.is_none() {
            // If we already set all paint metrics or we have not set navigation start yet,
            // we just bail out.
            return;
        }

        if let Some(pending_metric) = self.pending_metrics.borrow_mut().remove(&epoch) {
            let profiler_metadata = pending_metric.0;
            set_metric(
                self,
                profiler_metadata.clone(),
                ProgressiveWebMetricType::FirstPaint,
                ProfilerCategory::TimeToFirstPaint,
                &self.first_paint,
                Some(paint_time),
            );

            if pending_metric.1 {
                set_metric(
                    self,
                    profiler_metadata,
                    ProgressiveWebMetricType::FirstContentfulPaint,
                    ProfilerCategory::TimeToFirstContentfulPaint,
                    &self.first_contentful_paint,
                    Some(paint_time),
                );
            }
        }
    }

    pub fn get_first_paint(&self) -> Option<f64> {
        self.first_paint.get()
    }

    pub fn get_first_contentful_paint(&self) -> Option<f64> {
        self.first_contentful_paint.get()
    }
}

impl ProgressiveWebMetric for PaintTimeMetrics {
    fn get_navigation_start(&self) -> Option<f64> {
        self.navigation_start
    }

    fn set_navigation_start(&mut self, time: f64) {
        self.navigation_start = Some(time);
    }

    fn send_queued_constellation_msg(&self, name: ProgressiveWebMetricType, time: f64) {
        let msg = ConstellationControlMsg::PaintMetric(self.pipeline_id, name, time);
        if let Err(e) = self.script_chan.send(msg) {
            warn!("Sending metric to script thread failed ({}).", e);
        }
    }

    fn get_time_profiler_chan(&self) -> &ProfilerChan {
        &self.time_profiler_chan
    }
}
