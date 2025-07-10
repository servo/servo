/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use base::cross_process_instant::CrossProcessInstant;
use compositing_traits::largest_contentful_paint_record::{
    IMAGE_ENTROPY_THEAHOLD, ImageRecord, LCPCandidateRecord, LargestContentfulPaint, TextRecord,
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
                text_calculator: Default::default(),
                lcp: None,
            })
    }

    pub fn appen_lcp_candidate_records(
        &mut self,
        pipeline_id: PipelineId,
        mut records: LCPCandidateRecord,
    ) {
        let lcp_calculator = self.ensure_lcp_calculator(pipeline_id);
        lcp_calculator
            .image_calculator
            .record_list
            .append(&mut records.image_records);
        let mut text_records: Vec<TextRecord> = records.text_records.into_values().collect();
        lcp_calculator
            .text_calculator
            .record_list
            .append(&mut text_records);
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
        let text_candidate = lcp_calculator
            .text_calculator
            .calculate_largest_contentful_paint(paint_time, epoch);

        let mut candidate = Self::pick_largest_contentful_paint(image_candidate, text_candidate);
        candidate = Self::pick_largest_contentful_paint(lcp_calculator.lcp, candidate);

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
    text_calculator: TextLargestContentfulPaintCalculator,
    lcp: Option<LargestContentfulPaint>,
}

#[derive(Default)]
pub struct ImageLargestContentfulPaintCalculator {
    record_list: Vec<ImageRecord>,
    largest_image: Option<ImageRecord>,
}

impl ImageLargestContentfulPaintCalculator {
    fn calculate_largest_contentful_paint(
        &mut self,
        paint_time: CrossProcessInstant,
        cur_epoch: Epoch,
    ) -> Option<LargestContentfulPaint> {
        if self.record_list.is_empty() {
            return self.largest_image.as_ref().map(|record| record.into());
        }

        let candidate_records = std::mem::take(&mut self.record_list);
        for mut candidate_record in candidate_records {
            if candidate_record.epoch() > cur_epoch {
                self.record_list.push(candidate_record);
                continue;
            }

            if candidate_record.image_entropy() < IMAGE_ENTROPY_THEAHOLD {
                continue;
            }

            candidate_record.paint_time = Some(paint_time);
            match self.largest_image {
                None => self.largest_image = Some(candidate_record),
                Some(ref largest_image) => {
                    if largest_image.size() < candidate_record.size() ||
                        (largest_image.size() == candidate_record.size() &&
                            largest_image.paint_time() > candidate_record.paint_time())
                    {
                        self.largest_image = Some(candidate_record)
                    }
                },
            }
        }

        self.largest_image.as_ref().map(|record| record.into())
    }
}

#[derive(Default)]
pub struct TextLargestContentfulPaintCalculator {
    record_list: Vec<TextRecord>,
    largest_text: Option<TextRecord>,
}

impl TextLargestContentfulPaintCalculator {
    fn calculate_largest_contentful_paint(
        &mut self,
        paint_time: CrossProcessInstant,
        cur_epoch: Epoch,
    ) -> Option<LargestContentfulPaint> {
        if self.record_list.is_empty() {
            return self.largest_text.as_ref().map(|record| record.into());
        }

        let candidate_records = std::mem::take(&mut self.record_list);
        for mut candidate_record in candidate_records {
            if candidate_record.epoch() > cur_epoch {
                self.record_list.push(candidate_record);
                continue;
            }

            candidate_record.paint_time = Some(paint_time);
            match self.largest_text {
                None => self.largest_text = Some(candidate_record),
                Some(ref largest_text) => {
                    if largest_text.size() < candidate_record.size() ||
                        (largest_text.size() == candidate_record.size() &&
                            largest_text.paint_time() > candidate_record.paint_time())
                    {
                        self.largest_text = Some(candidate_record)
                    }
                },
            }
        }

        self.largest_text.as_ref().map(|record| record.into())
    }
}
