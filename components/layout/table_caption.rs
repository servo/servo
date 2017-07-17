/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_code)]

use app_units::Au;
use block::BlockFlow;
use context::LayoutContext;
use display_list_builder::DisplayListBuildState;
use euclid::Point2D;
use flow::{Flow, FlowClass, OpaqueFlow};
use fragment::{Fragment, FragmentBorderBoxIterator, Overflow};
use gfx_traits::print_tree::PrintTree;
use std::fmt;
use style::logical_geometry::LogicalSize;
use style::properties::ComputedValues;

/// A table formatting context.
pub struct TableCaptionFlow {
    pub block_flow: BlockFlow,
}

impl TableCaptionFlow {
    pub fn from_fragment(fragment: Fragment) -> TableCaptionFlow {
        TableCaptionFlow {
            block_flow: BlockFlow::from_fragment(fragment),
        }
    }
}

impl Flow for TableCaptionFlow {
    fn class(&self) -> FlowClass {
        FlowClass::TableCaption
    }

    fn as_mut_table_caption(&mut self) -> &mut TableCaptionFlow {
        self
    }

    fn as_mut_block(&mut self) -> &mut BlockFlow {
        &mut self.block_flow
    }

    fn as_block(&self) -> &BlockFlow {
        &self.block_flow
    }

    fn bubble_inline_sizes(&mut self) {
        self.block_flow.bubble_inline_sizes();
    }

    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        debug!("assign_inline_sizes({}): assigning inline_size for flow", "table_caption");
        self.block_flow.assign_inline_sizes(layout_context);
    }

    fn assign_block_size(&mut self, layout_context: &LayoutContext) {
        debug!("assign_block_size: assigning block_size for table_caption");
        self.block_flow.assign_block_size(layout_context);
    }

    fn compute_absolute_position(&mut self, layout_context: &LayoutContext) {
        self.block_flow.compute_absolute_position(layout_context)
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        self.block_flow.update_late_computed_inline_position_if_necessary(inline_position)
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        self.block_flow.update_late_computed_block_position_if_necessary(block_position)
    }

    fn build_display_list(&mut self, state: &mut DisplayListBuildState) {
        debug!("build_display_list_table_caption: same process as block flow");
        self.block_flow.build_display_list(state);
    }

    fn collect_stacking_contexts(&mut self, state: &mut DisplayListBuildState) {
        self.block_flow.collect_stacking_contexts(state);
    }

    fn repair_style(&mut self, new_style: &::StyleArc<ComputedValues>) {
        self.block_flow.repair_style(new_style)
    }

    fn compute_overflow(&self) -> Overflow {
        self.block_flow.compute_overflow()
    }

    fn generated_containing_block_size(&self, flow: OpaqueFlow) -> LogicalSize<Au> {
        self.block_flow.generated_containing_block_size(flow)
    }

    fn iterate_through_fragment_border_boxes(&self,
                                             iterator: &mut FragmentBorderBoxIterator,
                                             level: i32,
                                             stacking_context_position: &Point2D<Au>) {
        self.block_flow.iterate_through_fragment_border_boxes(iterator, level, stacking_context_position)
    }

    fn mutate_fragments(&mut self, mutator: &mut FnMut(&mut Fragment)) {
        self.block_flow.mutate_fragments(mutator)
    }

    fn print_extra_flow_children(&self, print_tree: &mut PrintTree) {
        self.block_flow.print_extra_flow_children(print_tree);
    }
}

impl fmt::Debug for TableCaptionFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableCaptionFlow: {:?}", self.block_flow)
    }
}
