/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::display_list::DisplayListBuilder;
use crate::fragments::{AnonymousFragment, BoxFragment, Fragment};
use crate::geom::{PhysicalRect, ToWebRender};
use gfx_traits::{combine_id_with_fragment_type, FragmentType};
use std::cmp::Ordering;
use std::mem;
use style::computed_values::float::T as ComputedFloat;
use style::computed_values::mix_blend_mode::T as ComputedMixBlendMode;
use style::computed_values::overflow_x::T as ComputedOverflow;
use style::computed_values::position::T as ComputedPosition;
use style::computed_values::transform_style::T as ComputedTransformStyle;
use style::values::computed::Length;
use webrender_api::units::LayoutVector2D;
use webrender_api::{ExternalScrollId, ScrollSensitivity, SpaceAndClipInfo, SpatialId};

pub(crate) struct StackingContextFragment<'a> {
    space_and_clip: SpaceAndClipInfo,
    containing_block: PhysicalRect<Length>,
    fragment: &'a Fragment,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum StackingContextType {
    Real,
    PseudoPositioned,
    PseudoFloat,
}

pub(crate) struct StackingContext<'a> {
    /// The type of this StackingContext. Used for collecting and sorting.
    context_type: StackingContextType,

    /// The `z-index` for this stacking context.
    pub z_index: i32,

    /// Fragments that make up the content of this stacking context.
    fragments: Vec<StackingContextFragment<'a>>,

    /// All non-float stacking context and pseudo stacking context children
    /// of this stacking context.
    stacking_contexts: Vec<StackingContext<'a>>,

    /// All float pseudo stacking context children of this stacking context.
    float_stacking_contexts: Vec<StackingContext<'a>>,
}

impl<'a> StackingContext<'a> {
    pub(crate) fn new(context_type: StackingContextType, z_index: i32) -> Self {
        Self {
            context_type,
            z_index,
            fragments: vec![],
            stacking_contexts: vec![],
            float_stacking_contexts: vec![],
        }
    }

    pub(crate) fn sort_stacking_contexts(&mut self) {
        self.stacking_contexts.sort_by(|a, b| {
            if a.z_index != 0 || b.z_index != 0 {
                return a.z_index.cmp(&b.z_index);
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

    pub(crate) fn build_display_list(&'a self, builder: &'a mut DisplayListBuilder) {
        // Properly order display items that make up a stacking context. "Steps" here
        // refer to the steps in CSS 2.1 Appendix E.
        //
        // TODO(mrobinson): The fragment content of the stacking context needs to be
        // organized or sorted into the different sections according to the appropriate
        // paint order.

        // Step 3: Positioned descendants with negative z-indices.
        let mut child_stacking_contexts = self.stacking_contexts.iter().peekable();
        while child_stacking_contexts
            .peek()
            .map_or(false, |child| child.z_index < 0)
        {
            let child_context = child_stacking_contexts.next().unwrap();
            child_context.build_display_list(builder);
        }

        // Step 4: Block backgrounds and borders.
        for child in &self.fragments {
            builder.current_space_and_clip = child.space_and_clip;
            child
                .fragment
                .build_display_list(builder, &child.containing_block);
        }

        // Step 5: Floats.
        for child_context in &self.float_stacking_contexts {
            child_context.build_display_list(builder);
        }

        // Step 7, 8 & 9: Inlines that generate stacking contexts and positioned
        // descendants with nonnegative, numeric z-indices.
        for child_context in child_stacking_contexts {
            child_context.build_display_list(builder);
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

        if self.style.get_box().position != ComputedPosition::Static {
            return Some(StackingContextType::PseudoPositioned);
        }

        if self.style.get_box().float != ComputedFloat::None {
            return Some(StackingContextType::PseudoFloat);
        }

        None
    }

    /// Returns true if this fragment establishes a new stacking context and false otherwise.
    fn establishes_stacking_context(&self) -> bool {
        if self.style.get_effects().opacity != 1.0 {
            return true;
        }

        if self.style.get_effects().mix_blend_mode != ComputedMixBlendMode::Normal {
            return true;
        }

        if self.has_filter_transform_or_perspective() {
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

    /// Returns true if this fragment has a filter, transform, or perspective property set.
    fn has_filter_transform_or_perspective(&self) -> bool {
        // TODO(mrobinson): We need to handle perspective here.
        !self.style.get_box().transform.0.is_empty() ||
            !self.style.get_effects().filter.0.is_empty()
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
                        containing_block,
                        stacking_context,
                    );
                    return;
                },
            };

            let mut child_stacking_context =
                StackingContext::new(context_type, self.effective_z_index());
            self.build_stacking_context_tree_for_children(
                fragment,
                builder,
                containing_block,
                &mut child_stacking_context,
            );

            if context_type == StackingContextType::Real {
                child_stacking_context.sort_stacking_contexts();
                stacking_context
                    .stacking_contexts
                    .push(child_stacking_context);
            } else {
                let mut children =
                    mem::replace(&mut child_stacking_context.stacking_contexts, Vec::new());
                stacking_context
                    .stacking_contexts
                    .push(child_stacking_context);
                stacking_context.stacking_contexts.append(&mut children);
            }
        });
    }

    fn build_stacking_context_tree_for_children<'a>(
        &'a self,
        fragment: &'a Fragment,
        builder: &mut DisplayListBuilder,
        containing_block: &PhysicalRect<Length>,
        stacking_context: &mut StackingContext<'a>,
    ) {
        stacking_context.fragments.push(StackingContextFragment {
            space_and_clip: builder.current_space_and_clip,
            containing_block: *containing_block,
            fragment,
        });

        // We want to build the scroll frame after the background and border, because
        // they shouldn't scroll with the rest of the box content.
        self.build_scroll_frame_if_necessary(builder, containing_block);

        let new_containing_block = self
            .content_rect
            .to_physical(self.style.writing_mode, containing_block)
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
            SpatialId::root_reference_frame(builder.wr.pipeline_id);
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
            let external_id = ExternalScrollId(id, builder.wr.pipeline_id);

            let sensitivity = if ComputedOverflow::Hidden == overflow_x &&
                ComputedOverflow::Hidden == overflow_y
            {
                ScrollSensitivity::Script
            } else {
                ScrollSensitivity::ScriptAndInputEvents
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
