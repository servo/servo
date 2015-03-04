/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Layout for elements with a CSS `display` property of `list-item`. These elements consist of a
//! block and an extra inline fragment for the marker.

#![deny(unsafe_blocks)]

use block::BlockFlow;
use context::LayoutContext;
use display_list_builder::ListItemFlowDisplayListBuilding;
use floats::FloatKind;
use flow::{Flow, FlowClass};
use fragment::{CoordinateSystem, Fragment, FragmentBorderBoxIterator, GeneratedContentInfo};
use generated_content;
use incremental::RESOLVE_GENERATED_CONTENT;
use wrapper::ThreadSafeLayoutNode;

use geom::{Point2D, Rect};
use gfx::display_list::DisplayList;
use servo_util::geometry::Au;
use servo_util::logical_geometry::LogicalRect;
use servo_util::opts;
use style::properties::ComputedValues;
use style::computed_values::list_style_type;
use std::sync::Arc;

/// A block with the CSS `display` property equal to `list-item`.
#[derive(Debug)]
pub struct ListItemFlow {
    /// Data common to all block flows.
    pub block_flow: BlockFlow,
    /// The marker, if outside. (Markers that are inside are instead just fragments on the interior
    /// `InlineFlow`.)
    pub marker: Option<Fragment>,
}

impl ListItemFlow {
    pub fn from_node_fragments_and_flotation(node: &ThreadSafeLayoutNode,
                                             main_fragment: Fragment,
                                             marker_fragment: Option<Fragment>,
                                             flotation: Option<FloatKind>)
                                             -> ListItemFlow {
        let mut this = ListItemFlow {
            block_flow: if let Some(flotation) = flotation {
                BlockFlow::float_from_node_and_fragment(node, main_fragment, flotation)
            } else {
                BlockFlow::from_node_and_fragment(node, main_fragment)
            },
            marker: marker_fragment,
        };

        if let Some(ref marker) = this.marker {
            match marker.style().get_list().list_style_type {
                list_style_type::T::disc |
                list_style_type::T::none |
                list_style_type::T::circle |
                list_style_type::T::square |
                list_style_type::T::disclosure_open |
                list_style_type::T::disclosure_closed => {}
                _ => this.block_flow.base.restyle_damage.insert(RESOLVE_GENERATED_CONTENT),
            }
        }

        this
    }
}

impl Flow for ListItemFlow {
    fn class(&self) -> FlowClass {
        FlowClass::ListItem
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
    }

    fn bubble_inline_sizes(&mut self) {
        // The marker contributes no intrinsic inline-size, so…
        self.block_flow.bubble_inline_sizes()
    }

    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        self.block_flow.assign_inline_sizes(layout_context);

        match self.marker {
            None => {}
            Some(ref mut marker) => {
                // Do this now. There's no need to do this in bubble-widths, since markers do not
                // contribute to the inline size of this flow.
                let intrinsic_inline_sizes = marker.compute_intrinsic_inline_sizes();

                marker.border_box.size.inline =
                    intrinsic_inline_sizes.content_intrinsic_sizes.preferred_inline_size;
                marker.border_box.start.i = self.block_flow.fragment.border_box.start.i -
                    marker.border_box.size.inline;
            }
        }
    }

    fn assign_block_size<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        self.block_flow.assign_block_size(layout_context);

        match self.marker {
            None => {}
            Some(ref mut marker) => {
                marker.border_box.start.b = Au(0);
                marker.border_box.size.block = marker.calculate_line_height(layout_context);
            }
        }
    }

    fn compute_absolute_position(&mut self) {
        self.block_flow.compute_absolute_position()
    }

    fn place_float_if_applicable<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        self.block_flow.place_float_if_applicable(layout_context)
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        self.block_flow.update_late_computed_inline_position_if_necessary(inline_position)
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        self.block_flow.update_late_computed_block_position_if_necessary(block_position)
    }

    fn build_display_list(&mut self, layout_context: &LayoutContext) {
        self.build_display_list_for_list_item(box DisplayList::new(), layout_context);
        if opts::get().validate_display_list_geometry {
            self.block_flow.base.validate_display_list_geometry();
        }
    }

    fn repair_style(&mut self, new_style: &Arc<ComputedValues>) {
        self.block_flow.repair_style(new_style)
    }

    fn compute_overflow(&self) -> Rect<Au> {
        self.block_flow.compute_overflow()
    }

    fn generated_containing_block_rect(&self) -> LogicalRect<Au> {
        self.block_flow.generated_containing_block_rect()
    }

    fn iterate_through_fragment_border_boxes(&self,
                                             iterator: &mut FragmentBorderBoxIterator,
                                             stacking_context_position: &Point2D<Au>) {
        self.block_flow.iterate_through_fragment_border_boxes(iterator, stacking_context_position);

        if let Some(ref marker) = self.marker {
            if iterator.should_process(marker) {
                iterator.process(
                    marker,
                    &marker.stacking_relative_border_box(&self.block_flow
                                                              .base
                                                              .stacking_relative_position,
                                                         &self.block_flow
                                                              .base
                                                              .absolute_position_info
                                                              .relative_containing_block_size,
                                                         CoordinateSystem::Parent)
                           .translate(stacking_context_position));
            }
        }
    }

    fn mutate_fragments(&mut self, mutator: &mut FnMut(&mut Fragment)) {
        self.block_flow.mutate_fragments(mutator);

        if let Some(ref mut marker) = self.marker {
            (*mutator)(marker)
        }
    }
}

/// The kind of content that `list-style-type` results in.
pub enum ListStyleTypeContent {
    None,
    StaticText(char),
    GeneratedContent(Box<GeneratedContentInfo>),
}

impl ListStyleTypeContent {
    /// Returns the content to be used for the given value of the `list-style-type` property.
    pub fn from_list_style_type(list_style_type: list_style_type::T) -> ListStyleTypeContent {
        // Just to keep things simple, use a nonbreaking space (Unicode 0xa0) to provide the marker
        // separation.
        match list_style_type {
            list_style_type::T::none => ListStyleTypeContent::None,
            list_style_type::T::disc | list_style_type::T::circle | list_style_type::T::square |
            list_style_type::T::disclosure_open | list_style_type::T::disclosure_closed => {
                let text = generated_content::static_representation(list_style_type);
                ListStyleTypeContent::StaticText(text)
            }
            _ => ListStyleTypeContent::GeneratedContent(box GeneratedContentInfo::ListItem),
        }
    }
}

