/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::display_list::conversions::ToWebRender;
use crate::display_list::DisplayListBuilder;
use crate::fragments::{AnonymousFragment, BoxFragment, Fragment};
use crate::geom::PhysicalRect;
use euclid::default::Rect;
use gfx_traits::{combine_id_with_fragment_type, FragmentType};
use std::cmp::Ordering;
use std::mem;
use style::computed_values::float::T as ComputedFloat;
use style::computed_values::mix_blend_mode::T as ComputedMixBlendMode;
use style::computed_values::overflow_x::T as ComputedOverflow;
use style::computed_values::position::T as ComputedPosition;
use style::computed_values::transform_style::T as ComputedTransformStyle;
use style::values::computed::Length;
use style::values::generics::box_::Perspective;
use style::values::generics::transform;
use style::values::specified::box_::DisplayOutside;
use webrender_api as wr;
use webrender_api::units::{LayoutPoint, LayoutTransform, LayoutVector2D};

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum StackingContextSection {
    BackgroundsAndBorders,
    BlockBackgroundsAndBorders,
    Content,
}

pub(crate) struct StackingContextFragment<'a> {
    space_and_clip: wr::SpaceAndClipInfo,
    section: StackingContextSection,
    containing_block: PhysicalRect<Length>,
    fragment: &'a Fragment,
}

impl<'a> StackingContextFragment<'a> {
    fn build_display_list(&self, builder: &mut DisplayListBuilder) {
        builder.current_space_and_clip = self.space_and_clip;
        self.fragment
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

pub(crate) struct StackingContext<'a> {
    /// The fragment that established this stacking context.
    initializing_fragment: Option<&'a BoxFragment>,

    /// The type of this StackingContext. Used for collecting and sorting.
    context_type: StackingContextType,

    /// Fragments that make up the content of this stacking context.
    fragments: Vec<StackingContextFragment<'a>>,

    /// All non-float stacking context and pseudo stacking context children
    /// of this stacking context.
    stacking_contexts: Vec<StackingContext<'a>>,

    /// All float pseudo stacking context children of this stacking context.
    float_stacking_contexts: Vec<StackingContext<'a>>,
}

impl<'a> StackingContext<'a> {
    pub(crate) fn new(
        initializing_fragment: &'a BoxFragment,
        context_type: StackingContextType,
    ) -> Self {
        Self {
            initializing_fragment: Some(initializing_fragment),
            context_type,
            fragments: vec![],
            stacking_contexts: vec![],
            float_stacking_contexts: vec![],
        }
    }

    pub(crate) fn create_root() -> Self {
        Self {
            initializing_fragment: None,
            context_type: StackingContextType::Real,
            fragments: vec![],
            stacking_contexts: vec![],
            float_stacking_contexts: vec![],
        }
    }

    fn z_index(&self) -> i32 {
        match self.initializing_fragment {
            Some(fragment) => fragment.effective_z_index(),
            None => 0,
        }
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
        builder: &'a mut DisplayListBuilder,
    ) -> bool {
        let fragment = match self.initializing_fragment {
            Some(fragment) => fragment,
            None => return false,
        };

        // WebRender only uses the stacking context to apply certain effects. If we don't
        // actually need to create a stacking context, just avoid creating one.
        let effects = fragment.style.get_effects();
        if effects.filter.0.is_empty() &&
            effects.opacity == 1.0 &&
            effects.mix_blend_mode == ComputedMixBlendMode::Normal
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

        builder.wr.push_stacking_context(
            LayoutPoint::zero(),                       // origin
            builder.current_space_and_clip.spatial_id, // spatial_id
            wr::PrimitiveFlags::default(),
            None, // clip_id
            wr::TransformStyle::Flat,
            effects.mix_blend_mode.to_webrender(),
            &filters,
            &vec![], // filter_datas
            &vec![], // filter_primitives
            wr::RasterSpace::Screen,
            false, // cache_tiles,
            false, // false
        );

        true
    }

    pub(crate) fn build_display_list(&'a self, builder: &'a mut DisplayListBuilder) {
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

impl Fragment {
    pub(crate) fn build_stacking_context_tree<'a>(
        &'a self,
        builder: &mut DisplayListBuilder,
        containing_block: &PhysicalRect<Length>,
        stacking_context: &mut StackingContext<'a>,
    ) {
        match self {
            Fragment::Box(fragment) => fragment.build_stacking_context_tree(
                self,
                builder,
                containing_block,
                stacking_context,
            ),
            Fragment::Anonymous(fragment) => {
                fragment.build_stacking_context_tree(builder, containing_block, stacking_context)
            },
            Fragment::Text(_) | Fragment::Image(_) => {
                stacking_context.fragments.push(StackingContextFragment {
                    section: StackingContextSection::Content,
                    space_and_clip: builder.current_space_and_clip,
                    containing_block: *containing_block,
                    fragment: self,
                });
            },
        }
    }
}

impl BoxFragment {
    fn get_stacking_context_type(&self) -> Option<StackingContextType> {
        if self.establishes_stacking_context() {
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

    /// Returns true if this fragment establishes a new stacking context and false otherwise.
    fn establishes_stacking_context(&self) -> bool {
        let effects = self.style.get_effects();
        if effects.opacity != 1.0 {
            return true;
        }

        if effects.mix_blend_mode != ComputedMixBlendMode::Normal {
            return true;
        }

        if self.has_transform_or_perspective() {
            return true;
        }

        if !self.style.get_effects().filter.0.is_empty() {
            return true;
        }

        if self.style.get_box().transform_style == ComputedTransformStyle::Preserve3d ||
            self.style.overrides_transform_style()
        {
            return true;
        }

        // Fixed position and sticky position always create stacking contexts.
        // TODO(mrobinson): We need to handle sticky positioning here when we support it.
        if self.style.get_box().position == ComputedPosition::Fixed {
            return true;
        }

        // Statically positioned fragments don't establish stacking contexts if the previous
        // conditions are not fulfilled. Furthermore, z-index doesn't apply to statically
        // positioned fragments.
        if self.style.get_box().position == ComputedPosition::Static {
            return false;
        }

        // For absolutely and relatively positioned fragments we only establish a stacking
        // context if there is a z-index set.
        // See https://www.w3.org/TR/CSS2/visuren.html#z-index
        !self.style.get_position().z_index.is_auto()
    }

    // Get the effective z-index of this fragment. Z-indices only apply to positioned element
    // per CSS 2 9.9.1 (http://www.w3.org/TR/CSS2/visuren.html#z-index), so this value may differ
    // from the value specified in the style.
    fn effective_z_index(&self) -> i32 {
        match self.style.get_box().position {
            ComputedPosition::Static => {},
            _ => return self.style.get_position().z_index.integer_or(0),
        }

        0
    }

    /// Returns true if this fragment has a transform, or perspective property set.
    fn has_transform_or_perspective(&self) -> bool {
        !self.style.get_box().transform.0.is_empty() ||
            self.style.get_box().perspective != Perspective::None
    }

    fn build_stacking_context_tree<'a>(
        &'a self,
        fragment: &'a Fragment,
        builder: &mut DisplayListBuilder,
        containing_block: &PhysicalRect<Length>,
        stacking_context: &mut StackingContext<'a>,
    ) {
        builder.clipping_and_scrolling_scope(|builder| {
            self.adjust_spatial_id_for_positioning(builder);

            let context_type = match self.get_stacking_context_type() {
                Some(context_type) => context_type,
                None => {
                    self.build_stacking_context_tree_for_children(
                        fragment,
                        builder,
                        *containing_block,
                        stacking_context,
                    );
                    return;
                },
            };

            let mut child_stacking_context = StackingContext::new(self, context_type);
            self.build_stacking_context_tree_for_children(
                fragment,
                builder,
                *containing_block,
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
            stacking_context
                .stacking_contexts
                .push(child_stacking_context);
            stacking_context
                .stacking_contexts
                .append(&mut stolen_children);
        });
    }

    fn build_stacking_context_tree_for_children<'a>(
        &'a self,
        fragment: &'a Fragment,
        builder: &mut DisplayListBuilder,
        mut containing_block: PhysicalRect<Length>,
        stacking_context: &mut StackingContext<'a>,
    ) {
        let relative_border_rect = self
            .border_rect()
            .to_physical(self.style.writing_mode, &containing_block);
        let border_rect = relative_border_rect.translate(containing_block.origin.to_vector());
        let established_reference_frame =
            self.build_reference_frame_if_necessary(builder, &border_rect);

        // WebRender reference frames establish a new coordinate system at their origin
        // (the border box of the fragment). We need to ensure that any coordinates we
        // give to WebRender in this reference frame are relative to the fragment border
        // box. We do this by adjusting the containing block origin.
        if established_reference_frame {
            containing_block.origin = (-relative_border_rect.origin.to_vector()).to_point();
        }

        stacking_context.fragments.push(StackingContextFragment {
            space_and_clip: builder.current_space_and_clip,
            section: self.get_stacking_context_section(),
            containing_block: containing_block,
            fragment,
        });

        // We want to build the scroll frame after the background and border, because
        // they shouldn't scroll with the rest of the box content.
        self.build_scroll_frame_if_necessary(builder, &containing_block);

        let new_containing_block = self
            .content_rect
            .to_physical(self.style.writing_mode, &containing_block)
            .translate(containing_block.origin.to_vector());
        for child in &self.children {
            child.build_stacking_context_tree(builder, &new_containing_block, stacking_context);
        }
    }

    fn adjust_spatial_id_for_positioning(&self, builder: &mut DisplayListBuilder) {
        if self.style.get_box().position != ComputedPosition::Fixed {
            return;
        }

        // TODO(mrobinson): Eventually this should use the spatial id of the reference
        // frame that is the parent of this one once we have full support for stacking
        // contexts and transforms.
        builder.current_space_and_clip.spatial_id =
            wr::SpatialId::root_reference_frame(builder.wr.pipeline_id);
    }

    fn build_scroll_frame_if_necessary(
        &self,
        builder: &mut DisplayListBuilder,
        containing_block: &PhysicalRect<Length>,
    ) {
        let overflow_x = self.style.get_box().overflow_x;
        let overflow_y = self.style.get_box().overflow_y;
        let original_scroll_and_clip_info = builder.current_space_and_clip;
        if overflow_x != ComputedOverflow::Visible || overflow_y != ComputedOverflow::Visible {
            // TODO(mrobinson): We should use the correct fragment type, once we generate
            // fragments from ::before and ::after generated content selectors.
            let id =
                combine_id_with_fragment_type(self.tag.id() as usize, FragmentType::FragmentBody)
                    as u64;
            let external_id = wr::ExternalScrollId(id, builder.wr.pipeline_id);

            let sensitivity = if ComputedOverflow::Hidden == overflow_x &&
                ComputedOverflow::Hidden == overflow_y
            {
                wr::ScrollSensitivity::Script
            } else {
                wr::ScrollSensitivity::ScriptAndInputEvents
            };

            let padding_rect = self
                .padding_rect()
                .to_physical(self.style.writing_mode, containing_block)
                .translate(containing_block.origin.to_vector())
                .to_webrender();
            builder.current_space_and_clip = builder.wr.define_scroll_frame(
                &original_scroll_and_clip_info,
                Some(external_id),
                self.scrollable_overflow().to_webrender(),
                padding_rect,
                vec![], // complex_clips
                None,   // image_mask
                sensitivity,
                LayoutVector2D::zero(),
            );
        }
    }

    /// Build a reference frame for this fragment if it is necessary. Returns `true` if
    /// a reference was built and `false` otherwise.
    fn build_reference_frame_if_necessary(
        &self,
        builder: &mut DisplayListBuilder,
        border_rect: &PhysicalRect<Length>,
    ) -> bool {
        if !self.has_transform_or_perspective() {
            return false;
        }
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

        builder.current_space_and_clip.spatial_id = builder.wr.push_reference_frame(
            border_rect.origin.to_webrender(),
            builder.current_space_and_clip.spatial_id,
            self.style.get_box().transform_style.to_webrender(),
            wr::PropertyBinding::Value(reference_frame_transform),
            reference_frame_kind,
        );
        true
    }

    /// Returns the 4D matrix representing this fragment's transform.
    pub fn calculate_transform_matrix(
        &self,
        border_rect: &Rect<Length>,
    ) -> Option<LayoutTransform> {
        let list = &self.style.get_box().transform;
        let transform =
            LayoutTransform::from_untyped(&list.to_transform_3d_matrix(Some(&border_rect)).ok()?.0);

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
    fn build_stacking_context_tree<'a>(
        &'a self,
        builder: &mut DisplayListBuilder,
        containing_block: &PhysicalRect<Length>,
        stacking_context: &mut StackingContext<'a>,
    ) {
        let new_containing_block = self
            .rect
            .to_physical(self.mode, containing_block)
            .translate(containing_block.origin.to_vector());
        for child in &self.children {
            child.build_stacking_context_tree(builder, &new_containing_block, stacking_context);
        }
    }
}
