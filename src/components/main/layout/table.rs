/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::block::BlockFlow;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{BaseFlow, TableFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow;
use layout::table_wrapper::{TableLayout, FixedLayout, AutoLayout};

use std::cell::RefCell;
use style::computed_values::table_layout;
use geom::{Point2D, Rect};
use gfx::display_list::DisplayList;
use servo_util::geometry::Au;

/// A table flow corresponded to the table's internal table box under a table wrapper flow.
/// The properties `position`, `float`, and `margin-*` are used on the table wrapper box,
/// not table box per CSS 2.1 § 10.5.
pub struct TableFlow {
    block_flow: BlockFlow,

    /// Column widths
    col_widths: ~[Au],

    /// Table-layout property
    table_layout: TableLayout,
}

impl TableFlow {
    pub fn new(base: BaseFlow) -> TableFlow {
        TableFlow {
            block_flow: BlockFlow::new(base),
            col_widths: ~[],
            table_layout: AutoLayout,
        }
    }

    pub fn from_box(base: BaseFlow, box_: Box) -> TableFlow {
        let table_layout = if (box_.style().Table.table_layout == table_layout::fixed) {
            FixedLayout
        } else {
            AutoLayout
        };
        TableFlow {
            block_flow: BlockFlow::from_box(base, box_, false),
            col_widths: ~[],
            table_layout: table_layout,
        }
    }

    pub fn teardown(&mut self) {
        self.block_flow.teardown();
        self.col_widths = ~[];
    }

    // inline(always) because this is only ever called by in-order or non-in-order top-level
    // methods
    #[inline(always)]
    fn assign_height_table_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {

        let (_, top_offset, bottom_offset, left_offset) = self.block_flow.initialize_offset(true);

        let mut float_ctx = self.block_flow.handle_children_floats_if_inorder(ctx,
                                                                              Point2D(-left_offset, -top_offset),
                                                                              inorder);

        let mut cur_y = top_offset;
        for kid in self.block_flow.base.child_iter() {
            let child_node = flow::mut_base(*kid);
            child_node.position.origin.y = cur_y;
            cur_y = cur_y + child_node.position.size.height;
        }

        let height = cur_y - top_offset;

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

    pub fn build_display_list_table<E:ExtraDisplayListData>(
                                    &mut self,
                                    builder: &DisplayListBuilder,
                                    dirty: &Rect<Au>,
                                    list: &RefCell<DisplayList<E>>)
                                    -> bool {
        debug!("build_display_list_table: same process as block flow");
        self.block_flow.build_display_list_block(builder, dirty, list)
    }
}

impl Flow for TableFlow {
    fn class(&self) -> FlowClass {
        TableFlowClass
    }

    fn as_table<'a>(&'a mut self) -> &'a mut TableFlow {
        self
    }

    /// This function finds the specified column widths from column group and the first row.
    /// Those are used in fixed table layout calculation.
    /* FIXME: automatic table layout calculation */
    fn bubble_widths(&mut self, ctx: &mut LayoutContext) {
        let mut did_first_row = false;

        /* find max width from child block contexts */
        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_proper_table_child());

            if kid.is_table_colgroup() {
                self.col_widths.push_all(kid.as_table_colgroup().widths);
            } else if kid.is_table_rowgroup() || kid.is_table_row() {
                // read column widths from table-row-group/table-row, and assign
                // width=0 for the columns not defined in column-group
                // FIXME: need to read widths from either table-header-group OR
                // first table-row
                let kid_col_widths = if kid.is_table_rowgroup() {
                    &kid.as_table_rowgroup().col_widths
                } else {
                    &kid.as_table_row().col_widths
                };
                match self.table_layout {
                    FixedLayout if !did_first_row => {
                        did_first_row = true;
                        let mut child_widths = kid_col_widths.iter();
                        for col_width in self.col_widths.mut_iter() {
                            match child_widths.next() {
                                Some(child_width) => {
                                    if *col_width == Au::new(0) {
                                        *col_width = *child_width;
                                    }
                                },
                                None => break
                            }
                        }
                    },
                    _ => {}
                }
                let num_child_cols = kid_col_widths.len();
                let num_cols = self.col_widths.len();
                debug!("colgroup has {} column(s) and child has {} column(s)", num_cols, num_child_cols);
                for i in range(num_cols, num_child_cols) {
                    self.col_widths.push( kid_col_widths[i] );
                }
            }
        }
        self.block_flow.bubble_widths(ctx);
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    fn assign_widths(&mut self, _: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow {}", "table", self.block_flow.base.id);

        // The position was set to the containing block by the flow's parent.
        let mut remaining_width = self.block_flow.base.position.size.width;
        let mut x_offset = Au::new(0);

        let mut num_unspecified_widths = 0;
        let mut total_columns_widths = Au::new(0);
        for col_width in self.col_widths.iter() {
            if *col_width == Au::new(0) {
                num_unspecified_widths += 1;
            } else {
                total_columns_widths = total_columns_widths.add(col_width);
            }
        }

        for box_ in self.block_flow.box_.iter() {
            let style = box_.style();

            // The text alignment of a table_wrapper flow is the text alignment of its box's style.
            self.block_flow.base.flags_info.flags.set_text_align(style.Text.text_align);
            self.block_flow.initial_box_setting(box_, style, remaining_width, true, false);
            self.block_flow.set_box_x_and_width(box_, Au(0), remaining_width);

            x_offset = box_.padding.get().left + box_.border.get().left;
            let padding_and_borders = box_.padding.get().left + box_.padding.get().right +
                                      box_.border.get().left + box_.border.get().right;
            remaining_width = remaining_width - padding_and_borders;
        }

        // In fixed table layout, we distribute extra space among the unspecified columns if there are
        // any, or among all the columns if all are specified.
        let extra_column_width = if (total_columns_widths < remaining_width) &&
                                    (num_unspecified_widths == 0) {
            let ratio = remaining_width.to_f64().unwrap() / total_columns_widths.to_f64().unwrap();
            for col_width in self.col_widths.mut_iter() {
                *col_width = (*col_width).scale_by(ratio);
            }
            Au(0)
        } else if num_unspecified_widths != 0 {
            (remaining_width - total_columns_widths) / Au::new(num_unspecified_widths)
        } else {
            Au(0)
        };

        self.block_flow.propagate_assigned_width_to_children(x_offset, remaining_width);
        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_proper_table_child());
            if kid.is_table_colgroup() {
                continue;
            }
            let final_col_widths = self.col_widths.map(|width| {
                if *width == Au(0) {
                    extra_column_width
                } else {
                    *width
                }
            });
            if kid.is_table_rowgroup() {
                kid.as_table_rowgroup().col_widths = final_col_widths;
            } else if kid.is_table_row() {
                kid.as_table_row().col_widths = final_col_widths;
            }
        }
    }

    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height_inorder: assigning height for table {}", self.block_flow.base.id);
        self.assign_height_table_base(ctx, true);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height: assigning height for table {}", self.block_flow.base.id);
        self.assign_height_table_base(ctx, false);
    }

    fn collapse_margins(&mut self,
                        _: bool,
                        _: &mut bool,
                        _: &mut Au,
                        _: &mut Au,
                        collapsing: &mut Au,
                        collapsible: &mut Au) {
        // `margin` is not used on table box.
        *collapsing = Au::new(0);
        *collapsible = Au::new(0);
    }

    fn debug_str(&self) -> ~str {
        let txt = ~"TableFlow: ";
        txt.append(match self.block_flow.box_ {
            Some(ref rb) => rb.debug_str(),
            None => ~"",
        })
    }
}

