/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::f32;
use std::cell::RefCell;
use std::mem;

use app_units::Au;
use base::id::ScrollTreeNodeId;
use base::print_tree::PrintTree;
use compositing_traits::display_list::{AxesScrollSensitivity, ScrollableNodeInfo};
use euclid::SideOffsets2D;
use euclid::default::{Point2D, Rect, Size2D};
use log::warn;
use servo_arc::Arc as ServoArc;
use servo_config::opts::DebugOptions;
use style::Zero;
use style::computed_values::float::T as ComputedFloat;
use style::computed_values::mix_blend_mode::T as ComputedMixBlendMode;
use style::computed_values::overflow_x::T as ComputedOverflow;
use style::computed_values::position::T as ComputedPosition;
use style::properties::ComputedValues;
use style::values::computed::angle::Angle;
use style::values::computed::basic_shape::ClipPath;
use style::values::computed::{ClipRectOrAuto, Length};
use style::values::generics::box_::Perspective;
use style::values::generics::transform::{self, GenericRotate, GenericScale, GenericTranslate};
use style::values::specified::box_::DisplayOutside;
use webrender_api::units::{LayoutPoint, LayoutRect, LayoutTransform, LayoutVector2D};
use webrender_api::{self as wr, BorderRadius};
use wr::units::{LayoutPixel, LayoutSize};
use wr::{ClipChainId, SpatialTreeItemKey, StickyOffsetBounds};

use super::DisplayList;
use super::clip_path::build_clip_path_clip_chain_if_necessary;
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

    /// The WebRender ClipId to use for this children of this containing
    /// block.
    clip_chain_id: wr::ClipChainId,

    /// The physical rect of this containing block.
    rect: PhysicalRect<Au>,
}

impl ContainingBlock {
    pub(crate) fn new(
        rect: PhysicalRect<Au>,
        scroll_node_id: ScrollTreeNodeId,
        scroll_frame_size: Option<LayoutSize>,
        clip_chain_id: wr::ClipChainId,
    ) -> Self {
        ContainingBlock {
            scroll_node_id,
            scroll_frame_size,
            clip_chain_id,
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

impl DisplayList {
    /// Produce a new SpatialTreeItemKey. This is currently unused by WebRender,
    /// but has to be unique to the entire scene.
    fn get_next_spatial_tree_item_key(&mut self) -> SpatialTreeItemKey {
        self.spatial_tree_count += 1;
        let pipeline_tag = ((self.wr.pipeline_id.0 as u64) << 32) | self.wr.pipeline_id.1 as u64;
        SpatialTreeItemKey::new(pipeline_tag, self.spatial_tree_count)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "display_list::build_stacking_context_tree",
            skip_all,
            fields(servo_profiling = true),
            level = "trace",
        )
    )]
    pub fn build_stacking_context_tree(
        &mut self,
        fragment_tree: &FragmentTree,
        debug: &DebugOptions,
    ) -> StackingContext {
        let cb_for_non_fixed_descendants = ContainingBlock::new(
            fragment_tree.initial_containing_block,
            self.compositor_info.root_scroll_node_id,
            Some(self.compositor_info.viewport_size),
            ClipChainId::INVALID,
        );
        let cb_for_fixed_descendants = ContainingBlock::new(
            fragment_tree.initial_containing_block,
            self.compositor_info.root_reference_frame_id,
            None,
            ClipChainId::INVALID,
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

        let mut root_stacking_context = StackingContext::create_root(&self.wr, debug);
        for fragment in &fragment_tree.root_fragments {
            fragment.build_stacking_context_tree(
                self,
                &containing_block_info,
                &mut root_stacking_context,
                StackingContextBuildMode::SkipHoisted,
            );
        }
        root_stacking_context.sort();
        root_stacking_context
    }

    fn push_reference_frame(
        &mut self,
        origin: LayoutPoint,
        parent_scroll_node_id: &ScrollTreeNodeId,
        transform_style: wr::TransformStyle,
        transform: wr::PropertyBinding<LayoutTransform>,
        kind: wr::ReferenceFrameKind,
    ) -> ScrollTreeNodeId {
        let spatial_tree_item_key = self.get_next_spatial_tree_item_key();
        let new_spatial_id = self.wr.push_reference_frame(
            origin,
            parent_scroll_node_id.spatial_id,
            transform_style,
            transform,
            kind,
            spatial_tree_item_key,
        );
        self.compositor_info.scroll_tree.add_scroll_tree_node(
            Some(parent_scroll_node_id),
            new_spatial_id,
            None,
        )
    }

    fn pop_reference_frame(&mut self) {
        self.wr.pop_reference_frame();
    }

    fn clip_overflow_frame(
        &mut self,
        parent_scroll_node_id: &ScrollTreeNodeId,
        parent_clip_id: &ClipChainId,
        clip_rect: LayoutRect,
        radii: wr::BorderRadius,
    ) -> ClipChainId {
        let new_clip_id = if radii.is_zero() {
            self.wr
                .define_clip_rect(parent_scroll_node_id.spatial_id, clip_rect)
        } else {
            self.wr.define_clip_rounded_rect(
                parent_scroll_node_id.spatial_id,
                webrender_api::ComplexClipRegion {
                    rect: clip_rect,
                    radii,
                    mode: webrender_api::ClipMode::Clip,
                },
            )
        };

        self.define_clip_chain(*parent_clip_id, [new_clip_id])
    }

    fn define_scroll_frame(
        &mut self,
        parent_scroll_node_id: &ScrollTreeNodeId,
        external_id: wr::ExternalScrollId,
        content_rect: LayoutRect,
        clip_rect: LayoutRect,
        scroll_sensitivity: AxesScrollSensitivity,
    ) -> ScrollTreeNodeId {
        let spatial_tree_item_key = self.get_next_spatial_tree_item_key();

        let new_spatial_id = self.wr.define_scroll_frame(
            parent_scroll_node_id.spatial_id,
            external_id,
            content_rect,
            clip_rect,
            LayoutVector2D::zero(), /* external_scroll_offset */
            0,                      /* scroll_offset_generation */
            wr::HasScrollLinkedEffect::No,
            spatial_tree_item_key,
        );

        self.compositor_info.scroll_tree.add_scroll_tree_node(
            Some(parent_scroll_node_id),
            new_spatial_id,
            Some(ScrollableNodeInfo {
                external_id,
                scrollable_size: content_rect.size() - clip_rect.size(),
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
        let spatial_tree_item_key = self.get_next_spatial_tree_item_key();
        let new_spatial_id = self.wr.define_sticky_frame(
            parent_scroll_node_id.spatial_id,
            frame_rect,
            margins,
            vertical_offset_bounds,
            horizontal_offset_bounds,
            LayoutVector2D::zero(), /* previously_applied_offset */
            spatial_tree_item_key,
            None, /* transform */
        );
        self.compositor_info.scroll_tree.add_scroll_tree_node(
            Some(parent_scroll_node_id),
            new_spatial_id,
            None,
        )
    }
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
        clip_chain_id: wr::ClipChainId,
        section: StackingContextSection,
        containing_block: PhysicalRect<Au>,
        fragment: Fragment,
        is_hit_test_for_scrollable_overflow: bool,
        is_collapsed_table_borders: bool,
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
                clip_chain_id,
                section,
                containing_block,
                fragment,
                is_hit_test_for_scrollable_overflow,
                is_collapsed_table_borders,
            } => {
                builder.current_scroll_node_id = *scroll_node_id;
                builder.current_reference_frame_scroll_node_id = *reference_frame_scroll_node_id;
                builder.current_clip_chain_id = *clip_chain_id;
                fragment.build_display_list(
                    builder,
                    containing_block,
                    *section,
                    *is_hit_test_for_scrollable_overflow,
                    *is_collapsed_table_borders,
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
    spatial_id: wr::SpatialId,

    /// The clip chain id of this stacking context if it has one. Used for filter clipping.
    clip_chain_id: Option<wr::ClipChainId>,

    /// The style of the fragment that established this stacking context.
    initializing_fragment_style: Option<ServoArc<ComputedValues>>,

    /// The [`FragmentFlags`] of the [`Fragment`] that established this stacking context.
    initializing_fragment_flags: FragmentFlags,

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
        spatial_id: wr::SpatialId,
        clip_chain_id: wr::ClipChainId,
        initializing_fragment_style: ServoArc<ComputedValues>,
        initializing_fragment_flags: FragmentFlags,
        context_type: StackingContextType,
    ) -> Self {
        // WebRender has two different ways of expressing "no clip." ClipChainId::INVALID should be
        // used for primitives, but `None` is used for stacking contexts and clip chains. We convert
        // to the `Option<ClipChainId>` representation here. Just passing Some(ClipChainId::INVALID)
        // leads to a crash.
        let clip_chain_id: Option<ClipChainId> = match clip_chain_id {
            ClipChainId::INVALID => None,
            clip_chain_id => Some(clip_chain_id),
        };
        Self {
            spatial_id,
            clip_chain_id,
            initializing_fragment_style: Some(initializing_fragment_style),
            initializing_fragment_flags,
            context_type,
            contents: vec![],
            real_stacking_contexts_and_positioned_stacking_containers: vec![],
            float_stacking_containers: vec![],
            atomic_inline_stacking_containers: vec![],
            debug_print_items: self.debug_print_items.is_some().then(|| vec![].into()),
        }
    }

    pub(crate) fn create_root(wr: &wr::DisplayListBuilder, debug: &DebugOptions) -> Self {
        Self {
            spatial_id: wr::SpaceAndClipInfo::root_scroll(wr.pipeline_id).spatial_id,
            clip_chain_id: None,
            initializing_fragment_style: None,
            initializing_fragment_flags: FragmentFlags::empty(),
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
        self.initializing_fragment_style
            .as_ref()
            .map_or(0, |style| {
                style.effective_z_index(self.initializing_fragment_flags)
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
        let style = match self.initializing_fragment_style.as_ref() {
            Some(style) => style,
            None => return false,
        };

        // WebRender only uses the stacking context to apply certain effects. If we don't
        // actually need to create a stacking context, just avoid creating one.
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
        builder.wr().push_stacking_context(
            LayoutPoint::zero(), // origin
            self.spatial_id,
            style.get_webrender_primitive_flags(),
            self.clip_chain_id,
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
        containing_block_rect: &PhysicalRect<Au>,
    ) {
        let style = if let Some(style) = &fragment_tree.canvas_background.style {
            style
        } else {
            // The root element has `display: none`,
            // or the canvas background is taken from `<body>` which has `display: none`
            return;
        };

        // The painting area is theoretically the infinite 2D plane,
        // but we need a rectangle with finite coordinates.
        //
        // If the document is smaller than the viewport (and doesn’t scroll),
        // we still want to paint the rest of the viewport.
        // If it’s larger, we also want to paint areas reachable after scrolling.
        let mut painting_area = fragment_tree
            .initial_containing_block
            .union(&fragment_tree.scrollable_overflow)
            .to_webrender();

        let background_color = style.resolve_color(&style.get_background().background_color);
        if background_color.alpha > 0.0 {
            let common = builder.common_properties(painting_area, style);
            let color = super::rgba(background_color);
            builder
                .display_list
                .wr
                .push_rect(&common, painting_area, color)
        }

        // `background-color` was comparatively easy,
        // but `background-image` needs a positioning area based on the root element.
        // Let’s find the corresponding fragment.

        // The fragment generated by the root element is the first one here, unless…
        let first_if_any = self.contents.first().or_else(|| {
            // There wasn’t any `StackingContextFragment` in the root `StackingContext`,
            // because the root element generates a stacking context. Let’s find that one.
            self.real_stacking_contexts_and_positioned_stacking_containers
                .first()
                .and_then(|first_child_stacking_context| {
                    first_child_stacking_context.contents.first()
                })
        });

        macro_rules! debug_panic {
            ($msg: expr) => {
                if cfg!(debug_assertions) {
                    panic!($msg);
                } else {
                    warn!($msg);
                    return;
                }
            };
        }

        let first_stacking_context_fragment = if let Some(first) = first_if_any {
            first
        } else {
            // This should only happen if the root element has `display: none`
            // TODO(servo#30569) revert to debug_panic!() once underlying bug is fixed
            log::warn!(
                "debug assertion failed! `CanvasBackground::for_root_element` should have returned `style: None`",
            );
            return;
        };

        let StackingContextContent::Fragment {
            fragment,
            scroll_node_id,
            containing_block,
            ..
        } = first_stacking_context_fragment
        else {
            debug_panic!("Expected a fragment, not a stacking container");
        };
        let box_fragment = match fragment {
            Fragment::Box(box_fragment) | Fragment::Float(box_fragment) => box_fragment,
            _ => debug_panic!("Expected a box-generated fragment"),
        };
        let box_fragment = &*box_fragment.borrow();

        // The `StackingContextFragment` we found is for the root DOM element:
        debug_assert_eq!(
            fragment.tag().map(|tag| tag.node),
            Some(fragment_tree.canvas_background.root_element),
        );

        // The root element may have a CSS transform, and we want the canvas’
        // background image to be transformed. To do so, take its `SpatialId`
        // (but not its `ClipId`)
        builder.current_scroll_node_id = *scroll_node_id;

        // Now we need express the painting area rectangle in the local coordinate system,
        // which differs from the top-level coordinate system based on…

        // Convert the painting area rectangle to the local coordinate system of this `SpatialId`
        if let Some(reference_frame_data) =
            box_fragment.reference_frame_data_if_necessary(containing_block_rect)
        {
            painting_area.min -= reference_frame_data.origin.to_webrender().to_vector();
            if let Some(transformed) = reference_frame_data
                .transform
                .inverse()
                .and_then(|inversed| inversed.outer_transformed_rect(&painting_area.to_rect()))
            {
                painting_area = transformed.to_box2d();
            } else {
                // The desired rect cannot be represented, so skip painting this background-image
                return;
            }
        }

        let mut fragment_builder = BuilderForBoxFragment::new(
            box_fragment,
            containing_block,
            false, /* is_hit_test_for_scrollable_overflow */
            false, /* is_collapsed_table_borders */
        );
        let painter = super::background::BackgroundPainter {
            style,
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
            builder.display_list.wr.pop_stacking_context();
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
        display_list: &mut DisplayList,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
        mode: StackingContextBuildMode,
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

                // If this fragment has a transform applied that makes it take up no space
                // then we don't need to create any stacking contexts for it.
                let has_non_invertible_transform = fragment
                    .has_non_invertible_transform_or_zero_scale(
                        &containing_block.rect.to_untyped(),
                    );
                if has_non_invertible_transform {
                    return;
                }

                fragment.build_stacking_context_tree(
                    fragment_clone,
                    display_list,
                    containing_block,
                    containing_block_info,
                    stacking_context,
                );
            },
            Fragment::AbsoluteOrFixedPositioned(fragment) => {
                let shared_fragment = fragment.borrow();
                let fragment_ref = match shared_fragment.fragment.as_ref() {
                    Some(fragment_ref) => fragment_ref,
                    None => unreachable!("Found hoisted box with missing fragment."),
                };

                fragment_ref.build_stacking_context_tree(
                    display_list,
                    containing_block_info,
                    stacking_context,
                    StackingContextBuildMode::IncludeHoisted,
                );
            },
            Fragment::Positioning(fragment) => {
                let fragment = fragment.borrow();
                fragment.build_stacking_context_tree(
                    display_list,
                    containing_block,
                    containing_block_info,
                    stacking_context,
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
                        clip_chain_id: containing_block.clip_chain_id,
                        containing_block: containing_block.rect,
                        fragment: fragment_clone,
                        is_hit_test_for_scrollable_overflow: false,
                        is_collapsed_table_borders: false,
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
    clip_chain_id: wr::ClipChainId,
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
        display_list: &mut DisplayList,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        parent_stacking_context: &mut StackingContext,
    ) {
        self.build_stacking_context_tree_maybe_creating_reference_frame(
            fragment,
            display_list,
            containing_block,
            containing_block_info,
            parent_stacking_context,
        );
    }

    fn build_stacking_context_tree_maybe_creating_reference_frame(
        &self,
        fragment: Fragment,
        display_list: &mut DisplayList,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        parent_stacking_context: &mut StackingContext,
    ) {
        let reference_frame_data =
            match self.reference_frame_data_if_necessary(&containing_block.rect) {
                Some(reference_frame_data) => reference_frame_data,
                None => {
                    return self.build_stacking_context_tree_maybe_creating_stacking_context(
                        fragment,
                        display_list,
                        containing_block,
                        containing_block_info,
                        parent_stacking_context,
                    );
                },
            };

        let new_spatial_id = display_list.push_reference_frame(
            reference_frame_data.origin.to_webrender(),
            &containing_block.scroll_node_id,
            self.style.get_box().transform_style.to_webrender(),
            wr::PropertyBinding::Value(reference_frame_data.transform),
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
            containing_block.clip_chain_id,
        );
        let new_containing_block_info =
            containing_block_info.new_for_non_absolute_descendants(&adjusted_containing_block);

        self.build_stacking_context_tree_maybe_creating_stacking_context(
            fragment,
            display_list,
            &adjusted_containing_block,
            &new_containing_block_info,
            parent_stacking_context,
        );

        display_list.pop_reference_frame();
    }

    fn build_stacking_context_tree_maybe_creating_stacking_context(
        &self,
        fragment: Fragment,
        display_list: &mut DisplayList,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        parent_stacking_context: &mut StackingContext,
    ) {
        let context_type = match self.get_stacking_context_type() {
            Some(context_type) => context_type,
            None => {
                self.build_stacking_context_tree_for_children(
                    fragment,
                    display_list,
                    containing_block,
                    containing_block_info,
                    parent_stacking_context,
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
        let stacking_context_clip_chain_id = build_clip_path_clip_chain_if_necessary(
            self.style.clone_clip_path(),
            display_list,
            &containing_block.scroll_node_id,
            &containing_block.clip_chain_id,
            BuilderForBoxFragment::new(
                self,
                &containing_block.rect,
                false, /* is_hit_test_for_scrollable_overflow */
                false, /* is_collapsed_table_borders */
            ),
        )
        .unwrap_or(containing_block.clip_chain_id);

        let mut child_stacking_context = parent_stacking_context.create_descendant(
            containing_block.scroll_node_id.spatial_id,
            stacking_context_clip_chain_id,
            self.style.clone(),
            self.base.flags,
            context_type,
        );
        self.build_stacking_context_tree_for_children(
            fragment,
            display_list,
            containing_block,
            containing_block_info,
            &mut child_stacking_context,
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
        display_list: &mut DisplayList,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
    ) {
        let mut new_scroll_node_id = containing_block.scroll_node_id;
        let mut new_clip_chain_id = containing_block.clip_chain_id;
        let mut new_scroll_frame_size = containing_block_info
            .for_non_absolute_descendants
            .scroll_frame_size;

        if let Some(scroll_node_id) = self.build_sticky_frame_if_necessary(
            display_list,
            &new_scroll_node_id,
            &containing_block.rect,
            &new_scroll_frame_size,
        ) {
            new_scroll_node_id = scroll_node_id;
        }

        if let Some(clip_chain_id) = self.build_clip_frame_if_necessary(
            display_list,
            &new_scroll_node_id,
            &new_clip_chain_id,
            &containing_block.rect,
        ) {
            new_clip_chain_id = clip_chain_id;
        }

        if let Some(clip_chain_id) = build_clip_path_clip_chain_if_necessary(
            self.style.clone_clip_path(),
            display_list,
            &new_scroll_node_id,
            &new_clip_chain_id,
            BuilderForBoxFragment::new(
                self,
                &containing_block.rect,
                false, /* is_hit_test_for_scrollable_overflow*/
                false, /* is_collapsed_table_borders */
            ),
        ) {
            new_clip_chain_id = clip_chain_id;
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
                    clip_chain_id: new_clip_chain_id,
                    section,
                    containing_block: containing_block.rect,
                    fragment: fragment.clone(),
                    is_hit_test_for_scrollable_overflow: false,
                    is_collapsed_table_borders: false,
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
            display_list,
            &new_scroll_node_id,
            &new_clip_chain_id,
            &containing_block.rect,
        ) {
            new_clip_chain_id = overflow_frame_data.clip_chain_id;
            if let Some(scroll_frame_data) = overflow_frame_data.scroll_frame_data {
                new_scroll_node_id = scroll_frame_data.scroll_tree_node_id;
                new_scroll_frame_size = Some(scroll_frame_data.scroll_frame_rect.size());

                stacking_context
                    .contents
                    .push(StackingContextContent::Fragment {
                        scroll_node_id: new_scroll_node_id,
                        reference_frame_scroll_node_id:
                            reference_frame_scroll_node_id_for_fragments,
                        clip_chain_id: new_clip_chain_id,
                        section,
                        containing_block: containing_block.rect,
                        fragment: fragment.clone(),
                        is_hit_test_for_scrollable_overflow: true,
                        is_collapsed_table_borders: false,
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
            new_clip_chain_id,
        );
        let for_non_absolute_descendants = ContainingBlock::new(
            content_rect,
            new_scroll_node_id,
            new_scroll_frame_size,
            new_clip_chain_id,
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

        for child in &self.children {
            child.build_stacking_context_tree(
                display_list,
                &new_containing_block_info,
                stacking_context,
                StackingContextBuildMode::SkipHoisted,
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
                    clip_chain_id: new_clip_chain_id,
                    section,
                    containing_block: containing_block.rect,
                    fragment: fragment.clone(),
                    is_hit_test_for_scrollable_overflow: false,
                    is_collapsed_table_borders: true,
                });
        }
    }

    fn build_clip_frame_if_necessary(
        &self,
        display_list: &mut DisplayList,
        parent_scroll_node_id: &ScrollTreeNodeId,
        parent_clip_chain_id: &wr::ClipChainId,
        containing_block_rect: &PhysicalRect<Au>,
    ) -> Option<wr::ClipChainId> {
        let position = self.style.get_box().position;
        // https://drafts.csswg.org/css2/#clipping
        // The clip property applies only to absolutely positioned elements
        if position != ComputedPosition::Absolute && position != ComputedPosition::Fixed {
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

        let clip_id = display_list
            .wr
            .define_clip_rect(parent_scroll_node_id.spatial_id, clip_rect);
        Some(display_list.define_clip_chain(*parent_clip_chain_id, [clip_id]))
    }

    fn build_overflow_frame_if_necessary(
        &self,
        display_list: &mut DisplayList,
        parent_scroll_node_id: &ScrollTreeNodeId,
        parent_clip_chain_id: &wr::ClipChainId,
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

            let clip_chain_id = display_list.clip_overflow_frame(
                parent_scroll_node_id,
                parent_clip_chain_id,
                overflow_clip_rect,
                radii,
            );

            return Some(OverflowFrameData {
                clip_chain_id,
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

        let clip_chain_id = display_list.clip_overflow_frame(
            parent_scroll_node_id,
            parent_clip_chain_id,
            scroll_frame_rect,
            BuilderForBoxFragment::new(self, containing_block_rect, false, false).border_radius,
        );

        let tag = self.base.tag?;
        let external_id = wr::ExternalScrollId(
            tag.to_display_list_fragment_id(),
            display_list.wr.pipeline_id,
        );

        let sensitivity = AxesScrollSensitivity {
            x: overflow.x.into(),
            y: overflow.y.into(),
        };

        let content_rect = self.scrollable_overflow().to_webrender();

        let scroll_tree_node_id = display_list.define_scroll_frame(
            parent_scroll_node_id,
            external_id,
            content_rect,
            scroll_frame_rect,
            sensitivity,
        );

        Some(OverflowFrameData {
            clip_chain_id,
            scroll_frame_data: Some(ScrollFrameData {
                scroll_tree_node_id,
                scroll_frame_rect,
            }),
        })
    }

    fn build_sticky_frame_if_necessary(
        &self,
        display_list: &mut DisplayList,
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
                &display_list.compositor_info.viewport_size
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

        let sticky_node_id = display_list.define_sticky_frame(
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

    /// Returns true if the given style contains a transform that is not invertible.
    fn has_non_invertible_transform_or_zero_scale(&self, containing_block: &Rect<Au>) -> bool {
        let list = &self.style.get_box().transform;
        match list.to_transform_3d_matrix(Some(&au_rect_to_length_rect(containing_block))) {
            Ok(t) => !t.0.is_invertible() || t.0.m11 == 0. || t.0.m22 == 0.,
            Err(_) => false,
        }
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
        // WebRender will end up dividing by the scale value of this transform, so we
        // want to ensure we don't feed it a divisor of 0.
        if transform.m11 == 0. || transform.m22 == 0. {
            return Some(LayoutTransform::identity());
        }

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
        display_list: &mut DisplayList,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
    ) {
        let rect = self
            .rect
            .translate(containing_block.rect.origin.to_vector());
        let new_containing_block = containing_block.new_replacing_rect(&rect);
        let new_containing_block_info =
            containing_block_info.new_for_non_absolute_descendants(&new_containing_block);

        for child in &self.children {
            child.build_stacking_context_tree(
                display_list,
                &new_containing_block_info,
                stacking_context,
                StackingContextBuildMode::SkipHoisted,
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
