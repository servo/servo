/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::Rect;
use paint_api::largest_contentful_paint_candidate::LCPCandidate;
use servo_geometry::{FastLayoutTransform, au_rect_to_f32_rect, f32_rect_to_au_rect};
use webrender_api::units::{LayoutRect, LayoutSize};

use crate::query::transform_au_rectangle;

pub(crate) struct PaintCandidateCollector {
    /// The LCP candidate, it may be a image or text.
    pub lcp_candidate: Option<LCPCandidate>,
    /// The rect of viewport.
    pub viewport_rect: LayoutRect,
    /// Flag to indicate if there is an update to LCP candidate.
    /// This is used to avoid sending duplicate LCP candidates to `Paint`.
    pub did_lcp_candidate_update: bool,
}

impl PaintCandidateCollector {
    pub fn new(viewport_size: LayoutSize) -> Self {
        Self {
            lcp_candidate: None,
            viewport_rect: LayoutRect::from_size(viewport_size),
            did_lcp_candidate_update: true,
        }
    }

    pub fn candidate_size(
        &self,
        clip_rect: LayoutRect,
        bounds: LayoutRect,
        transform: FastLayoutTransform,
    ) -> usize {
        let clipped_rect = bounds
            .intersection(&clip_rect)
            .unwrap_or(LayoutRect::zero());
        let transformed_rect = transform_au_rectangle(
            f32_rect_to_au_rect(clipped_rect.to_rect().cast_unit()),
            transform,
        )
        .unwrap_or_default();
        let transformed_rect = au_rect_to_f32_rect(transformed_rect);
        let visual_rect = transformed_rect
            .intersection(&self.viewport_rect.to_rect().cast_unit())
            .unwrap_or(Rect::zero());
        let area = visual_rect.size.width * visual_rect.size.height;

        area as usize
    }

    pub fn add_or_update_lcp_candidate(&mut self, candidate: LCPCandidate) {
        if let Some(ref mut latest_candidate) = self.lcp_candidate {
            if candidate.area > latest_candidate.area {
                *latest_candidate = candidate;
                self.did_lcp_candidate_update = true;
            }
        } else {
            self.lcp_candidate = Some(candidate);
        }
    }

    pub fn largest_contentful_paint(&self) -> Option<LCPCandidate> {
        self.lcp_candidate
    }
}
