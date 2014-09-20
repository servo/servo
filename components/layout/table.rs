/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_block)]

use block::{BlockFlow, MarginsMayNotCollapse, ISizeAndMarginsComputer};
use block::{ISizeConstraintInput, ISizeConstraintSolution};
use construct::FlowConstructor;
use context::LayoutContext;
use floats::FloatKind;
use flow::{TableFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use fragment::Fragment;
use layout_debug;
use table_wrapper::{TableLayout, FixedLayout, AutoLayout};
use wrapper::ThreadSafeLayoutNode;

use servo_util::geometry::Au;
use servo_util::logical_geometry::LogicalRect;
use std::cmp::max;
use std::fmt;
use style::computed_values::table_layout;

/// A table flow corresponded to the table's internal table fragment under a table wrapper flow.
/// The properties `position`, `float`, and `margin-*` are used on the table wrapper fragment,
/// not table fragment per CSS 2.1 § 10.5.
#[deriving(Encodable)]
pub struct TableFlow {
    pub block_flow: BlockFlow,

    /// Column inline-sizes
    pub col_inline_sizes: Vec<Au>,

    /// Column min inline-sizes.
    pub col_min_inline_sizes: Vec<Au>,

    /// Column pref inline-sizes.
    pub col_pref_inline_sizes: Vec<Au>,

    /// Table-layout property
    pub table_layout: TableLayout,
}

impl TableFlow {
    pub fn from_node_and_fragment(node: &ThreadSafeLayoutNode,
                                  fragment: Fragment)
                                  -> TableFlow {
        let mut block_flow = BlockFlow::from_node_and_fragment(node, fragment);
        let table_layout = if block_flow.fragment().style().get_table().table_layout ==
                              table_layout::fixed {
            FixedLayout
        } else {
            AutoLayout
        };
        TableFlow {
            block_flow: block_flow,
            col_inline_sizes: vec!(),
            col_min_inline_sizes: vec!(),
            col_pref_inline_sizes: vec!(),
            table_layout: table_layout
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableFlow {
        let mut block_flow = BlockFlow::from_node(constructor, node);
        let table_layout = if block_flow.fragment().style().get_table().table_layout ==
                              table_layout::fixed {
            FixedLayout
        } else {
            AutoLayout
        };
        TableFlow {
            block_flow: block_flow,
            col_inline_sizes: vec!(),
            col_min_inline_sizes: vec!(),
            col_pref_inline_sizes: vec!(),
            table_layout: table_layout
        }
    }

    pub fn float_from_node(constructor: &mut FlowConstructor,
                           node: &ThreadSafeLayoutNode,
                           float_kind: FloatKind)
                           -> TableFlow {
        let mut block_flow = BlockFlow::float_from_node(constructor, node, float_kind);
        let table_layout = if block_flow.fragment().style().get_table().table_layout ==
                              table_layout::fixed {
            FixedLayout
        } else {
            AutoLayout
        };
        TableFlow {
            block_flow: block_flow,
            col_inline_sizes: vec!(),
            col_min_inline_sizes: vec!(),
            col_pref_inline_sizes: vec!(),
            table_layout: table_layout
        }
    }

    /// Update the corresponding value of self_inline-sizes if a value of kid_inline-sizes has larger value
    /// than one of self_inline-sizes.
    pub fn update_col_inline_sizes(self_inline_sizes: &mut Vec<Au>, kid_inline_sizes: &Vec<Au>) -> Au {
        let mut sum_inline_sizes = Au(0);
        let mut kid_inline_sizes_it = kid_inline_sizes.iter();
        for self_inline_size in self_inline_sizes.mut_iter() {
            match kid_inline_sizes_it.next() {
                Some(kid_inline_size) => {
                    if *self_inline_size < *kid_inline_size {
                        *self_inline_size = *kid_inline_size;
                    }
                },
                None => {}
            }
            sum_inline_sizes = sum_inline_sizes + *self_inline_size;
        }
        sum_inline_sizes
    }

    /// Assign block-size for table flow.
    ///
    /// TODO(#2014, pcwalton): This probably doesn't handle margin collapse right.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_block_size_table_base<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        self.block_flow.assign_block_size_block_base(layout_context, MarginsMayNotCollapse);
    }

    pub fn build_display_list_table(&mut self, layout_context: &LayoutContext) {
        debug!("build_display_list_table: same process as block flow");
        self.block_flow.build_display_list_block(layout_context);
    }
}

impl Flow for TableFlow {
    fn class(&self) -> FlowClass {
        TableFlowClass
    }

    fn as_table<'a>(&'a mut self) -> &'a mut TableFlow {
        self
    }

    fn as_immutable_table<'a>(&'a self) -> &'a TableFlow {
        self
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
    }

    fn col_inline_sizes<'a>(&'a mut self) -> &'a mut Vec<Au> {
        &mut self.col_inline_sizes
    }

    fn col_min_inline_sizes<'a>(&'a self) -> &'a Vec<Au> {
        &self.col_min_inline_sizes
    }

    fn col_pref_inline_sizes<'a>(&'a self) -> &'a Vec<Au> {
        &self.col_pref_inline_sizes
    }

    /// The specified column inline-sizes are set from column group and the first row for the fixed
    /// table layout calculation.
    /// The maximum min/pref inline-sizes of each column are set from the rows for the automatic
    /// table layout calculation.
    fn bubble_inline_sizes(&mut self, _: &LayoutContext) {
        let _scope = layout_debug_scope!("table::bubble_inline_sizes {:s}",
                                            self.block_flow.base.debug_id());

        let mut min_inline_size = Au(0);
        let mut pref_inline_size = Au(0);
        let mut did_first_row = false;

        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_proper_table_child());

            if kid.is_table_colgroup() {
                self.col_inline_sizes.push_all(kid.as_table_colgroup().inline_sizes.as_slice());
                self.col_min_inline_sizes = self.col_inline_sizes.clone();
                self.col_pref_inline_sizes = self.col_inline_sizes.clone();
            } else if kid.is_table_rowgroup() || kid.is_table_row() {
                // read column inline-sizes from table-row-group/table-row, and assign
                // inline-size=0 for the columns not defined in column-group
                // FIXME: need to read inline-sizes from either table-header-group OR
                // first table-row
                match self.table_layout {
                    FixedLayout => {
                        let kid_col_inline_sizes = kid.col_inline_sizes();
                        if !did_first_row {
                            did_first_row = true;
                            let mut child_inline_sizes = kid_col_inline_sizes.iter();
                            for col_inline_size in self.col_inline_sizes.mut_iter() {
                                match child_inline_sizes.next() {
                                    Some(child_inline_size) => {
                                        if *col_inline_size == Au::new(0) {
                                            *col_inline_size = *child_inline_size;
                                        }
                                    },
                                    None => break
                                }
                            }
                        }
                        let num_child_cols = kid_col_inline_sizes.len();
                        let num_cols = self.col_inline_sizes.len();
                        debug!("table until the previous row has {} column(s) and this row has {} column(s)",
                               num_cols, num_child_cols);
                        for i in range(num_cols, num_child_cols) {
                            self.col_inline_sizes.push((*kid_col_inline_sizes)[i]);
                        }
                    },
                    AutoLayout => {
                        min_inline_size = TableFlow::update_col_inline_sizes(&mut self.col_min_inline_sizes, kid.col_min_inline_sizes());
                        pref_inline_size = TableFlow::update_col_inline_sizes(&mut self.col_pref_inline_sizes, kid.col_pref_inline_sizes());

                        // update the number of column inline-sizes from table-rows.
                        let num_cols = self.col_min_inline_sizes.len();
                        let num_child_cols = kid.col_min_inline_sizes().len();
                        debug!("table until the previous row has {} column(s) and this row has {} column(s)",
                               num_cols, num_child_cols);
                        for i in range(num_cols, num_child_cols) {
                            self.col_inline_sizes.push(Au::new(0));
                            let new_kid_min = kid.col_min_inline_sizes()[i];
                            self.col_min_inline_sizes.push( new_kid_min );
                            let new_kid_pref = kid.col_pref_inline_sizes()[i];
                            self.col_pref_inline_sizes.push( new_kid_pref );
                            min_inline_size = min_inline_size + new_kid_min;
                            pref_inline_size = pref_inline_size + new_kid_pref;
                        }
                    }
                }
            }
        }

        let fragment_intrinsic_inline_sizes = self.block_flow.fragment.intrinsic_inline_sizes();
        self.block_flow.base.intrinsic_inline_sizes.minimum_inline_size = min_inline_size;
        self.block_flow.base.intrinsic_inline_sizes.preferred_inline_size =
            max(min_inline_size, pref_inline_size);
        self.block_flow.base.intrinsic_inline_sizes.surround_inline_size =
            fragment_intrinsic_inline_sizes.surround_inline_size;
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments. When
    /// called on this context, the context has had its inline-size set by the parent context.
    fn assign_inline_sizes(&mut self, ctx: &LayoutContext) {
        let _scope = layout_debug_scope!("table::assign_inline_sizes {:s}",
                                            self.block_flow.base.debug_id());
        debug!("assign_inline_sizes({}): assigning inline_size for flow", "table");

        // The position was set to the containing block by the flow's parent.
        let containing_block_inline_size = self.block_flow.base.position.size.inline;

        let mut num_unspecified_inline_sizes = 0;
        let mut total_column_inline_size = Au::new(0);
        for col_inline_size in self.col_inline_sizes.iter() {
            if *col_inline_size == Au::new(0) {
                num_unspecified_inline_sizes += 1;
            } else {
                total_column_inline_size = total_column_inline_size.add(col_inline_size);
            }
        }

        let inline_size_computer = InternalTable;
        inline_size_computer.compute_used_inline_size(&mut self.block_flow, ctx, containing_block_inline_size);

        let inline_start_content_edge = self.block_flow.fragment.border_padding.inline_start;
        let padding_and_borders = self.block_flow.fragment.border_padding.inline_start_end();
        let content_inline_size = self.block_flow.fragment.border_box.size.inline - padding_and_borders;

        match self.table_layout {
            FixedLayout => {
                // In fixed table layout, we distribute extra space among the unspecified columns if there are
                // any, or among all the columns if all are specified.
                if (total_column_inline_size < content_inline_size) && (num_unspecified_inline_sizes == 0) {
                    let ratio = content_inline_size.to_f64().unwrap() / total_column_inline_size.to_f64().unwrap();
                    for col_inline_size in self.col_inline_sizes.mut_iter() {
                        *col_inline_size = (*col_inline_size).scale_by(ratio);
                    }
                } else if num_unspecified_inline_sizes != 0 {
                    let extra_column_inline_size = (content_inline_size - total_column_inline_size) / num_unspecified_inline_sizes;
                    for col_inline_size in self.col_inline_sizes.mut_iter() {
                        if *col_inline_size == Au(0) {
                            *col_inline_size = extra_column_inline_size;
                        }
                    }
                }
            }
            _ => {}
        }

        self.block_flow.propagate_assigned_inline_size_to_children(inline_start_content_edge, content_inline_size, Some(self.col_inline_sizes.clone()));
    }

    fn assign_block_size<'a>(&mut self, ctx: &'a LayoutContext<'a>) {
        debug!("assign_block_size: assigning block_size for table");
        self.assign_block_size_table_base(ctx);
    }

    fn compute_absolute_position(&mut self) {
        self.block_flow.compute_absolute_position()
    }

    fn generated_containing_block_rect(&self) -> LogicalRect<Au> {
        self.block_flow.generated_containing_block_rect()
    }
}

impl fmt::Show for TableFlow {
    /// Outputs a debugging string describing this table flow.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableFlow: {}", self.block_flow)
    }
}

/// Table, TableRowGroup, TableRow, TableCell types.
/// Their inline-sizes are calculated in the same way and do not have margins.
pub struct InternalTable;

impl ISizeAndMarginsComputer for InternalTable {
    /// Compute the used value of inline-size, taking care of min-inline-size and max-inline-size.
    ///
    /// CSS Section 10.4: Minimum and Maximum inline-sizes
    fn compute_used_inline_size(&self,
                          block: &mut BlockFlow,
                          ctx: &LayoutContext,
                          parent_flow_inline_size: Au) {
        let input = self.compute_inline_size_constraint_inputs(block, parent_flow_inline_size, ctx);
        let solution = self.solve_inline_size_constraints(block, &input);
        self.set_inline_size_constraint_solutions(block, solution);
    }

    /// Solve the inline-size and margins constraints for this block flow.
    fn solve_inline_size_constraints(&self, _: &mut BlockFlow, input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution {
        ISizeConstraintSolution::new(input.available_inline_size, Au::new(0), Au::new(0))
    }
}
