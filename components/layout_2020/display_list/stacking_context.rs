/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::display_list::DisplayListBuilder;
use crate::fragments::{AnonymousFragment, BoxFragment, Fragment};
use crate::geom::{PhysicalRect, ToWebRender};
use gfx_traits::{combine_id_with_fragment_type, FragmentType};
use std::default::Default;
use style::computed_values::overflow_x::T as ComputedOverflow;
use style::computed_values::position::T as ComputedPosition;
use style::values::computed::Length;
use webrender_api::units::LayoutVector2D;
use webrender_api::{ExternalScrollId, ScrollSensitivity, SpaceAndClipInfo, SpatialId};

pub(crate) struct StackingContextFragment<'a> {
    space_and_clip: SpaceAndClipInfo,
    containing_block: PhysicalRect<Length>,
    fragment: &'a Fragment,
}

#[derive(Default)]
pub(crate) struct StackingContext<'a> {
    fragments: Vec<StackingContextFragment<'a>>,
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
    fn build_stacking_context_tree<'a>(
        &'a self,
        fragment: &'a Fragment,
        builder: &mut DisplayListBuilder,
        containing_block: &PhysicalRect<Length>,
        stacking_context: &mut StackingContext<'a>,
    ) {
        builder.clipping_and_scrolling_scope(|builder| {
            self.adjust_spatial_id_for_positioning(builder);

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
        });
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

impl<'a> StackingContext<'a> {
    pub(crate) fn build_display_list(&'a self, builder: &'a mut DisplayListBuilder) {
        for child in &self.fragments {
            builder.current_space_and_clip = child.space_and_clip;
            child
                .fragment
                .build_display_list(builder, &child.containing_block);
        }
    }
}
