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
    /// The root pipeline at the time of the last composition. When the root changes
    /// (the `WebView` navigated), captures unreachable from the new root are
    /// discarded.
    root: Option<PipelineId>,
}

impl WebViewDisplayListCaptures {
    /// Integrate a new per-pipeline capture and compose a `WebView`-level snapshot
    /// rooted at `root_pipeline_id`. Returns `None` if the root pipeline's capture has
    /// not arrived yet.
    pub(crate) fn update(
        &mut self,
        capture: DisplayList,
        root_pipeline_id: Option<PipelineId>,
    ) -> Option<DisplayList> {
        self.captures.insert(capture.pipeline_id, capture);

        let root = match root_pipeline_id {
            Some(root) => root,
            // The renderer may not know the root pipeline yet when the first capture
            // arrives; a lone capture can only be the root.
            None if self.captures.len() == 1 => *self.captures.keys().next().unwrap(),
            None => return None,
        };

        // Discard stale captures when the `WebView` navigates to a new root pipeline.
        // Nothing is discarded when the root is first learned: subframe captures can
        // legitimately arrive before the root pipeline's first capture.
        if let Some(previous_root) = self.root.replace(root) &&
            previous_root != root
        {
            self.discard_unreachable_from(root);
        }

        self.compose(root)
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

    /// Discard captures for pipelines that are not reachable from the given root
    /// through `Iframe` items, so captures from before a navigation do not linger.
    fn discard_unreachable_from(&mut self, root: PipelineId) {
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
                            text("visible", rect(10., 40., 50., 10.), DisplayListItemSpace::Document),
                            text("scrolled away", rect(10., 0., 50., 10.), DisplayListItemSpace::Document),
                            text("fixed", rect(0., 0., 20., 10.), DisplayListItemSpace::Viewport),
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
        assert_eq!(rects.len(), 3, "The fully scrolled-out subframe item is culled");
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
}
