/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::block::BlockFlow;
use layout::block::{WidthAndMarginsComputer, WidthConstraintInput, WidthConstraintSolution};
use layout::construct::FlowConstructor;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::floats::{FloatKind};
use layout::flow::{TableFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow;
use layout::table_wrapper::{TableLayout, FixedLayout, AutoLayout};
use layout::wrapper::ThreadSafeLayoutNode;

use std::cell::RefCell;
use style::computed_values::table_layout;
use geom::{Point2D, Rect, Size2D};
use gfx::display_list::DisplayListCollection;
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
    pub fn from_node_and_box(node: &ThreadSafeLayoutNode,
                             box_: Box)
                             -> TableFlow {
        let mut block_flow = BlockFlow::from_node_and_box(node, box_);
        let table_layout = if block_flow.box_().style().Table.get().table_layout ==
                              table_layout::fixed {
            FixedLayout
        } else {
            AutoLayout
        };
        TableFlow {
            block_flow: block_flow,
            col_widths: ~[],
            table_layout: table_layout
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableFlow {
        let mut block_flow = BlockFlow::from_node(constructor, node);
        let table_layout = if block_flow.box_().style().Table.get().table_layout ==
                              table_layout::fixed {
            FixedLayout
        } else {
            AutoLayout
        };
        TableFlow {
            block_flow: block_flow,
            col_widths: ~[],
            table_layout: table_layout
        }
    }

    pub fn float_from_node(constructor: &mut FlowConstructor,
                           node: &ThreadSafeLayoutNode,
                           float_kind: FloatKind)
                           -> TableFlow {
        let mut block_flow = BlockFlow::float_from_node(constructor, node, float_kind);
        let table_layout = if block_flow.box_().style().Table.get().table_layout ==
                              table_layout::fixed {
            FixedLayout
        } else {
            AutoLayout
        };
        TableFlow {
            block_flow: block_flow,
            col_widths: ~[],
            table_layout: table_layout
        }
    }

    pub fn teardown(&mut self) {
        self.block_flow.teardown();
        self.col_widths = ~[];
    }

    /// Assign height for table flow.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_height_table_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {

        let (_, top_offset, bottom_offset, left_offset) = self.block_flow.initialize_offsets(true);

        self.block_flow.handle_children_floats_if_necessary(ctx, inorder,
                                                            left_offset, top_offset);

        let mut cur_y = top_offset;
        for kid in self.block_flow.base.child_iter() {
            let child_node = flow::mut_base(kid);
            child_node.position.origin.y = cur_y;
            cur_y = cur_y + child_node.position.size.height;
        }

        let height = cur_y - top_offset;

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

        self.block_flow.set_floats_out_if_inorder(inorder, height, cur_y,
                                                  top_offset, bottom_offset, left_offset);
    }

    pub fn build_display_list_table<E:ExtraDisplayListData>(
                                    &mut self,
                                    builder: &DisplayListBuilder,
                                    container_block_size: &Size2D<Au>,
                                    absolute_cb_abs_position: Point2D<Au>,
                                    dirty: &Rect<Au>,
                                    index: uint,
                                    lists: &RefCell<DisplayListCollection<E>>)
                                    -> uint {
        debug!("build_display_list_table: same process as block flow");
        self.block_flow.build_display_list_block(builder, container_block_size,
                                                 absolute_cb_abs_position,
                                                 dirty, index, lists)
    }
}

impl Flow for TableFlow {
    fn class(&self) -> FlowClass {
        TableFlowClass
    }

    fn as_table<'a>(&'a mut self) -> &'a mut TableFlow {
        self
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
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
    fn assign_widths(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow", "table");

        // The position was set to the containing block by the flow's parent.
        let containing_block_width = self.block_flow.base.position.size.width;
        let mut left_content_edge = Au::new(0);
        let mut content_width = containing_block_width;

        let mut num_unspecified_widths = 0;
        let mut total_column_width = Au::new(0);
        for col_width in self.col_widths.iter() {
            if *col_width == Au::new(0) {
                num_unspecified_widths += 1;
            } else {
                total_column_width = total_column_width.add(col_width);
            }
        }

        let width_computer = InternalTable;
        width_computer.compute_used_width(&mut self.block_flow, ctx, containing_block_width);

        for box_ in self.block_flow.box_.iter() {
            left_content_edge = box_.padding.get().left + box_.border.get().left;
            let padding_and_borders = box_.padding.get().left + box_.padding.get().right +
                                      box_.border.get().left + box_.border.get().right;
            content_width = box_.border_box.get().size.width - padding_and_borders;
        }

        // In fixed table layout, we distribute extra space among the unspecified columns if there are
        // any, or among all the columns if all are specified.
        if (total_column_width < content_width) && (num_unspecified_widths == 0) {
            let ratio = content_width.to_f64().unwrap() / total_column_width.to_f64().unwrap();
            for col_width in self.col_widths.mut_iter() {
                *col_width = (*col_width).scale_by(ratio);
            }
        } else if num_unspecified_widths != 0 {
            let extra_column_width = (content_width - total_column_width) / Au::new(num_unspecified_widths);
            for col_width in self.col_widths.mut_iter() {
                if *col_width == Au(0) {
                    *col_width = extra_column_width;
                }
            }
        }

        self.block_flow.propagate_assigned_width_to_children(left_content_edge, content_width, Some(self.col_widths.clone()));
    }

    /// This is called on kid flows by a parent.
    ///
    /// Hence, we can assume that assign_height has already been called on the
    /// kid (because of the bottom-up traversal).
    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height_inorder: assigning height for table");
        self.assign_height_table_base(ctx, true);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height: assigning height for table");
        self.assign_height_table_base(ctx, false);
    }

    // CSS Section 8.3.1 - Collapsing Margins
    // Since `margin` is not used on table box, `collapsing` and `collapsible` are set to 0
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

/// Table, TableRowGroup, TableRow, TableCell types.
/// Their widths are calculated in the same way and do not have margins.
pub struct InternalTable;
impl WidthAndMarginsComputer for InternalTable {

    /// Compute the used value of width, taking care of min-width and max-width.
    ///
    /// CSS Section 10.4: Minimum and Maximum widths
    fn compute_used_width(&self,
                          block: &mut BlockFlow,
                          ctx: &mut LayoutContext,
                          parent_flow_width: Au) {
        let input = self.compute_width_constraint_inputs(block, parent_flow_width, ctx);

        let solution = self.solve_width_constraints(block, input);

        self.set_width_constraint_solutions(block, solution);
    }

    /// Solve the width and margins constraints for this block flow.
    fn solve_width_constraints(&self,
                               _: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution {
        WidthConstraintSolution::new(input.available_width, Au::new(0), Au::new(0))
    }
}
