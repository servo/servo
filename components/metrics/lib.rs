/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::Duration;

use base::cross_process_instant::CrossProcessInstant;
use base::id::PipelineId;
use base::Epoch;
use ipc_channel::ipc::IpcSender;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use profile_traits::time::{send_profile_data, ProfilerCategory, ProfilerChan, TimerMetadata};
use script_traits::{LayoutMsg, ProgressiveWebMetricType, ScriptThreadMessage};
use servo_config::opts;
use servo_url::ServoUrl;

pub trait ProfilerMetadataFactory {
    fn new_metadata(&self) -> Option<TimerMetadata>;
}

pub trait ProgressiveWebMetric {
    fn get_navigation_start(&self) -> Option<CrossProcessInstant>;
    fn set_navigation_start(&mut self, time: CrossProcessInstant);
    fn get_time_profiler_chan(&self) -> &ProfilerChan;
    fn send_queued_constellation_msg(
        &self,
        name: ProgressiveWebMetricType,
        time: CrossProcessInstant,
    );
    fn get_url(&self) -> &ServoUrl;
}

/// TODO make this configurable
/// maximum task time is 50ms (in ns)
pub const MAX_TASK_NS: u64 = 50000000;
/// 10 second window
const INTERACTIVE_WINDOW_SECONDS: Duration = Duration::from_secs(10);

pub trait ToMs<T> {
    fn to_ms(&self) -> T;
}

impl ToMs<f64> for u64 {
    fn to_ms(&self) -> f64 {
        *self as f64 / 1000000.
    }
}

fn set_metric<U: ProgressiveWebMetric>(
    pwm: &U,
    metadata: Option<TimerMetadata>,
    metric_type: ProgressiveWebMetricType,
    category: ProfilerCategory,
    attr: &Cell<Option<CrossProcessInstant>>,
    metric_time: CrossProcessInstant,
    url: &ServoUrl,
) {
    attr.set(Some(metric_time));

    // Queue performance observer notification.
    pwm.send_queued_constellation_msg(metric_type, metric_time);

    // Send the metric to the time profiler.
    send_profile_data(
        category,
        metadata,
        pwm.get_time_profiler_chan(),
        metric_time,
        metric_time,
    );

    // Print the metric to console if the print-pwm option was given.
    if opts::get().print_pwm {
        let navigation_start = pwm
            .get_navigation_start()
            .unwrap_or_else(CrossProcessInstant::epoch);
        println!(
            "{:?} {:?} {:?}",
            url,
            metric_type,
            (metric_time - navigation_start).as_seconds_f64()
        );
    }
}

// spec: https://github.com/WICG/time-to-interactive
// https://github.com/GoogleChrome/lighthouse/issues/27
// we can look at three different metrics here:
// navigation start -> visually ready (dom content loaded)
// navigation start -> thread ready (main thread available)
// visually ready -> thread ready
#[derive(MallocSizeOf)]
pub struct InteractiveMetrics {
    /// when we navigated to the page
    navigation_start: Option<CrossProcessInstant>,
    /// indicates if the page is visually ready
    dom_content_loaded: Cell<Option<CrossProcessInstant>>,
    /// main thread is available -- there's been a 10s window with no tasks longer than 50ms
    main_thread_available: Cell<Option<CrossProcessInstant>>,
    // max(main_thread_available, dom_content_loaded)
    time_to_interactive: Cell<Option<CrossProcessInstant>>,
    #[ignore_malloc_size_of = "can't measure channels"]
    time_profiler_chan: ProfilerChan,
    url: ServoUrl,
}

#[derive(Clone, Copy, Debug, MallocSizeOf)]
pub struct InteractiveWindow {
    start: CrossProcessInstant,
}

impl Default for InteractiveWindow {
    fn default() -> Self {
        Self {
            start: CrossProcessInstant::now(),
        }
    }
}

impl InteractiveWindow {
    // We need to either start or restart the 10s window
    //   start: we've added a new document
    //   restart: there was a task > 50ms
    //   not all documents are interactive
    pub fn start_window(&mut self) {
        self.start = CrossProcessInstant::now();
    }

    /// check if 10s has elapsed since start
    pub fn needs_check(&self) -> bool {
        CrossProcessInstant::now() - self.start > INTERACTIVE_WINDOW_SECONDS
    }

    pub fn get_start(&self) -> CrossProcessInstant {
        self.start
    }
}

#[derive(Debug)]
pub enum InteractiveFlag {
    DOMContentLoaded,
    TimeToInteractive(CrossProcessInstant),
}

impl InteractiveMetrics {
    pub fn new(time_profiler_chan: ProfilerChan, url: ServoUrl) -> InteractiveMetrics {
        InteractiveMetrics {
            navigation_start: None,
            dom_content_loaded: Cell::new(None),
            main_thread_available: Cell::new(None),
            time_to_interactive: Cell::new(None),
            time_profiler_chan,
            url,
        }
    }

    pub fn set_dom_content_loaded(&self) {
        if self.dom_content_loaded.get().is_none() {
            self.dom_content_loaded
                .set(Some(CrossProcessInstant::now()));
        }
    }

    pub fn set_main_thread_available(&self, time: CrossProcessInstant) {
        if self.main_thread_available.get().is_none() {
            self.main_thread_available.set(Some(time));
        }
    }

    pub fn get_dom_content_loaded(&self) -> Option<CrossProcessInstant> {
        self.dom_content_loaded.get()
    }

    pub fn get_main_thread_available(&self) -> Option<CrossProcessInstant> {
        self.main_thread_available.get()
    }

    // can set either dlc or tti first, but both must be set to actually calc metric
    // when the second is set, set_tti is called with appropriate time
    pub fn maybe_set_tti<T>(&self, profiler_metadata_factory: &T, metric: InteractiveFlag)
    where
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
            metric_time,
            &self.url,
        );
    }

    pub fn get_tti(&self) -> Option<CrossProcessInstant> {
        self.time_to_interactive.get()
    }

    pub fn needs_tti(&self) -> bool {
        self.get_tti().is_none()
    }
}

impl ProgressiveWebMetric for InteractiveMetrics {
    fn get_navigation_start(&self) -> Option<CrossProcessInstant> {
        self.navigation_start
    }

    fn set_navigation_start(&mut self, time: CrossProcessInstant) {
        self.navigation_start = Some(time);
    }

    fn send_queued_constellation_msg(
        &self,
        _name: ProgressiveWebMetricType,
        _time: CrossProcessInstant,
    ) {
    }

    fn get_time_profiler_chan(&self) -> &ProfilerChan {
        &self.time_profiler_chan
    }

    fn get_url(&self) -> &ServoUrl {
        &self.url
    }
}

// https://w3c.github.io/paint-timing/
pub struct PaintTimeMetrics {
    pending_metrics: RefCell<HashMap<Epoch, (Option<TimerMetadata>, bool)>>,
    navigation_start: CrossProcessInstant,
    first_paint: Cell<Option<CrossProcessInstant>>,
    first_contentful_paint: Cell<Option<CrossProcessInstant>>,
    pipeline_id: PipelineId,
    time_profiler_chan: ProfilerChan,
    constellation_chan: IpcSender<LayoutMsg>,
    script_chan: IpcSender<ScriptThreadMessage>,
    url: ServoUrl,
}

impl PaintTimeMetrics {
    pub fn new(
        pipeline_id: PipelineId,
        time_profiler_chan: ProfilerChan,
        constellation_chan: IpcSender<LayoutMsg>,
        script_chan: IpcSender<ScriptThreadMessage>,
        url: ServoUrl,
        navigation_start: CrossProcessInstant,
    ) -> PaintTimeMetrics {
        PaintTimeMetrics {
            pending_metrics: RefCell::new(HashMap::new()),
            navigation_start,
            first_paint: Cell::new(None),
            first_contentful_paint: Cell::new(None),
            pipeline_id,
            time_profiler_chan,
            constellation_chan,
            script_chan,
            url,
        }
    }

    pub fn maybe_observe_paint_time<T>(
        &self,
        profiler_metadata_factory: &T,
        epoch: Epoch,
        display_list_is_contentful: bool,
    ) where
        T: ProfilerMetadataFactory,
    {
        if self.first_paint.get().is_some() && self.first_contentful_paint.get().is_some() {
            // If we already set all paint metrics, we just bail out.
            return;
        }

        self.pending_metrics.borrow_mut().insert(
            epoch,
            (
                profiler_metadata_factory.new_metadata(),
                display_list_is_contentful,
            ),
        );

        // Send the pending metric information to the compositor thread.
        // The compositor will record the current time after painting the
        // frame with the given ID and will send the metric back to us.
        let msg = LayoutMsg::PendingPaintMetric(self.pipeline_id, epoch);
        if let Err(e) = self.constellation_chan.send(msg) {
            warn!("Failed to send PendingPaintMetric {:?}", e);
        }
    }

    pub fn maybe_set_metric(&self, epoch: Epoch, paint_time: CrossProcessInstant) {
        if self.first_paint.get().is_some() && self.first_contentful_paint.get().is_some() {
            // If we already set all paint metrics we just bail out.
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
                paint_time,
                &self.url,
            );

            if pending_metric.1 {
                set_metric(
                    self,
                    profiler_metadata,
                    ProgressiveWebMetricType::FirstContentfulPaint,
                    ProfilerCategory::TimeToFirstContentfulPaint,
                    &self.first_contentful_paint,
                    paint_time,
                    &self.url,
                );
            }
        }
    }

    pub fn get_first_paint(&self) -> Option<CrossProcessInstant> {
        self.first_paint.get()
    }

    pub fn get_first_contentful_paint(&self) -> Option<CrossProcessInstant> {
        self.first_contentful_paint.get()
    }
}

impl ProgressiveWebMetric for PaintTimeMetrics {
    fn get_navigation_start(&self) -> Option<CrossProcessInstant> {
        Some(self.navigation_start)
    }

    fn set_navigation_start(&mut self, time: CrossProcessInstant) {
        self.navigation_start = time;
    }

    fn send_queued_constellation_msg(
        &self,
        name: ProgressiveWebMetricType,
        time: CrossProcessInstant,
    ) {
        let msg = ScriptThreadMessage::PaintMetric(self.pipeline_id, name, time);
        if let Err(e) = self.script_chan.send(msg) {
            warn!("Sending metric to script thread failed ({}).", e);
        }
    }

    fn get_time_profiler_chan(&self) -> &ProfilerChan {
        &self.time_profiler_chan
    }

    fn get_url(&self) -> &ServoUrl {
        &self.url
    }
}
