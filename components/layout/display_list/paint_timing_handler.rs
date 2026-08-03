/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;

use app_units::Au;
use euclid::Rect;
use paint_api::container_timing_candidate::ContainerTimingRecord;
use paint_api::largest_contentful_paint_candidate::{LCPCandidate, LCPCandidateID};
use rustc_hash::FxHashMap;
use servo_geometry::{FastLayoutTransform, au_rect_to_f32_rect, f32_rect_to_au_rect};
use servo_url::ServoUrl;
use style::dom::OpaqueNode;
use webrender_api::units::{LayoutRect, LayoutSize};

use crate::fragment_tree::Tag;
use crate::query::transform_f32_rectangle;

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
    /// The set of nodes that have been reported as LCP candidates.
    reported_lcp_nodes: HashSet<OpaqueNode>,
    /// Container timing records accumulated during this display list build, keyed by
    /// container root node. Sent to the paint thread at the end of the build.
    container_timing_records: FxHashMap<OpaqueNode, ContainerTimingRecord>,
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
            reported_lcp_nodes: HashSet::new(),
            container_timing_records: FxHashMap::default(),
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

    /// <https://www.w3.org/TR/largest-contentful-paint/#sec-effective-visual-size>
    fn effective_visual_size(
        &self,
        bounds: LayoutRect,
        clip_rect: LayoutRect,
        intersection_rect: LayoutRect,
        transform: FastLayoutTransform,
        natural_width: Option<Au>,
        natural_height: Option<Au>,
    ) -> Option<f32> {
        // Step 1. Let width be intersectionRect's width, rounded up to the
        // nearest integer.
        // Step 2. Let height be intersectionRect's height, rounded up to the
        // nearest integer.
        // Step 3. Let size be width * height.
        let size = intersection_rect.area();

        // Step 4. Let root be document's browsing context's top-level browsing
        // context's active document.
        // Note: This is not needed as we already have the viewport rect.

        // Step 5. Let rootWidth be root's visual viewport's width,
        // excluding any scrollbars.
        // Step 6. Let rootHeight be root's visual viewport's height excluding
        // any scrollbars.
        // Step 7. If size is equal to rootWidth times rootHeight, return null.
        if size >= self.viewport_rect.area() {
            return None;
        }

        // Step 8: If imageRequest is not null, run the following steps to
        // adjust for image position and upscaling:
        // Note: This is handled by check for [showing_broken_image_icon] earlier

        // TODO Step 8.1: If imageRequest's response's content length in bytes
        // is less than size * 0.004, then return null. (Not Implemented)

        // Step 8.2: Let concreteDimensions be imageRequest's concrete object
        // size within element.
        // Step 8.3: Let visibleDimensions be concreteDimensions, adjusted for
        // positioning by object-position or background-position and element's
        // content box.
        // Note: bounds are already adjusted for positioning and content box
        let visible_dimensions = bounds
            .intersection(&clip_rect)
            .unwrap_or(LayoutRect::zero());

        // Step 8.4: Let clientContentRect be the smallest DOMRectReadOnly
        // containing visibleDimensions with element's transforms applied.
        let client_content_rect =
            transform_f32_rectangle(visible_dimensions.to_rect(), transform).unwrap_or_default();

        // Step 8.5: Let intersectingClientContentRect be the intersection of
        // clientContentRect with intersectionRect.
        let intersecting_client_content_rect = client_content_rect
            .intersection(&intersection_rect.to_rect())
            .unwrap_or(Rect::zero());

        // Step 8.6: Set width to intersectingClientContentRect's width,
        // rounded up to the nearest integer.
        // Step 8.7: Set height to intersectingClientContentRect's height,
        // rounded up to the nearest integer.
        // Step 8.8: Set size to width * height.
        let mut size = intersecting_client_content_rect.area();

        // Step 8.9: Let naturalArea be imageRequest's natural width * imageRequest's natural height.
        if let (Some(natural_width), Some(natural_height)) = (natural_width, natural_height) {
            let natural_area = natural_width.to_f32_px() * natural_height.to_f32_px();

            // Step 8.10: If naturalArea is 0, then return null.
            if natural_area == 0.0 {
                return None;
            }
            // Step 8.11: Let boundingClientArea be clientContentRect's width *
            // clientContentRect's height.
            let bounding_client_area = client_content_rect.width() * client_content_rect.height();

            // Step 8.12: Let scaleFactor be boundingClientArea / naturalArea.
            let scale_factor = bounding_client_area / natural_area;

            // Step 8.13: If scaleFactor is greater than 1, then divide size by scaleFactor.
            if scale_factor > 1.0 {
                size /= scale_factor;
            }
        }

        // Step 9: Return an effective visual size result with size set to size,
        // width set to width, and height set to height.
        Some(size)
    }

    /// <https://www.w3.org/TR/largest-contentful-paint/#compute-a-new-largest-contentful-paint-candidate>
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute_new_lcp_candidate(
        &mut self,
        tag: Option<Tag>,
        bounds: LayoutRect,
        clip_rect: LayoutRect,
        transform: FastLayoutTransform,
        url: Option<ServoUrl>,
        natural_width: Option<Au>,
        natural_height: Option<Au>,
    ) {
        // From <https://www.w3.org/TR/largest-contentful-paint/#sec-report-largest-contentful-paint>:
        // Each pending image record in paintedImages and text element in
        // paintedTextNodes will only be reported exactly once, from mark paint
        // timing, for the first paint where the element is considered
        // paintable (i.e. has opacity and visibility) and contentful
        // (i.e. image resource or blocking fonts are sufficiently loaded).
        if let Some(node) = tag.map(|tag| tag.node) &&
            !self.reported_lcp_nodes.insert(node)
        {
            return;
        }

        // Step 4.1. Let imageElement be record’s element.
        // TODO Step 4.2. If imageElement is not exposed for paint timing, given
        // document, continue.
        // Note: We are only dealing with images for now.

        // Step 4.3. Let intersectionRect be the value returned by the intersection rect
        // algorithm using imageElement as the target and viewport as the root.
        let intersection_rect = transform_f32_rectangle(clip_rect.to_rect(), transform)
            .unwrap_or_default()
            .intersection(&self.viewport_rect.to_rect())
            .map(|rect| rect.to_box2d())
            .unwrap_or_default();

        // Step 4.4. Let result be the effective visual size of imageElement
        // given intersectionRect and record’s request.
        let result = self.effective_visual_size(
            bounds,
            clip_rect,
            intersection_rect,
            transform,
            natural_width,
            natural_height,
        );

        // Step 4.5. If result is null, continue.
        let Some(result) = result else {
            return;
        };

        // Step 4.6. If result’s size is less than or equal to largestSize, continue.
        if result <= self.lcp_size {
            return;
        }

        // Step 4.7. Set largestSize to result’s size.
        self.lcp_size = result;

        let uuid = self.lcp_next_uuid;
        self.lcp_next_uuid += 1;
        self.lcp_node = tag.map(|tag| tag.node);

        // Step 4.8. Set newCandidate to be a new largest contentful paint candidate ...
        self.lcp_candidate = Some(LCPCandidate::new(
            LCPCandidateID(uuid),
            self.lcp_size as usize,
            url,
        ));

        self.lcp_candidate_updated = true;
    }

    /// Record a container timing candidate for a painted text or image fragment.
    /// Walks ancestors to find the container root; detached nodes (no root) are ignored.
    /// <https://wicg.github.io/container-timing/>
    pub(crate) fn update_container_timing(
        &mut self,
        node: OpaqueNode,
        bounds: LayoutRect,
        clip_rect: LayoutRect,
        transform: FastLayoutTransform,
    ) {
        let Some(container_root) = get_container_root(node) else {
            return;
        };

        let intersection_rect = self.calculate_intersection_rect(bounds, clip_rect, transform);

        let size = (intersection_rect.size.width * intersection_rect.size.height).round() as usize;
        if size == 0 {
            return;
        }

        let record = self.get_or_create_record(container_root);
        record.size += size;
        let (new_x, new_y, new_w, new_h) = (
            intersection_rect.origin.x,
            intersection_rect.origin.y,
            intersection_rect.size.width,
            intersection_rect.size.height,
        );
        if record.rect_width == 0.0 && record.rect_height == 0.0 {
            record.rect_x = new_x;
            record.rect_y = new_y;
            record.rect_width = new_w;
            record.rect_height = new_h;
        } else {
            let min_x = record.rect_x.min(new_x);
            let min_y = record.rect_y.min(new_y);
            let max_x = (record.rect_x + record.rect_width).max(new_x + new_w);
            let max_y = (record.rect_y + record.rect_height).max(new_y + new_h);
            record.rect_x = min_x;
            record.rect_y = min_y;
            record.rect_width = max_x - min_x;
            record.rect_height = max_y - min_y;
        }
    }

    /// Returns the entry for `container_root`, creating a default one if absent.
    fn get_or_create_record(&mut self, container_root: OpaqueNode) -> &mut ContainerTimingRecord {
        self.container_timing_records
            .entry(container_root)
            .or_insert_with(|| {
                let identifier =
                    script::layout_dom::container_timing_identifier_for_root(container_root)
                        .map(|id| id.to_string())
                        .unwrap_or_default();
                ContainerTimingRecord::new(identifier, 0, 0.0, 0.0, 0.0, 0.0)
            })
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
    /// Returns accumulated container timing candidates and clears the internal map.
    pub(crate) fn take_container_timing_candidates(&mut self) -> Vec<ContainerTimingRecord> {
        self.container_timing_records
            .drain()
            .map(|(_, record)| record)
            .collect()
    }
}

/// Walk DOM ancestors of `node` to find the nearest element with a `containertiming`
/// attribute. Returns `None` for detached nodes (no document root in the ancestor chain).
/// Equivalent to Chromium's `getContainerRoot`.
/// <https://wicg.github.io/container-timing/>
fn get_container_root(node: OpaqueNode) -> Option<OpaqueNode> {
    script::layout_dom::container_timing_root_for_node(node)
}
