/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};

use app_units::Au;
use euclid::Rect;
use paint_api::largest_contentful_paint_candidate::{LCPCandidate, LCPCandidateID};
use servo_geometry::FastLayoutTransform;
use servo_url::ServoUrl;
use style::dom::OpaqueNode;
use webrender_api::units::{LayoutRect, LayoutSize};

use crate::fragment_tree::Tag;
use crate::query::transform_f32_rectangle;

/// <https://w3c.github.io/paint-timing/#pending-image-record>
/// Different struct from spec, but fulfulling the same purpose.
struct PendingImageRecord {
    /// The image element this record belongs to.
    /// for <https://w3c.github.io/paint-timing/#pending-image-record-element>
    tag: Option<Tag>,
    /// The image rect (adjusted for object-fit/object-position).
    bounds: LayoutRect,
    /// The element's content box.
    clip_rect: LayoutRect,
    /// Cumulative transform to root space, computed at collection time.
    transform: FastLayoutTransform,
    /// The image URL. `None` for background images.
    url: Option<ServoUrl>,
    /// Intrinsic width, used for upscaling normalization.
    natural_width: Option<Au>,
    /// Intrinsic height, used for upscaling normalization.
    natural_height: Option<Au>,
}

/// <https://w3c.github.io/paint-timing/#sec-recording-paint-timing>
/// > Each Element has a set of owned text nodes, which is an ordered set of
/// > Text nodes, initially empty.
///
/// This struct corresponds to an Element for accumulating set of owned text
/// nodes by nearest ancestor box fragment's tag during display list building.
struct TextRecord {
    /// The tag of containing box fragment these texts belongs to.
    tag: Tag,
    /// <https://w3c.github.io/paint-timing/#set-of-owned-text-nodes>
    /// Collection of border_boxes of all Text nodes accumulated
    border_boxes: Vec<LayoutRect>,
}

enum LCPCandidateType<'a> {
    Image(&'a PendingImageRecord),
    Text,
}

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
    /// The set of image nodes that have been reported as LCP candidates.
    reported_image_nodes: HashSet<OpaqueNode>,
    /// <https://w3c.github.io/paint-timing/#paintedImages>
    painted_images: Vec<PendingImageRecord>,
    /// The set of text nodes that have been reported as LCP candidates.
    reported_text_nodes: HashSet<OpaqueNode>,
    /// <https://w3c.github.io/paint-timing/#paintedTextNodes>
    painted_text_nodes: HashMap<OpaqueNode, TextRecord>,
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
            reported_image_nodes: HashSet::new(),
            painted_images: Vec::new(),
            reported_text_nodes: HashSet::new(),
            painted_text_nodes: HashMap::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn append_image_record(
        &mut self,
        tag: Option<Tag>,
        bounds: LayoutRect,
        clip_rect: LayoutRect,
        transform: FastLayoutTransform,
        url: Option<ServoUrl>,
        natural_width: Option<Au>,
        natural_height: Option<Au>,
    ) {
        self.painted_images.push(PendingImageRecord {
            tag,
            bounds,
            clip_rect,
            transform,
            url,
            natural_width,
            natural_height,
        });
    }

    pub(crate) fn accumulate_text_rect(
        &mut self,
        tag: Tag,
        rect: LayoutRect,
        transform: FastLayoutTransform,
    ) {
        let border_box = transform_f32_rectangle(rect.to_rect(), transform)
            .unwrap_or_default()
            .to_box2d();
        self.painted_text_nodes
            .entry(tag.node)
            .and_modify(|record| record.border_boxes.push(border_box))
            .or_insert(TextRecord {
                tag,
                border_boxes: vec![border_box],
            });
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
        intersection_rect: LayoutRect,
        candidate_type: LCPCandidateType<'_>,
    ) -> Option<f32> {
        // Step 1. Let width be intersectionRect's width, rounded up to the
        // nearest integer.
        // Step 2. Let height be intersectionRect's height, rounded up to the
        // nearest integer.
        // Step 3. Let size be width * height.
        let mut size = intersection_rect.area();

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
        // adjust for image position and upscaling.
        // Note: This is skipped for Text aka the case of null request from specs
        if let LCPCandidateType::Image(record) = candidate_type {
            // TODO Step 8.1: If imageRequest's response's content length in bytes
            // is less than size * 0.004, then return null. (Not Implemented)

            // Step 8.2: Let concreteDimensions be imageRequest's concrete object
            // size within element.
            // Step 8.3: Let visibleDimensions be concreteDimensions, adjusted for
            // positioning by object-position or background-position and element's
            // content box.
            // Note: bounds are already adjusted for positioning and content box
            let visible_dimensions = record
                .bounds
                .intersection(&record.clip_rect)
                .unwrap_or(LayoutRect::zero());

            // Step 8.4: Let clientContentRect be the smallest DOMRectReadOnly
            // containing visibleDimensions with element's transforms applied.
            let client_content_rect =
                transform_f32_rectangle(visible_dimensions.to_rect(), record.transform)
                    .unwrap_or_default();

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
            size = intersecting_client_content_rect.area();

            // Step 8.9: Let naturalArea be imageRequest's natural width * imageRequest's natural height.
            if let (Some(natural_width), Some(natural_height)) =
                (record.natural_width, record.natural_height)
            {
                let natural_area = natural_width.to_f32_px() * natural_height.to_f32_px();

                // Step 8.10: If naturalArea is 0, then return null.
                if natural_area == 0.0 {
                    return None;
                }
                // Step 8.11: Let boundingClientArea be clientContentRect's width *
                // clientContentRect's height.
                let bounding_client_area =
                    client_content_rect.width() * client_content_rect.height();

                // Step 8.12: Let scaleFactor be boundingClientArea / naturalArea.
                let scale_factor = bounding_client_area / natural_area;

                // Step 8.13: If scaleFactor is greater than 1, then divide size by scaleFactor.
                if scale_factor > 1.0 {
                    size /= scale_factor;
                }
            }
        }

        // Step 9: Return an effective visual size result with size set to size,
        // width set to width, and height set to height.
        Some(size)
    }

    /// <https://www.w3.org/TR/largest-contentful-paint/#compute-a-new-largest-contentful-paint-candidate>
    fn compute_new_lcp_candidate(&mut self) {
        // Step 1. Let currentSize be currentCandidate’s size if
        // currentCandidate is not null or 0 otherwise.
        // Step 2. Let largestSize be currentSize.
        let mut largest_size = self.lcp_size;

        // Step 3. Let newCandidate be null.
        let mut new_candidate = None;
        let mut new_candidate_tag = None;

        // Step 4. For each record of paintedImages:
        for record in std::mem::take(&mut self.painted_images) {
            // Step 4.1. Let imageElement be record’s element.

            // TODO Step 4.2. If imageElement is not exposed for paint timing,
            // given document, continue.
            // Step 4.3. Let intersectionRect be the value returned by the
            // intersection rect algorithm using imageElement as the target
            // and viewport as the root.
            let intersection_rect =
                transform_f32_rectangle(record.clip_rect.to_rect(), record.transform)
                    .unwrap_or_default()
                    .intersection(&self.viewport_rect.to_rect())
                    .map(|rect| rect.to_box2d())
                    .unwrap_or_default();

            // Step 4.4. Let result be the effective visual size of imageElement
            // given intersectionRect and record's request.
            let result =
                self.effective_visual_size(intersection_rect, LCPCandidateType::Image(&record));

            // Step 4.5. If result is null, continue.
            let Some(result) = result else {
                continue;
            };
            // Step 4.6. If result's size is less than or equal to
            // largestSize, continue.
            if result <= largest_size {
                continue;
            }

            // Step 4.7. Set largestSize to result’s size.
            largest_size = result;

            // Step 4.8. Set newCandidate to be a new largest contentful paint candidate ...
            let uuid = self.lcp_next_uuid;
            self.lcp_next_uuid += 1;
            new_candidate = Some(LCPCandidate::new(
                LCPCandidateID(uuid),
                result as usize,
                record.url,
            ));
            new_candidate_tag = record.tag;
        }

        // Step 5. For each textNode of paintedTextNodes,
        for (_, record) in std::mem::take(&mut self.painted_text_nodes) {
            // TODO Step 5.1. If textNode is not exposed for paint timing,
            // given document, continue.
            // TODO Step 5.2. If textNode has alpha channel value <=0 or
            // opacity value <=0:
            // Step 5.3. Let intersectionRect be the union of the border boxes of
            // all Text nodes in textNode’s set of owned text nodes,
            // intersected with the visual viewport.
            let intersection_rect = record
                .border_boxes
                .into_iter()
                .reduce(|a, b| a.union(&b))
                .unwrap_or_default()
                .intersection(&self.viewport_rect)
                .unwrap_or_default();
            // Step 5.4. Let result be the effective visual size of textNode
            // given intersectionRect and null.
            let result = self.effective_visual_size(intersection_rect, LCPCandidateType::Text);

            // Step 5.5. If result is null, continue.
            let Some(result) = result else {
                continue;
            };
            // Step 5.6. If result's size is less than or equal to
            // largestSize, continue.
            if result <= largest_size {
                continue;
            }

            // Step 5.7. Set largestSize to result’s size.
            largest_size = result;

            // Step 5.8. Set newCandidate to be a new largest contentful paint candidate ...
            let uuid = self.lcp_next_uuid;
            self.lcp_next_uuid += 1;
            new_candidate = Some(LCPCandidate::new(
                LCPCandidateID(uuid),
                result as usize,
                None,
            ));
            new_candidate_tag = Some(record.tag);
        }

        // Step 6. If newCandidate is not null and currentSize is greater than 0:
        // TODO Step 6.1. If newCandidate’s width minus currentCandidate’s
        // width is less than or equal to 3, and newCandidate’s height minus
        // currentCandidate’s height is less than or equal to 3, return null.
        if new_candidate.is_some() {
            self.lcp_size = largest_size;
            self.lcp_candidate = new_candidate;
            self.lcp_node = new_candidate_tag.map(|tag| tag.node);
            self.lcp_candidate_updated = true;
        }

        // Step 7. Return newCandidate.
        // Note: We use flag lcp_candidate_updated for updating, needs revisit
    }

    /// <https://www.w3.org/TR/paint-timing/#mark-paint-timing>
    pub(crate) fn mark_paint_timing(&mut self) {
        // > From: <https://www.w3.org/TR/largest-contentful-paint/#sec-report-largest-contentful-paint>
        // > Note: Each pending image record in paintedImages and text
        // > element in paintedTextNodes will only be reported exactly
        // > once, from mark paint timing, for the first paint where the
        // > element is considered paintable (i.e. has opacity and
        // > visibility) and contentful (i.e. image resource or blocking
        // > fonts are sufficiently loaded).
        self.painted_images.retain(|record| {
            record
                .tag
                .is_none_or(|tag| self.reported_image_nodes.insert(tag.node))
        });
        self.painted_text_nodes
            .retain(|node, _record| self.reported_text_nodes.insert(*node));

        self.compute_new_lcp_candidate();
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
