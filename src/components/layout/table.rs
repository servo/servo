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
use table_wrapper::{TableLayout, FixedLayout, AutoLayout};
use wrapper::ThreadSafeLayoutNode;

use servo_util::geometry::Au;
use servo_util::geometry;
use std::fmt;
use style::computed_values::table_layout;

/// A table flow corresponded to the table's internal table fragment under a table wrapper flow.
/// The properties `position`, `float`, and `margin-*` are used on the table wrapper fragment,
/// not table fragment per CSS 2.1 § 10.5.
pub struct TableFlow {
    pub block_flow: BlockFlow,

    /// Column isizes
    pub col_isizes: Vec<Au>,

    /// Column min isizes.
    pub col_min_isizes: Vec<Au>,

    /// Column pref isizes.
    pub col_pref_isizes: Vec<Au>,

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
            col_isizes: vec!(),
            col_min_isizes: vec!(),
            col_pref_isizes: vec!(),
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
            col_isizes: vec!(),
            col_min_isizes: vec!(),
            col_pref_isizes: vec!(),
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
            col_isizes: vec!(),
            col_min_isizes: vec!(),
            col_pref_isizes: vec!(),
            table_layout: table_layout
        }
    }

    /// Update the corresponding value of self_isizes if a value of kid_isizes has larger value
    /// than one of self_isizes.
    pub fn update_col_isizes(self_isizes: &mut Vec<Au>, kid_isizes: &Vec<Au>) -> Au {
        let mut sum_isizes = Au(0);
        let mut kid_isizes_it = kid_isizes.iter();
        for self_isize in self_isizes.mut_iter() {
            match kid_isizes_it.next() {
                Some(kid_isize) => {
                    if *self_isize < *kid_isize {
                        *self_isize = *kid_isize;
                    }
                },
                None => {}
            }
            sum_isizes = sum_isizes + *self_isize;
        }
        sum_isizes
    }

    /// Assign bsize for table flow.
    ///
    /// TODO(#2014, pcwalton): This probably doesn't handle margin collapse right.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_bsize_table_base(&mut self, layout_context: &mut LayoutContext) {
        self.block_flow.assign_bsize_block_base(layout_context, MarginsMayNotCollapse);
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

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
    }

    fn col_isizes<'a>(&'a mut self) -> &'a mut Vec<Au> {
        &mut self.col_isizes
    }

    fn col_min_isizes<'a>(&'a self) -> &'a Vec<Au> {
        &self.col_min_isizes
    }

    fn col_pref_isizes<'a>(&'a self) -> &'a Vec<Au> {
        &self.col_pref_isizes
    }

    /// The specified column isizes are set from column group and the first row for the fixed
    /// table layout calculation.
    /// The maximum min/pref isizes of each column are set from the rows for the automatic
    /// table layout calculation.
    fn bubble_isizes(&mut self, _: &mut LayoutContext) {
        let mut min_isize = Au(0);
        let mut pref_isize = Au(0);
        let mut did_first_row = false;

        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_proper_table_child());

            if kid.is_table_colgroup() {
                self.col_isizes.push_all(kid.as_table_colgroup().isizes.as_slice());
                self.col_min_isizes = self.col_isizes.clone();
                self.col_pref_isizes = self.col_isizes.clone();
            } else if kid.is_table_rowgroup() || kid.is_table_row() {
                // read column isizes from table-row-group/table-row, and assign
                // isize=0 for the columns not defined in column-group
                // FIXME: need to read isizes from either table-header-group OR
                // first table-row
                match self.table_layout {
                    FixedLayout => {
                        let kid_col_isizes = kid.col_isizes();
                        if !did_first_row {
                            did_first_row = true;
                            let mut child_isizes = kid_col_isizes.iter();
                            for col_isize in self.col_isizes.mut_iter() {
                                match child_isizes.next() {
                                    Some(child_isize) => {
                                        if *col_isize == Au::new(0) {
                                            *col_isize = *child_isize;
                                        }
                                    },
                                    None => break
                                }
                            }
                        }
                        let num_child_cols = kid_col_isizes.len();
                        let num_cols = self.col_isizes.len();
                        debug!("table until the previous row has {} column(s) and this row has {} column(s)",
                               num_cols, num_child_cols);
                        for i in range(num_cols, num_child_cols) {
                            self.col_isizes.push( *kid_col_isizes.get(i) );
                        }
                    },
                    AutoLayout => {
                        min_isize = TableFlow::update_col_isizes(&mut self.col_min_isizes, kid.col_min_isizes());
                        pref_isize = TableFlow::update_col_isizes(&mut self.col_pref_isizes, kid.col_pref_isizes());

                        // update the number of column isizes from table-rows.
                        let num_cols = self.col_min_isizes.len();
                        let num_child_cols = kid.col_min_isizes().len();
                        debug!("table until the previous row has {} column(s) and this row has {} column(s)",
                               num_cols, num_child_cols);
                        for i in range(num_cols, num_child_cols) {
                            self.col_isizes.push(Au::new(0));
                            let new_kid_min = *kid.col_min_isizes().get(i);
                            self.col_min_isizes.push( new_kid_min );
                            let new_kid_pref = *kid.col_pref_isizes().get(i);
                            self.col_pref_isizes.push( new_kid_pref );
                            min_isize = min_isize + new_kid_min;
                            pref_isize = pref_isize + new_kid_pref;
                        }
                    }
                }
            }
        }
        self.block_flow.base.intrinsic_isizes.minimum_isize = min_isize;
        self.block_flow.base.intrinsic_isizes.preferred_isize =
            geometry::max(min_isize, pref_isize);
    }

    /// Recursively (top-down) determines the actual isize of child contexts and fragments. When
    /// called on this context, the context has had its isize set by the parent context.
    fn assign_isizes(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_isizes({}): assigning isize for flow", "table");

        // The position was set to the containing block by the flow's parent.
        let containing_block_isize = self.block_flow.base.position.size.isize;

        let mut num_unspecified_isizes = 0;
        let mut total_column_isize = Au::new(0);
        for col_isize in self.col_isizes.iter() {
            if *col_isize == Au::new(0) {
                num_unspecified_isizes += 1;
            } else {
                total_column_isize = total_column_isize.add(col_isize);
            }
        }

        let isize_computer = InternalTable;
        isize_computer.compute_used_isize(&mut self.block_flow, ctx, containing_block_isize);

        let istart_content_edge = self.block_flow.fragment.border_padding.istart;
        let padding_and_borders = self.block_flow.fragment.border_padding.istart_end();
        let content_isize = self.block_flow.fragment.border_box.size.isize - padding_and_borders;

        match self.table_layout {
            FixedLayout => {
                // In fixed table layout, we distribute extra space among the unspecified columns if there are
                // any, or among all the columns if all are specified.
                if (total_column_isize < content_isize) && (num_unspecified_isizes == 0) {
                    let ratio = content_isize.to_f64().unwrap() / total_column_isize.to_f64().unwrap();
                    for col_isize in self.col_isizes.mut_iter() {
                        *col_isize = (*col_isize).scale_by(ratio);
                    }
                } else if num_unspecified_isizes != 0 {
                    let extra_column_isize = (content_isize - total_column_isize) / Au::new(num_unspecified_isizes);
                    for col_isize in self.col_isizes.mut_iter() {
                        if *col_isize == Au(0) {
                            *col_isize = extra_column_isize;
                        }
                    }
                }
            }
            _ => {}
        }

        self.block_flow.propagate_assigned_isize_to_children(istart_content_edge, content_isize, Some(self.col_isizes.clone()));
    }

    fn assign_bsize(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_bsize: assigning bsize for table");
        self.assign_bsize_table_base(ctx);
    }

    fn compute_absolute_position(&mut self) {
        self.block_flow.compute_absolute_position()
    }
}

impl fmt::Show for TableFlow {
    /// Outputs a debugging string describing this table flow.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableFlow: {}", self.block_flow)
    }
}

/// Table, TableRowGroup, TableRow, TableCell types.
/// Their isizes are calculated in the same way and do not have margins.
pub struct InternalTable;

impl ISizeAndMarginsComputer for InternalTable {
    /// Compute the used value of isize, taking care of min-isize and max-isize.
    ///
    /// CSS Section 10.4: Minimum and Maximum isizes
    fn compute_used_isize(&self,
                          block: &mut BlockFlow,
                          ctx: &mut LayoutContext,
                          parent_flow_isize: Au) {
        let input = self.compute_isize_constraint_inputs(block, parent_flow_isize, ctx);
        let solution = self.solve_isize_constraints(block, &input);
        self.set_isize_constraint_solutions(block, solution);
    }

    /// Solve the isize and margins constraints for this block flow.
    fn solve_isize_constraints(&self, _: &mut BlockFlow, input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution {
        ISizeConstraintSolution::new(input.available_isize, Au::new(0), Au::new(0))
    }
}
