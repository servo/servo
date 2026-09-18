/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};

use app_units::Au;
use euclid::Rect;
use layout_api::{ContainerTimingRecord, LCPCandidate};
use paint_api::display_list::PaintTimingReport;
use rustc_hash::{FxHashMap, FxHashSet};
use servo_arc::Arc as ServoArc;
use servo_base::id::{ContainerTimingID, LCPCandidateID};
use servo_geometry::FastLayoutTransform;
use servo_url::ServoUrl;
use style::dom::OpaqueNode;
use style::properties::ComputedValues;
use webrender_api::units::{LayoutRect, LayoutSize};

use super::painted_region::PaintedRegion;
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
    /// The containing element's computed style
    style: ServoArc<ComputedValues>,
}

enum LCPCandidateType<'a> {
    Image(&'a PendingImageRecord),
    Text,
}

/// The Container Timing state accumulated for one container root, i.e. one element
/// carrying a `containertiming` attribute.
///
/// <https://wicg.github.io/container-timing/>
struct ContainerRecord {
    /// The value of the container's `containertiming` attribute.
    identifier: String,
    /// Descendant nodes that have already contributed painted area to this container, at
    /// any point in the past. Once a node is in this set it is never reconsidered
    /// TODO: This algorithm could change in future https://github.com/WICG/container-timing/issues/72
    contributing_nodes: FxHashSet<OpaqueNode>,
    /// Everything painted inside this container so far, de-overlapped. Only ever grows by
    /// merging in a genuinely new node's region (see `contributing_nodes`); persists across
    painted_region: PaintedRegion,
    /// The descendant whose paint most recently grew `painted_region`, for
    /// `PerformanceContainerTiming::lastPaintedElement`.
    last_painted_element: Option<OpaqueNode>,
}

/// Container Timing state accumulated for one container root during the display list
/// build currently in progress. Its later added to ContainerTiming Record, see
/// [`PaintTimingHandler::finalize_container_timing`].
#[derive(Default)]
struct PendingContainer {
    /// Every fragment touched this build, unioned per node
    node_regions: FxHashMap<OpaqueNode, PaintedRegion>,
    /// Nodes touched this build, in the order the display list traversal visited them.
    /// Only used so that, if this build does grow the container, `last_painted_element`
    /// is deterministic.
    /// TODO: This will change: https://github.com/WICGcontainer-timing/issues/53
    touched_order: Vec<OpaqueNode>,
}

pub(crate) struct PaintTimingHandler {
    /// The rect of viewport.
    viewport_rect: LayoutRect,
    /// Whether the current display list contains paintable items.
    is_document_paintable: bool,
    /// Whether the current display list contains contentful items.
    is_document_contentful: bool,
    /// <https://www.w3.org/TR/paint-timing/#set-of-previously-reported-paints>
    previously_reported_paints: PaintTimingReport,
    /// Counter for generating unique LCP candidate UUIDs.
    lcp_next_uuid: u64,
    /// The LCP candidate, it may be a image or text.
    lcp_candidate: Option<LCPCandidate>,
    /// The set of image nodes that have been reported as LCP candidates.
    reported_image_nodes: HashSet<OpaqueNode>,
    /// <https://www.w3.org/TR/paint-timing/#images-pending-rendering>
    images_pending_rendering: Vec<PendingImageRecord>,
    /// <https://www.w3.org/TR/paint-timing/#set-of-elements-with-rendered-text>
    /// The set of text nodes that have been reported as LCP candidates.
    elements_with_rendered_text: HashSet<OpaqueNode>,
    /// The set of pending text nodes that will fight for LCP candidate.
    elements_with_pending_rendered_text: HashMap<OpaqueNode, TextRecord>,
    /// Counter for generating unique container timing UUIDs.
    container_timing_next_uuid: u64,
    /// Accumulated Container Timing state, keyed by container root node. Persists
    /// across display list builds, like the LCP candidate does.
    container_timing_records: FxHashMap<OpaqueNode, ContainerRecord>,
    /// Container Timing state for the display list build currently in progress, keyed by
    /// container root node. Reset every build by [`Self::finalize_container_timing`].
    container_timing_pending: FxHashMap<OpaqueNode, PendingContainer>,
    /// Container roots whose painted area grew during the display list build currently
    /// in progress. Drained by [`Self::take_container_timing_records`] once the build
    /// finishes and the records are handed to script.
    dirty_containers: FxHashSet<OpaqueNode>,
}

impl PaintTimingHandler {
    pub(crate) fn new(viewport_size: LayoutSize) -> Self {
        Self {
            is_document_paintable: false,
            is_document_contentful: false,
            previously_reported_paints: PaintTimingReport::default(),
            lcp_next_uuid: 0,
            lcp_candidate: None,
            viewport_rect: LayoutRect::from_size(viewport_size),
            reported_image_nodes: HashSet::new(),
            images_pending_rendering: Vec::new(),
            elements_with_rendered_text: HashSet::new(),
            elements_with_pending_rendered_text: HashMap::new(),
            container_timing_next_uuid: 0,
            container_timing_records: FxHashMap::default(),
            container_timing_pending: FxHashMap::default(),
            dirty_containers: FxHashSet::default(),
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
        transform: FastLayoutTransform,
        url: Option<ServoUrl>,
        natural_width: Option<Au>,
        natural_height: Option<Au>,
    ) {
        self.images_pending_rendering.push(PendingImageRecord {
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
        style: &ServoArc<ComputedValues>,
    ) {
        let border_box = transform_f32_rectangle(rect.to_rect(), transform)
            .unwrap_or_default()
            .to_box2d();
        self.elements_with_pending_rendered_text
            .entry(tag.node)
            .and_modify(|record| {
                record.border_boxes.push(border_box);
            })
            .or_insert(TextRecord {
                tag,
                border_boxes: vec![border_box],
                style: ServoArc::clone(style),
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
                record.tag.map(|tag| tag.node),
            ));
        }

        // Step 5. For each textNode of paintedTextNodes,
        for (_, record) in painted_text_nodes {
            // Step 5.1. If textNode is not exposed for paint timing, given
            // document, continue.
            // Note: Satisfied, as the display-list builder only visits the
            // connected DOM tree of the fully-active document being laid out.

            // Step 5.2. If textNode has alpha channel value <=0 or opacity
            // value <=0:
            if record.style.clone_color().alpha <= 0.0 || record.style.clone_opacity() <= 0.0 {
                // Step 5.2.1. If textNode's text-shadow value is none,
                // textNode's stroke-color value is transparent and textNode's
                // stroke-image value is none, continue.
                // TODO: Update when we implement the `stroke-color`/`stroke-image`
                // properties, as of now they are default (`transparent`/`none`)
                if record.style.get_inherited_text().text_shadow.0.is_empty() {
                    continue;
                }
            }
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
                Some(record.tag.node),
            ));
        }

        // TODO Step 6. If newCandidate is not null and currentSize is greater than 0:
        // TODO Step 6.1. If newCandidate’s width minus currentCandidate’s
        // width is less than or equal to 3, and newCandidate’s height minus
        // currentCandidate’s height is less than or equal to 3, return null.

        // Step 7. Return newCandidate.
        new_candidate
    }

    /// <https://www.w3.org/TR/largest-contentful-paint/#sec-report-largest-contentful-paint>
    fn report_largest_contentful_paint(
        &mut self,
        halt_lcp: bool,
        painted_images: Vec<PendingImageRecord>,
        painted_text_nodes: HashMap<OpaqueNode, TextRecord>,
    ) {
        // Step 1. Let window be document’s relevant global object.
        // Step 2. If either of window’s has dispatched scroll event or has
        // dispatched input event is true, return.
        if halt_lcp {
            return;
        }

        // Step 3. Let newCandidate be the result of computing a new largest
        // contentful paint candidate given document, paintedImages,
        // paintedTextNodes, and document’s current largest contentful paint
        // candidate.
        self.lcp_candidate = self.compute_new_lcp_candidate(painted_images, painted_text_nodes);

        // Step 4. If newCandidate is null, return.
        // Step 5. Set document’s current largest contentful paint candidate to
        // newCandidate.
        // Note: Step 4-5 are fulfilled by the assignment above, as the new
        // candidate wether `None or Some` is computed and assigned to the
        // current candidate.

        // Step 6. Let entry be the result of creating a LargestContentfulPaint
        // entry with newCandidate, paintTimingInfo, and document.
        // Step 7. Queue the PerformanceEntry entry.
        // Note: Step 6-7 are handled in script.
    }

    /// Accumulate a painted text or image fragment into the Container Timing state of
    /// its nearest ancestor carrying a `containertiming` attribute, if any.
    ///
    /// Called during display list building, unlike the LCP collection which only
    /// accumulates here and computes in [`Self::mark_paint_timing`]. Container Timing has
    /// no per-frame winner to pick: each fragment either adds new painted area to its
    /// container or it does not, so the work can be done as fragments are visited.
    ///
    /// https://wicg.github.io/container-timing/#maybe-update-last-new-painted-area
    #[servo_tracing::instrument(name = "Update Container Timing", skip_all)]
    pub(crate) fn update_container_timing(
        &mut self,
        node: OpaqueNode,
        bounds: LayoutRect,
        clip_rect: LayoutRect,
        transform: FastLayoutTransform,
        natural_width: Option<Au>,
        natural_height: Option<Au>,
    ) {
        let Some(container_root) = script::layout_dom::container_timing_root_for_node(node) else {
            return;
        };

        let intersection_rect = transform_f32_rectangle(clip_rect.to_rect(), transform)
            .unwrap_or_default()
            .intersection(&self.viewport_rect.to_rect())
            .map(|rect| rect.to_box2d())
            .unwrap_or_default();

        // Gate on the same "is this fragment eligible to be reported at all" check LCP
        // uses.
        let candidate_type = PendingImageRecord {
            tag: None,
            bounds,
            clip_rect,
            transform,
            url: None,
            natural_width,
            natural_height,
        };
        let candidate_type = match (natural_width, natural_height) {
            (Some(_), Some(_)) => LCPCandidateType::Image(&candidate_type),
            _ => LCPCandidateType::Text,
        };
        if self
            .effective_visual_size(intersection_rect, candidate_type)
            .is_none()
        {
            return;
        }

        let pending = self
            .container_timing_pending
            .entry(container_root)
            .or_default();
        if !pending.node_regions.contains_key(&node) {
            pending.touched_order.push(node);
        }
        pending
            .node_regions
            .entry(node)
            .or_default()
            .union(intersection_rect);
    }

    /// Decide, once per display list build, which containers touched by
    /// [`Self::update_container_timing`] this build actually grew.
    ///
    /// A node only counts the first time it is ever seen contributing to a container
    /// (tracked by [`ContainerRecord::contributing_nodes`]);
    fn finalize_container_timing(&mut self) {
        for (container_root, pending) in std::mem::take(&mut self.container_timing_pending) {
            let record = self
                .container_timing_records
                .entry(container_root)
                .or_insert_with(|| ContainerRecord {
                    identifier: script::layout_dom::container_timing_identifier_for_root(
                        container_root,
                    )
                    .map(|identifier| identifier.to_string())
                    .unwrap_or_default(),
                    contributing_nodes: FxHashSet::default(),
                    painted_region: PaintedRegion::default(),
                    last_painted_element: None,
                });

            let mut grew = false;
            for node in pending.touched_order {
                if !record.contributing_nodes.insert(node) {
                    continue;
                }
                record.painted_region.merge(&pending.node_regions[&node]);
                record.last_painted_element = Some(node);
                grew = true;
            }

            if grew {
                self.dirty_containers.insert(container_root);
            }
        }
    }

    pub(crate) fn take_container_timing_records(&mut self) -> Vec<ContainerTimingRecord> {
        self.finalize_container_timing();
        let mut records = Vec::with_capacity(self.dirty_containers.len());
        for container_root in std::mem::take(&mut self.dirty_containers) {
            let Some(record) = self.container_timing_records.get(&container_root) else {
                continue;
            };
            let id = ContainerTimingID(self.container_timing_next_uuid);
            self.container_timing_next_uuid += 1;
            records.push(ContainerTimingRecord {
                id,
                container_id: container_root.id(),
                identifier: record.identifier.clone(),
                size: record.painted_region.area(),
                intersection_rect: record.painted_region.bounds(),
                root_element: container_root,
                last_painted_element: record.last_painted_element,
            });
        }
        records
    }

    /// <https://www.w3.org/TR/paint-timing/#first-paint>
    fn should_report_first_paint(&self) -> bool {
        // Step 1. If document's set of previously reported paints contains
        // "first-paint", then return false.
        if self
            .previously_reported_paints
            .contains(PaintTimingReport::FirstPaint)
        {
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
        if self
            .previously_reported_paints
            .contains(PaintTimingReport::FirstContentfulPaint)
        {
            return false;
        }
        // Step 2. If document contains at least one element that is both
        // paintable and contentful, then return true.
        // Step 3. Otherwise, return false.
        self.is_document_paintable && self.is_document_contentful
    }

    /// <https://www.w3.org/TR/paint-timing/#mark-paint-timing>
    ///
    /// Note: Step 10 is not in the current version of the specifications.
    /// Refer <https://github.com/w3c/paint-timing/issues/122> for details on
    /// the issue and for the modified steps yet to be merged.
    #[servo_tracing::instrument(name = "Mark Paint Timing", skip_all, fields(halt_lcp = halt_lcp))]
    pub(crate) fn mark_paint_timing(&mut self, halt_lcp: bool) -> PaintTimingReport {
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

        // Note: A new PaintTimingReport to accumulate the paints.
        let mut paint_timing_report = PaintTimingReport::default();

        // Step 10.1. If document should report first paint, then:
        // Note: This step is not in the current version of the specifications,
        // are waiting to be merged at w3c/paint-timing#123.
        if self.should_report_first_paint() {
            // Step 10.1.1. Report paint timing given document, "first-paint",
            // and paintTimingInfo.
            paint_timing_report.insert(PaintTimingReport::FirstPaint);
        }

        // Step 10.2. If document should report first contentful paint, then:
        if self.should_report_first_contentful_paint() {
            // Step 10.2.1. Report paint timing given document,
            // "first-contentful-paint", and paintTimingInfo.
            paint_timing_report.insert(PaintTimingReport::FirstContentfulPaint);
        }

        // Step 10.3. Report largest contentful paint given document,
        // paintTimingInfo, paintedImages and paintedTextNodes.
        self.report_largest_contentful_paint(halt_lcp, painted_images, painted_text_nodes);

        // Note: Append the newly reported paints aka [`PaintTimingReport`] to
        // the document's set of previously reported paints.
        self.previously_reported_paints |= paint_timing_report;

        paint_timing_report
    }

    pub(crate) fn largest_contentful_paint_candidate(&self) -> Option<LCPCandidate> {
        self.lcp_candidate.clone()
    }
}
