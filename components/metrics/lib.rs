/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::cmp::Ordering;
use std::time::Duration;

use base::cross_process_instant::CrossProcessInstant;
use malloc_size_of_derive::MallocSizeOf;
use profile_traits::time::{
    ProfilerCategory, ProfilerChan, TimerMetadata, TimerMetadataFrameType, TimerMetadataReflowType,
    send_profile_data,
};
use script_traits::ProgressiveWebMetricType;
use servo_config::opts;
use servo_url::ServoUrl;

/// TODO make this configurable
/// maximum task time is 50ms (in ns)
pub const MAX_TASK_NS: u128 = 50000000;
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

fn set_metric(
    pwm: &ProgressiveWebMetrics,
    metadata: Option<TimerMetadata>,
    metric_type: ProgressiveWebMetricType,
    category: ProfilerCategory,
    attr: &Cell<Option<CrossProcessInstant>>,
    metric_time: CrossProcessInstant,
    url: &ServoUrl,
) {
    attr.set(Some(metric_time));

    // Send the metric to the time profiler.
    send_profile_data(
        category,
        metadata,
        pwm.time_profiler_chan(),
        metric_time,
        metric_time,
    );

    // Print the metric to console if the print-pwm option was given.
    if opts::get().print_pwm {
        let navigation_start = pwm
            .navigation_start()
            .unwrap_or_else(CrossProcessInstant::epoch);
        println!(
            "{:?} {:?} {:?}",
            url,
            metric_type,
            (metric_time - navigation_start).as_seconds_f64()
        );
    }
}

/// A data structure to track web metrics dfined in various specifications:
///
///  - <https://w3c.github.io/paint-timing/>
///  - <https://github.com/WICG/time-to-interactive> / <https://github.com/GoogleChrome/lighthouse/issues/27>
///
///  We can look at three different metrics here:
///    - navigation start -> visually ready (dom content loaded)
///    - navigation start -> thread ready (main thread available)
///    - visually ready -> thread ready
#[derive(MallocSizeOf)]
pub struct ProgressiveWebMetrics {
    /// Whether or not this metric is for an `<iframe>` or a top level frame.
    frame_type: TimerMetadataFrameType,
    /// when we navigated to the page
    navigation_start: Option<CrossProcessInstant>,
    /// indicates if the page is visually ready
    dom_content_loaded: Cell<Option<CrossProcessInstant>>,
    /// main thread is available -- there's been a 10s window with no tasks longer than 50ms
    main_thread_available: Cell<Option<CrossProcessInstant>>,
    // max(main_thread_available, dom_content_loaded)
    time_to_interactive: Cell<Option<CrossProcessInstant>>,
    /// The first paint of a particular document.
    /// TODO(mrobinson): It's unclear if this particular metric is reflected in the specification.
    ///
    /// See <https://w3c.github.io/paint-timing/#sec-reporting-paint-timing>.
    first_paint: Cell<Option<CrossProcessInstant>>,
    /// The first "contentful" paint of a particular document.
    ///
    /// See <https://w3c.github.io/paint-timing/#first-contentful-paint>
    first_contentful_paint: Cell<Option<CrossProcessInstant>>,
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

impl ProgressiveWebMetrics {
    pub fn new(
        time_profiler_chan: ProfilerChan,
        url: ServoUrl,
        frame_type: TimerMetadataFrameType,
    ) -> ProgressiveWebMetrics {
        ProgressiveWebMetrics {
            frame_type,
            navigation_start: None,
            dom_content_loaded: Cell::new(None),
            main_thread_available: Cell::new(None),
            time_to_interactive: Cell::new(None),
            first_paint: Cell::new(None),
            first_contentful_paint: Cell::new(None),
            time_profiler_chan,
            url,
        }
    }

    fn make_metadata(&self, first_reflow: bool) -> TimerMetadata {
        TimerMetadata {
            url: self.url.to_string(),
            iframe: self.frame_type.clone(),
            incremental: match first_reflow {
                true => TimerMetadataReflowType::FirstReflow,
                false => TimerMetadataReflowType::Incremental,
            },
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

    pub fn dom_content_loaded(&self) -> Option<CrossProcessInstant> {
        self.dom_content_loaded.get()
    }

    pub fn first_paint(&self) -> Option<CrossProcessInstant> {
        self.first_paint.get()
    }

    pub fn first_contentful_paint(&self) -> Option<CrossProcessInstant> {
        self.first_contentful_paint.get()
    }

    pub fn main_thread_available(&self) -> Option<CrossProcessInstant> {
        self.main_thread_available.get()
    }

    pub fn set_first_paint(&self, paint_time: CrossProcessInstant, first_reflow: bool) {
        set_metric(
            self,
            Some(self.make_metadata(first_reflow)),
            ProgressiveWebMetricType::FirstPaint,
            ProfilerCategory::TimeToFirstPaint,
            &self.first_paint,
            paint_time,
            &self.url,
        );
    }

    pub fn set_first_contentful_paint(&self, paint_time: CrossProcessInstant, first_reflow: bool) {
        set_metric(
            self,
            Some(self.make_metadata(first_reflow)),
            ProgressiveWebMetricType::FirstContentfulPaint,
            ProfilerCategory::TimeToFirstContentfulPaint,
            &self.first_contentful_paint,
            paint_time,
            &self.url,
        );
    }

    // can set either dlc or tti first, but both must be set to actually calc metric
    // when the second is set, set_tti is called with appropriate time
    pub fn maybe_set_tti(&self, metric: InteractiveFlag) {
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
            Some(self.make_metadata(true)),
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

    pub fn navigation_start(&self) -> Option<CrossProcessInstant> {
        self.navigation_start
    }

    pub fn set_navigation_start(&mut self, time: CrossProcessInstant) {
        self.navigation_start = Some(time);
    }

    pub fn time_profiler_chan(&self) -> &ProfilerChan {
        &self.time_profiler_chan
    }
}

#[cfg(test)]
fn test_metrics() -> ProgressiveWebMetrics {
    let (sender, _) = ipc_channel::ipc::channel().unwrap();
    let profiler_chan = ProfilerChan(sender);
    let mut metrics = ProgressiveWebMetrics::new(
        profiler_chan,
        ServoUrl::parse("about:blank").unwrap(),
        TimerMetadataFrameType::RootWindow,
    );

    assert!((&metrics).navigation_start().is_none());
    assert!(metrics.get_tti().is_none());
    assert!(metrics.first_contentful_paint().is_none());
    assert!(metrics.first_paint().is_none());

    metrics.set_navigation_start(CrossProcessInstant::now());

    metrics
}

#[test]
fn test_set_dcl() {
    let metrics = test_metrics();
    metrics.maybe_set_tti(InteractiveFlag::DOMContentLoaded);
    let dcl = metrics.dom_content_loaded();
    assert!(dcl.is_some());

    //try to overwrite
    metrics.maybe_set_tti(InteractiveFlag::DOMContentLoaded);
    assert_eq!(metrics.dom_content_loaded(), dcl);
    assert_eq!(metrics.get_tti(), None);
}

#[test]
fn test_set_mta() {
    let metrics = test_metrics();
    let now = CrossProcessInstant::now();
    metrics.maybe_set_tti(InteractiveFlag::TimeToInteractive(now));
    let main_thread_available_time = metrics.main_thread_available();
    assert!(main_thread_available_time.is_some());
    assert_eq!(main_thread_available_time, Some(now));

    //try to overwrite
    metrics.maybe_set_tti(InteractiveFlag::TimeToInteractive(
        CrossProcessInstant::now(),
    ));
    assert_eq!(metrics.main_thread_available(), main_thread_available_time);
    assert_eq!(metrics.get_tti(), None);
}

#[test]
fn test_set_tti_dcl() {
    let metrics = test_metrics();
    let now = CrossProcessInstant::now();
    metrics.maybe_set_tti(InteractiveFlag::TimeToInteractive(now));
    let main_thread_available_time = metrics.main_thread_available();
    assert!(main_thread_available_time.is_some());

    metrics.maybe_set_tti(InteractiveFlag::DOMContentLoaded);
    let dom_content_loaded_time = metrics.dom_content_loaded();
    assert!(dom_content_loaded_time.is_some());

    assert_eq!(metrics.get_tti(), dom_content_loaded_time);
}

#[test]
fn test_set_tti_mta() {
    let metrics = test_metrics();
    metrics.maybe_set_tti(InteractiveFlag::DOMContentLoaded);
    let dcl = metrics.dom_content_loaded();
    assert!(dcl.is_some());

    let time = CrossProcessInstant::now();
    metrics.maybe_set_tti(InteractiveFlag::TimeToInteractive(time));
    let mta = metrics.main_thread_available();
    assert!(mta.is_some());

    assert_eq!(metrics.get_tti(), mta);
}

#[test]
fn test_first_paint_setter() {
    let metrics = test_metrics();
    metrics.set_first_paint(CrossProcessInstant::now(), false);
    assert!(metrics.first_paint().is_some());
}

#[test]
fn test_first_contentful_paint_setter() {
    let metrics = test_metrics();
    metrics.set_first_contentful_paint(CrossProcessInstant::now(), false);
    assert!(metrics.first_contentful_paint().is_some());
}
