/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_block)]

use block::BlockFlow;
use block::ISizeAndMarginsComputer;
use construct::FlowConstructor;
use context::LayoutContext;
use flow::{TableRowGroupFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use flow;
use fragment::Fragment;
use table::{InternalTable, TableFlow};
use wrapper::ThreadSafeLayoutNode;

use servo_util::geometry::Au;
use servo_util::geometry;
use std::fmt;

/// A table formatting context.
pub struct TableRowGroupFlow {
    pub block_flow: BlockFlow,

    /// Column isizes
    pub col_isizes: Vec<Au>,

    /// Column min isizes.
    pub col_min_isizes: Vec<Au>,

    /// Column pref isizes.
    pub col_pref_isizes: Vec<Au>,
}

impl TableRowGroupFlow {
    pub fn from_node_and_fragment(node: &ThreadSafeLayoutNode,
                                  fragment: Fragment)
                                  -> TableRowGroupFlow {
        TableRowGroupFlow {
            block_flow: BlockFlow::from_node_and_fragment(node, fragment),
            col_isizes: vec!(),
            col_min_isizes: vec!(),
            col_pref_isizes: vec!(),
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableRowGroupFlow {
        TableRowGroupFlow {
            block_flow: BlockFlow::from_node(constructor, node),
            col_isizes: vec!(),
            col_min_isizes: vec!(),
            col_pref_isizes: vec!(),
        }
    }

    pub fn fragment<'a>(&'a mut self) -> &'a Fragment {
        &self.block_flow.fragment
    }

    fn initialize_offsets(&mut self) -> (Au, Au, Au) {
        // TODO: If border-collapse: collapse, bstart_offset, bend_offset, and istart_offset
        // should be updated. Currently, they are set as Au(0).
        (Au(0), Au(0), Au(0))
    }

    /// Assign bsize for table-rowgroup flow.
    ///
    /// FIXME(pcwalton): This doesn't handle floats right.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_bsize_table_rowgroup_base(&mut self, layout_context: &mut LayoutContext) {
        let (bstart_offset, _, _) = self.initialize_offsets();

        let mut cur_y = bstart_offset;

        for kid in self.block_flow.base.child_iter() {
            kid.assign_bsize_for_inorder_child_if_necessary(layout_context);

            let child_node = flow::mut_base(kid);
            child_node.position.start.b = cur_y;
            cur_y = cur_y + child_node.position.size.bsize;
        }

        let bsize = cur_y - bstart_offset;

        let mut position = self.block_flow.fragment.border_box;
        position.size.bsize = bsize;
        self.block_flow.fragment.border_box = position;
        self.block_flow.base.position.size.bsize = bsize;
    }

    pub fn build_display_list_table_rowgroup(&mut self, layout_context: &LayoutContext) {
        debug!("build_display_list_table_rowgroup: same process as block flow");
        self.block_flow.build_display_list_block(layout_context)
    }
}

impl Flow for TableRowGroupFlow {
    fn class(&self) -> FlowClass {
        TableRowGroupFlowClass
    }

    fn as_table_rowgroup<'a>(&'a mut self) -> &'a mut TableRowGroupFlow {
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

    /// Recursively (bottom-up) determines the context's preferred and minimum isizes. When called
    /// on this context, all child contexts have had their min/pref isizes set. This function must
    /// decide min/pref isizes based on child context isizes and dimensions of any fragments it is
    /// responsible for flowing.
    /// Min/pref isizes set by this function are used in automatic table layout calculation.
    /// Also, this function finds the specified column isizes from the first row.
    /// Those are used in fixed table layout calculation
    fn bubble_isizes(&mut self, _: &mut LayoutContext) {
        let mut min_isize = Au(0);
        let mut pref_isize = Au(0);

        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_row());

            // calculate min_isize & pref_isize for automatic table layout calculation
            // 'self.col_min_isizes' collects the maximum value of cells' min-isizes for each column.
            // 'self.col_pref_isizes' collects the maximum value of cells' pref-isizes for each column.
            if self.col_isizes.is_empty() { // First Row
                assert!(self.col_min_isizes.is_empty() && self.col_pref_isizes.is_empty());
                // 'self.col_isizes' collects the specified column isizes from the first table-row for fixed table layout calculation.
                self.col_isizes = kid.col_isizes().clone();
                self.col_min_isizes = kid.col_min_isizes().clone();
                self.col_pref_isizes = kid.col_pref_isizes().clone();
            } else {
                min_isize = TableFlow::update_col_isizes(&mut self.col_min_isizes, kid.col_min_isizes());
                pref_isize = TableFlow::update_col_isizes(&mut self.col_pref_isizes, kid.col_pref_isizes());

                // update the number of column isizes from table-rows.
                let num_cols = self.col_isizes.len();
                let num_child_cols = kid.col_min_isizes().len();
                for i in range(num_cols, num_child_cols) {
                    self.col_isizes.push(Au::new(0));
                    let new_kid_min = *kid.col_min_isizes().get(i);
                    self.col_min_isizes.push(*kid.col_min_isizes().get(i));
                    let new_kid_pref = *kid.col_pref_isizes().get(i);
                    self.col_pref_isizes.push(*kid.col_pref_isizes().get(i));
                    min_isize = min_isize + new_kid_min;
                    pref_isize = pref_isize + new_kid_pref;
                }
            }
        }

        self.block_flow.base.intrinsic_isizes.minimum_isize = min_isize;
        self.block_flow.base.intrinsic_isizes.preferred_isize = geometry::max(min_isize,
                                                                              pref_isize);
    }

    /// Recursively (top-down) determines the actual isize of child contexts and fragments. When
    /// called on this context, the context has had its isize set by the parent context.
    fn assign_isizes(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_isizes({}): assigning isize for flow", "table_rowgroup");

        // The position was set to the containing block by the flow's parent.
        let containing_block_isize = self.block_flow.base.position.size.isize;
        // FIXME: In case of border-collapse: collapse, istart_content_edge should be
        // the border width on the inline-start side.
        let istart_content_edge = Au::new(0);
        let content_isize = containing_block_isize;

        let isize_computer = InternalTable;
        isize_computer.compute_used_isize(&mut self.block_flow, ctx, containing_block_isize);

        self.block_flow.propagate_assigned_isize_to_children(istart_content_edge, content_isize, Some(self.col_isizes.clone()));
    }

    fn assign_bsize(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_bsize: assigning bsize for table_rowgroup");
        self.assign_bsize_table_rowgroup_base(ctx);
    }

    fn compute_absolute_position(&mut self) {
        self.block_flow.compute_absolute_position()
    }
}

impl fmt::Show for TableRowGroupFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableRowGroupFlow: {}", self.block_flow.fragment)
    }
}
