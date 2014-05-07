/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::block::BlockFlow;
use layout::construct::FlowConstructor;
use layout::context::LayoutContext;
use layout::flow::{TableCaptionFlowClass, FlowClass, Flow};
use layout::wrapper::ThreadSafeLayoutNode;

use std::fmt;

/// A table formatting context.
pub struct TableCaptionFlow {
    pub block_flow: BlockFlow,
}

impl TableCaptionFlow {
    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableCaptionFlow {
        TableCaptionFlow {
            block_flow: BlockFlow::from_node(constructor, node)
        }
    }

    pub fn build_display_list_table_caption(&mut self, layout_context: &LayoutContext) {
        debug!("build_display_list_table_caption: same process as block flow");
        self.block_flow.build_display_list_block(layout_context)
    }
}

impl Flow for TableCaptionFlow {
    fn class(&self) -> FlowClass {
        TableCaptionFlowClass
    }

    fn as_table_caption<'a>(&'a mut self) -> &'a mut TableCaptionFlow {
        self
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
    }

    fn bubble_widths(&mut self, ctx: &mut LayoutContext) {
        self.block_flow.bubble_widths(ctx);
    }

    fn assign_widths(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow", "table_caption");
        self.block_flow.assign_widths(ctx);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height: assigning height for table_caption");
        self.block_flow.assign_height(ctx);
    }

    fn compute_absolute_position(&mut self) {
        self.block_flow.compute_absolute_position()
    }
}

impl fmt::Show for TableCaptionFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f.buf, "TableCaptionFlow: {}", self.block_flow)
    }
}
