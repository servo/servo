/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_block)]

use block::BlockFlow;
use block::ISizeAndMarginsComputer;
use construct::FlowConstructor;
use context::LayoutContext;
use flow::{TableRowFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use flow;
use fragment::Fragment;
use table::InternalTable;
use model::{MaybeAuto, Specified, Auto};
use wrapper::ThreadSafeLayoutNode;

use servo_util::geometry::Au;
use servo_util::geometry;
use std::fmt;

/// A table formatting context.
pub struct TableRowFlow {
    pub block_flow: BlockFlow,

    /// Column inline-sizes.
    pub col_inline_sizes: Vec<Au>,

    /// Column min inline-sizes.
    pub col_min_inline_sizes: Vec<Au>,

    /// Column pref inline-sizes.
    pub col_pref_inline_sizes: Vec<Au>,
}

impl TableRowFlow {
    pub fn from_node_and_fragment(node: &ThreadSafeLayoutNode,
                                  fragment: Fragment)
                                  -> TableRowFlow {
        TableRowFlow {
            block_flow: BlockFlow::from_node_and_fragment(node, fragment),
            col_inline_sizes: vec!(),
            col_min_inline_sizes: vec!(),
            col_pref_inline_sizes: vec!(),
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableRowFlow {
        TableRowFlow {
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

    /// Assign block-size for table-row flow.
    ///
    /// TODO(pcwalton): This doesn't handle floats and positioned elements right.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_block_size_table_row_base(&mut self, layout_context: &mut LayoutContext) {
        let (block_start_offset, _, _) = self.initialize_offsets();

        let /* mut */ cur_y = block_start_offset;

        // Per CSS 2.1 § 17.5.3, find max_y = max( computed `block-size`, minimum block-size of all cells )
        let mut max_y = Au::new(0);
        for kid in self.block_flow.base.child_iter() {
            kid.assign_block_size_for_inorder_child_if_necessary(layout_context);

            {
                let child_fragment = kid.as_table_cell().fragment();
                // TODO: Percentage block-size
                let child_specified_block_size = MaybeAuto::from_style(child_fragment.style().content_block_size(),
                                                                   Au::new(0)).specified_or_zero();
                max_y =
                    geometry::max(max_y,
                                  child_specified_block_size + child_fragment.border_padding.block_start_end());
            }
            let child_node = flow::mut_base(kid);
            child_node.position.start.b = cur_y;
            max_y = geometry::max(max_y, child_node.position.size.block);
        }

        let mut block_size = max_y;
        // TODO: Percentage block-size
        block_size = match MaybeAuto::from_style(self.block_flow.fragment.style().content_block_size(), Au(0)) {
            Auto => block_size,
            Specified(value) => geometry::max(value, block_size)
        };
        // cur_y = cur_y + block-size;

        // Assign the block-size of own fragment
        //
        // FIXME(pcwalton): Take `cur_y` into account.
        let mut position = self.block_flow.fragment.border_box;
        position.size.block = block_size;
        self.block_flow.fragment.border_box = position;
        self.block_flow.base.position.size.block = block_size;

        // Assign the block-size of kid fragments, which is the same value as own block-size.
        for kid in self.block_flow.base.child_iter() {
            {
                let kid_fragment = kid.as_table_cell().mut_fragment();
                let mut position = kid_fragment.border_box;
                position.size.block = block_size;
                kid_fragment.border_box = position;
            }
            let child_node = flow::mut_base(kid);
            child_node.position.size.block = block_size;
        }
    }

    pub fn build_display_list_table_row(&mut self, layout_context: &LayoutContext) {
        debug!("build_display_list_table_row: same process as block flow");
        self.block_flow.build_display_list_block(layout_context)
    }
}

impl Flow for TableRowFlow {
    fn class(&self) -> FlowClass {
        TableRowFlowClass
    }

    fn as_table_row<'a>(&'a mut self) -> &'a mut TableRowFlow {
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
    /// The specified column inline-sizes of children cells are used in fixed table layout calculation.
    fn bubble_inline_sizes(&mut self, _: &mut LayoutContext) {
        let mut min_inline_size = Au(0);
        let mut pref_inline_size = Au(0);
        /* find the specified inline_sizes from child table-cell contexts */
        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_cell());

            // collect the specified column inline-sizes of cells. These are used in fixed table layout calculation.
            {
                let child_fragment = kid.as_table_cell().fragment();
                let child_specified_inline_size = MaybeAuto::from_style(child_fragment.style().content_inline_size(),
                                                                  Au::new(0)).specified_or_zero();
                self.col_inline_sizes.push(child_specified_inline_size);
            }

            // collect min_inline-size & pref_inline-size of children cells for automatic table layout calculation.
            let child_base = flow::mut_base(kid);
            self.col_min_inline_sizes.push(child_base.intrinsic_inline_sizes.minimum_inline_size);
            self.col_pref_inline_sizes.push(child_base.intrinsic_inline_sizes.preferred_inline_size);
            min_inline_size = min_inline_size + child_base.intrinsic_inline_sizes.minimum_inline_size;
            pref_inline_size = pref_inline_size + child_base.intrinsic_inline_sizes.preferred_inline_size;
        }
        self.block_flow.base.intrinsic_inline_sizes.minimum_inline_size = min_inline_size;
        self.block_flow.base.intrinsic_inline_sizes.preferred_inline_size = geometry::max(min_inline_size,
                                                                              pref_inline_size);
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments. When called
    /// on this context, the context has had its inline-size set by the parent context.
    fn assign_inline_sizes(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_inline_sizes({}): assigning inline_size for flow", "table_row");

        // The position was set to the containing block by the flow's parent.
        let containing_block_inline_size = self.block_flow.base.position.size.inline;
        // FIXME: In case of border-collapse: collapse, inline-start_content_edge should be border-inline-start
        let inline_start_content_edge = Au::new(0);

        let inline_size_computer = InternalTable;
        inline_size_computer.compute_used_inline_size(&mut self.block_flow, ctx, containing_block_inline_size);

        self.block_flow.propagate_assigned_inline_size_to_children(inline_start_content_edge, Au(0), Some(self.col_inline_sizes.clone()));
    }

    fn assign_block_size(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_block_size: assigning block_size for table_row");
        self.assign_block_size_table_row_base(ctx);
    }

    fn compute_absolute_position(&mut self) {
        self.block_flow.compute_absolute_position()
    }
}

impl fmt::Show for TableRowFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableRowFlow: {}", self.block_flow.fragment)
    }
}
