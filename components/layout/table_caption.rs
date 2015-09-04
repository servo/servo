/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_code)]

use block::BlockFlow;
use context::LayoutContext;
use flow::{FlowClass, Flow, OpaqueFlow};
use fragment::{Fragment, FragmentBorderBoxIterator};

use euclid::{Point2D, Rect};
use std::fmt;
use std::sync::Arc;
use style::properties::ComputedValues;
use util::geometry::Au;
use util::logical_geometry::LogicalSize;

/// A table formatting context.
pub struct TableCaptionFlow {
    pub block_flow: BlockFlow,
}

impl TableCaptionFlow {
    pub fn from_fragment(fragment: Fragment) -> TableCaptionFlow {
        TableCaptionFlow {
            block_flow: BlockFlow::from_fragment(fragment, None)
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

    fn assign_inline_sizes(&mut self, ctx: &LayoutContext) {
        debug!("assign_inline_sizes({}): assigning inline_size for flow", "table_caption");
        self.block_flow.assign_inline_sizes(ctx);
    }

    fn assign_block_size<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
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

    fn build_display_list(&mut self, layout_context: &LayoutContext) {
        debug!("build_display_list_table_caption: same process as block flow");
        self.block_flow.build_display_list(layout_context)
    }

    fn repair_style(&mut self, new_style: &Arc<ComputedValues>) {
        self.block_flow.repair_style(new_style)
    }

    fn compute_overflow(&self) -> Rect<Au> {
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
}

impl fmt::Debug for TableCaptionFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableCaptionFlow: {:?}", self.block_flow)
    }
}
