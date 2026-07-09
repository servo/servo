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

use embedder_traits::{DisplayList, DisplayListItem, DisplayListItemContent, DisplayListItemSpace};
use paint_api::display_list::{PaintDisplayListInfo, ScrollTree};
use rustc_hash::FxHashMap;
use servo_base::id::{PipelineId, ScrollTreeNodeId};
use servo_geometry::FastLayoutTransform;
use webrender_api::units::{LayoutRect, LayoutVector2D};

use super::clip::{ClipId, StackingContextTreeClipStore};
use super::paint_traversal::TraversalState;

/// A display list item recorded during the paint traversal, before it has been
/// resolved out of the coordinate space of the spatial node it was painted in.
struct RecordedItem {
    /// The bounding rectangle, in the coordinate space of [`Self::spatial_node_id`].
    rect: LayoutRect,
    /// The spatial node the item is painted in.
    spatial_node_id: ScrollTreeNodeId,
    /// The clip chain that applies to the item.
    clip_id: ClipId,
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
        state: &TraversalState,
        rect: LayoutRect,
        content: DisplayListItemContent,
    ) {
        self.items.push(RecordedItem {
            rect,
            spatial_node_id: state.spatial_id,
            clip_id: state.clip_id,
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
    /// Accumulated clip rectangles in root (viewport) space, memoized by [`ClipId`].
    /// `None` means the chain does not clip.
    resolved_clips: FxHashMap<ClipId, Option<LayoutRect>>,
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
            return *resolved;
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
        self.resolved_clips.insert(clip_id, Some(resolved));
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
        FastLayoutTransform::Transform { transform, .. } => {
            transform.outer_transformed_box2d(rect)
        },
    }
}
