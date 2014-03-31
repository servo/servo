/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::block::{BlockFlow, WidthAndMarginsComputer};
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{TableCellFlowClass, FlowClass, Flow};
use layout::table::InternalTable;
use layout::wrapper::ThreadSafeLayoutNode;

use std::cell::RefCell;
use geom::{Point2D, Rect, Size2D};
use gfx::display_list::{DisplayListCollection};
use servo_util::geometry::Au;

/// A table formatting context.
pub struct TableCellFlow {
    /// Data common to all flows.
    block_flow: BlockFlow,
}

impl TableCellFlow {
    pub fn from_node_and_box(node: &ThreadSafeLayoutNode,
                             box_: Box)
                             -> TableCellFlow {
        TableCellFlow {
            block_flow: BlockFlow::from_node_and_box(node, box_)
        }
    }

    pub fn teardown(&mut self) {
        self.block_flow.teardown()
    }

    pub fn box_<'a>(&'a mut self) -> &'a Option<Box>{
        &self.block_flow.box_
    }

    /// Assign height for table-cell flow.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_height_table_cell_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {
        let (_, mut top_offset, bottom_offset, left_offset) = self.block_flow
                                                                  .initialize_offsets(true);

        self.block_flow.handle_children_floats_if_necessary(ctx, inorder,
                                                            left_offset, top_offset);
        let mut cur_y = top_offset;
        // Since table cell does not have `margin`, the first child's top margin and
        // the last child's bottom margin do not collapse.
        self.block_flow.compute_margin_collapse(&mut cur_y,
                                                &mut top_offset,
                                                &mut Au(0),
                                                &mut Au(0),
                                                false,
                                                false);

        // CSS 2.1 § 17.5.3. Table cell box height is the minimum height required by the content.
        let height = cur_y - top_offset;

        // TODO(june0cho): vertical-align of table-cell should be calculated.
        let mut noncontent_height = Au::new(0);
        for box_ in self.block_flow.box_.iter() {
            let mut position = box_.border_box.get();

            // noncontent_height = border_top/bottom + padding_top/bottom of box
            noncontent_height = box_.noncontent_height();

            position.origin.y = Au(0);
            position.size.height = height + noncontent_height;

            box_.border_box.set(position);
        }

        self.block_flow.base.position.size.height = height + noncontent_height;

        self.block_flow.set_floats_out_if_inorder(inorder, height, cur_y, top_offset,
                                                  bottom_offset, left_offset);
        self.block_flow.assign_height_absolute_flows(ctx);
    }

    pub fn build_display_list_table_cell<E:ExtraDisplayListData>(
                                           &mut self,
                                           builder: &DisplayListBuilder,
                                           container_block_size: &Size2D<Au>,
                                           absolute_cb_abs_position: Point2D<Au>,
                                           dirty: &Rect<Au>,
                                           index: uint,
                                           lists: &RefCell<DisplayListCollection<E>>)
                                           -> uint {
        debug!("build_display_list_table_cell: same process as block flow");
        self.block_flow.build_display_list_block(builder, container_block_size,
                                                 absolute_cb_abs_position,
                                                 dirty, index, lists)
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
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent table row.
    fn assign_widths(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow", "table_cell");

        // The position was set to the column width by the parent flow, table row flow.
        let containing_block_width = self.block_flow.base.position.size.width;
        let mut left_content_edge = Au::new(0);
        let mut content_width = containing_block_width;

        let width_computer = InternalTable;
        width_computer.compute_used_width(&mut self.block_flow, ctx, containing_block_width);

        for box_ in self.block_flow.box_.iter() {
            left_content_edge = box_.border_box.get().origin.x + box_.padding.get().left + box_.border.get().left;
            let padding_and_borders = box_.padding.get().left + box_.padding.get().right +
                                      box_.border.get().left + box_.border.get().right;
            content_width = box_.border_box.get().size.width - padding_and_borders;
        }

        self.block_flow.propagate_assigned_width_to_children(left_content_edge, content_width, None);
    }

    /// This is called on kid flows by a parent.
    ///
    /// Hence, we can assume that assign_height has already been called on the
    /// kid (because of the bottom-up traversal).
    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height_inorder: assigning height for table_cell");
        self.assign_height_table_cell_base(ctx, true);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height: assigning height for table_cell");
        self.assign_height_table_cell_base(ctx, false);
    }

    /// TableCellBox and their parents(TableRowBox) do not have margins.
    /// Therefore, margins to be collapsed do not exist.
    fn collapse_margins(&mut self, _: bool, _: &mut bool, _: &mut Au,
                        _: &mut Au, _: &mut Au, _: &mut Au) {
    }

    fn debug_str(&self) -> ~str {
        let txt = ~"TableCellFlow: ";
        txt.append(match self.block_flow.box_ {
            Some(ref rb) => rb.debug_str(),
            None => ~"",
        })
    }
}

