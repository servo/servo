/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::cross_process_instant::CrossProcessInstant;
use compositing_traits::largest_contentful_paint_candidate::{
    LCPCandidate, LargestContentfulPaint,
};
use fnv::{FnvBuildHasher, FnvHashMap};
use webrender_api::{Epoch, PipelineId};

/// Holds the [`LargestContentfulPaintCalculator`] for each pipeline.
#[derive(Default)]
pub(crate) struct LargestContentfulPaintDetector {
    lcp_calculators: FnvHashMap<PipelineId, LargestContentfulPaintCalculator>,
}

impl LargestContentfulPaintDetector {
    pub const fn new() -> Self {
        Self {
            lcp_calculators: FnvHashMap::with_hasher(FnvBuildHasher::new()),
        }
    }

    pub fn append_lcp_candidate(&mut self, pipeline_id: PipelineId, candidate: LCPCandidate) {
        self.lcp_calculators
            .entry(pipeline_id)
            .and_modify(|calculator| calculator.lcp_candidates.push(candidate))
            .or_insert(LargestContentfulPaintCalculator {
                lcp_candidates: vec![candidate],
                latest_lcp: None,
            });
    }

    pub fn calculate_largest_contentful_paint(
        &mut self,
        paint_time: CrossProcessInstant,
        cur_epoch: Epoch,
        pipeline_id: PipelineId,
    ) -> Option<LargestContentfulPaint> {
        match self.lcp_calculators.get_mut(&pipeline_id) {
            Some(lcp_calculator) => {
                lcp_calculator.calculate_largest_contentful_paint(paint_time, cur_epoch)
            },

            None => None,
        }
    }
}

#[derive(Default)]
struct LargestContentfulPaintCalculator {
    lcp_candidates: Vec<LCPCandidate>,
    latest_lcp: Option<LargestContentfulPaint>,
}

impl LargestContentfulPaintCalculator {
    fn calculate_largest_contentful_paint(
        &mut self,
        paint_time: CrossProcessInstant,
        cur_epoch: Epoch,
    ) -> Option<LargestContentfulPaint> {
        if self.lcp_candidates.is_empty() {
            return self.latest_lcp;
        }

        let candidates = std::mem::take(&mut self.lcp_candidates);
        for candidate in candidates {
            if candidate.epoch > cur_epoch {
                self.lcp_candidates.push(candidate);
                continue;
            }

            match self.latest_lcp {
                None => {
                    self.latest_lcp = Some(LargestContentfulPaint::from(candidate, paint_time));
                },
                Some(ref latest_lcp) => {
                    if latest_lcp.area < candidate.area {
                        self.latest_lcp = Some(LargestContentfulPaint::from(candidate, paint_time));
                    }
                },
            };
        }

        self.latest_lcp
    }
}
