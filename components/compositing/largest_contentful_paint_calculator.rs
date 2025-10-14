/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::cross_process_instant::CrossProcessInstant;
use compositing_traits::largest_contentful_paint_candidate::{
    LCPCandidate, LargestContentfulPaint,
};
use fnv::{FnvBuildHasher, FnvHashMap};
use webrender_api::PipelineId;

/// Holds the [`LargestContentfulPaintsContainer`] for each pipeline.
#[derive(Default)]
pub(crate) struct LargestContentfulPaintCalculator {
    lcp_containers: FnvHashMap<PipelineId, LargestContentfulPaintsContainer>,
}

impl LargestContentfulPaintCalculator {
    pub(crate) const fn new() -> Self {
        Self {
            lcp_containers: FnvHashMap::with_hasher(FnvBuildHasher::new()),
        }
    }

    pub fn append_lcp_candidate(&mut self, pipeline_id: PipelineId, candidate: LCPCandidate) {
        self.lcp_containers
            .entry(pipeline_id)
            .or_default()
            .lcp_candidates
            .push(candidate);
    }

    pub fn remove_lcp_candidates_for_pipeline(&mut self, pipeline_id: PipelineId) {
        self.lcp_containers.remove(&pipeline_id);
    }

    pub fn calculate_largest_contentful_paint(
        &mut self,
        paint_time: CrossProcessInstant,
        pipeline_id: PipelineId,
    ) -> Option<&LargestContentfulPaint> {
        self.lcp_containers
            .get_mut(&pipeline_id)
            .and_then(|container| container.calculate_largest_contentful_paint(paint_time))
    }

    pub fn clear(&mut self) {
        self.lcp_containers.clear();
    }
}

/// Holds the LCP candidates and the latest LCP for a specific pipeline.
#[derive(Default)]
struct LargestContentfulPaintsContainer {
    /// List of candidates for Largest Contentful Paint in this pipeline.
    lcp_candidates: Vec<LCPCandidate>,
    /// The most recent Largest Contentful Paint, if any.
    latest_lcp: Option<LargestContentfulPaint>,
}

impl LargestContentfulPaintsContainer {
    fn calculate_largest_contentful_paint(
        &mut self,
        paint_time: CrossProcessInstant,
    ) -> Option<&LargestContentfulPaint> {
        if self.lcp_candidates.is_empty() {
            return self.latest_lcp.as_ref();
        }

        let candidates = std::mem::take(&mut self.lcp_candidates);
        if let Some(max_candidate) = candidates.into_iter().max_by_key(|c| c.area) {
            match self.latest_lcp {
                None => {
                    self.latest_lcp = Some(LargestContentfulPaint::from(max_candidate, paint_time));
                },
                Some(ref latest_lcp) => {
                    if max_candidate.area > latest_lcp.area {
                        self.latest_lcp =
                            Some(LargestContentfulPaint::from(max_candidate, paint_time));
                    }
                },
            }
        }

        self.latest_lcp.as_ref()
    }
}
