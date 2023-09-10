/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::DisplayList;
use crate::cell::ArcRefCell;
use crate::display_list::conversions::ToWebRender;
use crate::display_list::DisplayListBuilder;
use crate::fragment_tree::{
    AnonymousFragment, BoxFragment, ContainingBlockManager, Fragment, FragmentTree,
};
use crate::geom::PhysicalRect;
use crate::style_ext::ComputedValuesExt;
use euclid::default::Rect;
use script_traits::compositor::{ScrollTreeNodeId, ScrollableNodeInfo};
use servo_arc::Arc as ServoArc;
use std::cmp::Ordering;
use std::mem;
use style::computed_values::float::T as ComputedFloat;
use style::computed_values::mix_blend_mode::T as ComputedMixBlendMode;
use style::computed_values::overflow_x::T as ComputedOverflow;
use style::computed_values::position::T as ComputedPosition;
use style::properties::ComputedValues;
use style::values::computed::ClipRectOrAuto;
use style::values::computed::Length;
use style::values::generics::box_::Perspective;
use style::values::generics::transform;
use style::values::specified::box_::DisplayOutside;
use webrender_api as wr;
use webrender_api::units::{LayoutPoint, LayoutRect, LayoutTransform, LayoutVector2D};
use webrender_api::ScrollSensitivity;

#[derive(Clone)]
pub(crate) struct ContainingBlock {
    /// The SpatialId of the spatial node that contains the children
    /// of this containing block.
    scroll_node_id: ScrollTreeNodeId,

    /// The WebRender ClipId to use for this children of this containing
    /// block.
    clip_chain_id: wr::ClipChainId,

    /// The physical rect of this containing block.
    rect: PhysicalRect<Length>,
}

impl ContainingBlock {
    pub(crate) fn new(
        rect: &PhysicalRect<Length>,
        scroll_node_id: ScrollTreeNodeId,
        clip_chain_id: wr::ClipChainId,
    ) -> Self {
        ContainingBlock {
            scroll_node_id,
            clip_chain_id,
            rect: *rect,
        }
    }

    pub(crate) fn new_replacing_rect(&self, rect: &PhysicalRect<Length>) -> Self {
        ContainingBlock {
            rect: *rect,
            ..*self
        }
    }
}

pub(crate) type ContainingBlockInfo<'a> = ContainingBlockManager<'a, ContainingBlock>;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum StackingContextSection {
    BackgroundsAndBorders,
    BlockBackgroundsAndBorders,
    Content,
    Outline,
}

impl DisplayList {
    pub fn build_stacking_context_tree(&mut self, fragment_tree: &FragmentTree) -> StackingContext {
        let root_clip_chain_id = self
            .wr
            .define_clip_chain(None, [wr::ClipId::root(self.wr.pipeline_id)]);

        let cb_for_non_fixed_descendants = ContainingBlock::new(
            &fragment_tree.initial_containing_block,
            self.compositor_info.root_scroll_node_id,
            root_clip_chain_id,
        );
        let cb_for_fixed_descendants = ContainingBlock::new(
            &fragment_tree.initial_containing_block,
            self.compositor_info.root_reference_frame_id,
            root_clip_chain_id,
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

        let mut root_stacking_context = StackingContext::create_root(&self.wr);
        for fragment in &fragment_tree.root_fragments {
            fragment.borrow().build_stacking_context_tree(
                fragment,
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
        let new_spatial_id = self.wr.push_reference_frame(
            origin,
            parent_scroll_node_id.spatial_id,
            transform_style,
            transform,
            kind,
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

    fn define_scroll_frame(
        &mut self,
        parent_scroll_node_id: &ScrollTreeNodeId,
        parent_clip_chain_id: &wr::ClipChainId,
        external_id: wr::ExternalScrollId,
        content_rect: LayoutRect,
        clip_rect: LayoutRect,
        scroll_sensitivity: ScrollSensitivity,
    ) -> (ScrollTreeNodeId, wr::ClipChainId) {
        let parent_space_and_clip_info = wr::SpaceAndClipInfo {
            spatial_id: parent_scroll_node_id.spatial_id,
            clip_id: wr::ClipId::root(self.wr.pipeline_id),
        };
        let new_clip_id = self
            .wr
            .define_clip_rect(&parent_space_and_clip_info, clip_rect);

        let new_clip_chain_id = self
            .wr
            .define_clip_chain(Some(*parent_clip_chain_id), [new_clip_id]);

        let new_spatial_id = self
            .wr
            .define_scroll_frame(
                &parent_space_and_clip_info,
                external_id,
                content_rect,
                clip_rect,
                scroll_sensitivity,
                LayoutVector2D::zero(), /* external_scroll_offset */
            )
            .spatial_id;

        let new_scroll_node_id = self.compositor_info.scroll_tree.add_scroll_tree_node(
            Some(&parent_scroll_node_id),
            new_spatial_id,
            Some(ScrollableNodeInfo {
                external_id,
                scrollable_size: content_rect.size - clip_rect.size,
                scroll_sensitivity,
                offset: LayoutVector2D::zero(),
            }),
        );
        (new_scroll_node_id, new_clip_chain_id)
    }
}

pub(crate) struct StackingContextFragment {
    scroll_node_id: ScrollTreeNodeId,
    clip_chain_id: wr::ClipChainId,
    section: StackingContextSection,
    containing_block: PhysicalRect<Length>,
    fragment: ArcRefCell<Fragment>,
}

impl StackingContextFragment {
    fn build_display_list(&self, builder: &mut DisplayListBuilder) {
        builder.current_scroll_node_id = self.scroll_node_id;
        builder.current_clip_chain_id = self.clip_chain_id;
        self.fragment
            .borrow()
            .build_display_list(builder, &self.containing_block, self.section);
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum StackingContextType {
    Real,
    PseudoPositioned,
    PseudoFloat,
    PseudoAtomicInline,
}

/// A [StackingContext] represents either a stacking context or a stacking
/// container according to the definitions outlined in
/// <https://drafts.csswg.org/css-position-4/#painting-order>
/// Stacking containers are sometimes called "pseudo-stacking contexts"
/// in the Servo source.
pub struct StackingContext {
    /// The spatial id of this fragment. This is used to properly handle
    /// things like preserve-3d.
    spatial_id: wr::SpatialId,

    /// The fragment that established this stacking context.
    initializing_fragment_style: Option<ServoArc<ComputedValues>>,

    /// The type of this StackingContext. Used for collecting and sorting.
    context_type: StackingContextType,

    /// Fragments that make up the content of this stacking context.
    fragments: Vec<StackingContextFragment>,

    /// All non-float stacking context and pseudo stacking context children
    /// of this stacking context.
    stacking_contexts: Vec<StackingContext>,

    /// All float stacking container children of this stacking context.
    /// These are stored separately because they should not be passed up to
    /// their real stacking context ancestors. From the definition of stacking
    /// containers from <https://drafts.csswg.org/css-position-4#painting-order>:
    ///
    /// > To paint a stacking container, given a box root and a canvas canvas:
    /// >  1. Paint a stacking context given root and canvas, treating root as
    /// >     if it created a new stacking context, but omitting any positioned
    /// >     descendants or descendants that actually create a stacking context
    /// >     (letting the parent stacking context paint them, instead).
    ///
    /// Note that all stacking containers / pseudo stacking contexts are passed up
    /// to parent stacking contexts, except in the case of floats.
    float_stacking_contexts: Vec<StackingContext>,
}

impl StackingContext {
    pub(crate) fn new(
        spatial_id: wr::SpatialId,
        initializing_fragment_style: ServoArc<ComputedValues>,
        context_type: StackingContextType,
    ) -> Self {
        Self {
            spatial_id,
            initializing_fragment_style: Some(initializing_fragment_style),
            context_type,
            fragments: vec![],
            stacking_contexts: vec![],
            float_stacking_contexts: vec![],
        }
    }

    pub(crate) fn create_root(wr: &wr::DisplayListBuilder) -> Self {
        Self {
            spatial_id: wr::SpaceAndClipInfo::root_scroll(wr.pipeline_id).spatial_id,
            initializing_fragment_style: None,
            context_type: StackingContextType::Real,
            fragments: vec![],
            stacking_contexts: vec![],
            float_stacking_contexts: vec![],
        }
    }

    /// Add a child stacking context to this stacking context.
    fn add_stacking_context(&mut self, stacking_context: StackingContext) {
        if stacking_context.context_type == StackingContextType::PseudoFloat {
            self.float_stacking_contexts.push(stacking_context);
        } else {
            self.stacking_contexts.push(stacking_context);
        }
    }

    fn z_index(&self) -> i32 {
        self.initializing_fragment_style
            .as_ref()
            .map_or(0, |style| style.effective_z_index())
    }

    pub(crate) fn sort(&mut self) {
        self.fragments.sort_by(|a, b| a.section.cmp(&b.section));

        self.stacking_contexts.sort_by(|a, b| {
            let a_z_index = a.z_index();
            let b_z_index = b.z_index();
            if a_z_index != 0 || b_z_index != 0 {
                return a_z_index.cmp(&b_z_index);
            }

            match (a.context_type, b.context_type) {
                (StackingContextType::PseudoFloat, StackingContextType::PseudoFloat) => {
                    Ordering::Equal
                },
                (StackingContextType::PseudoFloat, _) => Ordering::Less,
                (_, StackingContextType::PseudoFloat) => Ordering::Greater,
                (_, _) => Ordering::Equal,
            }
        });
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
            !style.has_transform_or_perspective()
        {
            return false;
        }

        // Create the filter pipeline.
        let mut filters: Vec<wr::FilterOp> = effects
            .filter
            .0
            .iter()
            .map(ToWebRender::to_webrender)
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
            None, // clip_chain_id
            style.get_used_transform_style().to_webrender(),
            effects.mix_blend_mode.to_webrender(),
            &filters,
            &vec![], // filter_datas
            &vec![], // filter_primitives
            wr::RasterSpace::Screen,
            wr::StackingContextFlags::empty(),
        );

        true
    }

    /// https://drafts.csswg.org/css-backgrounds/#special-backgrounds
    ///
    /// This is only called for the root `StackingContext`
    pub(crate) fn build_canvas_background_display_list(
        &self,
        builder: &mut DisplayListBuilder,
        fragment_tree: &crate::FragmentTree,
        containing_block_rect: &PhysicalRect<Length>,
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

        let background_color = style.resolve_color(style.get_background().background_color.clone());
        if background_color.alpha > 0 {
            let common = builder.common_properties(painting_area, &style);
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
        let first_if_any = self.fragments.first().or_else(|| {
            // There wasn’t any `StackingContextFragment` in the root `StackingContext`,
            // because the root element generates a stacking context. Let’s find that one.
            self.stacking_contexts
                .first()
                .and_then(|first_child_stacking_context| {
                    first_child_stacking_context.fragments.first()
                })
        });

        macro_rules! debug_panic {
            ($msg: expr) => {
                if cfg!(debug_assertions) {
                    panic!($msg)
                }
            };
        }

        let first_stacking_context_fragment = if let Some(first) = first_if_any {
            first
        } else {
            // This should only happen if the root element has `display: none`
            debug_panic!("`CanvasBackground::for_root_element` should have returned `style: None`");
            return;
        };

        let fragment = first_stacking_context_fragment.fragment.borrow();
        let box_fragment = match &*fragment {
            Fragment::Box(box_fragment) | Fragment::Float(box_fragment) => box_fragment,
            _ => {
                debug_panic!("Expected a box-generated fragment");
                return;
            },
        };

        // The `StackingContextFragment` we found is for the root DOM element:
        debug_assert_eq!(
            fragment.tag().map(|tag| tag.node),
            Some(fragment_tree.canvas_background.root_element),
        );

        // The root element may have a CSS transform, and we want the canvas’
        // background image to be transformed. To do so, take its `SpatialId`
        // (but not its `ClipId`)
        builder.current_scroll_node_id = first_stacking_context_fragment.scroll_node_id;

        // Now we need express the painting area rectangle in the local coordinate system,
        // which differs from the top-level coordinate system based on…

        // Convert the painting area rectangle to the local coordinate system of this `SpatialId`
        if let Some(reference_frame_data) =
            box_fragment.reference_frame_data_if_necessary(containing_block_rect)
        {
            painting_area.origin -= reference_frame_data.origin.to_webrender().to_vector();
            if let Some(transformed) = reference_frame_data
                .transform
                .inverse()
                .and_then(|inversed| inversed.outer_transformed_rect(&painting_area))
            {
                painting_area = transformed
            } else {
                // The desired rect cannot be represented, so skip painting this background-image
                return;
            }
        }

        let mut fragment_builder = super::BuilderForBoxFragment::new(
            box_fragment,
            &first_stacking_context_fragment.containing_block,
        );
        let source = super::background::Source::Canvas {
            style,
            painting_area,
        };
        fragment_builder.build_background_image(builder, source);
    }

    pub(crate) fn build_display_list(&self, builder: &mut DisplayListBuilder) {
        let pushed_context = self.push_webrender_stacking_context_if_necessary(builder);

        // Properly order display items that make up a stacking context. "Steps" here
        // refer to the steps in CSS 2.1 Appendix E.

        // Steps 1 and 2: Borders and background for the root
        let mut child_fragments = self.fragments.iter().peekable();
        while child_fragments.peek().map_or(false, |child| {
            child.section == StackingContextSection::BackgroundsAndBorders
        }) {
            child_fragments.next().unwrap().build_display_list(builder);
        }

        // Step 3: Positioned descendants with negative z-indices
        let mut child_stacking_contexts = self.stacking_contexts.iter().peekable();
        while child_stacking_contexts
            .peek()
            .map_or(false, |child| child.z_index() < 0)
        {
            let child_context = child_stacking_contexts.next().unwrap();
            child_context.build_display_list(builder);
        }

        // Step 4: Block backgrounds and borders
        while child_fragments.peek().map_or(false, |child| {
            child.section == StackingContextSection::BlockBackgroundsAndBorders
        }) {
            child_fragments.next().unwrap().build_display_list(builder);
        }

        // Step 5: Floats
        for child_context in &self.float_stacking_contexts {
            child_context.build_display_list(builder);
        }

        // Step 6: Content
        while child_fragments.peek().map_or(false, |child| {
            child.section == StackingContextSection::Content
        }) {
            child_fragments.next().unwrap().build_display_list(builder);
        }

        // Step 7, 8 & 9: Inlines that generate stacking contexts and positioned
        // descendants with nonnegative, numeric z-indices
        for child_context in child_stacking_contexts {
            child_context.build_display_list(builder);
        }

        // Step 10: Outline
        while child_fragments.peek().map_or(false, |child| {
            child.section == StackingContextSection::Outline
        }) {
            child_fragments.next().unwrap().build_display_list(builder);
        }

        if pushed_context {
            builder.display_list.wr.pop_stacking_context();
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
        fragment_ref: &ArcRefCell<Fragment>,
        display_list: &mut DisplayList,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
        mode: StackingContextBuildMode,
    ) {
        let containing_block = containing_block_info.get_containing_block_for_fragment(self);
        match self {
            Fragment::Box(fragment) | Fragment::Float(fragment) => {
                if mode == StackingContextBuildMode::SkipHoisted &&
                    fragment.style.clone_position().is_absolutely_positioned()
                {
                    return;
                }

                // If this fragment has a transform applied that makes it take up no space
                // then we don't need to create any stacking contexts for it.
                let has_non_invertible_transform =
                    fragment.has_non_invertible_transform(&containing_block.rect.to_untyped());
                if has_non_invertible_transform {
                    return;
                }

                fragment.build_stacking_context_tree(
                    fragment_ref,
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

                fragment_ref.borrow().build_stacking_context_tree(
                    fragment_ref,
                    display_list,
                    containing_block_info,
                    stacking_context,
                    StackingContextBuildMode::IncludeHoisted,
                );
            },
            Fragment::Anonymous(fragment) => {
                fragment.build_stacking_context_tree(
                    display_list,
                    containing_block,
                    containing_block_info,
                    stacking_context,
                );
            },
            Fragment::Text(_) | Fragment::Image(_) | Fragment::IFrame(_) => {
                stacking_context.fragments.push(StackingContextFragment {
                    section: StackingContextSection::Content,
                    scroll_node_id: containing_block.scroll_node_id,
                    clip_chain_id: containing_block.clip_chain_id,
                    containing_block: containing_block.rect,
                    fragment: fragment_ref.clone(),
                });
            },
        }
    }
}

struct ReferenceFrameData {
    origin: crate::geom::PhysicalPoint<Length>,
    transform: LayoutTransform,
    kind: wr::ReferenceFrameKind,
}

impl BoxFragment {
    fn get_stacking_context_type(&self) -> Option<StackingContextType> {
        if self.style.establishes_stacking_context() {
            return Some(StackingContextType::Real);
        }

        let box_style = &self.style.get_box();
        if box_style.position != ComputedPosition::Static {
            return Some(StackingContextType::PseudoPositioned);
        }

        if box_style.float != ComputedFloat::None {
            return Some(StackingContextType::PseudoFloat);
        }

        if box_style.display.is_atomic_inline_level() {
            return Some(StackingContextType::PseudoAtomicInline);
        }

        None
    }

    fn get_stacking_context_section(&self) -> StackingContextSection {
        if self.get_stacking_context_type().is_some() {
            return StackingContextSection::BackgroundsAndBorders;
        }

        if self.style.get_box().display.outside() == DisplayOutside::Inline {
            return StackingContextSection::Content;
        }

        StackingContextSection::BlockBackgroundsAndBorders
    }

    fn build_stacking_context_tree(
        &self,
        fragment: &ArcRefCell<Fragment>,
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
        fragment: &ArcRefCell<Fragment>,
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
        assert!(self
            .style
            .establishes_containing_block_for_all_descendants());
        let adjusted_containing_block = ContainingBlock::new(
            &containing_block
                .rect
                .translate(-reference_frame_data.origin.to_vector()),
            new_spatial_id,
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
        fragment: &ArcRefCell<Fragment>,
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

        let mut child_stacking_context = StackingContext::new(
            containing_block.scroll_node_id.spatial_id,
            self.style.clone(),
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
        if context_type != StackingContextType::Real {
            stolen_children = mem::replace(
                &mut child_stacking_context.stacking_contexts,
                stolen_children,
            );
        }

        child_stacking_context.sort();
        parent_stacking_context.add_stacking_context(child_stacking_context);
        parent_stacking_context
            .stacking_contexts
            .append(&mut stolen_children);
    }

    fn build_stacking_context_tree_for_children(
        &self,
        fragment: &ArcRefCell<Fragment>,
        display_list: &mut DisplayList,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
    ) {
        let mut new_scroll_node_id = containing_block.scroll_node_id;
        let mut new_clip_chain_id = containing_block.clip_chain_id;
        if let Some(clip_chain_id) = self.build_clip_frame_if_necessary(
            display_list,
            &new_scroll_node_id,
            &new_clip_chain_id,
            &containing_block.rect,
        ) {
            new_clip_chain_id = clip_chain_id;
        }

        stacking_context.fragments.push(StackingContextFragment {
            scroll_node_id: new_scroll_node_id,
            clip_chain_id: new_clip_chain_id,
            section: self.get_stacking_context_section(),
            containing_block: containing_block.rect,
            fragment: fragment.clone(),
        });
        if self.style.get_outline().outline_width.px() > 0.0 {
            stacking_context.fragments.push(StackingContextFragment {
                scroll_node_id: new_scroll_node_id,
                clip_chain_id: new_clip_chain_id,
                section: StackingContextSection::Outline,
                containing_block: containing_block.rect,
                fragment: fragment.clone(),
            });
        }

        // We want to build the scroll frame after the background and border, because
        // they shouldn't scroll with the rest of the box content.
        if let Some((scroll_node_id, clip_chain_id)) = self.build_scroll_frame_if_necessary(
            display_list,
            &new_scroll_node_id,
            &new_clip_chain_id,
            &containing_block.rect,
        ) {
            new_scroll_node_id = scroll_node_id;
            new_clip_chain_id = clip_chain_id;
        }

        let padding_rect = self
            .padding_rect()
            .to_physical(self.style.writing_mode, &containing_block.rect)
            .translate(containing_block.rect.origin.to_vector());
        let content_rect = self
            .content_rect
            .to_physical(self.style.writing_mode, &containing_block.rect)
            .translate(containing_block.rect.origin.to_vector());

        let for_absolute_descendants =
            ContainingBlock::new(&padding_rect, new_scroll_node_id, new_clip_chain_id);
        let for_non_absolute_descendants =
            ContainingBlock::new(&content_rect, new_scroll_node_id, new_clip_chain_id);

        // Create a new `ContainingBlockInfo` for descendants depending on
        // whether or not this fragment establishes a containing block for
        // absolute and fixed descendants.
        let new_containing_block_info = if self
            .style
            .establishes_containing_block_for_all_descendants()
        {
            containing_block_info.new_for_absolute_and_fixed_descendants(
                &for_non_absolute_descendants,
                &for_absolute_descendants,
            )
        } else if self
            .style
            .establishes_containing_block_for_absolute_descendants()
        {
            containing_block_info.new_for_absolute_descendants(
                &for_non_absolute_descendants,
                &for_absolute_descendants,
            )
        } else {
            containing_block_info.new_for_non_absolute_descendants(&for_non_absolute_descendants)
        };

        for child in &self.children {
            child.borrow().build_stacking_context_tree(
                child,
                display_list,
                &new_containing_block_info,
                stacking_context,
                StackingContextBuildMode::SkipHoisted,
            );
        }
    }

    fn build_clip_frame_if_necessary(
        &self,
        display_list: &mut DisplayList,
        parent_scroll_node_id: &ScrollTreeNodeId,
        parent_clip_chain_id: &wr::ClipChainId,
        containing_block_rect: &PhysicalRect<Length>,
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

        let border_rect = self
            .border_rect()
            .to_physical(self.style.writing_mode, &containing_block_rect);
        let clip_rect = clip_rect
            .for_border_rect(border_rect)
            .translate(containing_block_rect.origin.to_vector())
            .to_webrender();

        let parent_space_and_clip = &wr::SpaceAndClipInfo {
            spatial_id: parent_scroll_node_id.spatial_id,
            clip_id: wr::ClipId::root(display_list.wr.pipeline_id),
        };

        let clip_id = display_list
            .wr
            .define_clip_rect(&parent_space_and_clip, clip_rect);
        Some(
            display_list
                .wr
                .define_clip_chain(Some(*parent_clip_chain_id), [clip_id]),
        )
    }

    fn build_scroll_frame_if_necessary(
        &self,
        display_list: &mut DisplayList,
        parent_scroll_node_id: &ScrollTreeNodeId,
        parent_clip_id: &wr::ClipChainId,
        containing_block_rect: &PhysicalRect<Length>,
    ) -> Option<(ScrollTreeNodeId, wr::ClipChainId)> {
        let overflow_x = self.style.get_box().overflow_x;
        let overflow_y = self.style.get_box().overflow_y;
        if overflow_x == ComputedOverflow::Visible && overflow_y == ComputedOverflow::Visible {
            return None;
        }

        let tag = self.base.tag?;
        let external_id = wr::ExternalScrollId(
            tag.to_display_list_fragment_id(),
            display_list.wr.pipeline_id,
        );

        let sensitivity =
            if ComputedOverflow::Hidden == overflow_x && ComputedOverflow::Hidden == overflow_y {
                ScrollSensitivity::Script
            } else {
                ScrollSensitivity::ScriptAndInputEvents
            };

        let padding_rect = self
            .padding_rect()
            .to_physical(self.style.writing_mode, &containing_block_rect)
            .translate(containing_block_rect.origin.to_vector())
            .to_webrender();

        Some(
            display_list.define_scroll_frame(
                parent_scroll_node_id,
                parent_clip_id,
                external_id,
                self.scrollable_overflow(&containing_block_rect)
                    .to_webrender(),
                padding_rect,
                sensitivity,
            ),
        )
    }

    /// Optionally returns the data for building a reference frame, without yet building it.
    fn reference_frame_data_if_necessary(
        &self,
        containing_block_rect: &PhysicalRect<Length>,
    ) -> Option<ReferenceFrameData> {
        if !self.style.has_transform_or_perspective() {
            return None;
        }

        let relative_border_rect = self
            .border_rect()
            .to_physical(self.style.writing_mode, &containing_block_rect);
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
    fn has_non_invertible_transform(&self, containing_block: &Rect<Length>) -> bool {
        let list = &self.style.get_box().transform;
        match list.to_transform_3d_matrix(Some(containing_block)) {
            Ok(t) => !t.0.is_invertible(),
            Err(_) => false,
        }
    }

    /// Returns the 4D matrix representing this fragment's transform.
    pub fn calculate_transform_matrix(
        &self,
        border_rect: &Rect<Length>,
    ) -> Option<LayoutTransform> {
        let list = &self.style.get_box().transform;
        let transform =
            LayoutTransform::from_untyped(&list.to_transform_3d_matrix(Some(&border_rect)).ok()?.0);
        // WebRender will end up dividing by the scale value of this transform, so we
        // want to ensure we don't feed it a divisor of 0.
        assert_ne!(transform.m11, 0.);
        assert_ne!(transform.m22, 0.);

        let transform_origin = &self.style.get_box().transform_origin;
        let transform_origin_x = transform_origin
            .horizontal
            .percentage_relative_to(border_rect.size.width)
            .px();
        let transform_origin_y = transform_origin
            .vertical
            .percentage_relative_to(border_rect.size.height)
            .px();
        let transform_origin_z = transform_origin.depth.px();

        let pre_transform = LayoutTransform::translation(
            transform_origin_x,
            transform_origin_y,
            transform_origin_z,
        );
        let post_transform = LayoutTransform::translation(
            -transform_origin_x,
            -transform_origin_y,
            -transform_origin_z,
        );

        Some(post_transform.then(&transform).then(&pre_transform))
    }

    /// Returns the 4D matrix representing this fragment's perspective.
    pub fn calculate_perspective_matrix(
        &self,
        border_rect: &Rect<Length>,
    ) -> Option<LayoutTransform> {
        match self.style.get_box().perspective {
            Perspective::Length(length) => {
                let perspective_origin = &self.style.get_box().perspective_origin;
                let perspective_origin = LayoutPoint::new(
                    perspective_origin
                        .horizontal
                        .percentage_relative_to(border_rect.size.width)
                        .px(),
                    perspective_origin
                        .vertical
                        .percentage_relative_to(border_rect.size.height)
                        .px(),
                );

                let pre_transform =
                    LayoutTransform::translation(perspective_origin.x, perspective_origin.y, 0.0);
                let post_transform =
                    LayoutTransform::translation(-perspective_origin.x, -perspective_origin.y, 0.0);

                let perspective_matrix = LayoutTransform::from_untyped(
                    &transform::create_perspective_matrix(length.px()),
                );

                Some(
                    post_transform
                        .then(&perspective_matrix)
                        .then(&pre_transform),
                )
            },
            Perspective::None => None,
        }
    }
}

impl AnonymousFragment {
    fn build_stacking_context_tree(
        &self,
        display_list: &mut DisplayList,
        containing_block: &ContainingBlock,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
    ) {
        let rect = self
            .rect
            .to_physical(self.mode, &containing_block.rect)
            .translate(containing_block.rect.origin.to_vector());
        let new_containing_block = containing_block.new_replacing_rect(&rect);
        let new_containing_block_info =
            containing_block_info.new_for_non_absolute_descendants(&new_containing_block);

        for child in &self.children {
            child.borrow().build_stacking_context_tree(
                child,
                display_list,
                &new_containing_block_info,
                stacking_context,
                StackingContextBuildMode::SkipHoisted,
            );
        }
    }
}
