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
use layout_debug;
use table::{InternalTable, TableFlow};
use wrapper::ThreadSafeLayoutNode;

use servo_util::geometry::Au;
use std::cmp::max;
use std::fmt;

/// A table formatting context.
#[deriving(Encodable)]
pub struct TableRowGroupFlow {
    pub block_flow: BlockFlow,

    /// Column inline-sizes
    pub col_inline_sizes: Vec<Au>,

    /// Column min inline-sizes.
    pub col_min_inline_sizes: Vec<Au>,

    /// Column pref inline-sizes.
    pub col_pref_inline_sizes: Vec<Au>,
}

impl TableRowGroupFlow {
    pub fn from_node_and_fragment(node: &ThreadSafeLayoutNode,
                                  fragment: Fragment)
                                  -> TableRowGroupFlow {
        TableRowGroupFlow {
            block_flow: BlockFlow::from_node_and_fragment(node, fragment),
            col_inline_sizes: vec!(),
            col_min_inline_sizes: vec!(),
            col_pref_inline_sizes: vec!(),
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableRowGroupFlow {
        TableRowGroupFlow {
            block_flow: BlockFlow::from_node(constructor, node),
            col_inline_sizes: vec!(),
            col_min_inline_sizes: vec!(),
            col_pref_inline_sizes: vec!(),
        }
    }

    pub fn fragment<'a>(&'a mut self) -> &'a Fragment {
        &self.block_flow.fragment
    }

    fn initialize_offsets(&mut self) -> (Au, Au, Au) {
        // TODO: If border-collapse: collapse, block-start_offset, block-end_offset, and inline-start_offset
        // should be updated. Currently, they are set as Au(0).
        (Au(0), Au(0), Au(0))
    }

    /// Assign block-size for table-rowgroup flow.
    ///
    /// FIXME(pcwalton): This doesn't handle floats right.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_block_size_table_rowgroup_base<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        let (block_start_offset, _, _) = self.initialize_offsets();

        let mut cur_y = block_start_offset;

        for kid in self.block_flow.base.child_iter() {
            kid.assign_block_size_for_inorder_child_if_necessary(layout_context);

            let child_node = flow::mut_base(kid);
            child_node.position.start.b = cur_y;
            cur_y = cur_y + child_node.position.size.block;
        }

        let block_size = cur_y - block_start_offset;

        let mut position = self.block_flow.fragment.border_box;
        position.size.block = block_size;
        self.block_flow.fragment.border_box = position;
        self.block_flow.base.position.size.block = block_size;
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

    fn as_immutable_table_rowgroup<'a>(&'a self) -> &'a TableRowGroupFlow {
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

    /// Recursively (bottom-up) determines the context's preferred and minimum inline-sizes. When called
    /// on this context, all child contexts have had their min/pref inline-sizes set. This function must
    /// decide min/pref inline-sizes based on child context inline-sizes and dimensions of any fragments it is
    /// responsible for flowing.
    /// Min/pref inline-sizes set by this function are used in automatic table layout calculation.
    /// Also, this function finds the specified column inline-sizes from the first row.
    /// Those are used in fixed table layout calculation
    fn bubble_inline_sizes(&mut self, _: &LayoutContext) {
        let _scope = layout_debug_scope!("table_rowgroup::bubble_inline_sizes {:s}",
                                            self.block_flow.base.debug_id());

        let mut min_inline_size = Au(0);
        let mut pref_inline_size = Au(0);

        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_row());

            // calculate min_inline-size & pref_inline-size for automatic table layout calculation
            // 'self.col_min_inline-sizes' collects the maximum value of cells' min-inline-sizes for each column.
            // 'self.col_pref_inline-sizes' collects the maximum value of cells' pref-inline-sizes for each column.
            if self.col_inline_sizes.is_empty() { // First Row
                assert!(self.col_min_inline_sizes.is_empty() && self.col_pref_inline_sizes.is_empty());
                // 'self.col_inline-sizes' collects the specified column inline-sizes from the first table-row for fixed table layout calculation.
                self.col_inline_sizes = kid.col_inline_sizes().clone();
                self.col_min_inline_sizes = kid.col_min_inline_sizes().clone();
                self.col_pref_inline_sizes = kid.col_pref_inline_sizes().clone();
            } else {
                min_inline_size = TableFlow::update_col_inline_sizes(&mut self.col_min_inline_sizes, kid.col_min_inline_sizes());
                pref_inline_size = TableFlow::update_col_inline_sizes(&mut self.col_pref_inline_sizes, kid.col_pref_inline_sizes());

                // update the number of column inline-sizes from table-rows.
                let num_cols = self.col_inline_sizes.len();
                let num_child_cols = kid.col_min_inline_sizes().len();
                for i in range(num_cols, num_child_cols) {
                    self.col_inline_sizes.push(Au::new(0));
                    let new_kid_min = kid.col_min_inline_sizes()[i];
                    self.col_min_inline_sizes.push(kid.col_min_inline_sizes()[i]);
                    let new_kid_pref = kid.col_pref_inline_sizes()[i];
                    self.col_pref_inline_sizes.push(kid.col_pref_inline_sizes()[i]);
                    min_inline_size = min_inline_size + new_kid_min;
                    pref_inline_size = pref_inline_size + new_kid_pref;
                }
            }
        }

        self.block_flow.base.intrinsic_inline_sizes.minimum_inline_size = min_inline_size;
        self.block_flow.base.intrinsic_inline_sizes.preferred_inline_size = max(
            min_inline_size, pref_inline_size);
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments. When
    /// called on this context, the context has had its inline-size set by the parent context.
    fn assign_inline_sizes(&mut self, ctx: &LayoutContext) {
        let _scope = layout_debug_scope!("table_rowgroup::assign_inline_sizes {:s}",
                                            self.block_flow.base.debug_id());
        debug!("assign_inline_sizes({}): assigning inline_size for flow", "table_rowgroup");

        // The position was set to the containing block by the flow's parent.
        let containing_block_inline_size = self.block_flow.base.position.size.inline;
        // FIXME: In case of border-collapse: collapse, inline-start_content_edge should be
        // the border width on the inline-start side.
        let inline_start_content_edge = Au::new(0);
        let content_inline_size = containing_block_inline_size;

        let inline_size_computer = InternalTable;
        inline_size_computer.compute_used_inline_size(&mut self.block_flow, ctx, containing_block_inline_size);

        self.block_flow.propagate_assigned_inline_size_to_children(inline_start_content_edge, content_inline_size, Some(self.col_inline_sizes.clone()));
    }

    fn assign_block_size<'a>(&mut self, ctx: &'a LayoutContext<'a>) {
        debug!("assign_block_size: assigning block_size for table_rowgroup");
        self.assign_block_size_table_rowgroup_base(ctx);
    }

    fn compute_absolute_position(&mut self) {
        self.block_flow.compute_absolute_position()
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        self.block_flow.update_late_computed_inline_position_if_necessary(inline_position)
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        self.block_flow.update_late_computed_block_position_if_necessary(block_position)
    }
}

impl fmt::Show for TableRowGroupFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableRowGroupFlow: {}", self.block_flow.fragment)
    }
}
