/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::Rect;
use paint_api::largest_contentful_paint_candidate::{LCPCandidate, LCPCandidateID};
use servo_geometry::{FastLayoutTransform, au_rect_to_f32_rect, f32_rect_to_au_rect};
use servo_url::ServoUrl;
use style::dom::OpaqueNode;
use style_traits::CSSPixel;
use webrender_api::units::{LayoutRect, LayoutSize};

use crate::fragment_tree::Tag;
use crate::query::transform_au_rectangle;

pub(crate) struct PaintTimingHandler {
    /// The rect of viewport.
    viewport_rect: LayoutRect,
    /// The document’s largest contentful paint size
    lcp_size: f32,
    /// Counter for generating unique LCP candidate UUIDs.
    lcp_next_uuid: u64,
    /// The LCP candidate, it may be a image or text.
    lcp_candidate: Option<LCPCandidate>,
    /// The DOM node for the LCP candidate. Only used in ReflowResult
    lcp_node: Option<OpaqueNode>,
    /// Flag to indicate if there is an update to LCP candidate.
    /// This is used to avoid sending duplicate LCP candidates to `Paint`.
    lcp_candidate_updated: bool,
}

impl PaintTimingHandler {
    pub(crate) fn new(viewport_size: LayoutSize) -> Self {
        Self {
            lcp_size: 0.0,
            lcp_next_uuid: 0,
            lcp_node: None,
            lcp_candidate: None,
            lcp_candidate_updated: false,
            viewport_rect: LayoutRect::from_size(viewport_size),
        }
    }

    // Returns true if has non-zero width and height values.
    pub(crate) fn check_bounding_rect(&self, bounds: LayoutRect, clip_rect: LayoutRect) -> bool {
        let clipped_rect = bounds
            .intersection(&clip_rect)
            .unwrap_or(LayoutRect::zero())
            .to_rect();

        let bounding_rect = clipped_rect
            .intersection(&self.viewport_rect.to_rect().cast_unit())
            .unwrap_or(Rect::zero());

        !bounding_rect.is_empty()
    }

    fn calculate_intersection_rect(
        &self,
        bounds: LayoutRect,
        clip_rect: LayoutRect,
        transform: FastLayoutTransform,
    ) -> Rect<f32, CSSPixel> {
        let clipped_rect = bounds
            .intersection(&clip_rect)
            .unwrap_or(LayoutRect::zero());

        let transformed_rect = transform_au_rectangle(
            f32_rect_to_au_rect(clipped_rect.to_rect().cast_unit()),
            transform,
        )
        .unwrap_or_default();

        let transformed_rect = au_rect_to_f32_rect(transformed_rect);

        let intersection_rect =
            transformed_rect.intersection(&self.viewport_rect.to_rect().cast_unit());

        intersection_rect.unwrap_or(Rect::zero())
    }

    pub(crate) fn update_lcp_candidate(
        &mut self,
        tag: Option<Tag>,
        bounds: LayoutRect,
        clip_rect: LayoutRect,
        transform: FastLayoutTransform,
        url: Option<ServoUrl>,
    ) {
        // From <https://www.w3.org/TR/largest-contentful-paint/#sec-report-largest-contentful-paint>:
        //  Let intersectionRect be the value returned by the intersection rect algorithm using imageElement as the target and viewport as the root.
        let intersection_rect = self.calculate_intersection_rect(bounds, clip_rect, transform);

        // Let size be the effective visual size of candidate’s element given intersectionRect.
        let size = intersection_rect.size.width * intersection_rect.size.height;

        // If size is less than or equal to document’s largest contentful paint size, return.
        if size <= self.lcp_size {
            return;
        }

        let uuid = self.lcp_next_uuid;
        self.lcp_next_uuid += 1;

        self.lcp_size = size;
        self.lcp_node = tag.map(|tag| tag.node);
        self.lcp_candidate = Some(LCPCandidate::new(LCPCandidateID(uuid), size as usize, url));

        self.lcp_candidate_updated = true;
    }

    pub(crate) fn did_lcp_candidate_update(&self) -> bool {
        self.lcp_candidate_updated
    }

    pub(crate) fn unset_lcp_candidate_updated(&mut self) {
        self.lcp_candidate_updated = false;
    }

    pub(crate) fn largest_contentful_paint_candidate(&self) -> Option<LCPCandidate> {
        self.lcp_candidate.clone()
    }

    pub(crate) fn lcp_node(&self) -> Option<OpaqueNode> {
        self.lcp_node
    }
}
