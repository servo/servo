/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Composition of per-pipeline display-list captures into `WebView`-level snapshots.
//!
//! When display-list capture is enabled, every layout in a `WebView`'s frame tree
//! delivers a [`DisplayList`] snapshot for its own pipeline, in that pipeline's
//! coordinate spaces. [`WebViewDisplayListCaptures`] retains the most recent snapshot
//! from each pipeline and splices subframe snapshots into the `Iframe` items that
//! reference them, producing a single display list that covers the entire frame tree
//! in the root pipeline's coordinate spaces.

use embedder_traits::{DisplayList, DisplayListItem, DisplayListItemContent, DisplayListItemSpace};
use rustc_hash::FxHashMap;
use servo_base::id::PipelineId;
use webrender_api::units::{LayoutRect, LayoutVector2D};

/// Where a subframe's items land in the composed snapshot.
struct SubframePlacement {
    /// The origin of the subframe's viewport, in the composed snapshot's coordinates.
    origin: LayoutVector2D,
    /// The rectangle spliced items are clipped to: the accumulated intersection of the
    /// enclosing iframe rectangles.
    clip: LayoutRect,
    /// The coordinate space spliced items are reported in. All content of a subframe
    /// scrolls with the iframe element that displays it, so spliced items inherit the
    /// space of their outermost enclosing iframe item.
    space: DisplayListItemSpace,
}

/// The retained display-list captures for a single `WebView`.
#[derive(Default)]
pub(crate) struct WebViewDisplayListCaptures {
    /// The most recent capture from each pipeline in the frame tree.
    captures: FxHashMap<PipelineId, DisplayList>,
    /// The root pipeline most recently reported by Paint. This is deliberately tracked
    /// separately from the last capture: a capture can arrive before the frame tree update
    /// that identifies its pipeline as the root.
    root_pipeline_id: Option<PipelineId>,
    /// Whether a relevant capture or a frame-tree root change requires delivery.
    dirty: bool,
}

impl WebViewDisplayListCaptures {
    /// Integrate a new per-pipeline capture and compose a `WebView`-level snapshot
    /// rooted at `root_pipeline_id`. A capture that is not reachable from the currently
    /// known root is retained, rather than speculatively treating it as a root: it may be
    /// the first capture from a newly navigated root whose frame-tree update is still in
    /// flight.
    pub(crate) fn update(
        &mut self,
        capture: DisplayList,
        root_pipeline_id: Option<PipelineId>,
    ) -> Option<DisplayList> {
        let pipeline_id = capture.pipeline_id;
        self.captures.insert(pipeline_id, capture);
        self.observe_root(root_pipeline_id);

        // A single-pipeline `WebView` needs no traversal to know the capture is reachable.
        if let Some(root) = self.root_pipeline_id &&
            (pipeline_id == root || self.reachable_from(root).contains(&pipeline_id))
        {
            self.dirty = true;
        }
        self.compose_if_ready()
    }

    /// Re-evaluate retained captures after Paint processes frame-tree messages. This is
    /// necessary when captures arrived before Paint knew the root pipeline.
    pub(crate) fn refresh(&mut self, root_pipeline_id: Option<PipelineId>) -> Option<DisplayList> {
        self.observe_root(root_pipeline_id);
        self.compose_if_ready()
    }

    /// Remove a pipeline that Paint has retired. This bounds retained captures that were
    /// never referenced by a parent display list, and drops a retired subframe's content
    /// from the composed snapshot.
    pub(crate) fn remove_pipeline(&mut self, pipeline_id: PipelineId) {
        if self.captures.remove(&pipeline_id).is_some() {
            self.dirty = true;
        }
        if self.root_pipeline_id == Some(pipeline_id) {
            self.root_pipeline_id = None;
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.captures.is_empty()
    }

    fn observe_root(&mut self, root_pipeline_id: Option<PipelineId>) {
        if self.root_pipeline_id != root_pipeline_id {
            self.root_pipeline_id = root_pipeline_id;
            self.dirty = true;
        }
    }

    fn compose_if_ready(&mut self) -> Option<DisplayList> {
        let root = self.root_pipeline_id?;
        if !self.dirty || !self.captures.contains_key(&root) {
            return None;
        }

        // Pruning is safe only after the root itself is known. Until then, an
        // apparently-unreachable capture might be the incoming root of a navigation.
        self.discard_unreachable_from(root);
        let display_list = self.compose(root);
        self.dirty = false;
        display_list
    }

    /// Compose the retained captures into a single snapshot in the root pipeline's
    /// coordinate spaces.
    fn compose(&self, root: PipelineId) -> Option<DisplayList> {
        let root_capture = self.captures.get(&root)?;
        let mut items = Vec::with_capacity(root_capture.items.len());
        let mut visited = vec![root];
        self.splice(root_capture, None, &mut visited, &mut items);
        Some(DisplayList {
            pipeline_id: root_capture.pipeline_id,
            items,
            epoch: root_capture.epoch,
            scroll_offset: root_capture.scroll_offset,
            viewport_size: root_capture.viewport_size,
            content_size: root_capture.content_size,
        })
    }

    /// Append `capture`'s items — recursively spliced with the captures of the
    /// subframes they reference — to `items`. `placement` is `None` for the root
    /// pipeline, whose items pass through in their own coordinate spaces.
    fn splice(
        &self,
        capture: &DisplayList,
        placement: Option<&SubframePlacement>,
        visited: &mut Vec<PipelineId>,
        items: &mut Vec<DisplayListItem>,
    ) {
        for item in &capture.items {
            // Place the item into the composed snapshot's coordinates. Subframe items
            // are first converted to their own pipeline's viewport coordinates (the
            // rectangle the iframe displays), then offset to the iframe's position and
            // clipped by the enclosing iframe rectangles.
            let (rect, clipped_rect, space) = match placement {
                None => (item.rect, Some(item.rect), item.space),
                Some(placement) => {
                    let viewport_rect = match item.space {
                        DisplayListItemSpace::Document => {
                            item.rect.translate(-capture.scroll_offset)
                        },
                        DisplayListItemSpace::Viewport => item.rect,
                    };
                    let rect = viewport_rect.translate(placement.origin);
                    (rect, rect.intersection(&placement.clip), placement.space)
                },
            };
            let Some(clipped_rect) = clipped_rect else {
                continue;
            };

            items.push(DisplayListItem {
                rect: clipped_rect,
                space,
                content: item.content.clone(),
            });

            // Splice in the subframe's capture right after its iframe item, so the
            // composed list stays in back-to-front paint order. The `visited` guard
            // makes malformed capture sets (cycles, duplicate references) terminate.
            if let DisplayListItemContent::Iframe { pipeline_id } = item.content &&
                !visited.contains(&pipeline_id) &&
                let Some(subframe_capture) = self.captures.get(&pipeline_id)
            {
                visited.push(pipeline_id);
                self.splice(
                    subframe_capture,
                    Some(&SubframePlacement {
                        origin: rect.min.to_vector(),
                        clip: clipped_rect,
                        space,
                    }),
                    visited,
                    items,
                );
            }
        }
    }

    /// The pipelines reachable from `root` by following `Iframe` items, including `root`.
    fn reachable_from(&self, root: PipelineId) -> Vec<PipelineId> {
        let mut reachable = vec![root];
        let mut index = 0;
        while let Some(&pipeline_id) = reachable.get(index) {
            index += 1;
            let Some(capture) = self.captures.get(&pipeline_id) else {
                continue;
            };
            for item in &capture.items {
                if let DisplayListItemContent::Iframe { pipeline_id } = item.content &&
                    !reachable.contains(&pipeline_id)
                {
                    reachable.push(pipeline_id);
                }
            }
        }
        reachable
    }

    /// Discard captures for pipelines that are not reachable from the given root
    /// through `Iframe` items, so captures from before a navigation do not linger.
    fn discard_unreachable_from(&mut self, root: PipelineId) {
        let reachable = self.reachable_from(root);
        self.captures
            .retain(|pipeline_id, _| reachable.contains(pipeline_id));
    }
}

#[cfg(test)]
mod tests {
    use euclid::{Box2D, Point2D, Size2D};
    use servo_base::Epoch;
    use webrender_api::units::LayoutSize;

    use super::*;

    fn pipeline(index: u32) -> PipelineId {
        PipelineId::from(webrender_api::PipelineId(1, index))
    }

    fn rect(x: f32, y: f32, width: f32, height: f32) -> LayoutRect {
        Box2D::from_origin_and_size(Point2D::new(x, y), Size2D::new(width, height))
    }

    fn text(text: &str, rect: LayoutRect, space: DisplayListItemSpace) -> DisplayListItem {
        DisplayListItem {
            rect,
            space,
            content: DisplayListItemContent::Text {
                text: text.to_owned(),
                color: webrender_api::ColorF::BLACK,
            },
        }
    }

    fn iframe(pipeline_id: PipelineId, rect: LayoutRect) -> DisplayListItem {
        DisplayListItem {
            rect,
            space: DisplayListItemSpace::Document,
            content: DisplayListItemContent::Iframe { pipeline_id },
        }
    }

    fn capture(
        pipeline_id: PipelineId,
        scroll_offset: LayoutVector2D,
        items: Vec<DisplayListItem>,
    ) -> DisplayList {
        DisplayList {
            pipeline_id,
            items,
            epoch: Epoch(1),
            scroll_offset,
            viewport_size: LayoutSize::new(800., 600.),
            content_size: LayoutSize::new(800., 600.),
        }
    }

    #[test]
    fn composes_subframe_into_iframe_rect() {
        let (root, child) = (pipeline(1), pipeline(2));
        let mut captures = WebViewDisplayListCaptures::default();

        // The child arrives first and cannot be composed: it is not the root.
        assert!(
            captures
                .update(
                    capture(
                        child,
                        LayoutVector2D::new(0., 30.),
                        vec![
                            text(
                                "visible",
                                rect(10., 40., 50., 10.),
                                DisplayListItemSpace::Document
                            ),
                            text(
                                "scrolled away",
                                rect(10., 0., 50., 10.),
                                DisplayListItemSpace::Document
                            ),
                            text(
                                "fixed",
                                rect(0., 0., 20., 10.),
                                DisplayListItemSpace::Viewport
                            ),
                        ],
                    ),
                    Some(root),
                )
                .is_none()
        );

        let composed = captures
            .update(
                capture(
                    root,
                    LayoutVector2D::zero(),
                    vec![iframe(child, rect(100., 200., 300., 150.))],
                ),
                Some(root),
            )
            .expect("Should compose once the root capture arrives");

        let rects: Vec<_> = composed.items.iter().map(|item| item.rect).collect();
        assert_eq!(
            rects.len(),
            3,
            "The fully scrolled-out subframe item is culled"
        );
        // The iframe item itself.
        assert_eq!(rects[0], rect(100., 200., 300., 150.));
        // A document-space subframe item: shifted by the subframe scroll offset and
        // the iframe origin.
        assert_eq!(rects[1], rect(110., 210., 50., 10.));
        // A subframe-viewport-space item: shifted by the iframe origin only, and now
        // in the *root* document space, like its iframe.
        assert_eq!(rects[2], rect(100., 200., 20., 10.));
        assert!(
            composed
                .items
                .iter()
                .all(|item| item.space == DisplayListItemSpace::Document)
        );
    }

    #[test]
    fn clips_subframe_items_to_iframe_rect() {
        let (root, child) = (pipeline(1), pipeline(2));
        let mut captures = WebViewDisplayListCaptures::default();
        captures.update(
            capture(
                child,
                LayoutVector2D::zero(),
                vec![text(
                    "overflowing",
                    rect(0., 0., 500., 10.),
                    DisplayListItemSpace::Document,
                )],
            ),
            Some(root),
        );
        let composed = captures
            .update(
                capture(
                    root,
                    LayoutVector2D::zero(),
                    vec![iframe(child, rect(100., 100., 200., 100.))],
                ),
                Some(root),
            )
            .unwrap();
        assert_eq!(composed.items[1].rect, rect(100., 100., 200., 10.));
    }

    #[test]
    fn discards_captures_from_before_navigation() {
        let (old_root, new_root) = (pipeline(1), pipeline(2));
        let mut captures = WebViewDisplayListCaptures::default();
        captures.update(
            capture(old_root, LayoutVector2D::zero(), Vec::new()),
            Some(old_root),
        );
        let composed = captures
            .update(
                capture(new_root, LayoutVector2D::zero(), Vec::new()),
                Some(new_root),
            )
            .unwrap();
        assert_eq!(composed.pipeline_id, new_root);
        assert!(!captures.captures.contains_key(&old_root));
    }

    #[test]
    fn cyclic_references_terminate() {
        let (root, child) = (pipeline(1), pipeline(2));
        let mut captures = WebViewDisplayListCaptures::default();
        captures.update(
            capture(
                child,
                LayoutVector2D::zero(),
                vec![iframe(root, rect(0., 0., 100., 100.))],
            ),
            Some(root),
        );
        let composed = captures
            .update(
                capture(
                    root,
                    LayoutVector2D::zero(),
                    vec![iframe(child, rect(0., 0., 100., 100.))],
                ),
                Some(root),
            )
            .unwrap();
        // Both iframe items appear, but the cycle is not followed further.
        assert_eq!(composed.items.len(), 2);
    }

    #[test]
    fn subframe_navigation_evicts_stale_capture() {
        let (root, old_child, new_child) = (pipeline(1), pipeline(2), pipeline(3));
        let mut captures = WebViewDisplayListCaptures::default();
        captures.update(
            capture(
                old_child,
                LayoutVector2D::zero(),
                vec![text(
                    "stale",
                    rect(0., 0., 50., 10.),
                    DisplayListItemSpace::Document,
                )],
            ),
            Some(root),
        );
        captures.update(
            capture(
                root,
                LayoutVector2D::zero(),
                vec![iframe(old_child, rect(0., 0., 100., 100.))],
            ),
            Some(root),
        );

        // The iframe navigates: the root's rebuilt display list references the new
        // child pipeline. The old child's capture must not linger or be spliced.
        let composed = captures
            .update(
                capture(
                    root,
                    LayoutVector2D::zero(),
                    vec![iframe(new_child, rect(0., 0., 100., 100.))],
                ),
                Some(root),
            )
            .unwrap();
        assert_eq!(composed.items.len(), 1, "Only the iframe item remains");
        assert!(!captures.captures.contains_key(&old_child));
    }

    #[test]
    fn subframe_capture_before_root_capture_is_retained() {
        let (root, child) = (pipeline(1), pipeline(2));
        let mut captures = WebViewDisplayListCaptures::default();

        // The renderer already knows the root pipeline, but the root's capture has
        // not arrived. The child capture must not be treated as the root, nor be
        // discarded by pruning against the absent root capture.
        assert!(
            captures
                .update(
                    capture(
                        child,
                        LayoutVector2D::zero(),
                        vec![text(
                            "child",
                            rect(0., 0., 50., 10.),
                            DisplayListItemSpace::Document,
                        )],
                    ),
                    Some(root),
                )
                .is_none()
        );
        assert!(captures.captures.contains_key(&child));

        let composed = captures
            .update(
                capture(
                    root,
                    LayoutVector2D::zero(),
                    vec![iframe(child, rect(10., 10., 100., 100.))],
                ),
                Some(root),
            )
            .unwrap();
        assert_eq!(composed.items.len(), 2);
        assert_eq!(composed.items[1].rect, rect(10., 10., 50., 10.));
    }

    #[test]
    fn unreferenced_subframe_capture_is_evicted_by_the_next_root_capture() {
        let (root, hidden_child) = (pipeline(1), pipeline(2));
        let mut captures = WebViewDisplayListCaptures::default();
        captures.update(
            capture(root, LayoutVector2D::zero(), Vec::new()),
            Some(root),
        );

        // Keep an apparently-unreachable capture until the next root capture. It might
        // instead be the first capture from a new root while the frame-tree message is
        // in flight.
        assert!(
            captures
                .update(
                    capture(hidden_child, LayoutVector2D::zero(), Vec::new()),
                    Some(root),
                )
                .is_none()
        );
        assert!(captures.captures.contains_key(&hidden_child));

        captures.update(
            capture(root, LayoutVector2D::zero(), Vec::new()),
            Some(root),
        );
        assert!(!captures.captures.contains_key(&hidden_child));
    }

    #[test]
    fn capture_without_known_root_is_not_speculatively_delivered() {
        let child = pipeline(1);
        let mut captures = WebViewDisplayListCaptures::default();

        // A subframe can finish layout before its parent, so a lone capture does not
        // establish that it is the WebView's root.
        assert!(
            captures
                .update(capture(child, LayoutVector2D::zero(), Vec::new()), None)
                .is_none()
        );
        assert!(captures.captures.contains_key(&child));
    }

    #[test]
    fn refresh_composes_captures_that_arrived_before_root_was_known() {
        let (root, child) = (pipeline(1), pipeline(2));
        let mut captures = WebViewDisplayListCaptures::default();

        captures.update(
            capture(
                child,
                LayoutVector2D::zero(),
                vec![text(
                    "child",
                    rect(0., 0., 20., 10.),
                    DisplayListItemSpace::Document,
                )],
            ),
            None,
        );
        captures.update(
            capture(
                root,
                LayoutVector2D::zero(),
                vec![iframe(child, rect(10., 20., 100., 50.))],
            ),
            None,
        );

        let composed = captures.refresh(Some(root)).unwrap();
        assert_eq!(composed.pipeline_id, root);
        assert_eq!(composed.items.len(), 2);
        assert_eq!(composed.items[1].rect, rect(10., 20., 20., 10.));
    }

    #[test]
    fn stale_root_does_not_prune_an_incoming_root_capture() {
        let (old_root, new_root) = (pipeline(1), pipeline(2));
        let mut captures = WebViewDisplayListCaptures::default();
        captures.update(
            capture(old_root, LayoutVector2D::zero(), Vec::new()),
            Some(old_root),
        );

        // Paint still reports the old root while the new root's capture arrives.
        assert!(
            captures
                .update(
                    capture(new_root, LayoutVector2D::zero(), Vec::new()),
                    Some(old_root),
                )
                .is_none()
        );
        assert!(captures.captures.contains_key(&new_root));

        let composed = captures.refresh(Some(new_root)).unwrap();
        assert_eq!(composed.pipeline_id, new_root);
        assert!(!captures.captures.contains_key(&old_root));
    }

    #[test]
    fn retired_pipeline_capture_is_removed() {
        let root = pipeline(1);
        let mut captures = WebViewDisplayListCaptures::default();
        captures.update(
            capture(root, LayoutVector2D::zero(), Vec::new()),
            Some(root),
        );
        captures.remove_pipeline(root);
        assert!(captures.is_empty());
        assert!(captures.refresh(Some(root)).is_none());
    }

    #[test]
    fn cleared_capture_session_does_not_splice_an_old_child() {
        let (root, child) = (pipeline(1), pipeline(2));
        let mut old_session = WebViewDisplayListCaptures::default();
        old_session.update(
            capture(
                child,
                LayoutVector2D::zero(),
                vec![text(
                    "old child",
                    rect(0., 0., 20., 10.),
                    DisplayListItemSpace::Document,
                )],
            ),
            Some(root),
        );

        // `DisplayListCaptureCleared` discards the entire per-WebView state. A fresh
        // root may reference the same child pipeline, but it must wait for a capture
        // from the fresh session rather than splice the retained old one.
        let mut fresh_session = WebViewDisplayListCaptures::default();
        let composed = fresh_session
            .update(
                capture(
                    root,
                    LayoutVector2D::zero(),
                    vec![iframe(child, rect(10., 20., 100., 50.))],
                ),
                Some(root),
            )
            .unwrap();

        assert_eq!(composed.items.len(), 1);
        assert!(matches!(
            &composed.items[0].content,
            DisplayListItemContent::Iframe { pipeline_id } if *pipeline_id == child
        ));
    }

    #[test]
    fn retiring_a_subframe_recomposes_without_it() {
        let (root, child) = (pipeline(1), pipeline(2));
        let mut captures = WebViewDisplayListCaptures::default();
        captures.update(
            capture(
                child,
                LayoutVector2D::zero(),
                vec![text(
                    "child",
                    rect(0., 0., 50., 10.),
                    DisplayListItemSpace::Document,
                )],
            ),
            Some(root),
        );
        let composed = captures
            .update(
                capture(
                    root,
                    LayoutVector2D::zero(),
                    vec![iframe(child, rect(0., 0., 100., 100.))],
                ),
                Some(root),
            )
            .unwrap();
        assert_eq!(composed.items.len(), 2);

        // The child exits after an in-place iframe navigation, without the parent's
        // display list being rebuilt, so retiring the pipeline drops it from the snapshot.
        captures.remove_pipeline(child);
        let composed = captures
            .refresh(Some(root))
            .expect("Retiring a spliced subframe should recompose");
        assert_eq!(composed.items.len(), 1, "Only the iframe item remains");
    }

    #[test]
    fn nested_iframes_accumulate_origin_and_clip() {
        let (root, middle, inner) = (pipeline(1), pipeline(2), pipeline(3));
        let mut captures = WebViewDisplayListCaptures::default();
        captures.update(
            capture(
                inner,
                LayoutVector2D::zero(),
                vec![text(
                    "deep",
                    rect(5., 5., 500., 10.),
                    DisplayListItemSpace::Document,
                )],
            ),
            Some(root),
        );
        captures.update(
            capture(
                middle,
                LayoutVector2D::zero(),
                vec![iframe(inner, rect(20., 20., 60., 60.))],
            ),
            Some(root),
        );
        let composed = captures
            .update(
                capture(
                    root,
                    LayoutVector2D::zero(),
                    vec![iframe(middle, rect(100., 100., 200., 200.))],
                ),
                Some(root),
            )
            .unwrap();

        let rects: Vec<_> = composed.items.iter().map(|item| item.rect).collect();
        // The middle iframe, the inner iframe offset by it, and the text item offset
        // by both origins and clipped by the intersection of both iframe rects.
        assert_eq!(rects[0], rect(100., 100., 200., 200.));
        assert_eq!(rects[1], rect(120., 120., 60., 60.));
        assert_eq!(rects[2], rect(125., 125., 55., 10.));
    }

    #[test]
    fn viewport_space_subframe_items_inherit_iframe_space() {
        let (root, child) = (pipeline(1), pipeline(2));
        let mut captures = WebViewDisplayListCaptures::default();
        captures.update(
            capture(
                child,
                LayoutVector2D::new(0., 100.),
                vec![text(
                    "fixed",
                    rect(0., 0., 40., 10.),
                    DisplayListItemSpace::Viewport,
                )],
            ),
            Some(root),
        );
        let composed = captures
            .update(
                capture(
                    root,
                    LayoutVector2D::zero(),
                    vec![iframe(child, rect(50., 60., 100., 100.))],
                ),
                Some(root),
            )
            .unwrap();

        // A `position: fixed` item in the subframe is anchored to the iframe's
        // viewport: its rect is offset by the iframe origin without any scroll
        // compensation, and it inherits the iframe item's coordinate space.
        assert_eq!(composed.items[1].rect, rect(50., 60., 40., 10.));
        assert_eq!(composed.items[1].space, DisplayListItemSpace::Document);
    }
}
