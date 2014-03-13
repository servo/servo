/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::block::BlockFlow;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{BaseFlow, TableRowGroupFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow;

use std::cell::RefCell;
use geom::{Point2D, Rect};
use gfx::display_list::DisplayList;
use servo_util::geometry::Au;

/// A table formatting context.
pub struct TableRowGroupFlow {
    block_flow: BlockFlow,

    /// Column widths
    col_widths: ~[Au],
}

impl TableRowGroupFlow {
    pub fn new(base: BaseFlow) -> TableRowGroupFlow {
        TableRowGroupFlow {
            block_flow: BlockFlow::new(base),
            col_widths: ~[],
        }
    }

    pub fn from_box(base: BaseFlow, box_: Box) -> TableRowGroupFlow {
        TableRowGroupFlow {
            block_flow: BlockFlow::from_box(base, box_, false),
            col_widths: ~[],
        }
    }

    pub fn teardown(&mut self) {
        self.block_flow.teardown();
        self.col_widths = ~[];
    }

    pub fn box_<'a>(&'a mut self) -> &'a Option<Box>{
        &self.block_flow.box_
    }

    fn initialize_offsets(&mut self) -> (Au, Au, Au) {
        // TODO: If border-collapse: collapse, top_offset, bottom_offset, and left_offset
        // should be updated. Currently, they are set as Au(0).
        (Au(0), Au(0), Au(0))
    }

    // inline(always) because this is only ever called by in-order or non-in-order top-level
    // methods
    #[inline(always)]
    fn assign_height_table_rowgroup_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {
        let (top_offset, bottom_offset, left_offset) = self.initialize_offsets();

        let mut float_ctx = self.block_flow.handle_children_floats_if_inorder(ctx, Point2D(-left_offset, -top_offset), inorder);
        let mut cur_y = top_offset;

        for kid in self.block_flow.base.child_iter() {
            let child_node = flow::mut_base(*kid);
            child_node.position.origin.y = cur_y;
            cur_y = cur_y + child_node.position.size.height;
        }

        let height = cur_y - top_offset;

        for box_ in self.block_flow.box_.iter() {
            let mut position = box_.position.get();
            position.size.height = height;
            box_.position.set(position);
        }
        self.block_flow.base.position.size.height = height;

        self.block_flow.set_floats_out(&mut float_ctx, height, cur_y, top_offset,
                                       bottom_offset, left_offset, inorder);
    }

    pub fn build_display_list_table_rowgroup<E:ExtraDisplayListData>(
                                    &mut self,
                                    builder: &DisplayListBuilder,
                                    dirty: &Rect<Au>,
                                    list: &RefCell<DisplayList<E>>)
                                    -> bool {
        debug!("build_display_list_table_rowgroup: same process as block flow");
        self.block_flow.build_display_list_block(builder, dirty, list)
    }
}

impl Flow for TableRowGroupFlow {
    fn class(&self) -> FlowClass {
        TableRowGroupFlowClass
    }

    fn as_table_rowgroup<'a>(&'a mut self) -> &'a mut TableRowGroupFlow {
        self
    }

    /// Recursively (bottom-up) determines the context's preferred and minimum widths. When called
    /// on this context, all child contexts have had their min/pref widths set. This function must
    /// decide min/pref widths based on child context widths and dimensions of any boxes it is
    /// responsible for flowing.
    /// Min/pref widths set by this function are used in automatic table layout calculation.
    /// Also, this function finds the specified column widths from the first row.
    /// Those are used in fixed table layout calculation
    fn bubble_widths(&mut self, ctx: &mut LayoutContext) {
        /* find the specified column widths from the first table-row.
           update the number of column widths from other table-rows. */
        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_row());
            if self.col_widths.is_empty() {
                self.col_widths = kid.as_table_row().col_widths.clone();
            } else {
                let num_cols = self.col_widths.len();
                let num_child_cols = kid.as_table_row().col_widths.len();
                for _ in range(num_cols, num_child_cols) {
                    self.col_widths.push(Au::new(0));
                }
            }
        }

        // TODO: calculate min_width & pref_width for automatic table layout calculation
        self.block_flow.bubble_widths(ctx);
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    fn assign_widths(&mut self, _: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow {}", "table_rowgroup", self.block_flow.base.id);

        // The position was set to the containing block by the flow's parent.
        let remaining_width = self.block_flow.base.position.size.width;

        for box_ in self.block_flow.box_.iter() {
            let style = box_.style();

            // The text alignment of a table_rowgroup flow is the text alignment of its box's style.
            self.block_flow.base.flags_info.flags.set_text_align(style.Text.text_align);
            self.block_flow.compute_padding_and_margin_if_exists(box_, style, remaining_width, false, false);
            self.block_flow.set_box_x_and_width(box_, Au(0), remaining_width);
        }

        self.block_flow.propagate_assigned_width_to_children(Au(0), remaining_width, Some(self.col_widths.clone()));
    }

    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height_inorder: assigning height for table_rowgroup {}", self.block_flow.base.id);
        self.assign_height_table_rowgroup_base(ctx, true);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        //assign height for box
        for box_ in self.block_flow.box_.iter() {
            box_.assign_height();
        }

        debug!("assign_height: assigning height for table_rowgroup {}", self.block_flow.base.id);
        self.assign_height_table_rowgroup_base(ctx, false);
    }

    /// TableRowBox and their parents(TableBox) do not have margins.
    /// Therefore, margins to be collapsed do not exist.
    fn collapse_margins(&mut self, _: bool, _: &mut bool, _: &mut Au,
                        _: &mut Au, _: &mut Au, _: &mut Au) {
    }

    fn debug_str(&self) -> ~str {
        let txt = ~"TableRowGroupFlow: ";
        txt.append(match self.block_flow.box_ {
            Some(ref rb) => rb.debug_str(),
            None => ~"",
        })
    }
}

