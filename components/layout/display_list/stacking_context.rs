/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use app_units::Au;
use embedder_traits::ViewportDetails;
use euclid::{Point2D, Rect, SideOffsets2D, Size2D};
use malloc_size_of_derive::MallocSizeOf;
use paint_api::display_list::{
    AxesScrollSensitivity, PaintDisplayListInfo, ReferenceFrameNodeInfo, ScrollableNodeInfo,
    SpatialTreeNodeInfo, StickyNodeInfo,
};
use servo_base::id::ScrollTreeNodeId;
use servo_base::print_tree::PrintTree;
use servo_config::opts::{DiagnosticsLogging, DiagnosticsLoggingOption};
use servo_geometry::MaxRect;
use style::Zero;
use style::color::AbsoluteColor;
use style::computed_values::overflow_x::T as ComputedOverflow;
use style::computed_values::position::T as ComputedPosition;
use style::computed_values::text_decoration_style::T as TextDecorationStyle;
use style::values::computed::angle::Angle;
use style::values::computed::{ClipRectOrAuto, Length, TextDecorationLine};
use style::values::generics::box_::{OverflowClipMarginBox, Perspective};
use style::values::generics::transform::{self, GenericRotate, GenericScale, GenericTranslate};
use style_traits::CSSPixel;
use webrender_api::units::{LayoutPoint, LayoutRect, LayoutTransform, LayoutVector2D};
use webrender_api::{self as wr, BorderRadius};
use wr::StickyOffsetBounds;
use wr::units::{LayoutPixel, LayoutSize};

use super::ClipId;
use super::clip::StackingContextTreeClipStore;
use crate::display_list::conversions::ToWebRender;
use crate::display_list::{BuilderForBoxFragment, offset_radii};
use crate::fragment_tree::{
    BoxFragment, BoxFragmentWithStyle, ContainingBlockCalculation, ContainingBlockManager,
    Fragment, FragmentFlags, FragmentTree, PositioningFragment,
};
use crate::geom::{
    AuOrAuto, LengthPercentageOrAuto, PhysicalPoint, PhysicalRect, PhysicalSides, PhysicalVec,
};
use crate::style_ext::{ComputedValuesExt, TransformExt};

#[derive(Clone)]
pub(crate) struct ContainingBlock {
    /// The SpatialId of the spatial node that contains the children
    /// of this containing block.
    scroll_node_id: ScrollTreeNodeId,

    /// The size of the parent scroll frame of this containing block, used for resolving
    /// sticky margins. If this is None, then this is a direct descendant of a reference
    /// frame and sticky positioning isn't taken into account.
    scroll_frame_size: Option<LayoutSize>,

    /// The [`ClipId`] to use for the children of this containing block.
    clip_id: ClipId,

    /// The physical rect of this containing block.
    rect: PhysicalRect<Au>,

    /// Normally containing block offsets and display list items are positioned relative
    /// to their parent reference frame, but cumulative containing block boundaries on
    /// fragments need to disregard reference frames entirely. This value tracks the
    /// accumulated offset from the origin of the parent reference frame of this
    /// containing block.
    accumulated_reference_frame_offset: PhysicalVec<Au>,
}

impl ContainingBlock {
    pub(crate) fn new(
        rect: PhysicalRect<Au>,
        scroll_node_id: ScrollTreeNodeId,
        scroll_frame_size: Option<LayoutSize>,
        clip_id: ClipId,
        accumulated_reference_frame_offset: PhysicalVec<Au>,
    ) -> Self {
        ContainingBlock {
            scroll_node_id,
            scroll_frame_size,
            clip_id,
            rect,
            accumulated_reference_frame_offset,
        }
    }

    pub(crate) fn new_replacing_rect(&self, rect: &PhysicalRect<Au>) -> Self {
        ContainingBlock {
            rect: *rect,
            ..*self
        }
    }
}

pub(crate) type ContainingBlockInfo<'a> = ContainingBlockManager<'a, ContainingBlock>;

#[derive(MallocSizeOf)]
pub(crate) struct StackingContextTree {
    /// The root [`StackingContext`] of this [`StackingContextTree`].
    pub root_stacking_context: StackingContext,

    /// The information about the WebRender display list that `Paint`
    /// consumes. This currently contains the out-of-band hit testing information
    /// data structure that `Paint` uses to map hit tests to information
    /// about the item hit.
    pub paint_info: PaintDisplayListInfo,

    /// All of the clips collected for this [`StackingContextTree`]. These are added
    /// for things like `overflow`. More clips may be created later during WebRender
    /// display list construction, but they are never added here.
    pub clip_store: StackingContextTreeClipStore,
}

impl StackingContextTree {
    /// Create a new [DisplayList] given the dimensions of the layout and the WebRender
    /// pipeline id.
    pub fn new(
        fragment_tree: &FragmentTree,
        viewport_details: ViewportDetails,
        pipeline_id: wr::PipelineId,
        first_reflow: bool,
        debug: &DiagnosticsLogging,
    ) -> Self {
        let scrollable_overflow = fragment_tree.scrollable_overflow();
        let scroll_area = scrollable_overflow.union(&fragment_tree.initial_containing_block);
        let scroll_area = LayoutSize::from_untyped(Size2D::new(
            scroll_area.size.width.to_f32_px(),
            scroll_area.size.height.to_f32_px(),
        ));

        let viewport_size = viewport_details.layout_size();
        let paint_info = PaintDisplayListInfo::new(
            viewport_details,
            scroll_area,
            pipeline_id,
            // This epoch is set when the WebRender display list is built. For now use a dummy value.
            Default::default(),
            fragment_tree.viewport_scroll_sensitivity,
            first_reflow,
        );

        let root_scroll_node_id = paint_info.root_scroll_node_id;
        let cb_for_non_fixed_descendants = ContainingBlock::new(
            fragment_tree.initial_containing_block,
            root_scroll_node_id,
            Some(viewport_size),
            ClipId::INVALID,
            PhysicalVec::zero(),
        );
        let cb_for_fixed_descendants = ContainingBlock::new(
            fragment_tree.initial_containing_block,
            paint_info.root_reference_frame_id,
            None,
            ClipId::INVALID,
            PhysicalVec::zero(),
        );

        // We need to specify all three containing blocks here, because absolute
        // descendants of the root cannot share the containing block we specify
        // for fixed descendants. In this case, they need to have the spatial
        // id of the root scroll frame, whereas fixed descendants need the
        // spatial id of the root reference frame so that they do not scroll with
        // page content.
        let containing_block_info = ContainingBlockInfo {
            for_non_absolute_descendants: &cb_for_non_fixed_descendants,
            for_absolute_descendants: Some(&cb_for_non_fixed_descendants),
            for_absolute_and_fixed_descendants: &cb_for_fixed_descendants,
        };

        let mut stacking_context_tree = Self {
            // This is just a temporary value that will be replaced once we have finished
            // building the tree.
            root_stacking_context: StackingContext::root(root_scroll_node_id),
            paint_info,
            clip_store: Default::default(),
        };

        let text_decorations = Default::default();
        let mut root_stacking_context = StackingContext::root(root_scroll_node_id);
        if let Some(root_box_fragment) = fragment_tree.root_box_fragment() {
            Fragment::Box(root_box_fragment).build_stacking_context_tree(
                &mut stacking_context_tree,
                &containing_block_info,
                &mut root_stacking_context,
                // The root element might be absolutely positioned and we still want
                // to process it, if it is.
                StackingContextBuildMode::IncludeHoisted,
                &text_decorations,
            );
        }

        root_stacking_context.sort();
        stacking_context_tree.root_stacking_context = root_stacking_context;

        if debug.is_enabled(DiagnosticsLoggingOption::StackingContextTree) {
            stacking_context_tree
                .root_stacking_context
                .print(&mut PrintTree::new("Stacking Context Tree"));
        }

        stacking_context_tree
    }

    fn push_reference_frame(
        &mut self,
        origin: LayoutPoint,
        frame_origin_for_query: LayoutPoint,
        parent_scroll_node_id: ScrollTreeNodeId,
        transform_style: wr::TransformStyle,
        transform: LayoutTransform,
        kind: wr::ReferenceFrameKind,
    ) -> ScrollTreeNodeId {
        self.paint_info.scroll_tree.add_scroll_tree_node(
            Some(parent_scroll_node_id),
            SpatialTreeNodeInfo::ReferenceFrame(ReferenceFrameNodeInfo {
                origin,
                frame_origin_for_query,
                transform_style,
                transform: transform.into(),
                kind,
            }),
        )
    }

    fn define_scroll_frame(
        &mut self,
        parent_scroll_node_id: ScrollTreeNodeId,
        external_id: wr::ExternalScrollId,
        content_rect: LayoutRect,
        clip_rect: LayoutRect,
        scroll_sensitivity: AxesScrollSensitivity,
    ) -> ScrollTreeNodeId {
        self.paint_info.scroll_tree.add_scroll_tree_node(
            Some(parent_scroll_node_id),
            SpatialTreeNodeInfo::Scroll(ScrollableNodeInfo {
                external_id,
                content_rect,
                clip_rect,
                scroll_sensitivity,
                offset: LayoutVector2D::zero(),
                offset_changed: Cell::new(false),
            }),
        )
    }

    fn define_sticky_frame(
        &mut self,
        parent_scroll_node_id: ScrollTreeNodeId,
        frame_rect: LayoutRect,
        margins: SideOffsets2D<Option<f32>, LayoutPixel>,
        vertical_offset_bounds: StickyOffsetBounds,
        horizontal_offset_bounds: StickyOffsetBounds,
    ) -> ScrollTreeNodeId {
        self.paint_info.scroll_tree.add_scroll_tree_node(
            Some(parent_scroll_node_id),
            SpatialTreeNodeInfo::Sticky(StickyNodeInfo {
                frame_rect,
                margins,
                vertical_offset_bounds,
                horizontal_offset_bounds,
            }),
        )
    }

    /// Given a [`Fragment`] and a point in the viewport of the page, return the point in
    /// the [`Fragment`]'s content rectangle in its transformed coordinate system
    /// (untransformed CSS pixels). Note that the point may be outside the [`Fragment`]'s
    /// boundaries.
    ///
    /// TODO: Currently, this only works for [`BoxFragment`], but we should extend it to
    /// other types of [`Fragment`]s in the future.
    pub(crate) fn offset_in_fragment(
        &self,
        fragment: &Fragment,
        point_in_viewport: PhysicalPoint<Au>,
    ) -> Option<Point2D<Au, CSSPixel>> {
        let Fragment::Box(fragment) = fragment else {
            return None;
        };

        let spatial_tree_node = fragment.spatial_tree_node()?;
        let transform = self
            .paint_info
            .scroll_tree
            .cumulative_root_to_node_transform(spatial_tree_node)?;
        let transformed_point = transform
            .project_point2d(point_in_viewport.map(Au::to_f32_px).cast_unit())?
            .map(Au::from_f32_px)
            .cast_unit();

        // Find the origin of the fragment relative to its reference frame in the same coordinate system.
        let reference_frame_origin = self
            .paint_info
            .scroll_tree
            .reference_frame_offset(spatial_tree_node)
            .map(Au::from_f32_px);
        let fragment_origin = fragment
            .cumulative_content_box_rect(
                ContainingBlockCalculation::AlreadyDoneWithStackingContextTree,
            )
            .origin -
            reference_frame_origin.cast_unit();

        // Use that to find the offset from the fragment origin.
        Some(transformed_point - fragment_origin)
    }
}

/// The text decorations for a Fragment, collecting during [`StackingContextTree`] construction.
#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) struct FragmentTextDecoration {
    pub line: TextDecorationLine,
    pub color: AbsoluteColor,
    pub style: TextDecorationStyle,
}

#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq)]
pub(crate) enum StackingContextType {
    StackingContext,
    StackingContainer,
}

#[derive(MallocSizeOf)]
pub enum StackingContextFragments {
    Root,
    Fragment(#[conditional_malloc_size_of] Arc<BoxFragment>),
}

/// Either a stacking context or a stacking container, per the definitions in
/// <https://drafts.csswg.org/css-position-4/#painting-order>.
///
/// We use the term “real stacking context” in situations that call for a stacking context
/// but not a stacking container. Only positioned stacking containers every get a
/// `StackingContext`. The rest are handled inline during the `PaintTraversal`.
#[derive(MallocSizeOf)]
pub struct StackingContext {
    /// The [`BoxFragment`] that established this stacking context. This is used to paint this
    /// [`StackingContext`] and traverse its descendants for painting.
    ///
    /// This is `None` for the root stacking context.
    pub(crate) fragment: StackingContextFragments,

    /// The [`StackingContextType`] of this [`StackingContet`], which determines if it
    /// a stacking context or a stacking container.
    pub(crate) context_type: StackingContextType,

    /// Child [`StackingContext`]s of this [`StackingContext`].
    pub(crate) children: Vec<StackingContext>,

    /// The offset of the containing block, used to properly paint child fragments of this
    /// stacking context or stacking container.
    pub(crate) containing_block_origin: PhysicalPoint<Au>,

    /// The spatial id of this [`StackingContext`].
    pub(crate) scroll_tree_node_id: ScrollTreeNodeId,

    /// The clip id of this [`StackingContext`] if it has one.
    pub(crate) clip_id: ClipId,

    /// The z-index of this [`StackingContext`]. Note that `auto` is represented as 0.
    pub(crate) z_index: i32,

    /// The text decorations that apply to this [`StackingContext`] propagated via the box tree.
    #[conditional_malloc_size_of]
    pub(crate) text_decorations: Rc<Vec<FragmentTextDecoration>>,
}

impl StackingContext {
    fn root(scroll_tree_node_id: ScrollTreeNodeId) -> Self {
        Self {
            fragment: StackingContextFragments::Root,
            context_type: StackingContextType::StackingContext,
            children: Default::default(),
            containing_block_origin: Default::default(),
            scroll_tree_node_id,
            clip_id: ClipId::INVALID,
            z_index: 0,
            text_decorations: Default::default(),
        }
    }

    fn create_descendant(
        &self,
        context_type: StackingContextType,
        containing_block_offset: PhysicalPoint<Au>,
        spatial_id: ScrollTreeNodeId,
        clip_id: ClipId,
        initializing_fragment: Arc<BoxFragment>,
        text_decorations: Rc<Vec<FragmentTextDecoration>>,
    ) -> Self {
        let z_index = initializing_fragment
            .style()
            .effective_z_index(initializing_fragment.base.flags);
        Self {
            fragment: StackingContextFragments::Fragment(initializing_fragment),
            context_type,
            containing_block_origin: containing_block_offset,
            children: Default::default(),
            scroll_tree_node_id: spatial_id,
            clip_id,
            z_index,
            text_decorations,
        }
    }

    pub(crate) fn fragment(&self) -> Option<&Arc<BoxFragment>> {
        match &self.fragment {
            StackingContextFragments::Root => None,
            StackingContextFragments::Fragment(box_fragment) => Some(box_fragment),
        }
    }

    fn sort(&mut self) {
        self.children.sort_by_key(|child| child.z_index)
    }

    fn print(&self, tree: &mut PrintTree) {
        let fragment_string = match &self.fragment {
            StackingContextFragments::Root => "Root".into(),
            StackingContextFragments::Fragment(box_fragment) => format!(
                "{:?} rect={:?}",
                box_fragment.base.tag,
                box_fragment.content_rect()
            ),
        };

        tree.new_level(format!(
            "{fragment_string} z-index={:?} spatial={:?} clip={:?}",
            self.z_index, self.scroll_tree_node_id, self.clip_id
        ));

        for child in self.children.iter() {
            child.print(tree);
        }

        tree.end_level();
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum StackingContextBuildMode {
    IncludeHoisted,
    SkipHoisted,
}

impl Fragment {
    pub(crate) fn build_stacking_context_tree(
        &self,
        stacking_context_tree: &mut StackingContextTree,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
        mode: StackingContextBuildMode,
        text_decorations: &Rc<Vec<FragmentTextDecoration>>,
    ) {
        let containing_block = containing_block_info.get_containing_block_for_fragment(self);
        let cumulative_containing_block = containing_block
            .rect
            .translate(containing_block.accumulated_reference_frame_offset);
        self.set_containing_block(&cumulative_containing_block);

        if self
            .base()
            .is_some_and(|base| base.flags.contains(FragmentFlags::IS_COLLAPSED))
        {
            return;
        }

        let fragment_clone = self.clone();
        match self {
            Fragment::Box(fragment) | Fragment::Float(fragment) => {
                if mode == StackingContextBuildMode::SkipHoisted &&
                    fragment.style().clone_position().is_absolutely_positioned()
                {
                    return;
                }

                let text_decorations = match self {
                    Fragment::Float(..) => &Default::default(),
                    _ => text_decorations,
                };

                fragment.build_stacking_context_tree(
                    fragment_clone,
                    stacking_context_tree,
                    containing_block,
                    containing_block_info,
                    stacking_context,
                    text_decorations,
                );
            },
            Fragment::AbsoluteOrFixedPositioned(fragment) => {
                let shared_fragment = fragment.borrow();
                let fragment_ref = match shared_fragment.fragment.as_ref() {
                    Some(fragment_ref) => fragment_ref,
                    None => unreachable!("Found hoisted box with missing fragment."),
                };

                fragment_ref.build_stacking_context_tree(
                    stacking_context_tree,
                    containing_block_info,
                    stacking_context,
                    StackingContextBuildMode::IncludeHoisted,
                    &Default::default(),
                );
            },
            Fragment::Positioning(fragment) => {
                fragment.build_stacking_context_tree(
                    stacking_context_tree,
                    containing_block,
                    containing_block_info,
                    stacking_context,
                    text_decorations,
                );
            },
            Fragment::Text(_) | Fragment::Image(_) | Fragment::IFrame(_) => {},
        }
    }
}

struct ReferenceFrameData {
    origin: PhysicalPoint<Au>,
    transform: LayoutTransform,
    kind: wr::ReferenceFrameKind,
}
struct ScrollFrameData {
    scroll_tree_node_id: ScrollTreeNodeId,
    scroll_frame_rect: LayoutRect,
}

struct OverflowFrameData {
    clip_id: ClipId,
    scroll_frame_data: Option<ScrollFrameData>,
}

impl BoxFragment {
    pub(crate) fn stacking_context_type(&self) -> Option<StackingContextType> {
        let flags = self.base.flags;
        let style = self.style();
        if style.establishes_stacking_context(flags) {
            return Some(StackingContextType::StackingContext);
        }

        let box_style = &style.get_box();
        if box_style.position != ComputedPosition::Static {
            return Some(StackingContextType::StackingContainer);
        }

        None
    }

    fn build_stacking_context_tree(
        self: &Arc<Self>,
        fragment: Fragment,
        stacking_context_tree: &mut StackingContextTree,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        parent_stacking_context: &mut StackingContext,
        text_decorations: &Rc<Vec<FragmentTextDecoration>>,
    ) {
        self.build_stacking_context_tree_maybe_creating_reference_frame(
            fragment,
            stacking_context_tree,
            containing_block,
            containing_block_info,
            parent_stacking_context,
            text_decorations,
        );
    }

    fn build_stacking_context_tree_maybe_creating_reference_frame(
        self: &Arc<Self>,
        fragment: Fragment,
        stacking_context_tree: &mut StackingContextTree,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        parent_stacking_context: &mut StackingContext,
        text_decorations: &Rc<Vec<FragmentTextDecoration>>,
    ) {
        let reference_frame_data =
            match self.reference_frame_data_if_necessary(&containing_block.rect) {
                Some(reference_frame_data) => reference_frame_data,
                None => {
                    return self.build_stacking_context_tree_maybe_creating_stacking_context(
                        fragment,
                        stacking_context_tree,
                        containing_block,
                        containing_block_info,
                        parent_stacking_context,
                        text_decorations,
                    );
                },
            };

        // <https://drafts.csswg.org/css-transforms/#transform-function-lists>
        // > If a transform function causes the current transformation matrix of an object
        // > to be non-invertible, the object and its content do not get displayed.
        if !reference_frame_data.transform.is_invertible() {
            self.clear_spatial_tree_node_including_descendants();
            return;
        }

        let style = self.style();
        let frame_origin_for_query = self
            .cumulative_border_box_rect(
                ContainingBlockCalculation::AlreadyDoneWithStackingContextTree,
            )
            .origin
            .to_webrender();
        let new_spatial_id = stacking_context_tree.push_reference_frame(
            reference_frame_data.origin.to_webrender(),
            frame_origin_for_query,
            containing_block.scroll_node_id,
            style.get_box().transform_style.to_webrender(),
            reference_frame_data.transform,
            reference_frame_data.kind,
        );

        // WebRender reference frames establish a new coordinate system at their
        // origin (the border box of the fragment). We need to ensure that any
        // coordinates we give to WebRender in this reference frame are relative
        // to the fragment border box. We do this by adjusting the containing
        // block origin. Note that the `for_absolute_descendants` and
        // `for_all_absolute_and_fixed_descendants` properties are now bogus,
        // but all fragments that establish reference frames also establish
        // containing blocks for absolute and fixed descendants, so those
        // properties will be replaced before recursing into children.
        assert!(style.establishes_containing_block_for_all_descendants(self.base.flags));
        let reference_frame_offset = reference_frame_data.origin.to_vector();
        let adjusted_containing_block = ContainingBlock::new(
            containing_block.rect.translate(-reference_frame_offset),
            new_spatial_id,
            None,
            containing_block.clip_id,
            containing_block.accumulated_reference_frame_offset + reference_frame_offset,
        );
        let new_containing_block_info =
            containing_block_info.new_for_non_absolute_descendants(&adjusted_containing_block);

        self.build_stacking_context_tree_maybe_creating_stacking_context(
            fragment,
            stacking_context_tree,
            &adjusted_containing_block,
            &new_containing_block_info,
            parent_stacking_context,
            text_decorations,
        );
    }

    fn build_stacking_context_tree_maybe_creating_stacking_context(
        self: &Arc<Self>,
        fragment: Fragment,
        stacking_context_tree: &mut StackingContextTree,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        parent_stacking_context: &mut StackingContext,
        text_decorations: &Rc<Vec<FragmentTextDecoration>>,
    ) {
        let Some(stacking_context_type) = self.stacking_context_type() else {
            self.build_stacking_context_tree_for_children(
                stacking_context_tree,
                containing_block,
                containing_block_info,
                parent_stacking_context,
                text_decorations,
            );
            return;
        };

        let new_scroll_frame_size = containing_block_info
            .for_non_absolute_descendants
            .scroll_frame_size;
        let spatial_id = self.build_sticky_frame_if_necessary(
            stacking_context_tree,
            containing_block.scroll_node_id,
            &containing_block.rect,
            &new_scroll_frame_size,
        );

        let clip_id = self.build_clip_frame_if_necessary(
            stacking_context_tree,
            spatial_id.unwrap_or(containing_block.scroll_node_id),
            containing_block.clip_id,
            &containing_block.rect,
        );

        let with_style = &self.with_style();
        let style = with_style.style();
        let clip_id = stacking_context_tree
            .clip_store
            .add_for_clip_path(
                &style.get_svg().clip_path,
                spatial_id.unwrap_or(containing_block.scroll_node_id),
                clip_id.unwrap_or(containing_block.clip_id),
                with_style,
                containing_block.rect.origin,
            )
            .or(clip_id);

        let containing_block = if clip_id.is_some() || spatial_id.is_some() {
            if let Some(clip_id) = clip_id {
                self.set_generated_clip_id(clip_id);
            }
            if let Some(spatial_id) = spatial_id {
                self.set_generated_scroll_tree_node_id(spatial_id);
            }
            &ContainingBlock {
                scroll_node_id: spatial_id.unwrap_or(containing_block.scroll_node_id),
                clip_id: clip_id.unwrap_or(containing_block.clip_id),
                ..*containing_block
            }
        } else {
            containing_block
        };

        let box_fragment = fragment
            .retrieve_box_fragment()
            .expect("Should never try to make stacking context for non-BoxFragment")
            .clone();
        let mut child_stacking_context = parent_stacking_context.create_descendant(
            stacking_context_type,
            containing_block.rect.origin,
            containing_block.scroll_node_id,
            containing_block.clip_id,
            box_fragment,
            text_decorations.clone(),
        );
        self.build_stacking_context_tree_for_children(
            stacking_context_tree,
            containing_block,
            containing_block_info,
            &mut child_stacking_context,
            text_decorations,
        );

        let mut stolen_children = vec![];
        if stacking_context_type != StackingContextType::StackingContext {
            stolen_children =
                std::mem::replace(&mut child_stacking_context.children, stolen_children);
        } else {
            child_stacking_context.sort();
        }

        parent_stacking_context
            .children
            .push(child_stacking_context);
        parent_stacking_context
            .children
            .append(&mut stolen_children);
    }

    fn build_stacking_context_tree_for_children(
        self: &Arc<Self>,
        stacking_context_tree: &mut StackingContextTree,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
        text_decorations: &Rc<Vec<FragmentTextDecoration>>,
    ) {
        let with_style = self.with_style();
        let style = with_style.style();
        let establishes_containing_block_for_all_descendants =
            style.establishes_containing_block_for_all_descendants(self.base.flags);
        let establishes_containing_block_for_absolute_descendants =
            style.establishes_containing_block_for_absolute_descendants(self.base.flags);

        let mut new_scroll_node_id = containing_block.scroll_node_id;
        self.spatial_tree_node.set(Some(new_scroll_node_id));

        // We want to build the scroll frame after the background and border, because
        // they shouldn't scroll with the rest of the box content.
        let mut new_scroll_frame_size = containing_block_info
            .for_non_absolute_descendants
            .scroll_frame_size;
        let mut new_clip_id = containing_block.clip_id;
        if let Some(overflow_frame_data) = with_style.build_overflow_frame_if_necessary(
            stacking_context_tree,
            new_scroll_node_id,
            new_clip_id,
            &containing_block.rect,
        ) {
            new_clip_id = overflow_frame_data.clip_id;
            self.set_generated_clip_id(new_clip_id);

            if let Some(scroll_frame_data) = overflow_frame_data.scroll_frame_data {
                new_scroll_node_id = scroll_frame_data.scroll_tree_node_id;
                new_scroll_frame_size = Some(scroll_frame_data.scroll_frame_rect.size());
                self.set_generated_scroll_tree_node_id(new_scroll_node_id);
            }
        }

        let padding_rect = self
            .padding_rect()
            .translate(containing_block.rect.origin.to_vector());
        let content_rect = self
            .content_rect()
            .translate(containing_block.rect.origin.to_vector());

        let for_absolute_descendants = ContainingBlock::new(
            padding_rect,
            new_scroll_node_id,
            new_scroll_frame_size,
            new_clip_id,
            containing_block.accumulated_reference_frame_offset,
        );
        let for_non_absolute_descendants = ContainingBlock::new(
            content_rect,
            new_scroll_node_id,
            new_scroll_frame_size,
            new_clip_id,
            containing_block.accumulated_reference_frame_offset,
        );

        // Create a new `ContainingBlockInfo` for descendants depending on
        // whether or not this fragment establishes a containing block for
        // absolute and fixed descendants.
        let new_containing_block_info = if establishes_containing_block_for_all_descendants {
            containing_block_info.new_for_absolute_and_fixed_descendants(
                &for_non_absolute_descendants,
                &for_absolute_descendants,
            )
        } else if establishes_containing_block_for_absolute_descendants {
            containing_block_info.new_for_absolute_descendants(
                &for_non_absolute_descendants,
                &for_absolute_descendants,
            )
        } else {
            containing_block_info.new_for_non_absolute_descendants(&for_non_absolute_descendants)
        };

        // Text decorations are not propagated to atomic inline-level descendants.
        // From https://drafts.csswg.org/css2/#lining-striking-props:
        // > Note that text decorations are not propagated to floating and absolutely
        // > positioned descendants, nor to the contents of atomic inline-level descendants
        // > such as inline blocks and inline tables.
        let text_decorations = match with_style.is_atomic_inline_level() ||
            self.base
                .flags
                .contains(FragmentFlags::IS_OUTSIDE_LIST_ITEM_MARKER)
        {
            true => &Default::default(),
            false => text_decorations,
        };

        let new_text_decoration;
        let text_decorations = match style.clone_text_decoration_line() {
            TextDecorationLine::NONE => text_decorations,
            line => {
                let mut new_vector = (**text_decorations).clone();
                let color = &style.get_inherited_text().color;
                new_vector.push(FragmentTextDecoration {
                    line,
                    color: style
                        .clone_text_decoration_color()
                        .resolve_to_absolute(color),
                    style: style.clone_text_decoration_style(),
                });
                new_text_decoration = Rc::new(new_vector);
                &new_text_decoration
            },
        };

        for child in &self.children {
            child.build_stacking_context_tree(
                stacking_context_tree,
                &new_containing_block_info,
                stacking_context,
                StackingContextBuildMode::SkipHoisted,
                text_decorations,
            );
        }
    }

    fn build_clip_frame_if_necessary(
        &self,
        stacking_context_tree: &mut StackingContextTree,
        parent_scroll_node_id: ScrollTreeNodeId,
        parent_clip_id: ClipId,
        containing_block_rect: &PhysicalRect<Au>,
    ) -> Option<ClipId> {
        let style = self.style();
        let position = style.get_box().position;
        // https://drafts.csswg.org/css2/#clipping
        // The clip property applies only to absolutely positioned elements
        if !position.is_absolutely_positioned() {
            return None;
        }

        // Only rectangles are supported for now.
        let clip_rect = match style.get_effects().clip {
            ClipRectOrAuto::Rect(rect) => rect,
            _ => return None,
        };

        let border_rect = self.border_rect();
        let clip_rect = clip_rect
            .for_border_rect(border_rect)
            .translate(containing_block_rect.origin.to_vector())
            .to_webrender();
        Some(stacking_context_tree.clip_store.add(
            BorderRadius::zero(),
            clip_rect,
            parent_scroll_node_id,
            parent_clip_id,
        ))
    }
}

impl BoxFragmentWithStyle<'_> {
    fn build_overflow_frame_if_necessary(
        &self,
        stacking_context_tree: &mut StackingContextTree,
        parent_scroll_node_id: ScrollTreeNodeId,
        parent_clip_id: ClipId,
        containing_block_rect: &PhysicalRect<Au>,
    ) -> Option<OverflowFrameData> {
        let style = self.style();
        let overflow = style.effective_overflow(self.base.flags);

        if overflow.x == ComputedOverflow::Visible && overflow.y == ComputedOverflow::Visible {
            return None;
        }

        // Non-scrollable overflow path
        if overflow.x == ComputedOverflow::Clip || overflow.y == ComputedOverflow::Clip {
            let overflow_clip_margin = style.get_margin().overflow_clip_margin;
            let mut overflow_clip_rect = match overflow_clip_margin.visual_box {
                OverflowClipMarginBox::ContentBox => self.content_rect(),
                OverflowClipMarginBox::PaddingBox => self.padding_rect(),
                OverflowClipMarginBox::BorderBox => self.border_rect(),
            }
            .translate(containing_block_rect.origin.to_vector())
            .to_webrender();

            // Adjust by the overflow clip margin.
            // https://drafts.csswg.org/css-overflow-3/#overflow-clip-margin
            let clip_margin_offset = overflow_clip_margin.offset.px();
            overflow_clip_rect = overflow_clip_rect.inflate(clip_margin_offset, clip_margin_offset);

            // The clipping region only gets rounded corners if both axes have `overflow: clip`.
            // https://drafts.csswg.org/css-overflow-3/#corner-clipping
            let radii;
            if overflow.x == ComputedOverflow::Clip && overflow.y == ComputedOverflow::Clip {
                let builder = BuilderForBoxFragment::new(self, containing_block_rect.origin);
                let mut offsets_from_border = SideOffsets2D::new_all_same(clip_margin_offset);
                match overflow_clip_margin.visual_box {
                    OverflowClipMarginBox::ContentBox => {
                        offsets_from_border -= (self.border + self.padding).to_webrender();
                    },
                    OverflowClipMarginBox::PaddingBox => {
                        offsets_from_border -= self.border.to_webrender();
                    },
                    OverflowClipMarginBox::BorderBox => {},
                };
                radii = offset_radii(builder.border_radius(), offsets_from_border);
            } else if overflow.x != ComputedOverflow::Clip {
                let max = LayoutRect::max_rect();
                overflow_clip_rect.min.x = max.min.x;
                overflow_clip_rect.max.x = max.max.x;
                radii = BorderRadius::zero();
            } else {
                let max = LayoutRect::max_rect();
                overflow_clip_rect.min.y = max.min.y;
                overflow_clip_rect.max.y = max.max.y;
                radii = BorderRadius::zero();
            }

            let clip_id = stacking_context_tree.clip_store.add(
                radii,
                overflow_clip_rect,
                parent_scroll_node_id,
                parent_clip_id,
            );

            return Some(OverflowFrameData {
                clip_id,
                scroll_frame_data: None,
            });
        }

        let scroll_frame_rect = self
            .padding_rect()
            .translate(containing_block_rect.origin.to_vector())
            .to_webrender();

        let clip_id = stacking_context_tree.clip_store.add(
            BuilderForBoxFragment::new(self, containing_block_rect.origin).border_radius(),
            scroll_frame_rect,
            parent_scroll_node_id,
            parent_clip_id,
        );

        let tag = self.base.tag?;
        let external_scroll_id = wr::ExternalScrollId(
            tag.to_display_list_fragment_id(),
            stacking_context_tree.paint_info.pipeline_id,
        );

        let sensitivity = AxesScrollSensitivity {
            x: overflow.x.into(),
            y: overflow.y.into(),
        };

        let scroll_tree_node_id = stacking_context_tree.define_scroll_frame(
            parent_scroll_node_id,
            external_scroll_id,
            self.scrollable_overflow().to_webrender(),
            scroll_frame_rect,
            sensitivity,
        );

        Some(OverflowFrameData {
            clip_id,
            scroll_frame_data: Some(ScrollFrameData {
                scroll_tree_node_id,
                scroll_frame_rect,
            }),
        })
    }
}

impl BoxFragment {
    fn build_sticky_frame_if_necessary(
        &self,
        stacking_context_tree: &mut StackingContextTree,
        parent_scroll_node_id: ScrollTreeNodeId,
        containing_block_rect: &PhysicalRect<Au>,
        scroll_frame_size: &Option<LayoutSize>,
    ) -> Option<ScrollTreeNodeId> {
        let style = self.style();
        if style.get_box().position != ComputedPosition::Sticky {
            return None;
        }

        let scroll_frame_size_for_resolve = match scroll_frame_size {
            Some(size) => size,
            None => {
                // This is a direct descendant of a reference frame.
                &stacking_context_tree
                    .paint_info
                    .viewport_details
                    .layout_size()
            },
        };

        // Percentages sticky positions offsets are resovled against the size of the
        // nearest scroll frame instead of the containing block like for other types
        // of positioning.
        let scroll_frame_height = Au::from_f32_px(scroll_frame_size_for_resolve.height);
        let scroll_frame_width = Au::from_f32_px(scroll_frame_size_for_resolve.width);
        let offsets = style.physical_box_offsets();
        let offsets = PhysicalSides::<AuOrAuto>::new(
            offsets.top.map(|v| v.to_used_value(scroll_frame_height)),
            offsets.right.map(|v| v.to_used_value(scroll_frame_width)),
            offsets.bottom.map(|v| v.to_used_value(scroll_frame_height)),
            offsets.left.map(|v| v.to_used_value(scroll_frame_width)),
        );
        self.set_resolved_sticky_insets(offsets);

        if scroll_frame_size.is_none() {
            return None;
        }

        if offsets.top.is_auto() &&
            offsets.right.is_auto() &&
            offsets.bottom.is_auto() &&
            offsets.left.is_auto()
        {
            return None;
        }

        // https://drafts.csswg.org/css-position/#stickypos-insets
        // > For each side of the box, if the corresponding inset property is not `auto`, and the
        // > corresponding border edge of the box would be outside the corresponding edge of the
        // > sticky view rectangle, the box must be visually shifted (as for relative positioning)
        // > to be inward of that sticky view rectangle edge, insofar as it can while its position
        // > box remains contained within its containing block.
        // > The *position box* is its margin box, except that for any side for which the distance
        // > between its margin edge and the corresponding edge of its containing block is less
        // > than its corresponding margin, that distance is used in place of that margin.
        //
        // Amendments:
        // - Using the "margin edge" seems nonsensical, the spec must mean "border edge" instead:
        //   https://github.com/w3c/csswg-drafts/issues/12833
        // - `auto` margins need to be treated as zero:
        //   https://github.com/w3c/csswg-drafts/issues/12852
        //
        // We implement this by enforcing a minimum negative offset and a maximum positive offset.
        // The logic below is a simplified (but equivalent) version of the description above.
        let border_rect = self.border_rect();
        let computed_margin = style.physical_margin();

        // Signed distance between each side of the border box to the corresponding side of the
        // containing block. Note that |border_rect| is already in the coordinate system of the
        // containing block.
        let distance_from_border_box_to_cb = PhysicalSides::new(
            border_rect.min_y(),
            containing_block_rect.width() - border_rect.max_x(),
            containing_block_rect.height() - border_rect.max_y(),
            border_rect.min_x(),
        );

        // Shrinks the signed distance by the margin, producing a limit on how much we can shift
        // the sticky positioned box without forcing the margin to move outside of the containing
        // block.
        let offset_bound = |distance, used_margin, computed_margin: LengthPercentageOrAuto| {
            let used_margin = if computed_margin.is_auto() {
                Au::zero()
            } else {
                used_margin
            };
            Au::zero().max(distance - used_margin).to_f32_px()
        };

        // This is the minimum negative offset and then the maximum positive offset. We specify
        // all sides, but they will have no effect if the corresponding inset property is `auto`.
        let vertical_offset_bounds = wr::StickyOffsetBounds::new(
            -offset_bound(
                distance_from_border_box_to_cb.top,
                self.margin.top,
                computed_margin.top,
            ),
            offset_bound(
                distance_from_border_box_to_cb.bottom,
                self.margin.bottom,
                computed_margin.bottom,
            ),
        );
        let horizontal_offset_bounds = wr::StickyOffsetBounds::new(
            -offset_bound(
                distance_from_border_box_to_cb.left,
                self.margin.left,
                computed_margin.left,
            ),
            offset_bound(
                distance_from_border_box_to_cb.right,
                self.margin.right,
                computed_margin.right,
            ),
        );

        let frame_rect = border_rect
            .translate(containing_block_rect.origin.to_vector())
            .to_webrender();

        // These are the "margins" between the scrollport and |frame_rect|. They are not the same
        // as CSS margins.
        let margins = SideOffsets2D::new(
            offsets.top.non_auto().map(|v| v.to_f32_px()),
            offsets.right.non_auto().map(|v| v.to_f32_px()),
            offsets.bottom.non_auto().map(|v| v.to_f32_px()),
            offsets.left.non_auto().map(|v| v.to_f32_px()),
        );

        let sticky_node_id = stacking_context_tree.define_sticky_frame(
            parent_scroll_node_id,
            frame_rect,
            margins,
            vertical_offset_bounds,
            horizontal_offset_bounds,
        );

        Some(sticky_node_id)
    }

    /// Optionally returns the data for building a reference frame, without yet building it.
    fn reference_frame_data_if_necessary(
        &self,
        containing_block_rect: &PhysicalRect<Au>,
    ) -> Option<ReferenceFrameData> {
        if !self
            .style()
            .has_effective_transform_or_perspective(self.base.flags)
        {
            return None;
        }

        let relative_border_rect = self.border_rect();
        let border_rect = relative_border_rect.translate(containing_block_rect.origin.to_vector());
        let transform = self.calculate_transform_matrix(&border_rect);
        let perspective = self.calculate_perspective_matrix(&border_rect);
        let (reference_frame_transform, reference_frame_kind) = match (transform, perspective) {
            (None, Some(perspective)) => (
                perspective,
                wr::ReferenceFrameKind::Perspective {
                    scrolling_relative_to: None,
                },
            ),
            (Some(transform), None) => (
                transform,
                wr::ReferenceFrameKind::Transform {
                    is_2d_scale_translation: false,
                    should_snap: false,
                    paired_with_perspective: false,
                },
            ),
            (Some(transform), Some(perspective)) => (
                perspective.then(&transform),
                wr::ReferenceFrameKind::Perspective {
                    scrolling_relative_to: None,
                },
            ),
            (None, None) => unreachable!(),
        };

        Some(ReferenceFrameData {
            origin: border_rect.origin,
            transform: reference_frame_transform,
            kind: reference_frame_kind,
        })
    }

    /// Returns the 4D matrix representing this fragment's transform.
    pub fn calculate_transform_matrix(
        &self,
        border_rect: &Rect<Au, CSSPixel>,
    ) -> Option<LayoutTransform> {
        let style = self.style();
        let list = &style.get_box().transform;
        let length_rect = au_rect_to_length_rect(border_rect);
        // https://drafts.csswg.org/css-transforms-2/#individual-transforms
        let rotate = match style.clone_rotate() {
            GenericRotate::Rotate(angle) => (0., 0., 1., angle),
            GenericRotate::Rotate3D(x, y, z, angle) => (x, y, z, angle),
            GenericRotate::None => (0., 0., 1., Angle::zero()),
        };
        let scale = match style.clone_scale() {
            GenericScale::Scale(sx, sy, sz) => (sx, sy, sz),
            GenericScale::None => (1., 1., 1.),
        };
        let translation = match style.clone_translate() {
            GenericTranslate::Translate(x, y, z) => LayoutTransform::translation(
                x.resolve(length_rect.size.width).px(),
                y.resolve(length_rect.size.height).px(),
                z.px(),
            ),
            GenericTranslate::None => LayoutTransform::identity(),
        };

        let angle = euclid::Angle::radians(rotate.3.radians());
        let transform_base = list
            .to_transform_3d_matrix(Some(&length_rect.to_untyped()))
            .ok()?;
        let transform = LayoutTransform::from_untyped(&transform_base.0)
            .then_rotate(rotate.0, rotate.1, rotate.2, angle)
            .then_scale(scale.0, scale.1, scale.2)
            .then(&translation);

        let transform_origin = &style.get_box().transform_origin;
        let transform_origin_x = transform_origin
            .horizontal
            .to_used_value(border_rect.size.width)
            .to_f32_px();
        let transform_origin_y = transform_origin
            .vertical
            .to_used_value(border_rect.size.height)
            .to_f32_px();
        let transform_origin_z = transform_origin.depth.px();

        Some(transform.change_basis(transform_origin_x, transform_origin_y, transform_origin_z))
    }

    /// Returns the 4D matrix representing this fragment's perspective.
    pub fn calculate_perspective_matrix(
        &self,
        border_rect: &Rect<Au, CSSPixel>,
    ) -> Option<LayoutTransform> {
        let style = self.style();
        match style.get_box().perspective {
            Perspective::Length(length) => {
                let perspective_origin = &style.get_box().perspective_origin;
                let perspective_origin = LayoutPoint::new(
                    perspective_origin
                        .horizontal
                        .percentage_relative_to(border_rect.size.width.into())
                        .px(),
                    perspective_origin
                        .vertical
                        .percentage_relative_to(border_rect.size.height.into())
                        .px(),
                );

                let perspective_matrix = LayoutTransform::from_untyped(
                    &transform::create_perspective_matrix(length.px()),
                );

                Some(perspective_matrix.change_basis(
                    perspective_origin.x,
                    perspective_origin.y,
                    0.0,
                ))
            },
            Perspective::None => None,
        }
    }

    fn clear_spatial_tree_node_including_descendants(&self) {
        fn assign_spatial_tree_node_on_fragments(fragments: &[Fragment]) {
            for fragment in fragments.iter() {
                match fragment {
                    Fragment::Box(box_fragment) | Fragment::Float(box_fragment) => {
                        box_fragment.clear_spatial_tree_node_including_descendants();
                    },
                    Fragment::Positioning(positioning_fragment) => {
                        assign_spatial_tree_node_on_fragments(&positioning_fragment.children);
                    },
                    _ => {},
                }
            }
        }

        self.spatial_tree_node.set(None);
        assign_spatial_tree_node_on_fragments(&self.children);
    }
}

impl PositioningFragment {
    fn build_stacking_context_tree(
        &self,
        stacking_context_tree: &mut StackingContextTree,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
        text_decorations: &Rc<Vec<FragmentTextDecoration>>,
    ) {
        let rect = self
            .base
            .rect()
            .translate(containing_block.rect.origin.to_vector());
        let new_containing_block = containing_block.new_replacing_rect(&rect);
        let new_containing_block_info =
            containing_block_info.new_for_non_absolute_descendants(&new_containing_block);

        for child in &self.children {
            child.build_stacking_context_tree(
                stacking_context_tree,
                &new_containing_block_info,
                stacking_context,
                StackingContextBuildMode::SkipHoisted,
                text_decorations,
            );
        }
    }
}

pub(crate) fn au_rect_to_length_rect(rect: &Rect<Au, CSSPixel>) -> Rect<Length, CSSPixel> {
    Rect::new(
        Point2D::new(rect.origin.x.into(), rect.origin.y.into()),
        Size2D::new(rect.size.width.into(), rect.size.height.into()),
    )
}
