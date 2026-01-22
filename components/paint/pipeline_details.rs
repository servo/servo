/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use base::Epoch;
use base::id::PipelineId;
use euclid::Scale;
use paint_api::display_list::ScrollTree;
use paint_api::{CompositionPipeline, PipelineExitSource};
use style_traits::CSSPixel;
use webrender_api::units::DevicePixel;

use crate::painter::PaintMetricState;

pub(crate) struct PipelineDetails {
    /// The pipeline associated with this PipelineDetails object.
    pub pipeline: Option<CompositionPipeline>,

    /// The id of the parent pipeline, if any.
    pub parent_pipeline_id: Option<PipelineId>,

    /// Whether animations are running
    pub animations_running: bool,

    /// Whether there are animation callbacks
    pub animation_callbacks_running: bool,

    /// Whether to use less resources by stopping animations.
    pub throttled: bool,

    /// The `Paint`-side [ScrollTree]. This is used to allow finding and scrolling
    /// nodes in `Paint` before forwarding new offsets to WebRender.
    pub scroll_tree: ScrollTree,

    /// The paint metric status of the first paint.
    pub first_paint_metric: Cell<PaintMetricState>,

    /// The paint metric status of the first contentful paint.
    pub first_contentful_paint_metric: Cell<PaintMetricState>,

    /// The paint metric status of the largest contentful paint.
    pub largest_contentful_paint_metric: Cell<PaintMetricState>,

    /// The CSS pixel to device pixel scale of the viewport of this pipeline, including
    /// page zoom, but not including any pinch zoom amount. This is used to detect
    /// situations where the current display list is for an old scale.
    pub viewport_scale: Option<Scale<f32, CSSPixel, DevicePixel>>,

    /// Which parts of Servo have reported that this `Pipeline` has exited. Only when all
    /// have done so will it be discarded.
    pub exited: PipelineExitSource,

    /// The [`Epoch`] of the latest display list received for this `Pipeline` or `None` if no
    /// display list has been received.
    pub display_list_epoch: Option<Epoch>,
}

impl PipelineDetails {
    pub(crate) fn animation_callbacks_running(&self) -> bool {
        self.animation_callbacks_running
    }

    pub(crate) fn animating(&self) -> bool {
        !self.throttled && (self.animation_callbacks_running || self.animations_running)
    }
}

impl PipelineDetails {
    pub(crate) fn new() -> PipelineDetails {
        PipelineDetails {
            pipeline: None,
            parent_pipeline_id: None,
            viewport_scale: None,
            animations_running: false,
            animation_callbacks_running: false,
            throttled: false,
            scroll_tree: ScrollTree::default(),
            first_paint_metric: Cell::new(PaintMetricState::Waiting),
            first_contentful_paint_metric: Cell::new(PaintMetricState::Waiting),
            largest_contentful_paint_metric: Cell::new(PaintMetricState::Waiting),
            exited: PipelineExitSource::empty(),
            display_list_epoch: None,
        }
    }

    pub(crate) fn install_new_scroll_tree(&mut self, new_scroll_tree: ScrollTree) {
        let old_scroll_offsets = self.scroll_tree.scroll_offsets();
        self.scroll_tree = new_scroll_tree;
        self.scroll_tree.set_all_scroll_offsets(&old_scroll_offsets);
    }
}
