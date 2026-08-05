/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Capture of embedder-facing display-list snapshots.
//!
//! While the [`DisplayListBuilder`](super::DisplayListBuilder) pushes items to
//! WebRender it can also record them here, in the coordinate space of the spatial
//! node they are painted in. Once the paint traversal completes,
//! [`DisplayListCapture::finish`] resolves every recorded item using the same
//! [`ScrollTree`] that positions content in WebRender: reference-frame transforms,
//! the scroll offset of every scroll frame on the ancestor chain, and
//! sticky-positioning offsets are all applied, and each item is clipped by its
//! accumulated clip chain.
//!
//! The scroll offsets used are those in layout's scroll tree at build time, which
//! reflect the most recent scroll positions delivered back from the renderer.
//! Asynchronous scrolls that happen after the build are not reflected until the
//! next display-list build; for the root scroll frame the embedder can compensate
//! using the live offset from `WebView::root_scroll_offset`, but there is no such
//! mechanism for inner scrollers yet.

use embedder_traits::{DisplayList, DisplayListItem, DisplayListItemContent, DisplayListItemSpace};
use paint_api::display_list::{PaintDisplayListInfo, ScrollTree};
use rustc_hash::FxHashMap;
use servo_base::id::{PipelineId, ScrollTreeNodeId};
use servo_geometry::FastLayoutTransform;
use webrender_api::units::{LayoutRect, LayoutVector2D};

use super::clip::{ClipId, StackingContextTreeClipStore};

/// A display list item recorded during the paint traversal, before it has been
/// resolved out of the coordinate space of the spatial node it was painted in.
struct RecordedItem {
    /// The bounding rectangle, in the coordinate space of [`Self::spatial_node_id`].
    rect: LayoutRect,
    /// The spatial node the item is painted in.
    spatial_node_id: ScrollTreeNodeId,
    /// The clip chain that applies to the item.
    clip_id: ClipId,
    /// The primitive's own clip rectangle. WebRender applies this independently of
    /// the clip chain (for example, to crop `object-fit: cover` images).
    clip_rect: LayoutRect,
    /// What the item paints.
    content: DisplayListItemContent,
}

/// Records content display items in paint order during the paint traversal, when the
/// `layout_display_list_capture_enabled` preference is set.
#[derive(Default)]
pub(super) struct DisplayListCapture {
    items: Vec<RecordedItem>,
}

impl DisplayListCapture {
    pub(super) fn record(
        &mut self,
        rect: LayoutRect,
        clip_rect: LayoutRect,
        spatial_node_id: ScrollTreeNodeId,
        clip_id: ClipId,
        content: DisplayListItemContent,
    ) {
        // A fully transparent solid-color primitive has no visible contribution.
        // Text has an equivalent guard at its call site; images cannot be filtered
        // this way because their decoded alpha is not available during traversal.
        if matches!(&content, DisplayListItemContent::SolidColor { color } if color.a <= 0.0) {
            return;
        }
        self.items.push(RecordedItem {
            rect,
            clip_rect,
            spatial_node_id,
            clip_id,
            content,
        });
    }

    /// Resolve all recorded items into an embedder-facing [`DisplayList`].
    pub(super) fn finish(
        self,
        pipeline_id: PipelineId,
        paint_info: &PaintDisplayListInfo,
        clip_store: &StackingContextTreeClipStore,
    ) -> DisplayList {
        let scroll_offset = paint_info
            .scroll_tree
            .get_node(paint_info.root_scroll_node_id)
            .offset()
            .unwrap_or_default();
        let mut resolver = ItemResolver {
            scroll_tree: &paint_info.scroll_tree,
            clip_store,
            root_scroll_node_id: paint_info.root_scroll_node_id,
            root_scroll_offset: scroll_offset,
            resolved_clips: FxHashMap::default(),
        };
        DisplayList {
            pipeline_id,
            items: self
                .items
                .into_iter()
                .filter_map(|item| resolver.resolve(item))
                .collect(),
            epoch: paint_info.epoch,
            scroll_offset,
            viewport_size: paint_info.viewport_details.layout_size(),
            content_size: paint_info.content_size,
        }
    }
}

/// Resolves [`RecordedItem`]s from the coordinate spaces of their spatial nodes into
/// the pipeline's document and viewport spaces.
struct ItemResolver<'a> {
    scroll_tree: &'a ScrollTree,
    clip_store: &'a StackingContextTreeClipStore,
    root_scroll_node_id: ScrollTreeNodeId,
    root_scroll_offset: LayoutVector2D,
    /// Memoized accumulated clip rectangles in root (viewport) space.
    resolved_clips: FxHashMap<ClipId, LayoutRect>,
}

impl ItemResolver<'_> {
    fn resolve(&mut self, item: RecordedItem) -> Option<DisplayListItem> {
        // Map the rectangle into root (viewport) space. For transformed content this
        // is the axis-aligned bounding box of the transformed rectangle; content
        // collapsed by a degenerate transform is dropped, matching what is painted.
        let transform = self
            .scroll_tree
            .cumulative_node_to_root_transform(item.spatial_node_id);
        let rect = transform_rect(&transform, &item.rect)?;

        // `CommonItemProperties::clip_rect` is a primitive-local clip, not part of
        // the clip-chain store. Apply it separately so object-fit and tiled
        // backgrounds report only their visible bounds.
        let primitive_clip = transform_rect(&transform, &item.clip_rect)?;
        let rect = rect.intersection(&primitive_clip)?;

        // Clip by the item's accumulated clip chain, so that content hidden by
        // `overflow` clips or scrolled out of an ancestor scroll port is reduced or
        // culled exactly as it is when painted.
        let rect = match self.clip_chain_in_root_space(item.clip_id) {
            Some(clip) => rect.intersection(&clip)?,
            None => rect,
        };
        if rect.is_empty() {
            return None;
        }

        // Content that does not descend from the root scroll frame (e.g. `position:
        // fixed`) is anchored to the viewport; everything else is reported in document
        // space by undoing the root scroll offset applied above.
        let space = self.space_of(item.spatial_node_id);
        let rect = match space {
            DisplayListItemSpace::Document => rect.translate(self.root_scroll_offset),
            DisplayListItemSpace::Viewport => rect,
        };

        Some(DisplayListItem {
            rect,
            space,
            content: item.content,
        })
    }

    /// The accumulated intersection of the given clip chain in root (viewport) space,
    /// or `None` if the chain does not clip. Clip radii are ignored: rounded clips are
    /// treated as their bounding rectangle, which never under-reports content.
    fn clip_chain_in_root_space(&mut self, clip_id: ClipId) -> Option<LayoutRect> {
        // `ClipId::INVALID` (and any clip not created during `StackingContextTree`
        // construction) means "no clip."
        let clip = self.clip_store.0.get(clip_id.0)?;
        if let Some(resolved) = self.resolved_clips.get(&clip_id) {
            return Some(*resolved);
        }

        // A clip collapsed by a degenerate transform clips out everything.
        let transform = self
            .scroll_tree
            .cumulative_node_to_root_transform(clip.parent_scroll_node_id);
        let rect = transform_rect(&transform, &clip.rect).unwrap_or_else(LayoutRect::zero);
        let parent_clip_id = clip.parent_clip_id;

        let resolved = match self.clip_chain_in_root_space(parent_clip_id) {
            Some(parent) => rect.intersection(&parent).unwrap_or_else(LayoutRect::zero),
            None => rect,
        };
        self.resolved_clips.insert(clip_id, resolved);
        Some(resolved)
    }

    /// The coordinate space of content in the given spatial node. Content anchored to
    /// the viewport is exactly the content that does not have the root scroll frame as
    /// an ancestor (e.g. `position: fixed` content).
    fn space_of(&self, node_id: ScrollTreeNodeId) -> DisplayListItemSpace {
        let mut current = Some(node_id);
        while let Some(node_id) = current {
            if node_id == self.root_scroll_node_id {
                return DisplayListItemSpace::Document;
            }
            current = self.scroll_tree.get_node(node_id).parent;
        }
        DisplayListItemSpace::Viewport
    }
}

/// The axis-aligned bounding rectangle of `rect` mapped through `transform`, or `None`
/// if the transformation collapses the rectangle.
fn transform_rect(transform: &FastLayoutTransform, rect: &LayoutRect) -> Option<LayoutRect> {
    match transform {
        FastLayoutTransform::Offset(offset) => Some(rect.translate(*offset)),
        FastLayoutTransform::Transform { transform, .. } => transform.outer_transformed_box2d(rect),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use embedder_traits::ViewportDetails;
    use euclid::{Box2D, Point2D, Scale, Size2D};
    use paint_api::display_list::{
        AxesScrollSensitivity, ReferenceFrameNodeInfo, ScrollType, ScrollableNodeInfo,
        SpatialTreeNodeInfo,
    };
    use servo_base::Epoch;
    use webrender_api::units::{LayoutPoint, LayoutSize};
    use webrender_api::{BorderRadius, ExternalScrollId, ReferenceFrameKind, TransformStyle};

    use super::*;

    fn rect(x: f32, y: f32, width: f32, height: f32) -> LayoutRect {
        Box2D::from_origin_and_size(Point2D::new(x, y), Size2D::new(width, height))
    }

    fn test_paint_info() -> PaintDisplayListInfo {
        PaintDisplayListInfo::new(
            ViewportDetails {
                size: Size2D::new(800., 600.),
                hidpi_scale_factor: Scale::new(1.),
            },
            LayoutSize::new(800., 4000.),
            webrender_api::PipelineId(1, 1),
            Epoch(1),
            AxesScrollSensitivity {
                x: ScrollType::all(),
                y: ScrollType::all(),
            },
            true,
        )
    }

    fn set_scroll_offset(
        paint_info: &mut PaintDisplayListInfo,
        node_id: ScrollTreeNodeId,
        offset: LayoutVector2D,
    ) {
        if let SpatialTreeNodeInfo::Scroll(info) =
            &mut paint_info.scroll_tree.get_node_mut(node_id).info
        {
            info.offset = offset;
        }
    }

    fn add_scroll_frame(
        paint_info: &mut PaintDisplayListInfo,
        parent: ScrollTreeNodeId,
        external_id_index: u64,
        clip_rect: LayoutRect,
        content_rect: LayoutRect,
        offset: LayoutVector2D,
    ) -> ScrollTreeNodeId {
        paint_info.scroll_tree.add_scroll_tree_node(
            Some(parent),
            SpatialTreeNodeInfo::Scroll(ScrollableNodeInfo {
                external_id: ExternalScrollId(external_id_index, paint_info.pipeline_id),
                content_rect,
                clip_rect,
                scroll_sensitivity: AxesScrollSensitivity {
                    x: ScrollType::all(),
                    y: ScrollType::all(),
                },
                offset,
                offset_changed: Cell::new(false),
            }),
        )
    }

    fn item(rect: LayoutRect, spatial_node_id: ScrollTreeNodeId, clip_id: ClipId) -> RecordedItem {
        RecordedItem {
            rect,
            clip_rect: rect,
            spatial_node_id,
            clip_id,
            content: DisplayListItemContent::SolidColor {
                color: webrender_api::ColorF::BLACK,
            },
        }
    }

    fn finish(
        items: Vec<RecordedItem>,
        paint_info: &PaintDisplayListInfo,
        clip_store: &StackingContextTreeClipStore,
    ) -> DisplayList {
        DisplayListCapture { items }.finish(
            PipelineId::from(paint_info.pipeline_id),
            paint_info,
            clip_store,
        )
    }

    #[test]
    fn resolves_scroll_offsets_through_the_ancestor_chain() {
        let mut paint_info = test_paint_info();
        let root_scroll_node_id = paint_info.root_scroll_node_id;
        set_scroll_offset(
            &mut paint_info,
            root_scroll_node_id,
            LayoutVector2D::new(0., 300.),
        );
        let inner_scroller = add_scroll_frame(
            &mut paint_info,
            root_scroll_node_id,
            1,
            rect(0., 350., 100., 100.),
            rect(0., 350., 100., 1000.),
            LayoutVector2D::new(0., 40.),
        );

        let display_list = finish(
            vec![item(
                rect(0., 400., 100., 50.),
                inner_scroller,
                ClipId::INVALID,
            )],
            &paint_info,
            &StackingContextTreeClipStore::default(),
        );

        // The item is shifted up by the inner scroller's offset (40), while the root
        // scroll offset (300) cancels out of document-space coordinates. The root
        // offset is reported on the display list itself.
        assert_eq!(display_list.scroll_offset, LayoutVector2D::new(0., 300.));
        assert_eq!(display_list.items.len(), 1);
        assert_eq!(display_list.items[0].rect, rect(0., 360., 100., 50.));
        assert_eq!(display_list.items[0].space, DisplayListItemSpace::Document);
    }

    #[test]
    fn content_outside_the_root_scroll_frame_is_viewport_space() {
        let mut paint_info = test_paint_info();
        let root_scroll_node_id = paint_info.root_scroll_node_id;
        set_scroll_offset(
            &mut paint_info,
            root_scroll_node_id,
            LayoutVector2D::new(0., 300.),
        );

        // `position: fixed` content hangs off the root reference frame directly.
        let display_list = finish(
            vec![item(
                rect(10., 20., 30., 40.),
                paint_info.root_reference_frame_id,
                ClipId::INVALID,
            )],
            &paint_info,
            &StackingContextTreeClipStore::default(),
        );

        assert_eq!(display_list.items[0].rect, rect(10., 20., 30., 40.));
        assert_eq!(display_list.items[0].space, DisplayListItemSpace::Viewport);
    }

    #[test]
    fn clip_chains_reduce_and_cull_items() {
        let mut paint_info = test_paint_info();
        let root_scroll_node_id = paint_info.root_scroll_node_id;
        set_scroll_offset(
            &mut paint_info,
            root_scroll_node_id,
            LayoutVector2D::new(0., 300.),
        );

        // An `overflow` clip over the region currently visible through the scrolled
        // viewport, defined in the root scroll frame's (document) coordinates.
        let mut clip_store = StackingContextTreeClipStore::default();
        let clip_id = clip_store.add(
            BorderRadius::default(),
            rect(0., 300., 100., 100.),
            root_scroll_node_id,
            ClipId::INVALID,
        );

        let display_list = finish(
            vec![
                // Partially visible: wider than the clip.
                item(rect(0., 320., 200., 50.), root_scroll_node_id, clip_id),
                // Scrolled out of the clipped region entirely.
                item(rect(0., 0., 50., 10.), root_scroll_node_id, clip_id),
            ],
            &paint_info,
            &clip_store,
        );

        assert_eq!(
            display_list.items.len(),
            1,
            "The scrolled-out item is culled"
        );
        assert_eq!(display_list.items[0].rect, rect(0., 320., 100., 50.));
    }

    #[test]
    fn primitive_clip_rect_reduces_and_culls_items() {
        let paint_info = test_paint_info();
        let root_scroll_node_id = paint_info.root_scroll_node_id;
        let mut partially_clipped = item(
            rect(-50., 0., 200., 100.),
            root_scroll_node_id,
            ClipId::INVALID,
        );
        partially_clipped.clip_rect = rect(0., 0., 100., 100.);
        let mut fully_clipped = item(
            rect(-50., 0., 20., 20.),
            root_scroll_node_id,
            ClipId::INVALID,
        );
        fully_clipped.clip_rect = rect(0., 0., 100., 100.);

        let display_list = finish(
            vec![partially_clipped, fully_clipped],
            &paint_info,
            &StackingContextTreeClipStore::default(),
        );

        assert_eq!(display_list.items.len(), 1);
        assert_eq!(display_list.items[0].rect, rect(0., 0., 100., 100.));
    }

    #[test]
    fn primitive_clip_rect_uses_the_item_spatial_node() {
        let mut paint_info = test_paint_info();
        let root_scroll_node_id = paint_info.root_scroll_node_id;
        let reference_frame = paint_info.scroll_tree.add_scroll_tree_node(
            Some(root_scroll_node_id),
            SpatialTreeNodeInfo::ReferenceFrame(ReferenceFrameNodeInfo {
                origin: LayoutPoint::zero(),
                frame_origin_for_query: LayoutPoint::zero(),
                transform_style: TransformStyle::Flat,
                transform: FastLayoutTransform::Offset(LayoutVector2D::new(25., 15.)),
                kind: ReferenceFrameKind::default(),
            }),
        );
        let mut recorded_item = item(rect(0., 0., 100., 100.), reference_frame, ClipId::INVALID);
        recorded_item.clip_rect = rect(10., 20., 30., 40.);

        let display_list = finish(
            vec![recorded_item],
            &paint_info,
            &StackingContextTreeClipStore::default(),
        );

        assert_eq!(display_list.items[0].rect, rect(35., 35., 30., 40.));
    }

    #[test]
    fn transparent_solid_colors_are_not_recorded() {
        let paint_info = test_paint_info();
        let mut capture = DisplayListCapture::default();
        capture.record(
            rect(0., 0., 10., 10.),
            rect(0., 0., 10., 10.),
            paint_info.root_scroll_node_id,
            ClipId::INVALID,
            DisplayListItemContent::SolidColor {
                color: webrender_api::ColorF::TRANSPARENT,
            },
        );

        let display_list = capture.finish(
            PipelineId::from(paint_info.pipeline_id),
            &paint_info,
            &StackingContextTreeClipStore::default(),
        );

        assert!(display_list.items.is_empty());
    }

    #[test]
    fn reference_frame_translations_are_composed() {
        let mut paint_info = test_paint_info();
        let root_scroll_node_id = paint_info.root_scroll_node_id;
        let reference_frame = paint_info.scroll_tree.add_scroll_tree_node(
            Some(root_scroll_node_id),
            SpatialTreeNodeInfo::ReferenceFrame(ReferenceFrameNodeInfo {
                origin: LayoutPoint::zero(),
                frame_origin_for_query: LayoutPoint::zero(),
                transform_style: TransformStyle::Flat,
                transform: FastLayoutTransform::Offset(LayoutVector2D::new(25., 15.)),
                kind: ReferenceFrameKind::default(),
            }),
        );

        let display_list = finish(
            vec![item(
                rect(0., 0., 10., 10.),
                reference_frame,
                ClipId::INVALID,
            )],
            &paint_info,
            &StackingContextTreeClipStore::default(),
        );

        assert_eq!(display_list.items[0].rect, rect(25., 15., 10., 10.));
        assert_eq!(display_list.items[0].space, DisplayListItemSpace::Document);
    }
}
