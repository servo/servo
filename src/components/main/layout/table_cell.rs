/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::block::{BlockFlow, MarginsMayNotCollapse, WidthAndMarginsComputer};
use layout::context::LayoutContext;
use layout::flow::{TableCellFlowClass, FlowClass, Flow};
use layout::model::{MaybeAuto};
use layout::table::InternalTable;
use layout::wrapper::ThreadSafeLayoutNode;

use servo_util::geometry::Au;

/// A table formatting context.
pub struct TableCellFlow {
    /// Data common to all flows.
    pub block_flow: BlockFlow,
}

impl TableCellFlow {
    pub fn from_node_and_box(node: &ThreadSafeLayoutNode, box_: Box) -> TableCellFlow {
        TableCellFlow {
            block_flow: BlockFlow::from_node_and_box(node, box_)
        }
    }

    pub fn teardown(&mut self) {
        self.block_flow.teardown()
    }

    pub fn box_<'a>(&'a mut self) -> &'a Box {
        &self.block_flow.box_
    }

    pub fn mut_box<'a>(&'a mut self) -> &'a mut Box {
        &mut self.block_flow.box_
    }

    /// Assign height for table-cell flow.
    ///
    /// TODO(#2015, pcwalton): This doesn't handle floats right.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_height_table_cell_base(&mut self, layout_context: &mut LayoutContext) {
        self.block_flow.assign_height_block_base(layout_context, MarginsMayNotCollapse)
    }

    pub fn build_display_list_table_cell(&mut self, layout_context: &LayoutContext) {
        debug!("build_display_list_table: same process as block flow");
        self.block_flow.build_display_list_block(layout_context)
    }
}

impl Flow for TableCellFlow {
    fn class(&self) -> FlowClass {
        TableCellFlowClass
    }

    fn as_table_cell<'a>(&'a mut self) -> &'a mut TableCellFlow {
        self
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
    }

    /// Minimum/preferred widths set by this function are used in automatic table layout calculation.
    fn bubble_widths(&mut self, ctx: &mut LayoutContext) {
        self.block_flow.bubble_widths(ctx);
        let specified_width = MaybeAuto::from_style(self.block_flow.box_.style().Box.get().width,
                                                    Au::new(0)).specified_or_zero();
        if self.block_flow.base.intrinsic_widths.minimum_width < specified_width {
            self.block_flow.base.intrinsic_widths.minimum_width = specified_width;
        }
        if self.block_flow.base.intrinsic_widths.preferred_width <
            self.block_flow.base.intrinsic_widths.minimum_width {
            self.block_flow.base.intrinsic_widths.preferred_width =
                self.block_flow.base.intrinsic_widths.minimum_width;
        }
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent table row.
    fn assign_widths(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow", "table_cell");

        // The position was set to the column width by the parent flow, table row flow.
        let containing_block_width = self.block_flow.base.position.size.width;

        let width_computer = InternalTable;
        width_computer.compute_used_width(&mut self.block_flow, ctx, containing_block_width);

        let left_content_edge = self.block_flow.box_.border_box.origin.x +
            self.block_flow.box_.border_padding.left;
        let padding_and_borders = self.block_flow.box_.border_padding.horizontal();
        let content_width = self.block_flow.box_.border_box.size.width - padding_and_borders;

        self.block_flow.propagate_assigned_width_to_children(left_content_edge,
                                                             content_width,
                                                             None);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height: assigning height for table_cell");
        self.assign_height_table_cell_base(ctx);
    }

    fn compute_absolute_position(&mut self) {
        self.block_flow.compute_absolute_position()
    }

    fn debug_str(&self) -> ~str {
        let txt = ~"TableCellFlow: ";
        txt.append(self.block_flow.box_.debug_str())
    }
}

