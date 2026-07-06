/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Calculator for the Container Timing API.
//!
//! <https://wicg.github.io/container-timing/>

use paint_api::container_timing_candidate::{ContainerTiming, ContainerTimingRecord};
use rustc_hash::{FxHashMap, FxHashSet};
use servo_base::cross_process_instant::CrossProcessInstant;
use servo_base::id::WebViewId;
use webrender_api::PipelineId;

/// Holds per-pipeline container timing state for all active webviews.
#[derive(Default)]
pub(crate) struct ContainerTimingCalculator {
    containers: FxHashMap<PipelineId, PipelineContainerTimings>,
    disabled_webviews: FxHashSet<WebViewId>,
}

impl ContainerTimingCalculator {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn enabled_for_webview(&self, webview_id: &WebViewId) -> bool {
        !self.disabled_webviews.contains(webview_id)
    }

    pub(crate) fn enable_for_webview(&mut self, webview_id: &WebViewId) {
        self.disabled_webviews.remove(webview_id);
    }

    pub(crate) fn disable_for_webview(&mut self, webview_id: WebViewId) {
        self.disabled_webviews.insert(webview_id);
    }

    pub(crate) fn append_candidate(
        &mut self,
        candidate: ContainerTimingRecord,
        pipeline_id: PipelineId,
    ) {
        self.containers
            .entry(pipeline_id)
            .or_default()
            .candidates
            .push(candidate);
    }

    pub(crate) fn remove_candidates_for_pipeline(&mut self, pipeline_id: &PipelineId) {
        self.containers.remove(pipeline_id);
    }

    /// Drain all pending candidates for `pipeline_id`, compute the updated
    /// [`ContainerTiming`] entries, and return any that changed since the last
    /// call. `paint_time` is the time the frame was composited.
    pub(crate) fn calculate(
        &mut self,
        paint_time: CrossProcessInstant,
        pipeline_id: PipelineId,
    ) -> Vec<ContainerTiming> {
        let Some(state) = self.containers.get_mut(&pipeline_id) else {
            return Vec::new();
        };
        state.flush(paint_time)
    }
}

/// Per-pipeline accumulated state for container timing.
#[derive(Default)]
struct PipelineContainerTimings {
    /// Pending candidates collected during the most recent display list build.
    candidates: Vec<ContainerTimingRecord>,
    /// The current best [`ContainerTiming`] for each identifier.
    latest: FxHashMap<String, ContainerTiming>,
}

impl PipelineContainerTimings {
    /// Consume all pending candidates, update `latest`, and return all
    /// containers whose size or rect changed.
    fn flush(&mut self, paint_time: CrossProcessInstant) -> Vec<ContainerTiming> {
        if self.candidates.is_empty() {
            return Vec::new();
        }

        // Aggregate candidates by identifier: accumulate size and expand bounding rect.
        let mut aggregated: FxHashMap<String, AggregatedCandidate> = FxHashMap::default();
        for candidate in self.candidates.drain(..) {
            let entry = aggregated.entry(candidate.identifier.clone()).or_default();
            entry.size += candidate.size;
            entry.expand(
                candidate.rect_x,
                candidate.rect_y,
                candidate.rect_width,
                candidate.rect_height,
            );
        }

        let mut updated = Vec::new();

        for (identifier, agg) in aggregated {
            match self.latest.get_mut(&identifier) {
                None => {
                    // First paint for this container.
                    let entry = ContainerTiming {
                        identifier: identifier.clone(),
                        size: agg.size,
                        rect_x: agg.rect_x,
                        rect_y: agg.rect_y,
                        rect_width: agg.rect_width,
                        rect_height: agg.rect_height,
                        first_render_time: paint_time,
                        paint_time,
                    };
                    updated.push(entry.clone());
                    self.latest.insert(identifier, entry);
                },
                Some(existing) => {
                    // Update if the painted area grew.
                    if agg.size > existing.size {
                        existing.size = agg.size;
                        existing.rect_x = agg.rect_x;
                        existing.rect_y = agg.rect_y;
                        existing.rect_width = agg.rect_width;
                        existing.rect_height = agg.rect_height;
                        existing.paint_time = paint_time;
                        updated.push(existing.clone());
                    }
                },
            }
        }

        updated
    }
}

/// Intermediate accumulator for candidates within one display-list build.
#[derive(Default)]
struct AggregatedCandidate {
    size: usize,
    rect_x: f32,
    rect_y: f32,
    rect_width: f32,
    rect_height: f32,
}

impl AggregatedCandidate {
    fn expand(&mut self, x: f32, y: f32, w: f32, h: f32) {
        if self.rect_width == 0.0 && self.rect_height == 0.0 {
            self.rect_x = x;
            self.rect_y = y;
            self.rect_width = w;
            self.rect_height = h;
        } else {
            let min_x = self.rect_x.min(x);
            let min_y = self.rect_y.min(y);
            let max_x = (self.rect_x + self.rect_width).max(x + w);
            let max_y = (self.rect_y + self.rect_height).max(y + h);
            self.rect_x = min_x;
            self.rect_y = min_y;
            self.rect_width = max_x - min_x;
            self.rect_height = max_y - min_y;
        }
    }
}
