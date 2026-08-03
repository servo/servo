/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Calculator for the Container Timing API.
//!
//! <https://wicg.github.io/container-timing/>

use paint_api::container_timing_candidate::ContainerTimingRecord;
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
            .record_candidate(candidate);
    }

    pub(crate) fn remove_candidates_for_pipeline(&mut self, pipeline_id: &PipelineId) {
        self.containers.remove(pipeline_id);
    }

    /// Stamp `paint_time` on every container touched since the last call for
    /// `pipeline_id`, and return only those records. `paint_time` is the time the frame
    /// was composited.
    pub(crate) fn calculate(
        &mut self,
        paint_time: CrossProcessInstant,
        pipeline_id: PipelineId,
    ) -> Vec<ContainerTimingRecord> {
        let Some(state) = self.containers.get_mut(&pipeline_id) else {
            return Vec::new();
        };
        state.flush(paint_time)
    }
}

/// Per-pipeline accumulated state for container timing.
#[derive(Default)]
struct PipelineContainerTimings {
    /// Every container ever seen for this pipeline, for as long as the document (and
    /// its `containertiming` attribute) is around. Grows monotonically -- entries are
    /// only ever appended or replaced wholesale, never removed. This can't be keyed by
    /// `identifier` since that's optional and not guaranteed unique; entries are matched
    /// up by `container_id` instead.
    records: Vec<ContainerTimingRecord>,
    /// Indices into `records` that received a new paint since the last `flush`, and so
    /// need their `paint_time`/`first_render_time` set and reported to the constellation.
    dirty: Vec<usize>,
}

impl PipelineContainerTimings {
    /// Store a freshly-arrived candidate as the permanent record for its container.
    /// Layout only ever hands off a container once it's already accumulated that
    /// container's up-to-date total size and bounding rect (see
    /// `PaintTimingHandler::update_container_timing`), so there's nothing left to merge
    /// here -- except that an existing record's `first_render_time`/`paint_time` (set by
    /// a previous `flush`) must survive, since layout never knows about them and a
    /// wholesale overwrite would erase them. Marks the record dirty so the next `flush`
    /// reports it.
    fn record_candidate(&mut self, candidate: ContainerTimingRecord) {
        let index = self
            .records
            .iter()
            .position(|record| record.container_id == candidate.container_id);

        let index = match index {
            Some(index) => {
                self.records[index].update_metrics(candidate);
                index
            },
            None => {
                self.records.push(candidate);
                self.records.len() - 1
            },
        };
        self.dirty.push(index);
    }

    /// Stamp `paint_time` (and `first_render_time`, only the first time) on every record
    /// touched since the last flush, and return them.
    fn flush(&mut self, paint_time: CrossProcessInstant) -> Vec<ContainerTimingRecord> {
        if self.dirty.is_empty() {
            return Vec::new();
        }

        // The same container can be marked dirty more than once per round (e.g. painted
        // across multiple builds before we got around to flushing) -- dedup the indices
        // before reporting, so we don't send duplicate entries for one container.
        let dirty_indices: FxHashSet<usize> = self.dirty.drain(..).collect();
        let mut updated = Vec::with_capacity(dirty_indices.len());
        for index in dirty_indices {
            let record = &mut self.records[index];
            record.mark_painted(paint_time);
            updated.push(record.clone());
        }
        updated
    }
}
