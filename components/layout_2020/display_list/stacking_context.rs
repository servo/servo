/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cell::ArcRefCell;
use crate::display_list::conversions::ToWebRender;
use crate::display_list::DisplayListBuilder;
use crate::fragments::{AbsoluteOrFixedPositionedFragment, AnonymousFragment, BoxFragment};
use crate::fragments::{Fragment, HoistedFloatFragment};
use crate::geom::PhysicalRect;
use crate::style_ext::ComputedValuesExt;
use euclid::default::Rect;
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
use webrender_api::units::{LayoutPoint, LayoutTransform, LayoutVector2D};

#[derive(Clone)]
pub(crate) struct ContainingBlock {
    /// The SpaceAndClipInfo that contains the children of the fragment that
    /// established this containing block.
    space_and_clip: wr::SpaceAndClipInfo,

    /// The physical rect of this containing block.
    rect: PhysicalRect<Length>,
}

impl ContainingBlock {
    pub(crate) fn new(rect: &PhysicalRect<Length>, space_and_clip: wr::SpaceAndClipInfo) -> Self {
        ContainingBlock {
            space_and_clip,
            rect: *rect,
        }
    }
}

#[derive(Clone)]
pub(crate) struct ContainingBlockInfo {
    /// The positioning rectangle established by the parent. This is sometimes
    /// called the "containing block" in layout_2020.
    pub rect: PhysicalRect<Length>,

    /// The nearest real containing block at this point in the construction of
    /// the stacking context tree.
    pub nearest_containing_block: Option<ContainingBlock>,

    /// The nearest containing block for all descendants at this point in the
    /// stacking context tree. This containing blocks contains fixed position
    /// elements.
    pub containing_block_for_all_descendants: ContainingBlock,
}

pub(crate) struct StackingContextBuilder<'a> {
    /// The current SpatialId and ClipId information for this `DisplayListBuilder`.
    pub current_space_and_clip: wr::SpaceAndClipInfo,

    /// The id of the nearest ancestor reference frame for this `DisplayListBuilder`.
    nearest_reference_frame: wr::SpatialId,

    wr: &'a mut wr::DisplayListBuilder,
}

impl<'a> StackingContextBuilder<'a> {
    pub fn new(wr: &'a mut wr::DisplayListBuilder) -> Self {
        Self {
            current_space_and_clip: wr::SpaceAndClipInfo::root_scroll(wr.pipeline_id),
            nearest_reference_frame: wr::SpatialId::root_reference_frame(wr.pipeline_id),
            wr,
        }
    }

    fn clipping_and_scrolling_scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let previous_space_and_clip = self.current_space_and_clip;
        let previous_nearest_reference_frame = self.nearest_reference_frame;

        let result = f(self);

        self.current_space_and_clip = previous_space_and_clip;
        self.nearest_reference_frame = previous_nearest_reference_frame;

        result
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum StackingContextSection {
    BackgroundsAndBorders,
    BlockBackgroundsAndBorders,
    Content,
}

pub(crate) struct StackingContextFragment {
    space_and_clip: wr::SpaceAndClipInfo,
    section: StackingContextSection,
    containing_block: PhysicalRect<Length>,
    fragment: ArcRefCell<Fragment>,
}

impl StackingContextFragment {
    fn build_display_list(&self, builder: &mut DisplayListBuilder) {
        builder.current_space_and_clip = self.space_and_clip;
        self.fragment
            .borrow()
            .build_display_list(builder, &self.containing_block);
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum StackingContextType {
    Real,
    PseudoPositioned,
    PseudoFloat,
    PseudoAtomicInline,
}

pub(crate) struct StackingContext {
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

    /// All float pseudo stacking context children of this stacking context.
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

    fn push_webrender_stacking_context_if_necessary<'a>(
        &self,
        builder: &'a mut DisplayListBuilder,
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
        builder.wr.push_stacking_context(
            LayoutPoint::zero(), // origin
            self.spatial_id,
            style.get_webrender_primitive_flags(),
            None, // clip_id
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

        let background_color = style.resolve_color(style.get_background().background_color);
        if background_color.alpha > 0 {
            let common = builder.common_properties(painting_area, &style);
            let color = super::rgba(background_color);
            builder.wr.push_rect(&common, painting_area, color)
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
        let box_fragment = if let Fragment::Box(box_fragment) = &*fragment {
            box_fragment
        } else {
            debug_panic!("Expected a box-generated fragment");
            return;
        };

        // The `StackingContextFragment` we found is for the root DOM element:
        debug_assert_eq!(
            box_fragment.tag.node(),
            fragment_tree.canvas_background.root_element
        );

        // The root element may have a CSS transform,
        // and we want the canvas’ background image to be transformed.
        // To do so, take its `SpatialId` (but not its `ClipId`)
        builder.current_space_and_clip.spatial_id =
            first_stacking_context_fragment.space_and_clip.spatial_id;

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
                .and_then(|inversed| inversed.transform_rect(&painting_area))
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

        if pushed_context {
            builder.wr.pop_stacking_context();
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
        builder: &mut StackingContextBuilder,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
        mode: StackingContextBuildMode,
    ) {
        match self {
            Fragment::Box(fragment) => {
                if mode == StackingContextBuildMode::SkipHoisted &&
                    fragment.style.clone_position().is_absolutely_positioned()
                {
                    return;
                }

                // If this fragment has a transform applied that makes it take up no space
                // then we don't need to create any stacking contexts for it.
                let has_non_invertible_transform =
                    fragment.has_non_invertible_transform(&containing_block_info.rect.to_untyped());
                if has_non_invertible_transform {
                    return;
                }

                fragment.build_stacking_context_tree(
                    fragment_ref,
                    builder,
                    containing_block_info,
                    stacking_context,
                );
            },
            Fragment::HoistedFloat(fragment) => {
                fragment.build_stacking_context_tree(
                    builder,
                    containing_block_info,
                    stacking_context,
                );
            },
            Fragment::Float => {},
            Fragment::AbsoluteOrFixedPositioned(fragment) => {
                fragment.build_stacking_context_tree(
                    builder,
                    containing_block_info,
                    stacking_context,
                );
            },
            Fragment::Anonymous(fragment) => {
                fragment.build_stacking_context_tree(
                    builder,
                    containing_block_info,
                    stacking_context,
                );
            },
            Fragment::Text(_) | Fragment::Image(_) => {
                stacking_context.fragments.push(StackingContextFragment {
                    section: StackingContextSection::Content,
                    space_and_clip: builder.current_space_and_clip,
                    containing_block: containing_block_info.rect,
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

    fn build_containing_block<'a>(
        &'a self,
        builder: &mut StackingContextBuilder,
        padding_rect: &PhysicalRect<Length>,
        containing_block_info: &mut ContainingBlockInfo,
    ) {
        if !self.style.establishes_containing_block() {
            return;
        }

        let new_containing_block =
            ContainingBlock::new(padding_rect, builder.current_space_and_clip);

        if self
            .style
            .establishes_containing_block_for_all_descendants()
        {
            containing_block_info.nearest_containing_block = None;
            containing_block_info.containing_block_for_all_descendants = new_containing_block;
        } else {
            containing_block_info.nearest_containing_block = Some(new_containing_block);
        }
    }

    fn build_stacking_context_tree(
        &self,
        fragment: &ArcRefCell<Fragment>,
        builder: &mut StackingContextBuilder,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
    ) {
        builder.clipping_and_scrolling_scope(|builder| {
            self.adjust_spatial_id_for_positioning(builder);

            match self.get_stacking_context_type() {
                Some(context_type) => {
                    self.build_stacking_context_tree_creating_stacking_context(
                        fragment,
                        builder,
                        containing_block_info,
                        stacking_context,
                        context_type,
                    );
                },
                None => {
                    self.build_stacking_context_tree_for_children(
                        fragment,
                        builder,
                        containing_block_info,
                        stacking_context,
                    );
                },
            }
        });
    }

    fn build_stacking_context_tree_creating_stacking_context(
        &self,
        fragment: &ArcRefCell<Fragment>,
        builder: &mut StackingContextBuilder,
        containing_block_info: &ContainingBlockInfo,
        parent_stacking_context: &mut StackingContext,
        context_type: StackingContextType,
    ) {
        // If we are creating a stacking context, we may also need to create a reference
        // frame first.
        let reference_frame_data =
            self.reference_frame_data_if_necessary(&containing_block_info.rect);

        // WebRender reference frames establish a new coordinate system at their origin
        // (the border box of the fragment). We need to ensure that any coordinates we
        // give to WebRender in this reference frame are relative to the fragment border
        // box. We do this by adjusting the containing block origin.
        let mut new_containing_block_info = containing_block_info.clone();

        if let Some(reference_frame_data) = &reference_frame_data {
            new_containing_block_info.rect.origin -= reference_frame_data.origin.to_vector();
            builder.current_space_and_clip.spatial_id = builder.wr.push_reference_frame(
                reference_frame_data.origin.to_webrender(),
                builder.current_space_and_clip.spatial_id,
                self.style.get_box().transform_style.to_webrender(),
                wr::PropertyBinding::Value(reference_frame_data.transform),
                reference_frame_data.kind,
            );
            builder.nearest_reference_frame = builder.current_space_and_clip.spatial_id;
        }

        let mut child_stacking_context = StackingContext::new(
            builder.current_space_and_clip.spatial_id,
            self.style.clone(),
            context_type,
        );
        self.build_stacking_context_tree_for_children(
            fragment,
            builder,
            &new_containing_block_info,
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
        parent_stacking_context
            .stacking_contexts
            .push(child_stacking_context);
        parent_stacking_context
            .stacking_contexts
            .append(&mut stolen_children);

        if reference_frame_data.is_some() {
            builder.wr.pop_reference_frame();
        }
    }

    fn build_stacking_context_tree_for_children<'a>(
        &'a self,
        fragment: &ArcRefCell<Fragment>,
        builder: &mut StackingContextBuilder,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
    ) {
        self.build_clip_frame_if_necessary(builder, containing_block_info);
        stacking_context.fragments.push(StackingContextFragment {
            space_and_clip: builder.current_space_and_clip,
            section: self.get_stacking_context_section(),
            containing_block: containing_block_info.rect,
            fragment: fragment.clone(),
        });

        // We want to build the scroll frame after the background and border, because
        // they shouldn't scroll with the rest of the box content.
        self.build_scroll_frame_if_necessary(builder, containing_block_info);

        let padding_rect = self
            .padding_rect()
            .to_physical(self.style.writing_mode, &containing_block_info.rect)
            .translate(containing_block_info.rect.origin.to_vector());
        let mut new_containing_block_info = containing_block_info.clone();
        new_containing_block_info.rect = self
            .content_rect
            .to_physical(self.style.writing_mode, &new_containing_block_info.rect)
            .translate(new_containing_block_info.rect.origin.to_vector());

        // If we establish a containing block we use the padding rect as the offset. This is
        // because for all but the initial containing block, the padding rect determines
        // the size and position of the containing block.
        self.build_containing_block(builder, &padding_rect, &mut new_containing_block_info);

        for child in &self.children {
            child.borrow().build_stacking_context_tree(
                child,
                builder,
                &new_containing_block_info,
                stacking_context,
                StackingContextBuildMode::SkipHoisted,
            );
        }
    }

    fn adjust_spatial_id_for_positioning(&self, builder: &mut StackingContextBuilder) {
        if self.style.get_box().position != ComputedPosition::Fixed {
            return;
        }

        // TODO(mrobinson): Eventually this should use the spatial id of the reference
        // frame that is the parent of this one once we have full support for stacking
        // contexts and transforms.
        builder.current_space_and_clip.spatial_id = builder.nearest_reference_frame;
    }

    fn build_clip_frame_if_necessary(
        &self,
        builder: &mut StackingContextBuilder,
        containing_block_info: &ContainingBlockInfo,
    ) {
        let position = self.style.get_box().position;
        // https://drafts.csswg.org/css2/#clipping
        // The clip property applies only to absolutely positioned elements
        if position == ComputedPosition::Absolute || position == ComputedPosition::Fixed {
            let clip = self.style.get_effects().clip;
            if let ClipRectOrAuto::Rect(r) = clip {
                let border_rect = self
                    .border_rect()
                    .to_physical(self.style.writing_mode, &containing_block_info.rect);
                let clip_rect = r
                    .for_border_rect(border_rect)
                    .translate(containing_block_info.rect.origin.to_vector())
                    .to_webrender();

                let parent = builder.current_space_and_clip;
                builder.current_space_and_clip.clip_id =
                    builder.wr.define_clip_rect(&parent, clip_rect);
            }
        }
    }

    fn build_scroll_frame_if_necessary<'a>(
        &self,
        builder: &mut StackingContextBuilder,
        containing_block_info: &ContainingBlockInfo,
    ) {
        let overflow_x = self.style.get_box().overflow_x;
        let overflow_y = self.style.get_box().overflow_y;

        let original_scroll_and_clip_info = builder.current_space_and_clip;
        if overflow_x != ComputedOverflow::Visible || overflow_y != ComputedOverflow::Visible {
            let external_id = wr::ExternalScrollId(
                self.tag.to_display_list_fragment_id(),
                builder.wr.pipeline_id,
            );

            let sensitivity = if ComputedOverflow::Hidden == overflow_x &&
                ComputedOverflow::Hidden == overflow_y
            {
                wr::ScrollSensitivity::Script
            } else {
                wr::ScrollSensitivity::ScriptAndInputEvents
            };

            let padding_rect = self
                .padding_rect()
                .to_physical(self.style.writing_mode, &containing_block_info.rect)
                .translate(containing_block_info.rect.origin.to_vector())
                .to_webrender();
            builder.current_space_and_clip = builder.wr.define_scroll_frame(
                &original_scroll_and_clip_info,
                Some(external_id),
                self.scrollable_overflow(&containing_block_info.rect)
                    .to_webrender(),
                padding_rect,
                sensitivity,
                LayoutVector2D::zero(),
            );
        }
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
            (Some(transform), None) => (transform, wr::ReferenceFrameKind::Transform),
            (Some(transform), Some(perspective)) => (
                transform.pre_transform(&perspective),
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

        let pre_transform = LayoutTransform::create_translation(
            transform_origin_x,
            transform_origin_y,
            transform_origin_z,
        );
        let post_transform = LayoutTransform::create_translation(
            -transform_origin_x,
            -transform_origin_y,
            -transform_origin_z,
        );

        Some(
            pre_transform
                .pre_transform(&transform)
                .pre_transform(&post_transform),
        )
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

                let pre_transform = LayoutTransform::create_translation(
                    perspective_origin.x,
                    perspective_origin.y,
                    0.0,
                );
                let post_transform = LayoutTransform::create_translation(
                    -perspective_origin.x,
                    -perspective_origin.y,
                    0.0,
                );

                let perspective_matrix = LayoutTransform::from_untyped(
                    &transform::create_perspective_matrix(length.px()),
                );

                Some(
                    pre_transform
                        .pre_transform(&perspective_matrix)
                        .pre_transform(&post_transform),
                )
            },
            Perspective::None => None,
        }
    }
}

impl AnonymousFragment {
    fn build_stacking_context_tree(
        &self,
        builder: &mut StackingContextBuilder,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
    ) {
        let mut new_containing_block_info = containing_block_info.clone();
        new_containing_block_info.rect = self
            .rect
            .to_physical(self.mode, &containing_block_info.rect)
            .translate(containing_block_info.rect.origin.to_vector());
        for child in &self.children {
            child.borrow().build_stacking_context_tree(
                child,
                builder,
                &new_containing_block_info,
                stacking_context,
                StackingContextBuildMode::SkipHoisted,
            );
        }
    }
}

impl HoistedFloatFragment {
    fn build_stacking_context_tree(
        &self,
        builder: &mut StackingContextBuilder,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
    ) {
        self.fragment.borrow().build_stacking_context_tree(
            &self.fragment,
            builder,
            containing_block_info,
            stacking_context,
            StackingContextBuildMode::SkipHoisted,
        );
    }
}

impl AbsoluteOrFixedPositionedFragment {
    fn build_stacking_context_tree(
        &self,
        builder: &mut StackingContextBuilder,
        containing_block_info: &ContainingBlockInfo,
        stacking_context: &mut StackingContext,
    ) {
        let hoisted_fragment = self.hoisted_fragment.borrow();
        let fragment_ref = match hoisted_fragment.fragment.as_ref() {
            Some(fragment_ref) => fragment_ref,
            None => unreachable!("Found hoisted box with missing fragment."),
        };

        let containing_block = match self.position {
            ComputedPosition::Fixed => &containing_block_info.containing_block_for_all_descendants,
            ComputedPosition::Absolute => containing_block_info
                .nearest_containing_block
                .as_ref()
                .unwrap_or(&containing_block_info.containing_block_for_all_descendants),
            ComputedPosition::Static | ComputedPosition::Relative => unreachable!(
                "Found an AbsoluteOrFixedPositionedFragment for a \
                              non-absolutely or fixed position fragment."
            ),
        };

        builder.clipping_and_scrolling_scope(|builder| {
            let mut new_containing_block_info = containing_block_info.clone();
            new_containing_block_info.rect = containing_block.rect;
            builder.current_space_and_clip = containing_block.space_and_clip;

            fragment_ref.borrow().build_stacking_context_tree(
                fragment_ref,
                builder,
                &new_containing_block_info,
                stacking_context,
                StackingContextBuildMode::IncludeHoisted,
            );
        });
    }
}
