/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::TEST_PIPELINE_ID;
use base::Epoch;
use ipc_channel::ipc;
use metrics::{PaintTimeMetrics, ProfilerMetadataFactory, ProgressiveWebMetric};
use profile_traits::time::{ProfilerChan, TimerMetadata};
use servo_url::ServoUrl;

struct DummyProfilerMetadataFactory {}
impl ProfilerMetadataFactory for DummyProfilerMetadataFactory {
    fn new_metadata(&self) -> Option<TimerMetadata> {
        None
    }
}

#[test]
fn test_paint_metrics_construction() {
    let (sender, _) = ipc::channel().unwrap();
    let profiler_chan = ProfilerChan(sender);
    let (layout_sender, _) = ipc::channel().unwrap();
    let (script_sender, _) = ipc::channel().unwrap();
    let paint_time_metrics = PaintTimeMetrics::new(
        TEST_PIPELINE_ID,
        profiler_chan,
        layout_sender,
        script_sender,
        ServoUrl::parse("about:blank").unwrap(),
        0,
    );
    assert_eq!(
        (&paint_time_metrics).get_navigation_start(),
        Some(0),
        "navigation start is set properly"
    );
    assert_eq!(
        paint_time_metrics.get_first_paint(),
        None,
        "first paint is None"
    );
    assert_eq!(
        paint_time_metrics.get_first_contentful_paint(),
        None,
        "first contentful paint is None"
    );
}

fn test_common(display_list_is_contentful: bool, epoch: Epoch) -> PaintTimeMetrics {
    let (sender, _) = ipc::channel().unwrap();
    let profiler_chan = ProfilerChan(sender);
    let (layout_sender, _) = ipc::channel().unwrap();
    let (script_sender, _) = ipc::channel().unwrap();
    let mut paint_time_metrics = PaintTimeMetrics::new(
        TEST_PIPELINE_ID,
        profiler_chan,
        layout_sender,
        script_sender,
        ServoUrl::parse("about:blank").unwrap(),
        0,
    );
    let dummy_profiler_metadata_factory = DummyProfilerMetadataFactory {};

    paint_time_metrics.maybe_observe_paint_time(
        &dummy_profiler_metadata_factory,
        epoch,
        display_list_is_contentful,
    );

    assert_eq!(
        paint_time_metrics.get_first_paint(),
        None,
        "first paint is None"
    );
    assert_eq!(
        paint_time_metrics.get_first_contentful_paint(),
        None,
        "first contentful paint is None"
    );

    let navigation_start = time::precise_time_ns();
    paint_time_metrics.set_navigation_start(navigation_start);
    assert_eq!(
        (&paint_time_metrics).get_navigation_start().unwrap(),
        navigation_start,
        "navigation start is set"
    );

    paint_time_metrics
}

#[test]
fn test_first_paint_setter() {
    let epoch = Epoch(0);
    let paint_time_metrics = test_common(false, epoch);
    let now = time::precise_time_ns();
    paint_time_metrics.maybe_set_metric(epoch, now);
    assert!(
        paint_time_metrics.get_first_paint().is_some(),
        "first paint is set"
    );
    assert_eq!(
        paint_time_metrics.get_first_contentful_paint(),
        None,
        "first contentful paint is None"
    );
}

#[test]
fn test_first_contentful_paint_setter() {
    let epoch = Epoch(0);
    let paint_time_metrics = test_common(true, epoch);
    let now = time::precise_time_ns();
    paint_time_metrics.maybe_set_metric(epoch, now);
    assert!(
        paint_time_metrics.get_first_contentful_paint().is_some(),
        "first contentful paint is set"
    );
    assert!(
        paint_time_metrics.get_first_paint().is_some(),
        "first paint is set"
    );
}
