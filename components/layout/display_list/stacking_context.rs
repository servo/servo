/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::f32;
use std::cell::RefCell;
use std::mem;
use std::sync::Arc;

use app_units::Au;
use base::id::ScrollTreeNodeId;
use base::print_tree::PrintTree;
use compositing_traits::display_list::{
    AxesScrollSensitivity, CompositorDisplayListInfo, ReferenceFrameNodeInfo, ScrollableNodeInfo,
    SpatialTreeNodeInfo, StickyNodeInfo,
};
use euclid::SideOffsets2D;
use euclid::default::{Point2D, Rect, Size2D};
use log::warn;
use servo_config::opts::DebugOptions;
use style::Zero;
use style::color::AbsoluteColor;
use style::computed_values::float::T as ComputedFloat;
use style::computed_values::mix_blend_mode::T as ComputedMixBlendMode;
use style::computed_values::overflow_x::T as ComputedOverflow;
use style::computed_values::position::T as ComputedPosition;
use style::computed_values::text_decoration_style::T as TextDecorationStyle;
use style::values::computed::angle::Angle;
use style::values::computed::basic_shape::ClipPath;
use style::values::computed::{ClipRectOrAuto, Length, TextDecorationLine};
use style::values::generics::box_::Perspective;
use style::values::generics::transform::{self, GenericRotate, GenericScale, GenericTranslate};
use style::values::specified::box_::DisplayOutside;
use webrender_api::units::{LayoutPoint, LayoutRect, LayoutTransform, LayoutVector2D};
use webrender_api::{self as wr, BorderRadius};
use wr::StickyOffsetBounds;
use wr::units::{LayoutPixel, LayoutSize};

use super::ClipId;
use super::clip::StackingContextTreeClipStore;
use crate::ArcRefCell;
use crate::display_list::conversions::{FilterToWebRender, ToWebRender};
use crate::display_list::{BuilderForBoxFragment, DisplayListBuilder, offset_radii};
use crate::fragment_tree::{
    BoxFragment, ContainingBlockManager, Fragment, FragmentFlags, FragmentTree,
    PositioningFragment, SpecificLayoutInfo,
};
use crate::geom::{AuOrAuto, PhysicalRect, PhysicalSides};
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
}

impl ContainingBlock {
    pub(crate) fn new(
        rect: PhysicalRect<Au>,
        scroll_node_id: ScrollTreeNodeId,
        scroll_frame_size: Option<LayoutSize>,
        clip_id: ClipId,
    ) -> Self {
        ContainingBlock {
            scroll_node_id,
            scroll_frame_size,
            clip_id,
            rect,
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum StackingContextSection {
    OwnBackgroundsAndBorders,
    DescendantBackgroundsAndBorders,
    Foreground,
    Outline,
}

pub(crate) struct StackingContextTree {
    /// The root stacking context of this [`StackingContextTree`].
    pub root_stacking_context: StackingContext,

    /// The information about the WebRender display list that the compositor
    /// consumes. This curerntly contains the out-of-band hit testing information
    /// data structure that the compositor uses to map hit tests to information
    /// about the item hit.
    pub compositor_info: CompositorDisplayListInfo,

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
        viewport_size: LayoutSize,
        pipeline_id: wr::PipelineId,
        first_reflow: bool,
        debug: &DebugOptions,
    ) -> Self {
        let scrollable_overflow = fragment_tree.scrollable_overflow();
        let scrollable_overflow = LayoutSize::from_untyped(Size2D::new(
            scrollable_overflow.size.width.to_f32_px(),
            scrollable_overflow.size.height.to_f32_px(),
        ));

        let compositor_info = CompositorDisplayListInfo::new(
            viewport_size,
            scrollable_overflow,
            pipeline_id,
            // This epoch is set when the WebRender display list is built. For now use a dummy value.
            wr::Epoch(0),
            fragment_tree.viewport_scroll_sensitivity,
            first_reflow,
        );

        let root_scroll_node_id = compositor_info.root_scroll_node_id;
        let cb_for_non_fixed_descendants = ContainingBlock::new(
            fragment_tree.initial_containing_block,
            root_scroll_node_id,
            Some(compositor_info.viewport_size),
            ClipId::INVALID,
        );
        let cb_for_fixed_descendants = ContainingBlock::new(
            fragment_tree.initial_containing_block,
            compositor_info.root_reference_frame_id,
            None,
            ClipId::INVALID,
        );

        // We need to specify all three containing blocks here, because absolute
        // descdendants of the root cannot share the containing block we specify
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
            // This is just a temporary value that will be replaced once we have finished building the tree.
            root_stacking_context: StackingContext::create_root(root_scroll_node_id, debug),
            compositor_info,
            clip_store: Default::default(),
        };

        let mut root_stacking_context = StackingContext::create_root(root_scroll_node_id, debug);
        let text_decorations = Default::default();
        for fragment in &fragment_tree.root_fragments {
            fragment.build_stacking_context_tree(
                &mut stacking_context_tree,
                &containing_block_info,
                &mut root_stacking_context,
                StackingContextBuildMode::SkipHoisted,
                &text_decorations,
            );
        }
        root_stacking_context.sort();

        if debug.dump_stacking_context_tree {
            root_stacking_context.debug_print();
        }

        stacking_context_tree.root_stacking_context = root_stacking_context;

        stacking_context_tree
    }

    fn push_reference_frame(
        &mut self,
        origin: LayoutPoint,
        parent_scroll_node_id: &ScrollTreeNodeId,
        transform_style: wr::TransformStyle,
        transform: LayoutTransform,
        kind: wr::ReferenceFrameKind,
    ) -> ScrollTreeNodeId {
        self.compositor_info.scroll_tree.add_scroll_tree_node(
            Some(parent_scroll_node_id),
            SpatialTreeNodeInfo::ReferenceFrame(ReferenceFrameNodeInfo {
                origin,
                transform_style,
                transform,
                kind,
            }),
        )
    }

    fn define_scroll_frame(
        &mut self,
        parent_scroll_node_id: &ScrollTreeNodeId,
        external_id: wr::ExternalScrollId,
        content_rect: LayoutRect,
        clip_rect: LayoutRect,
        scroll_sensitivity: AxesScrollSensitivity,
    ) -> ScrollTreeNodeId {
        self.compositor_info.scroll_tree.add_scroll_tree_node(
            Some(parent_scroll_node_id),
            SpatialTreeNodeInfo::Scroll(ScrollableNodeInfo {
                external_id,
                content_rect,
                clip_rect,
                scroll_sensitivity,
                offset: LayoutVector2D::zero(),
            }),
        )
    }

    fn define_sticky_frame(
        &mut self,
        parent_scroll_node_id: &ScrollTreeNodeId,
        frame_rect: LayoutRect,
        margins: SideOffsets2D<Option<f32>, LayoutPixel>,
        vertical_offset_bounds: StickyOffsetBounds,
        horizontal_offset_bounds: StickyOffsetBounds,
    ) -> ScrollTreeNodeId {
        self.compositor_info.scroll_tree.add_scroll_tree_node(
            Some(parent_scroll_node_id),
            SpatialTreeNodeInfo::Sticky(StickyNodeInfo {
                frame_rect,
                margins,
                vertical_offset_bounds,
                horizontal_offset_bounds,
            }),
        )
    }
}

/// The text decorations for a Fragment, collecting during [`StackingContextTree`] construction.
#[derive(Clone, Debug)]
pub(crate) struct FragmentTextDecoration {
    pub line: TextDecorationLine,
    pub color: AbsoluteColor,
    pub style: TextDecorationStyle,
}

/// A piece of content that directly belongs to a section of a stacking context.
///
/// This is generally part of a fragment, like its borders or foreground, but it
/// can also be a stacking container that needs to be painted in fragment order.
pub(crate) enum StackingContextContent {
    /// A fragment that does not generate a stacking context or stacking container.
    Fragment {
        scroll_node_id: ScrollTreeNodeId,
        reference_frame_scroll_node_id: ScrollTreeNodeId,
        clip_id: ClipId,
        section: StackingContextSection,
        containing_block: PhysicalRect<Au>,
        fragment: Fragment,
        is_hit_test_for_scrollable_overflow: bool,
        is_collapsed_table_borders: bool,
        text_decorations: Arc<Vec<FragmentTextDecoration>>,
    },

    /// An index into [StackingContext::atomic_inline_stacking_containers].
    ///
    /// There is no section field, because these are always in [StackingContextSection::Foreground].
    AtomicInlineStackingContainer { index: usize },
}

impl StackingContextContent {
    fn section(&self) -> StackingContextSection {
        match self {
            Self::Fragment { section, .. } => *section,
            Self::AtomicInlineStackingContainer { .. } => StackingContextSection::Foreground,
        }
    }

    fn build_display_list(
        &self,
        builder: &mut DisplayListBuilder,
        inline_stacking_containers: &[StackingContext],
    ) {
        match self {
            Self::Fragment {
                scroll_node_id,
                reference_frame_scroll_node_id,
                clip_id,
                section,
                containing_block,
                fragment,
                is_hit_test_for_scrollable_overflow,
                is_collapsed_table_borders,
                text_decorations,
            } => {
                builder.current_scroll_node_id = *scroll_node_id;
                builder.current_reference_frame_scroll_node_id = *reference_frame_scroll_node_id;
                builder.current_clip_id = *clip_id;
                fragment.build_display_list(
                    builder,
                    containing_block,
                    *section,
                    *is_hit_test_for_scrollable_overflow,
                    *is_collapsed_table_borders,
                    text_decorations,
                );
            },
            Self::AtomicInlineStackingContainer { index } => {
                inline_stacking_containers[*index].build_display_list(builder);
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StackingContextType {
    RealStackingContext,
    PositionedStackingContainer,
    FloatStackingContainer,
    AtomicInlineStackingContainer,
}

/// Either a stacking context or a stacking container, per the definitions in
/// <https://drafts.csswg.org/css-position-4/#painting-order>.
///
/// We use the term “real stacking context” in situations that call for a
/// stacking context but not a stacking container.
pub struct StackingContext {
    /// The spatial id of this fragment. This is used to properly handle
    /// things like preserve-3d.
    scroll_tree_node_id: ScrollTreeNodeId,

    /// The clip chain id of this stacking context if it has one. Used for filter clipping.
    clip_id: Option<ClipId>,

    /// The [`BoxFragment`] that established this stacking context. We store the fragment here
    /// rather than just the style, so that incremental layout can automatically update the style.
    initializing_fragment: Option<ArcRefCell<BoxFragment>>,

    /// The type of this stacking context. Used for collecting and sorting.
    context_type: StackingContextType,

    /// The contents that need to be painted in fragment order.
    contents: Vec<StackingContextContent>,

    /// Stacking contexts that need to be stolen by the parent stacking context
    /// if this is a stacking container, that is, real stacking contexts and
    /// positioned stacking containers (where ‘z-index’ is auto).
    /// <https://drafts.csswg.org/css-position-4/#paint-a-stacking-container>
    /// > To paint a stacking container, given a box root and a canvas canvas:
    /// >  1. Paint a stacking context given root and canvas, treating root as
    /// >     if it created a new stacking context, but omitting any positioned
    /// >     descendants or descendants that actually create a stacking context
    /// >     (letting the parent stacking context paint them, instead).
    real_stacking_contexts_and_positioned_stacking_containers: Vec<StackingContext>,

    /// Float stacking containers.
    /// Separate from real_stacking_contexts_or_positioned_stacking_containers
    /// because they should never be stolen by the parent stacking context.
    /// <https://drafts.csswg.org/css-position-4/#paint-a-stacking-container>
    float_stacking_containers: Vec<StackingContext>,

    /// Atomic inline stacking containers.
    /// Separate from real_stacking_contexts_or_positioned_stacking_containers
    /// because they should never be stolen by the parent stacking context, and
    /// separate from float_stacking_containers so that [StackingContextContent]
    /// can index into this vec to paint them in fragment order.
    /// <https://drafts.csswg.org/css-position-4/#paint-a-stacking-container>
    /// <https://drafts.csswg.org/css-position-4/#paint-a-box-in-a-line-box>
    atomic_inline_stacking_containers: Vec<StackingContext>,

    /// Information gathered about the painting order, for [Self::debug_print].
    debug_print_items: Option<RefCell<Vec<DebugPrintItem>>>,
}

/// Refers to one of the child contents or stacking contexts of a [StackingContext].
#[derive(Clone, Copy)]
pub struct DebugPrintItem {
    field: DebugPrintField,
    index: usize,
}

/// Refers to one of the vecs of a [StackingContext].
#[derive(Clone, Copy)]
pub enum DebugPrintField {
    Contents,
    RealStackingContextsAndPositionedStackingContainers,
    FloatStackingContainers,
    AtomicInlineStackingContainers,
}

impl StackingContext {
    fn create_descendant(
        &self,
        spatial_id: ScrollTreeNodeId,
        clip_id: ClipId,
        initializing_fragment: ArcRefCell<BoxFragment>,
        context_type: StackingContextType,
    ) -> Self {
        // WebRender has two different ways of expressing "no clip." ClipChainId::INVALID should be
        // used for primitives, but `None` is used for stacking contexts and clip chains. We convert
        // to the `Option<ClipId>` representation here. Just passing Some(ClipChainId::INVALID)
        // leads to a crash.
        let clip_id = match clip_id {
            ClipId::INVALID => None,
            clip_id => Some(clip_id),
        };
        Self {
            scroll_tree_node_id: spatial_id,
            clip_id,
            initializing_fragment: Some(initializing_fragment),
            context_type,
            contents: vec![],
            real_stacking_contexts_and_positioned_stacking_containers: vec![],
            float_stacking_containers: vec![],
            atomic_inline_stacking_containers: vec![],
            debug_print_items: self.debug_print_items.is_some().then(|| vec![].into()),
        }
    }

    fn create_root(root_scroll_node_id: ScrollTreeNodeId, debug: &DebugOptions) -> Self {
        Self {
            scroll_tree_node_id: root_scroll_node_id,
            clip_id: None,
            initializing_fragment: None,
            context_type: StackingContextType::RealStackingContext,
            contents: vec![],
            real_stacking_contexts_and_positioned_stacking_containers: vec![],
            float_stacking_containers: vec![],
            atomic_inline_stacking_containers: vec![],
            debug_print_items: debug.dump_stacking_context_tree.then(|| vec![].into()),
        }
    }

    /// Add a child stacking context to this stacking context.
    fn add_stacking_context(&mut self, stacking_context: StackingContext) {
        match stacking_context.context_type {
            StackingContextType::RealStackingContext => {
                &mut self.real_stacking_contexts_and_positioned_stacking_containers
            },
            StackingContextType::PositionedStackingContainer => {
                &mut self.real_stacking_contexts_and_positioned_stacking_containers
            },
            StackingContextType::FloatStackingContainer => &mut self.float_stacking_containers,
            StackingContextType::AtomicInlineStackingContainer => {
                &mut self.atomic_inline_stacking_containers
            },
        }
        .push(stacking_context)
    }

    fn z_index(&self) -> i32 {
        self.initializing_fragment.as_ref().map_or(0, |fragment| {
            let fragment = fragment.borrow();
            fragment.style.effective_z_index(fragment.base.flags)
        })
    }

    pub(crate) fn sort(&mut self) {
        self.contents.sort_by_key(|a| a.section());
        self.real_stacking_contexts_and_positioned_stacking_containers
            .sort_by_key(|a| a.z_index());

        debug_assert!(
            self.real_stacking_contexts_and_positioned_stacking_containers
                .iter()
                .all(|c| matches!(
                    c.context_type,
                    StackingContextType::RealStackingContext |
                        StackingContextType::PositionedStackingContainer
                ))
        );
        debug_assert!(
            self.float_stacking_containers
                .iter()
                .all(
                    |c| c.context_type == StackingContextType::FloatStackingContainer &&
                        c.z_index() == 0
                )
        );
        debug_assert!(
            self.atomic_inline_stacking_containers
                .iter()
                .all(
                    |c| c.context_type == StackingContextType::AtomicInlineStackingContainer &&
                        c.z_index() == 0
                )
        );
    }

    fn push_webrender_stacking_context_if_necessary(
        &self,
        builder: &mut DisplayListBuilder,
    ) -> bool {
        let fragment = match self.initializing_fragment.as_ref() {
            Some(fragment) => fragment.borrow(),
            None => return false,
        };

        // WebRender only uses the stacking context to apply certain effects. If we don't
        // actually need to create a stacking context, just avoid creating one.
        let style = &fragment.style;
        let effects = style.get_effects();
        if effects.filter.0.is_empty() &&
            effects.opacity == 1.0 &&
            effects.mix_blend_mode == ComputedMixBlendMode::Normal &&
            !style.has_effective_transform_or_perspective(FragmentFlags::empty()) &&
            style.clone_clip_path() == ClipPath::None
        {
            return false;
        }

        // Create the filter pipeline.
        let current_color = style.clone_color();
        let mut filters: Vec<wr::FilterOp> = effects
            .filter
            .0
            .iter()
            .map(|filter| FilterToWebRender::to_webrender(filter, &current_color))
            .collect();
        if effects.opacity != 1.0 {
            filters.push(wr::FilterOp::Opacity(
                effects.opacity.into(),
                effects.opacity,
            ));
        }

        // TODO(jdm): WebRender now requires us to create stacking context items
        //            with the IS_BLEND_CONTAINER flag enabled if any children
        //            of the stacking context have a blend mode applied.
        //            This will require additional tracking during layout
        //            before we start collecting stacking contexts so that
        //            information will be available when we reach this point.
        let spatial_id = builder.spatial_id(self.scroll_tree_node_id);
        let clip_chain_id = self.clip_id.map(|clip_id| builder.clip_chain_id(clip_id));
        builder.wr().push_stacking_context(
            LayoutPoint::zero(), // origin
            spatial_id,
            style.get_webrender_primitive_flags(),
            clip_chain_id,
            style.get_used_transform_style().to_webrender(),
            effects.mix_blend_mode.to_webrender(),
            &filters,
            &[], // filter_datas
            &[], // filter_primitives
            wr::RasterSpace::Screen,
            wr::StackingContextFlags::empty(),
            None, // snapshot
        );

        true
    }

    /// <https://drafts.csswg.org/css-backgrounds/#special-backgrounds>
    ///
    /// This is only called for the root `StackingContext`
    pub(crate) fn build_canvas_background_display_list(
        &self,
        builder: &mut DisplayListBuilder,
        fragment_tree: &crate::FragmentTree,
    ) {
        let Some(root_fragment) = fragment_tree.root_fragments.iter().find(|fragment| {
            fragment
                .base()
                .is_some_and(|base| base.flags.intersects(FragmentFlags::IS_ROOT_ELEMENT))
        }) else {
            return;
        };
        let root_fragment = match root_fragment {
            Fragment::Box(box_fragment) | Fragment::Float(box_fragment) => box_fragment,
            _ => return,
        }
        .borrow();

        let source_style = {
            // > For documents whose root element is an HTML HTML element or an XHTML html element
            // > [HTML]: if the computed value of background-image on the root element is none and its
            // > background-color is transparent, user agents must instead propagate the computed
            // > values of the background properties from that element’s first HTML BODY or XHTML body
            // > child element.
            if root_fragment.style.background_is_transparent() {
                let body_fragment = fragment_tree.body_fragment();
                builder.paint_body_background = body_fragment.is_none();
                body_fragment
                    .map(|body_fragment| body_fragment.borrow().style.clone())
                    .unwrap_or(root_fragment.style.clone())
            } else {
                root_fragment.style.clone()
            }
        };

        // This can happen if the root fragment does not have a `<body>` child (either because it is
        // `display: none` or `display: contents`) or if the `<body>`'s background is transparent.
        if source_style.background_is_transparent() {
            return;
        }

        // The painting area is theoretically the infinite 2D plane,
        // but we need a rectangle with finite coordinates.
        //
        // If the document is smaller than the viewport (and doesn’t scroll),
        // we still want to paint the rest of the viewport.
        // If it’s larger, we also want to paint areas reachable after scrolling.
        let painting_area = fragment_tree
            .initial_containing_block
            .union(&fragment_tree.scrollable_overflow())
            .to_webrender();

        let background_color =
            source_style.resolve_color(&source_style.get_background().background_color);
        if background_color.alpha > 0.0 {
            let common = builder.common_properties(painting_area, &source_style);
            let color = super::rgba(background_color);
            builder.wr().push_rect(&common, painting_area, color)
        }

        let mut fragment_builder = BuilderForBoxFragment::new(
            &root_fragment,
            &fragment_tree.initial_containing_block,
            false, /* is_hit_test_for_scrollable_overflow */
            false, /* is_collapsed_table_borders */
        );
        let painter = super::background::BackgroundPainter {
            style: &source_style,
            painting_area_override: Some(painting_area),
            positioning_area_override: None,
        };
        fragment_builder.build_background_image(builder, &painter);
    }

    pub(crate) fn build_display_list(&self, builder: &mut DisplayListBuilder) {
        let pushed_context = self.push_webrender_stacking_context_if_necessary(builder);

        // Properly order display items that make up a stacking context.
        // “Steps” here refer to the steps in CSS 2.1 Appendix E.
        // Note that “positioned descendants” is generalised to include all descendants that
        // generate stacking contexts (csswg-drafts#2717), except in the phrase “any positioned
        // descendants or descendants that actually create a stacking context”, where the term
        // means positioned descendants that do not generate stacking contexts.

        // Steps 1 and 2: Borders and background for the root
        let mut contents = self.contents.iter().enumerate().peekable();
        while contents.peek().is_some_and(|(_, child)| {
            child.section() == StackingContextSection::OwnBackgroundsAndBorders
        }) {
            let (i, child) = contents.next().unwrap();
            self.debug_push_print_item(DebugPrintField::Contents, i);
            child.build_display_list(builder, &self.atomic_inline_stacking_containers);
        }

        // Step 3: Stacking contexts with negative ‘z-index’
        let mut real_stacking_contexts_and_positioned_stacking_containers = self
            .real_stacking_contexts_and_positioned_stacking_containers
            .iter()
            .enumerate()
            .peekable();
        while real_stacking_contexts_and_positioned_stacking_containers
            .peek()
            .is_some_and(|(_, child)| child.z_index() < 0)
        {
            let (i, child) = real_stacking_contexts_and_positioned_stacking_containers
                .next()
                .unwrap();
            self.debug_push_print_item(
                DebugPrintField::RealStackingContextsAndPositionedStackingContainers,
                i,
            );
            child.build_display_list(builder);
        }

        // Step 4: Block backgrounds and borders
        while contents.peek().is_some_and(|(_, child)| {
            child.section() == StackingContextSection::DescendantBackgroundsAndBorders
        }) {
            let (i, child) = contents.next().unwrap();
            self.debug_push_print_item(DebugPrintField::Contents, i);
            child.build_display_list(builder, &self.atomic_inline_stacking_containers);
        }

        // Step 5: Float stacking containers
        for (i, child) in self.float_stacking_containers.iter().enumerate() {
            self.debug_push_print_item(DebugPrintField::FloatStackingContainers, i);
            child.build_display_list(builder);
        }

        // Steps 6 and 7: Fragments and inline stacking containers
        while contents
            .peek()
            .is_some_and(|(_, child)| child.section() == StackingContextSection::Foreground)
        {
            let (i, child) = contents.next().unwrap();
            self.debug_push_print_item(DebugPrintField::Contents, i);
            child.build_display_list(builder, &self.atomic_inline_stacking_containers);
        }

        // Steps 8 and 9: Stacking contexts with non-negative ‘z-index’, and
        // positioned stacking containers (where ‘z-index’ is auto)
        for (i, child) in real_stacking_contexts_and_positioned_stacking_containers {
            self.debug_push_print_item(
                DebugPrintField::RealStackingContextsAndPositionedStackingContainers,
                i,
            );
            child.build_display_list(builder);
        }

        // Step 10: Outline
        while contents
            .peek()
            .is_some_and(|(_, child)| child.section() == StackingContextSection::Outline)
        {
            let (i, child) = contents.next().unwrap();
            self.debug_push_print_item(DebugPrintField::Contents, i);
            child.build_display_list(builder, &self.atomic_inline_stacking_containers);
        }

        if pushed_context {
            builder.wr().pop_stacking_context();
        }
    }

    /// Store the fact that something was painted, if [Self::debug_print_items] is not None.
    ///
    /// This is used to help reconstruct the original painting order in [Self::debug_print] without
    /// duplicating our painting order logic, since that could fall out of sync with the real logic.
    fn debug_push_print_item(&self, field: DebugPrintField, index: usize) {
        if let Some(items) = self.debug_print_items.as_ref() {
            items.borrow_mut().push(DebugPrintItem { field, index });
        }
    }

    /// Print the stacking context tree.
    pub fn debug_print(&self) {
        if self.debug_print_items.is_none() {
            warn!("failed to print stacking context tree: debug_print_items was None");
            return;
        }
        let mut tree = PrintTree::new("Stacking context tree".to_owned());
        self.debug_print_with_tree(&mut tree);
    }

    /// Print a subtree with the given [PrintTree], or panic if [Self::debug_print_items] is None.
    fn debug_print_with_tree(&self, tree: &mut PrintTree) {
        match self.context_type {
            StackingContextType::RealStackingContext => {
                tree.new_level(format!("{:?} z={}", self.context_type, self.z_index()));
            },
            StackingContextType::AtomicInlineStackingContainer => {
                // do nothing; we print the heading with its index in DebugPrintField::Contents
            },
            _ => {
                tree.new_level(format!("{:?}", self.context_type));
            },
        }
        for DebugPrintItem { field, index } in
            self.debug_print_items.as_ref().unwrap().borrow().iter()
        {
            match field {
                DebugPrintField::Contents => match self.contents[*index] {
                    StackingContextContent::Fragment { section, .. } => {
                        tree.add_item(format!("{section:?}"));
                    },
                    StackingContextContent::AtomicInlineStackingContainer { index } => {
                        tree.new_level(format!("AtomicInlineStackingContainer #{index}"));
                        self.atomic_inline_stacking_containers[index].debug_print_with_tree(tree);
                        tree.end_level();
                    },
                },
                DebugPrintField::RealStackingContextsAndPositionedStackingContainers => {
                    self.real_stacking_contexts_and_positioned_stacking_containers[*index]
                        .debug_print_with_tree(tree);
                },
                DebugPrintField::FloatStackingContainers => {
                    self.float_stacking_containers[*index].debug_print_with_tree(tree);
                },
                DebugPrintField::AtomicInlineStackingContainers => {
                    // do nothing; we print these in DebugPrintField::Contents
                },
            }
        }
        match self.context_type {
            StackingContextType::AtomicInlineStackingContainer => {
                // do nothing; we print the heading with its index in DebugPrintField::Contents
            },
            _ => {
                tree.end_level();
            },
        }
    }
}

#[derive(PartialEq)]
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
        text_decorations: &Arc<Vec<FragmentTextDecoration>>,
    ) {
        let containing_block = containing_block_info.get_containing_block_for_fragment(self);
        let fragment_clone = self.clone();
        match self {
            Fragment::Box(fragment) | Fragment::Float(fragment) => {
                let fragment = fragment.borrow();
                if mode == StackingContextBuildMode::SkipHoisted &&
                    fragment.style.clone_position().is_absolutely_positioned()
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
                let fragment = fragment.borrow();
                fragment.build_stacking_context_tree(
                    stacking_context_tree,
                    containing_block,
                    containing_block_info,
                    stacking_context,
                    text_decorations,
                );
            },
            Fragment::Text(_) | Fragment::Image(_) | Fragment::IFrame(_) => {
                stacking_context
                    .contents
                    .push(StackingContextContent::Fragment {
                        section: StackingContextSection::Foreground,
                        scroll_node_id: containing_block.scroll_node_id,
                        reference_frame_scroll_node_id: containing_block_info
                            .for_absolute_and_fixed_descendants
                            .scroll_node_id,
                        clip_id: containing_block.clip_id,
                        containing_block: containing_block.rect,
                        fragment: fragment_clone,
                        is_hit_test_for_scrollable_overflow: false,
                        is_collapsed_table_borders: false,
                        text_decorations: text_decorations.clone(),
                    });
            },
        }
    }
}

struct ReferenceFrameData {
    origin: crate::geom::PhysicalPoint<Au>,
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
    fn get_stacking_context_type(&self) -> Option<StackingContextType> {
        if self.style.establishes_stacking_context(self.base.flags) {
            return Some(StackingContextType::RealStackingContext);
        }

        let box_style = &self.style.get_box();
        if box_style.position != ComputedPosition::Static {
            return Some(StackingContextType::PositionedStackingContainer);
        }

        if box_style.float != ComputedFloat::None {
            return Some(StackingContextType::FloatStackingContainer);
        }

        if self.is_atomic_inline_level() {
            return Some(StackingContextType::AtomicInlineStackingContainer);
        }

        None
    }

    fn get_stacking_context_section(&self) -> StackingContextSection {
        if self.get_stacking_context_type().is_some() {
            return StackingContextSection::OwnBackgroundsAndBorders;
        }

        if self.style.get_box().display.outside() == DisplayOutside::Inline {
            return StackingContextSection::Foreground;
        }

        StackingContextSection::DescendantBackgroundsAndBorders
    }

    fn build_stacking_context_tree(
        &self,
        fragment: Fragment,
        stacking_context_tree: &mut StackingContextTree,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        parent_stacking_context: &mut StackingContext,
        text_decorations: &Arc<Vec<FragmentTextDecoration>>,
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
        &self,
        fragment: Fragment,
        stacking_context_tree: &mut StackingContextTree,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        parent_stacking_context: &mut StackingContext,
        text_decorations: &Arc<Vec<FragmentTextDecoration>>,
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
            return;
        }

        let new_spatial_id = stacking_context_tree.push_reference_frame(
            reference_frame_data.origin.to_webrender(),
            &containing_block.scroll_node_id,
            self.style.get_box().transform_style.to_webrender(),
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
        assert!(
            self.style
                .establishes_containing_block_for_all_descendants(self.base.flags)
        );
        let adjusted_containing_block = ContainingBlock::new(
            containing_block
                .rect
                .translate(-reference_frame_data.origin.to_vector()),
            new_spatial_id,
            None,
            containing_block.clip_id,
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
        &self,
        fragment: Fragment,
        stacking_context_tree: &mut StackingContextTree,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        parent_stacking_context: &mut StackingContext,
        text_decorations: &Arc<Vec<FragmentTextDecoration>>,
    ) {
        let context_type = match self.get_stacking_context_type() {
            Some(context_type) => context_type,
            None => {
                self.build_stacking_context_tree_for_children(
                    fragment,
                    stacking_context_tree,
                    containing_block,
                    containing_block_info,
                    parent_stacking_context,
                    text_decorations,
                );
                return;
            },
        };

        if context_type == StackingContextType::AtomicInlineStackingContainer {
            // Push a dummy fragment that indicates when the new stacking context should be painted.
            parent_stacking_context.contents.push(
                StackingContextContent::AtomicInlineStackingContainer {
                    index: parent_stacking_context
                        .atomic_inline_stacking_containers
                        .len(),
                },
            );
        }

        // `clip-path` needs to be applied before filters and creates a stacking context, so it can be
        // applied directly to the stacking context itself.
        // before
        let stacking_context_clip_id = stacking_context_tree
            .clip_store
            .add_for_clip_path(
                self.style.clone_clip_path(),
                &containing_block.scroll_node_id,
                &containing_block.clip_id,
                BuilderForBoxFragment::new(
                    self,
                    &containing_block.rect,
                    false, /* is_hit_test_for_scrollable_overflow */
                    false, /* is_collapsed_table_borders */
                ),
            )
            .unwrap_or(containing_block.clip_id);

        let box_fragment = match fragment {
            Fragment::Box(ref box_fragment) | Fragment::Float(ref box_fragment) => {
                box_fragment.clone()
            },
            _ => unreachable!("Should never try to make stacking context for non-BoxFragment"),
        };

        let mut child_stacking_context = parent_stacking_context.create_descendant(
            containing_block.scroll_node_id,
            stacking_context_clip_id,
            box_fragment,
            context_type,
        );
        self.build_stacking_context_tree_for_children(
            fragment,
            stacking_context_tree,
            containing_block,
            containing_block_info,
            &mut child_stacking_context,
            text_decorations,
        );

        let mut stolen_children = vec![];
        if context_type != StackingContextType::RealStackingContext {
            stolen_children = mem::replace(
                &mut child_stacking_context
                    .real_stacking_contexts_and_positioned_stacking_containers,
                stolen_children,
            );
        }

        child_stacking_context.sort();
        parent_stacking_context.add_stacking_context(child_stacking_context);
        parent_stacking_context
            .real_stacking_contexts_and_positioned_stacking_containers
            .append(&mut stolen_children);
    }

    fn build_stacking_context_tree_for_children(
        &self,
        fragment: Fragment,
        stacking_context_tree: &mut StackingContextTree,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
        text_decorations: &Arc<Vec<FragmentTextDecoration>>,
    ) {
        let mut new_scroll_node_id = containing_block.scroll_node_id;
        let mut new_clip_id = containing_block.clip_id;
        let mut new_scroll_frame_size = containing_block_info
            .for_non_absolute_descendants
            .scroll_frame_size;

        if let Some(scroll_node_id) = self.build_sticky_frame_if_necessary(
            stacking_context_tree,
            &new_scroll_node_id,
            &containing_block.rect,
            &new_scroll_frame_size,
        ) {
            new_scroll_node_id = scroll_node_id;
        }

        if let Some(clip_id) = self.build_clip_frame_if_necessary(
            stacking_context_tree,
            &new_scroll_node_id,
            new_clip_id,
            &containing_block.rect,
        ) {
            new_clip_id = clip_id;
        }

        if let Some(clip_id) = stacking_context_tree.clip_store.add_for_clip_path(
            self.style.clone_clip_path(),
            &new_scroll_node_id,
            &new_clip_id,
            BuilderForBoxFragment::new(
                self,
                &containing_block.rect,
                false, /* is_hit_test_for_scrollable_overflow*/
                false, /* is_collapsed_table_borders */
            ),
        ) {
            new_clip_id = clip_id;
        }

        let establishes_containing_block_for_all_descendants = self
            .style
            .establishes_containing_block_for_all_descendants(self.base.flags);
        let establishes_containing_block_for_absolute_descendants = self
            .style
            .establishes_containing_block_for_absolute_descendants(self.base.flags);

        let reference_frame_scroll_node_id_for_fragments =
            if establishes_containing_block_for_all_descendants {
                new_scroll_node_id
            } else {
                containing_block_info
                    .for_absolute_and_fixed_descendants
                    .scroll_node_id
            };

        let mut add_fragment = |section| {
            stacking_context
                .contents
                .push(StackingContextContent::Fragment {
                    scroll_node_id: new_scroll_node_id,
                    reference_frame_scroll_node_id: reference_frame_scroll_node_id_for_fragments,
                    clip_id: new_clip_id,
                    section,
                    containing_block: containing_block.rect,
                    fragment: fragment.clone(),
                    is_hit_test_for_scrollable_overflow: false,
                    is_collapsed_table_borders: false,
                    text_decorations: text_decorations.clone(),
                });
        };

        let section = self.get_stacking_context_section();
        add_fragment(section);
        if !self.style.get_outline().outline_width.is_zero() {
            add_fragment(StackingContextSection::Outline);
        }

        // We want to build the scroll frame after the background and border, because
        // they shouldn't scroll with the rest of the box content.
        if let Some(overflow_frame_data) = self.build_overflow_frame_if_necessary(
            stacking_context_tree,
            &new_scroll_node_id,
            new_clip_id,
            &containing_block.rect,
        ) {
            new_clip_id = overflow_frame_data.clip_id;
            if let Some(scroll_frame_data) = overflow_frame_data.scroll_frame_data {
                new_scroll_node_id = scroll_frame_data.scroll_tree_node_id;
                new_scroll_frame_size = Some(scroll_frame_data.scroll_frame_rect.size());

                stacking_context
                    .contents
                    .push(StackingContextContent::Fragment {
                        scroll_node_id: new_scroll_node_id,
                        reference_frame_scroll_node_id:
                            reference_frame_scroll_node_id_for_fragments,
                        clip_id: new_clip_id,
                        section,
                        containing_block: containing_block.rect,
                        fragment: fragment.clone(),
                        is_hit_test_for_scrollable_overflow: true,
                        is_collapsed_table_borders: false,
                        text_decorations: text_decorations.clone(),
                    });
            }
        }

        let padding_rect = self
            .padding_rect()
            .translate(containing_block.rect.origin.to_vector());
        let content_rect = self
            .content_rect
            .translate(containing_block.rect.origin.to_vector());

        let for_absolute_descendants = ContainingBlock::new(
            padding_rect,
            new_scroll_node_id,
            new_scroll_frame_size,
            new_clip_id,
        );
        let for_non_absolute_descendants = ContainingBlock::new(
            content_rect,
            new_scroll_node_id,
            new_scroll_frame_size,
            new_clip_id,
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
        let text_decorations = match self.is_atomic_inline_level() ||
            self.base
                .flags
                .contains(FragmentFlags::IS_OUTSIDE_LIST_ITEM_MARKER)
        {
            true => &Default::default(),
            false => text_decorations,
        };

        let new_text_decoration;
        let text_decorations = match self.style.clone_text_decoration_line() {
            TextDecorationLine::NONE => text_decorations,
            line => {
                let mut new_vector = (**text_decorations).clone();
                let color = &self.style.get_inherited_text().color;
                new_vector.push(FragmentTextDecoration {
                    line,
                    color: self
                        .style
                        .clone_text_decoration_color()
                        .resolve_to_absolute(color),
                    style: self.style.clone_text_decoration_style(),
                });
                new_text_decoration = Arc::new(new_vector);
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

        if matches!(&fragment, Fragment::Box(box_fragment) if matches!(
            box_fragment.borrow().specific_layout_info,
            Some(SpecificLayoutInfo::TableGridWithCollapsedBorders(_))
        )) {
            stacking_context
                .contents
                .push(StackingContextContent::Fragment {
                    scroll_node_id: new_scroll_node_id,
                    reference_frame_scroll_node_id: reference_frame_scroll_node_id_for_fragments,
                    clip_id: new_clip_id,
                    section,
                    containing_block: containing_block.rect,
                    fragment: fragment.clone(),
                    is_hit_test_for_scrollable_overflow: false,
                    is_collapsed_table_borders: true,
                    text_decorations: text_decorations.clone(),
                });
        }
    }

    fn build_clip_frame_if_necessary(
        &self,
        stacking_context_tree: &mut StackingContextTree,
        parent_scroll_node_id: &ScrollTreeNodeId,
        parent_clip_id: ClipId,
        containing_block_rect: &PhysicalRect<Au>,
    ) -> Option<ClipId> {
        let position = self.style.get_box().position;
        // https://drafts.csswg.org/css2/#clipping
        // The clip property applies only to absolutely positioned elements
        if !position.is_absolutely_positioned() {
            return None;
        }

        // Only rectangles are supported for now.
        let clip_rect = match self.style.get_effects().clip {
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
            *parent_scroll_node_id,
            parent_clip_id,
        ))
    }

    fn build_overflow_frame_if_necessary(
        &self,
        stacking_context_tree: &mut StackingContextTree,
        parent_scroll_node_id: &ScrollTreeNodeId,
        parent_clip_id: ClipId,
        containing_block_rect: &PhysicalRect<Au>,
    ) -> Option<OverflowFrameData> {
        let overflow = self.style.effective_overflow(self.base.flags);

        if overflow.x == ComputedOverflow::Visible && overflow.y == ComputedOverflow::Visible {
            return None;
        }

        // Non-scrollable overflow path
        if overflow.x == ComputedOverflow::Clip || overflow.y == ComputedOverflow::Clip {
            // TODO: The spec allows `overflow-clip-rect` to specify which box edge to use
            // as the overflow clip edge origin, but Stylo doesn't currently support that.
            // It will need to be handled here, for now always use the padding rect.
            let mut overflow_clip_rect = self
                .padding_rect()
                .translate(containing_block_rect.origin.to_vector())
                .to_webrender();

            // Adjust by the overflow clip margin.
            // https://drafts.csswg.org/css-overflow-3/#overflow-clip-margin
            let clip_margin = self.style.get_margin().overflow_clip_margin.px();
            overflow_clip_rect = overflow_clip_rect.inflate(clip_margin, clip_margin);

            // The clipping region only gets rounded corners if both axes have `overflow: clip`.
            // https://drafts.csswg.org/css-overflow-3/#corner-clipping
            let radii;
            if overflow.x == ComputedOverflow::Clip && overflow.y == ComputedOverflow::Clip {
                let builder = BuilderForBoxFragment::new(self, containing_block_rect, false, false);
                radii = offset_radii(builder.border_radius, clip_margin);
            } else if overflow.x != ComputedOverflow::Clip {
                overflow_clip_rect.min.x = f32::MIN;
                overflow_clip_rect.max.x = f32::MAX;
                radii = BorderRadius::zero();
            } else {
                overflow_clip_rect.min.y = f32::MIN;
                overflow_clip_rect.max.y = f32::MAX;
                radii = BorderRadius::zero();
            }

            let clip_id = stacking_context_tree.clip_store.add(
                radii,
                overflow_clip_rect,
                *parent_scroll_node_id,
                parent_clip_id,
            );

            return Some(OverflowFrameData {
                clip_id,
                scroll_frame_data: None,
            });
        }

        // scrollable overflow path
        // From https://drafts.csswg.org/css-overflow/#propdef-overflow:
        // > UAs must apply the overflow-* values set on the root element to the viewport when the
        // > root element’s display value is not none. However, when the root element is an [HTML]
        // > html element (including XML syntax for HTML) whose overflow value is visible (in both
        // > axes), and that element has as a child a body element whose display value is also not
        // > none, user agents must instead apply the overflow-* values of the first such child
        // > element to the viewport. The element from which the value is propagated must then have a
        // > used overflow value of visible.
        //
        // TODO: This should only happen when the `display` value is actually propagated.
        if self
            .base
            .flags
            .contains(FragmentFlags::IS_BODY_ELEMENT_OF_HTML_ELEMENT_ROOT)
        {
            return None;
        }

        let scroll_frame_rect = self
            .padding_rect()
            .translate(containing_block_rect.origin.to_vector())
            .to_webrender();

        let clip_id = stacking_context_tree.clip_store.add(
            BuilderForBoxFragment::new(self, containing_block_rect, false, false).border_radius,
            scroll_frame_rect,
            *parent_scroll_node_id,
            parent_clip_id,
        );

        let tag = self.base.tag?;
        let external_id = wr::ExternalScrollId(
            tag.to_display_list_fragment_id(),
            stacking_context_tree.compositor_info.pipeline_id,
        );

        let sensitivity = AxesScrollSensitivity {
            x: overflow.x.into(),
            y: overflow.y.into(),
        };

        let scroll_tree_node_id = stacking_context_tree.define_scroll_frame(
            parent_scroll_node_id,
            external_id,
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

    fn build_sticky_frame_if_necessary(
        &self,
        stacking_context_tree: &mut StackingContextTree,
        parent_scroll_node_id: &ScrollTreeNodeId,
        containing_block_rect: &PhysicalRect<Au>,
        scroll_frame_size: &Option<LayoutSize>,
    ) -> Option<ScrollTreeNodeId> {
        if self.style.get_box().position != ComputedPosition::Sticky {
            return None;
        }

        let scroll_frame_size_for_resolve = match scroll_frame_size {
            Some(size) => size,
            None => {
                // This is a direct descendant of a reference frame.
                &stacking_context_tree.compositor_info.viewport_size
            },
        };

        // Percentages sticky positions offsets are resovled against the size of the
        // nearest scroll frame instead of the containing block like for other types
        // of positioning.
        let scroll_frame_height = Au::from_f32_px(scroll_frame_size_for_resolve.height);
        let scroll_frame_width = Au::from_f32_px(scroll_frame_size_for_resolve.width);
        let offsets = self.style.physical_box_offsets();
        let offsets = PhysicalSides::<AuOrAuto>::new(
            offsets.top.map(|v| v.to_used_value(scroll_frame_height)),
            offsets.right.map(|v| v.to_used_value(scroll_frame_width)),
            offsets.bottom.map(|v| v.to_used_value(scroll_frame_height)),
            offsets.left.map(|v| v.to_used_value(scroll_frame_width)),
        );
        *self.resolved_sticky_insets.borrow_mut() = Some(offsets);

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

        let frame_rect = self
            .border_rect()
            .translate(containing_block_rect.origin.to_vector())
            .to_webrender();

        // Position:sticky elements are always restricted based on the size and position of their
        // containing block.
        let containing_block_rect = containing_block_rect.to_webrender();

        // This is the minimum negative offset and then the maximum positive offset. We just
        // specify every edge, but if the corresponding margin is None, that offset has no effect.
        let vertical_offset_bounds = wr::StickyOffsetBounds::new(
            containing_block_rect.min.y - frame_rect.min.y,
            containing_block_rect.max.y - frame_rect.max.y,
        );
        let horizontal_offset_bounds = wr::StickyOffsetBounds::new(
            containing_block_rect.min.x - frame_rect.min.x,
            containing_block_rect.max.x - frame_rect.max.x,
        );

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
            .style
            .has_effective_transform_or_perspective(self.base.flags)
        {
            return None;
        }

        let relative_border_rect = self.border_rect();
        let border_rect = relative_border_rect.translate(containing_block_rect.origin.to_vector());
        let untyped_border_rect = border_rect.to_untyped();

        let transform = self.calculate_transform_matrix(&untyped_border_rect);
        let perspective = self.calculate_perspective_matrix(&untyped_border_rect);
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
    pub fn calculate_transform_matrix(&self, border_rect: &Rect<Au>) -> Option<LayoutTransform> {
        let list = &self.style.get_box().transform;
        let length_rect = au_rect_to_length_rect(border_rect);
        // https://drafts.csswg.org/css-transforms-2/#individual-transforms
        let rotate = match self.style.clone_rotate() {
            GenericRotate::Rotate(angle) => (0., 0., 1., angle),
            GenericRotate::Rotate3D(x, y, z, angle) => (x, y, z, angle),
            GenericRotate::None => (0., 0., 1., Angle::zero()),
        };
        let scale = match self.style.clone_scale() {
            GenericScale::Scale(sx, sy, sz) => (sx, sy, sz),
            GenericScale::None => (1., 1., 1.),
        };
        let translation = match self.style.clone_translate() {
            GenericTranslate::Translate(x, y, z) => LayoutTransform::translation(
                x.resolve(length_rect.size.width).px(),
                y.resolve(length_rect.size.height).px(),
                z.px(),
            ),
            GenericTranslate::None => LayoutTransform::identity(),
        };

        let angle = euclid::Angle::radians(rotate.3.radians());
        let transform_base = list.to_transform_3d_matrix(Some(&length_rect)).ok()?;
        let transform = LayoutTransform::from_untyped(&transform_base.0)
            .then_rotate(rotate.0, rotate.1, rotate.2, angle)
            .then_scale(scale.0, scale.1, scale.2)
            .then(&translation);

        let transform_origin = &self.style.get_box().transform_origin;
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
    pub fn calculate_perspective_matrix(&self, border_rect: &Rect<Au>) -> Option<LayoutTransform> {
        match self.style.get_box().perspective {
            Perspective::Length(length) => {
                let perspective_origin = &self.style.get_box().perspective_origin;
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
}

impl PositioningFragment {
    fn build_stacking_context_tree(
        &self,
        stacking_context_tree: &mut StackingContextTree,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
        text_decorations: &Arc<Vec<FragmentTextDecoration>>,
    ) {
        let rect = self
            .rect
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

pub fn au_rect_to_length_rect(rect: &Rect<Au>) -> Rect<Length> {
    Rect::new(
        Point2D::new(rect.origin.x.into(), rect.origin.y.into()),
        Size2D::new(rect.size.width.into(), rect.size.height.into()),
    )
}
