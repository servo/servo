/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};

use app_units::Au;
use euclid::Rect;
use paint_api::display_list::{PaintTimingReport, ScrollTree};
use paint_api::largest_contentful_paint_candidate::{LCPCandidate, LCPCandidateID};
use servo_base::id::ScrollTreeNodeId;
use servo_config::pref;
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
    /// The spatial node of the image, used to compute the cumulative transform
    /// lazily when the candidate is computed.
    spatial_id: ScrollTreeNodeId,
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
    /// The spatial node of the text, used to compute the cumulative transform
    /// lazily when the candidate is computed.
    spatial_id: ScrollTreeNodeId,
    /// <https://w3c.github.io/paint-timing/#set-of-owned-text-nodes>
    /// Collection of border_boxes of all Text nodes accumulated
    border_boxes: Vec<LayoutRect>,
}

enum LCPCandidateType<'a> {
    Image(&'a PendingImageRecord, FastLayoutTransform),
    Text,
}

pub(crate) struct PaintTimingHandler {
    /// The rect of viewport.
    viewport_rect: LayoutRect,
    /// Whether the current display list contains paintable items.
    is_document_paintable: bool,
    /// Whether the current display list contains contentful items.
    is_document_contentful: bool,
    /// <https://www.w3.org/TR/paint-timing/#set-of-previously-reported-paints>
    reported_paints: PaintTimingReport,
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
    /// <https://www.w3.org/TR/paint-timing/#images-pending-rendering>
    images_pending_rendering: Vec<PendingImageRecord>,
    /// <https://www.w3.org/TR/paint-timing/#set-of-elements-with-rendered-text>
    /// The set of text nodes that have been reported as LCP candidates.
    elements_with_rendered_text: HashSet<OpaqueNode>,
    /// The set of pending text nodes that will fight for LCP candidate.
    elements_with_pending_rendered_text: HashMap<OpaqueNode, TextRecord>,
}

impl PaintTimingHandler {
    pub(crate) fn new(viewport_size: LayoutSize) -> Self {
        Self {
            is_document_paintable: false,
            is_document_contentful: false,
            reported_paints: PaintTimingReport::default(),
            viewport_rect: LayoutRect::from_size(viewport_size),
            lcp_next_uuid: 0,
            lcp_node: None,
            lcp_candidate: None,
            lcp_candidate_updated: false,
            reported_image_nodes: HashSet::new(),
            images_pending_rendering: Vec::new(),
            elements_with_rendered_text: HashSet::new(),
            elements_with_pending_rendered_text: HashMap::new(),
        }
    }

    /// Marks the current display list as containing a paintable item.
    pub(crate) fn mark_document_is_paintable(&mut self) {
        self.is_document_paintable = true;
    }

    /// Marks the current display list as containing a contentful item.
    pub(crate) fn mark_document_is_contentful(&mut self) {
        self.is_document_contentful = true;
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn append_image_record(
        &mut self,
        tag: Option<Tag>,
        bounds: LayoutRect,
        clip_rect: LayoutRect,
        spatial_id: ScrollTreeNodeId,
        url: Option<ServoUrl>,
        natural_width: Option<Au>,
        natural_height: Option<Au>,
    ) {
        // From <https://www.w3.org/TR/paint-timing/#contentful>:
        // An element target is contentful when one or more of the following apply:
        // > target is a replaced element representing an available image.
        self.mark_document_is_contentful();

        // Skip pushing records if LargestContentfulPaint is disabled
        if !pref!(largest_contentful_paint_enabled) {
            return;
        }

        self.images_pending_rendering.push(PendingImageRecord {
            tag,
            bounds,
            clip_rect,
            spatial_id,
            url,
            natural_width,
            natural_height,
        });
    }

    pub(crate) fn accumulate_text_rect(
        &mut self,
        containing_element_tag: Option<Tag>,
        rect: LayoutRect,
        spatial_id: ScrollTreeNodeId,
    ) {
        // From <https://www.w3.org/TR/paint-timing/#contentful>:
        // An element target is contentful when one or more of the following apply:
        // > target has a text node child, representing non-empty text, and the
        // > node’s used opacity is greater than zero.
        self.mark_document_is_contentful();

        // Skip pushing records if LargestContentfulPaint is disabled
        if !pref!(largest_contentful_paint_enabled) {
            return;
        }

        // `containing_element_tag` is manadatory for union of TextNodes
        let Some(tag) = containing_element_tag else {
            return;
        };
        self.elements_with_pending_rendered_text
            .entry(tag.node)
            .and_modify(|record| {
                record.border_boxes.push(rect);
            })
            .or_insert(TextRecord {
                tag,
                spatial_id,
                border_boxes: vec![rect],
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
        if let LCPCandidateType::Image(record, transform) = candidate_type {
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
                transform_f32_rectangle(visible_dimensions.to_rect(), transform)
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
    #[servo_tracing::instrument(
        name = "Compute New LCP Candidate",
        skip_all,
        fields(
            image_count = painted_images.len(),
            text_count = painted_text_nodes.len(),
        )
    )]
    fn compute_new_lcp_candidate(
        &mut self,
        scroll_tree: &ScrollTree,
        painted_images: Vec<PendingImageRecord>,
        painted_text_nodes: HashMap<OpaqueNode, TextRecord>,
    ) -> Option<LCPCandidate> {
        // Step 1. Let currentSize be currentCandidate’s size if
        // currentCandidate is not null or 0 otherwise.
        // Step 2. Let largestSize be currentSize.
        let mut largest_size = self
            .lcp_candidate
            .as_ref()
            .map_or(0.0, |candidate| candidate.area as f32);

        // Step 3. Let newCandidate be null.
        let mut new_candidate = None;
        let mut new_candidate_tag = None;

        // Step 4. For each record of paintedImages:
        for record in painted_images {
            // Step 4.1. Let imageElement be record’s element.

            // Step 4.2. If imageElement is not exposed for paint timing, given
            // document, continue.
            // Note: Satisfied, as the display-list builder only visits the
            // connected DOM tree of the fully-active document being laid out.

            // Step 4.3. Let intersectionRect be the value returned by the
            // intersection rect algorithm using imageElement as the target
            // and viewport as the root.
            let transform = scroll_tree.cumulative_node_to_root_transform(record.spatial_id);
            let intersection_rect = transform_f32_rectangle(record.clip_rect.to_rect(), transform)
                .unwrap_or_default()
                .intersection(&self.viewport_rect.to_rect())
                .map(|rect| rect.to_box2d())
                .unwrap_or_default();

            // Step 4.4. Let result be the effective visual size of imageElement
            // given intersectionRect and record's request.
            let result = self.effective_visual_size(
                intersection_rect,
                LCPCandidateType::Image(&record, transform),
            );

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
        for (_, record) in painted_text_nodes {
            // Step 5.1. If textNode is not exposed for paint timing, given
            // document, continue.
            // Note: Satisfied, as the display-list builder only visits the
            // connected DOM tree of the fully-active document being laid out.

            // TODO Step 5.2. If textNode has alpha channel value <=0 or opacity
            // value <=0, continue.
            // Step 5.3. Let intersectionRect be the union of the border boxes of
            // all Text nodes in textNode’s set of owned text nodes,
            // intersected with the visual viewport.
            let transform = scroll_tree.cumulative_node_to_root_transform(record.spatial_id);
            let intersection_rect = record
                .border_boxes
                .into_iter()
                .map(|border_box| {
                    transform_f32_rectangle(border_box.to_rect(), transform)
                        .unwrap_or_default()
                        .to_box2d()
                })
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
        if let Some(ref candidate) = new_candidate {
            self.lcp_candidate = Some(candidate.clone());
            // TODO: Append NodeAddress in LCPCandidate struct
            self.lcp_node = new_candidate_tag.map(|tag| tag.node);
        }

        // Step 7. Return newCandidate.
        new_candidate
    }

    /// <https://www.w3.org/TR/largest-contentful-paint/#sec-report-largest-contentful-paint>
    fn report_largest_contentful_paint(
        &mut self,
        halt_lcp: bool,
        scroll_tree: &ScrollTree,
        painted_images: Vec<PendingImageRecord>,
        painted_text_nodes: HashMap<OpaqueNode, TextRecord>,
    ) {
        // > Note: Each pending image record in paintedImages and text
        // > element in paintedTextNodes will only be reported exactly
        // > once, from mark paint timing, for the first paint where the
        // > element is considered paintable (i.e. has opacity and
        // > visibility) and contentful (i.e. image resource or blocking
        // > fonts are sufficiently loaded).

        // Step 1. Let window be document’s relevant global object.
        // Step 2. If either of window’s has dispatched scroll event or has
        // dispatched input event is true, return.
        if halt_lcp || !pref!(largest_contentful_paint_enabled) {
            return;
        }

        // Step 3. Let newCandidate be the result of computing a new largest
        // contentful paint candidate given document, paintedImages,
        // paintedTextNodes, and document’s current largest contentful paint
        // candidate.
        let new_candidate =
            self.compute_new_lcp_candidate(scroll_tree, painted_images, painted_text_nodes);

        // Step 4. If newCandidate is null, return.
        if new_candidate.is_none() {
            return;
        }
        // Step 5. Set document’s current largest contentful paint candidate to
        // newCandidate.
        self.lcp_candidate_updated = true;

        // Step 6. Let entry be the result of creating a LargestContentfulPaint
        // entry with newCandidate, paintTimingInfo, and document.
        // Step 7. Queue the PerformanceEntry entry.
        // Note: Step 6-7 are handled in script.
    }

    /// <https://www.w3.org/TR/paint-timing/#first-paint>
    fn should_report_first_paint(&self) -> bool {
        // Step 1. If document's set of previously reported paints contains
        // "first-paint", then return false.
        if self.reported_paints.first_paint {
            return false;
        }
        // Step 2. If document contains at least one element that is
        // paintable, then return true.
        // Step 3. Otherwise, return false.
        self.is_document_paintable
    }

    /// <https://www.w3.org/TR/paint-timing/#first-contentful-paint>
    fn should_report_first_contentful_paint(&self) -> bool {
        // Step 1. If document's set of previously reported paints contains
        // "first-contentful-paint", then return false.
        if self.reported_paints.first_contentful_paint {
            return false;
        }
        // Step 2. If document contains at least one element that is both
        // paintable and contentful, then return true.
        // Step 3. Otherwise, return false.
        self.is_document_paintable && self.is_document_contentful
    }

    /// <https://www.w3.org/TR/paint-timing/#mark-paint-timing>
    #[servo_tracing::instrument(name = "Mark Paint Timing", skip_all, fields(halt_lcp = halt_lcp))]
    pub(crate) fn mark_paint_timing(
        &mut self,
        halt_lcp: bool,
        scroll_tree: &ScrollTree,
    ) -> PaintTimingReport {
        // TODO Step 1. If the document's browsing context is not paint-timing
        // eligible, return.

        // TODO Step 2. Let paintTimingInfo be a new paint timing info, whose
        // rendering update end time is the current high resolution time given
        // document's relevant global object.

        // Step 3. Let paintedImages be a new ordered set.
        // Step 4. Let paintedTextNodes be a new ordered set.

        // Step 5. For each record in doc's images pending rendering list:
        // Step 5.1. If record's request is available and ready to be painted,
        // then run the following steps:
        // Note: Only available images are accumulated, hence it is fulfilled.
        // Step 5.1.1. Append record to paintedImages.
        // Step 5.1.2. Remove record from doc's images pending rendering list.
        let painted_images: Vec<_> = std::mem::take(&mut self.images_pending_rendering)
            .into_iter()
            .filter(|record| {
                record
                    .tag
                    .is_none_or(|tag| self.reported_image_nodes.insert(tag.node))
            })
            .collect();

        // Step 6. For each Element element in doc's descendants:
        // Step 6.1. If element is contained in doc's set of elements with
        // rendered text, continue.
        // Step 6.2. If element's set of owned text nodes is empty, continue.
        // Step 6.3. Append element to doc's set of elements with rendered text.
        // Step 6.4. Append element to paintedTextNodes.
        let painted_text_nodes: HashMap<_, _> =
            std::mem::take(&mut self.elements_with_pending_rendered_text)
                .into_iter()
                .filter(|(node, _record)| self.elements_with_rendered_text.insert(*node))
                .collect();

        // Step 7. Let reportedPaints be the document’s set of previously
        // reported paints. (Directly accessing)

        // TODO Step 8. Let frameTimingInfo be document’s current frame timing info.
        // TODO Step 9. Set document’s current frame timing info to null.

        // Step 10. Let flushPaintTimings be the following steps:
        // Step 10.1. If document should report first paint, then:
        let first_paint = self.should_report_first_paint();
        // Step 10.2.1. Report paint timing given document, "first-paint",
        // and paintTimingInfo.
        if first_paint {
            self.reported_paints.first_paint = true;
        }

        // Step 10.2. If document should report first contentful paint, then:
        let first_contentful_paint = self.should_report_first_contentful_paint();
        // Step 10.2.1. Report paint timing given document,
        // "first-contentful-paint", and paintTimingInfo.
        if first_contentful_paint {
            self.reported_paints.first_contentful_paint = true;
        }

        // Step 10.3. Report largest contentful paint given document,
        // paintTimingInfo, paintedImages and paintedTextNodes.
        self.report_largest_contentful_paint(
            halt_lcp,
            scroll_tree,
            painted_images,
            painted_text_nodes,
        );

        PaintTimingReport {
            first_paint,
            first_contentful_paint,
        }
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
