/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Layout for elements with a CSS `display` property of `list-item`. These elements consist of a
//! block and an extra inline fragment for the marker.

#![deny(unsafe_blocks)]

use block::BlockFlow;
use construct::FlowConstructor;
use context::LayoutContext;
use display_list_builder::ListItemFlowDisplayListBuilding;
use floats::FloatKind;
use flow::{Flow, FlowClass};
use fragment::{Fragment, FragmentBorderBoxIterator};
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
    pub fn from_node_marker_and_flotation(constructor: &mut FlowConstructor,
                                          node: &ThreadSafeLayoutNode,
                                          marker_fragment: Option<Fragment>,
                                          flotation: Option<FloatKind>)
                                          -> ListItemFlow {
        ListItemFlow {
            block_flow: if let Some(flotation) = flotation {
                BlockFlow::float_from_node(constructor, node, flotation)
            } else {
                BlockFlow::from_node(constructor, node)
            },
            marker: marker_fragment,
        }
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
        self.block_flow.iterate_through_fragment_border_boxes(iterator, stacking_context_position)
    }
}

/// Returns the static text to be used for the given value of the `list-style-type` property.
///
/// TODO(pcwalton): Return either a string or a counter descriptor, once we support counters.
pub fn static_text_for_list_style_type(list_style_type: list_style_type::T)
                                       -> Option<&'static str> {
    // Just to keep things simple, use a nonbreaking space (Unicode 0xa0) to provide the marker
    // separation.
    match list_style_type {
        list_style_type::T::none => None,
        list_style_type::T::disc => Some("•\u{a0}"),
        list_style_type::T::circle => Some("◦\u{a0}"),
        list_style_type::T::square => Some("▪\u{a0}"),
        list_style_type::T::disclosure_open => Some("▾\u{a0}"),
        list_style_type::T::disclosure_closed => Some("‣\u{a0}"),
    }
}

