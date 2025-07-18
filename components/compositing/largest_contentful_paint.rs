/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use base::cross_process_instant::CrossProcessInstant;
use compositing_traits::largest_contentful_paint_candidate::{
    ImageCandidate, LCPCandidates, LargestContentfulPaint,
};
use webrender_api::{Epoch, PipelineId};

#[derive(Default)]
pub struct LargestContentfulPaintDetector {
    lcp_calculators: HashMap<PipelineId, LargestContentfulPaintCalculator>,
}

impl LargestContentfulPaintDetector {
    pub fn new() -> Self {
        Self {
            lcp_calculators: HashMap::new(),
        }
    }

    fn ensure_lcp_calculator(
        &mut self,
        pipeline_id: PipelineId,
    ) -> &mut LargestContentfulPaintCalculator {
        self.lcp_calculators
            .entry(pipeline_id)
            .or_insert(LargestContentfulPaintCalculator {
                image_calculator: Default::default(),
                lcp: None,
            })
    }

    pub fn append_lcp_candidates(&mut self, pipeline_id: PipelineId, candidates: LCPCandidates) {
        if let Some(image_candiadate) = candidates.image_candidate {
            self.ensure_lcp_calculator(pipeline_id)
                .image_calculator
                .candidates
                .push(image_candiadate);
        }
    }

    pub fn calculate_largest_contentful_paint(
        &mut self,
        paint_time: CrossProcessInstant,
        epoch: Epoch,
        pipeline_id: PipelineId,
    ) -> Option<LargestContentfulPaint> {
        let lcp_calculator = self.ensure_lcp_calculator(pipeline_id);
        let image_candidate = lcp_calculator
            .image_calculator
            .calculate_largest_contentful_paint(paint_time, epoch);

        let candidate = Self::pick_largest_contentful_paint(image_candidate, lcp_calculator.lcp);
        if candidate == lcp_calculator.lcp {
            return None;
        }

        lcp_calculator.lcp = candidate;
        lcp_calculator.lcp
    }

    fn pick_largest_contentful_paint(
        candidate1: Option<LargestContentfulPaint>,
        candidate2: Option<LargestContentfulPaint>,
    ) -> Option<LargestContentfulPaint> {
        match (candidate1, candidate2) {
            (_, None) => candidate1,
            (None, _) => candidate2,
            (Some(c1), Some(c2)) => {
                if (c1.size > c2.size) || (c1.size == c2.size && c1.paint_time <= c2.paint_time) {
                    Some(c1)
                } else {
                    Some(c2)
                }
            },
        }
    }
}

struct LargestContentfulPaintCalculator {
    image_calculator: ImageLargestContentfulPaintCalculator,
    lcp: Option<LargestContentfulPaint>,
}

#[derive(Default)]
pub struct ImageLargestContentfulPaintCalculator {
    candidates: Vec<ImageCandidate>,
    largest_image: Option<ImageCandidate>,
}

impl ImageLargestContentfulPaintCalculator {
    fn calculate_largest_contentful_paint(
        &mut self,
        paint_time: CrossProcessInstant,
        cur_epoch: Epoch,
    ) -> Option<LargestContentfulPaint> {
        if self.candidates.is_empty() {
            return self
                .largest_image
                .as_ref()
                .map(|candidate| candidate.into());
        }

        let candidates = std::mem::take(&mut self.candidates);
        for mut candidate in candidates {
            if candidate.epoch() > cur_epoch {
                self.candidates.push(candidate);
                continue;
            }

            candidate.paint_time = Some(paint_time);
            match self.largest_image {
                None => self.largest_image = Some(candidate),
                Some(ref largest_image) => {
                    if largest_image.size() < candidate.size() ||
                        (largest_image.size() == candidate.size() &&
                            largest_image.paint_time() > candidate.paint_time())
                    {
                        self.largest_image = Some(candidate)
                    }
                },
            }
        }

        self.largest_image
            .as_ref()
            .map(|candidate| candidate.into())
    }
}
