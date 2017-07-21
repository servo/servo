/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::Size2D;
use gfx::display_list::{BaseDisplayItem, WebRenderImageInfo};
use gfx::display_list::{DisplayItem, DisplayList, ImageDisplayItem};
use ipc_channel::ipc;
use metrics::{PaintTimeMetrics, ProfilerMetadataFactory};
use msg::constellation_msg::{PipelineId, PipelineIndex, PipelineNamespaceId};
use net_traits::image::base::PixelFormat;
use profile_traits::time::{ProfilerChan, TimerMetadata};
use style::computed_values::image_rendering;
use time;

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
    let paint_time_metrics = PaintTimeMetrics::new(profiler_chan);
    assert_eq!(paint_time_metrics.get_navigation_start(), None, "navigation start is None");
    assert_eq!(paint_time_metrics.get_first_paint(), None, "first paint is None");
    assert_eq!(paint_time_metrics.get_first_contentful_paint(), None, "first contentful paint is None");
}

#[test]
fn test_first_paint_setter() {
    let (sender, _) = ipc::channel().unwrap();
    let profiler_chan = ProfilerChan(sender);
    let mut paint_time_metrics = PaintTimeMetrics::new(profiler_chan);
    let dummy_profiler_metadata_factory = DummyProfilerMetadataFactory {};

    // Should not set any metric until navigation start is set.
    paint_time_metrics.maybe_set_first_paint(&dummy_profiler_metadata_factory);
    assert_eq!(paint_time_metrics.get_first_paint(), None, "first paint is None");

    let navigation_start = time::precise_time_ns() as f64;
    paint_time_metrics.set_navigation_start(navigation_start);
    assert_eq!(paint_time_metrics.get_navigation_start().unwrap(),
               navigation_start, "navigation start is set");

    paint_time_metrics.maybe_set_first_paint(&dummy_profiler_metadata_factory);
    assert!(paint_time_metrics.get_first_paint().is_some(), "first paint is set");
    assert_eq!(paint_time_metrics.get_first_contentful_paint(), None, "first contentful paint is None");
}

#[test]
fn test_first_contentful_paint_setter() {
    let (sender, _) = ipc::channel().unwrap();
    let profiler_chan = ProfilerChan(sender);
    let mut paint_time_metrics = PaintTimeMetrics::new(profiler_chan);
    let dummy_profiler_metadata_factory = DummyProfilerMetadataFactory {};
    let empty_display_list = DisplayList {
        list: Vec::new()
    };

    // Should not set any metric until navigation start is set.
    paint_time_metrics.maybe_set_first_contentful_paint(&dummy_profiler_metadata_factory,
                                                        &empty_display_list);
    assert_eq!(paint_time_metrics.get_first_contentful_paint(), None, "first contentful paint is None");

    // Should not set first contentful paint if no appropriate display item is present.
    let navigation_start = time::precise_time_ns() as f64;
    paint_time_metrics.set_navigation_start(navigation_start);
    paint_time_metrics.maybe_set_first_contentful_paint(&dummy_profiler_metadata_factory,
                                                        &empty_display_list);
    assert_eq!(paint_time_metrics.get_first_contentful_paint(), None, "first contentful paint is None");

    let pipeline_id = PipelineId {
        namespace_id: PipelineNamespaceId(1),
        index: PipelineIndex(1),
    };
    let image = DisplayItem::Image(Box::new(ImageDisplayItem {
        base: BaseDisplayItem::empty(pipeline_id),
        webrender_image: WebRenderImageInfo {
            width: 1,
            height: 1,
            format: PixelFormat::RGB8,
            key: None,
        },
        image_data: None,
        stretch_size: Size2D::zero(),
        tile_spacing: Size2D::zero(),
        image_rendering: image_rendering::T::auto,
    }));
    let display_list = DisplayList {
        list: vec![image]
    };
    paint_time_metrics.maybe_set_first_contentful_paint(&dummy_profiler_metadata_factory,
                                                        &display_list);
    assert!(paint_time_metrics.get_first_contentful_paint().is_some(), "first contentful paint is set");
    assert_eq!(paint_time_metrics.get_first_paint(), None, "first paint is None");
}
