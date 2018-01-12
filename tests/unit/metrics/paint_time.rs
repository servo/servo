/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use gfx::display_list::{BaseDisplayItem, WebRenderImageInfo};
use gfx::display_list::{DisplayItem, DisplayList, ImageDisplayItem};
use gfx_traits::Epoch;
use ipc_channel::ipc;
use metrics::{PaintTimeMetrics, ProfilerMetadataFactory, ProgressiveWebMetric};
use msg::constellation_msg::TEST_PIPELINE_ID;
use net_traits::image::base::PixelFormat;
use profile_traits::time::{ProfilerChan, TimerMetadata};
use servo_url::ServoUrl;
use time;
use webrender_api::{ImageRendering, LayoutSize};

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
    );
    assert_eq!(
        (&paint_time_metrics).get_navigation_start(),
        None,
        "navigation start is None"
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

fn test_common(display_list: &DisplayList, epoch: Epoch) -> PaintTimeMetrics {
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
    );
    let dummy_profiler_metadata_factory = DummyProfilerMetadataFactory {};

    paint_time_metrics.maybe_observe_paint_time(
        &dummy_profiler_metadata_factory,
        epoch,
        &*display_list,
    );

    // Should not set any metric until navigation start is set.
    paint_time_metrics.maybe_set_metric(epoch, 0);
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
    let empty_display_list = DisplayList {
        list: Vec::new(),
        clip_scroll_nodes: Vec::new(),
    };
    let epoch = Epoch(0);
    let paint_time_metrics = test_common(&empty_display_list, epoch);
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
    let image = DisplayItem::Image(Box::new(ImageDisplayItem {
        base: BaseDisplayItem::empty(),
        webrender_image: WebRenderImageInfo {
            width: 1,
            height: 1,
            format: PixelFormat::RGB8,
            key: None,
        },
        image_data: None,
        stretch_size: LayoutSize::zero(),
        tile_spacing: LayoutSize::zero(),
        image_rendering: ImageRendering::Auto,
    }));
    let display_list = DisplayList {
        list: vec![image],
        clip_scroll_nodes: Vec::new(),
    };
    let epoch = Epoch(0);
    let paint_time_metrics = test_common(&display_list, epoch);
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
