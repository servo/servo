/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::block::BlockFlow;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{BaseFlow, TableCellFlowClass, FlowClass, Flow};

use std::cell::RefCell;
use geom::{Point2D, Rect};
use gfx::display_list::DisplayList;
use servo_util::geometry::Au;

/// A table formatting context.
pub struct TableCellFlow {
    /// Data common to all flows.
    block_flow: BlockFlow,
}

impl TableCellFlow {
    pub fn new(base: BaseFlow) -> TableCellFlow {
        TableCellFlow {
            block_flow: BlockFlow::new(base),
        }
    }

    pub fn from_box(base: BaseFlow, box_: Box) -> TableCellFlow {
        TableCellFlow {
            block_flow: BlockFlow::from_box(base, box_, false),
        }
    }

    pub fn teardown(&mut self) {
        self.block_flow.teardown()
    }

    pub fn box_<'a>(&'a mut self) -> &'a Option<Box>{
        &self.block_flow.box_
    }

    // inline(always) because this is only ever called by in-order or non-in-order top-level
    // methods
    #[inline(always)]
    fn assign_height_table_cell_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {
        let (_, top_offset, bottom_offset, left_offset) = self.block_flow.initialize_offsets(true);

        let mut float_ctx = self.block_flow.handle_children_floats_if_inorder(ctx,
                                                                   Point2D(-left_offset, -top_offset),
                                                                   inorder);
        let mut cur_y = top_offset;
        let mut top_offset = top_offset;
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
            let mut position = box_.position.get();

            noncontent_height = box_.noncontent_height();

            position.origin.y = Au(0);
            position.size.height = height + noncontent_height;

            box_.position.set(position);
        }

        self.block_flow.base.position.size.height = height + noncontent_height;

        self.block_flow.set_floats_out(&mut float_ctx, height, cur_y, top_offset,
                                       bottom_offset, left_offset, inorder);
    }

    pub fn build_display_list_table_cell<E:ExtraDisplayListData>(
                                    &mut self,
                                    builder: &DisplayListBuilder,
                                    dirty: &Rect<Au>,
                                    list: &RefCell<DisplayList<E>>)
                                    -> bool {
        debug!("build_display_list_table_cell: same process as block flow");
        self.block_flow.build_display_list_block(builder, dirty, list)
    }
}

impl Flow for TableCellFlow {
    fn class(&self) -> FlowClass {
        TableCellFlowClass
    }

    fn as_table_cell<'a>(&'a mut self) -> &'a mut TableCellFlow {
        self
    }

    /// Minimum/preferred widths set by this function are used in automatic table layout calculation.
    fn bubble_widths(&mut self, ctx: &mut LayoutContext) {
        self.block_flow.bubble_widths(ctx);
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent table row.
    fn assign_widths(&mut self, _: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow {}", "table_cell", self.block_flow.base.id);

        // The position was set to the column width by the parent flow, table row flow.
        let mut remaining_width = self.block_flow.base.position.size.width;
        let mut x_offset = Au::new(0);

        for box_ in self.block_flow.box_.iter() {
            let style = box_.style();

            // The text alignment of a table_cell flow is the text alignment of its box's style.
            self.block_flow.base.flags_info.flags.set_text_align(style.Text.text_align);
            self.block_flow.compute_padding_and_margin_if_exists(box_, style, remaining_width, true, false);
            self.block_flow.set_box_x_and_width(box_, Au(0), remaining_width);

            x_offset = box_.offset();
            let padding_and_borders = box_.padding.get().left + box_.padding.get().right +
                                      box_.border.get().left + box_.border.get().right;
            remaining_width = remaining_width - padding_and_borders;
        }

        self.block_flow.propagate_assigned_width_to_children(x_offset, remaining_width, None);
    }

    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height_inorder: assigning height for table_cell {}", self.block_flow.base.id);
        self.assign_height_table_cell_base(ctx, true);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        //assign height for box
        for box_ in self.block_flow.box_.iter() {
            box_.assign_height();
        }

        debug!("assign_height: assigning height for table_cell {}", self.block_flow.base.id);
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
