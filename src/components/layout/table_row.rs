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

    /// Column isizes.
    pub col_isizes: Vec<Au>,

    /// Column min isizes.
    pub col_min_isizes: Vec<Au>,

    /// Column pref isizes.
    pub col_pref_isizes: Vec<Au>,
}

impl TableRowFlow {
    pub fn from_node_and_fragment(node: &ThreadSafeLayoutNode,
                                  fragment: Fragment)
                                  -> TableRowFlow {
        TableRowFlow {
            block_flow: BlockFlow::from_node_and_fragment(node, fragment),
            col_isizes: vec!(),
            col_min_isizes: vec!(),
            col_pref_isizes: vec!(),
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableRowFlow {
        TableRowFlow {
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

    /// Assign bsize for table-row flow.
    ///
    /// TODO(pcwalton): This doesn't handle floats and positioned elements right.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_bsize_table_row_base(&mut self, layout_context: &mut LayoutContext) {
        let (bstart_offset, _, _) = self.initialize_offsets();

        let /* mut */ cur_y = bstart_offset;

        // Per CSS 2.1 § 17.5.3, find max_y = max( computed `bsize`, minimum bsize of all cells )
        let mut max_y = Au::new(0);
        for kid in self.block_flow.base.child_iter() {
            kid.assign_bsize_for_inorder_child_if_necessary(layout_context);

            {
                let child_fragment = kid.as_table_cell().fragment();
                // TODO: Percentage bsize
                let child_specified_bsize = MaybeAuto::from_style(child_fragment.style().content_bsize(),
                                                                   Au::new(0)).specified_or_zero();
                max_y =
                    geometry::max(max_y,
                                  child_specified_bsize + child_fragment.border_padding.bstart_end());
            }
            let child_node = flow::mut_base(kid);
            child_node.position.start.b = cur_y;
            max_y = geometry::max(max_y, child_node.position.size.bsize);
        }

        let mut bsize = max_y;
        // TODO: Percentage bsize
        bsize = match MaybeAuto::from_style(self.block_flow.fragment.style().content_bsize(), Au(0)) {
            Auto => bsize,
            Specified(value) => geometry::max(value, bsize)
        };
        // cur_y = cur_y + bsize;

        // Assign the bsize of own fragment
        //
        // FIXME(pcwalton): Take `cur_y` into account.
        let mut position = self.block_flow.fragment.border_box;
        position.size.bsize = bsize;
        self.block_flow.fragment.border_box = position;
        self.block_flow.base.position.size.bsize = bsize;

        // Assign the bsize of kid fragments, which is the same value as own bsize.
        for kid in self.block_flow.base.child_iter() {
            {
                let kid_fragment = kid.as_table_cell().mut_fragment();
                let mut position = kid_fragment.border_box;
                position.size.bsize = bsize;
                kid_fragment.border_box = position;
            }
            let child_node = flow::mut_base(kid);
            child_node.position.size.bsize = bsize;
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
    /// The specified column isizes of children cells are used in fixed table layout calculation.
    fn bubble_isizes(&mut self, _: &mut LayoutContext) {
        let mut min_isize = Au(0);
        let mut pref_isize = Au(0);
        /* find the specified isizes from child table-cell contexts */
        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_cell());

            // collect the specified column isizes of cells. These are used in fixed table layout calculation.
            {
                let child_fragment = kid.as_table_cell().fragment();
                let child_specified_isize = MaybeAuto::from_style(child_fragment.style().content_isize(),
                                                                  Au::new(0)).specified_or_zero();
                self.col_isizes.push(child_specified_isize);
            }

            // collect min_isize & pref_isize of children cells for automatic table layout calculation.
            let child_base = flow::mut_base(kid);
            self.col_min_isizes.push(child_base.intrinsic_isizes.minimum_isize);
            self.col_pref_isizes.push(child_base.intrinsic_isizes.preferred_isize);
            min_isize = min_isize + child_base.intrinsic_isizes.minimum_isize;
            pref_isize = pref_isize + child_base.intrinsic_isizes.preferred_isize;
        }
        self.block_flow.base.intrinsic_isizes.minimum_isize = min_isize;
        self.block_flow.base.intrinsic_isizes.preferred_isize = geometry::max(min_isize,
                                                                              pref_isize);
    }

    /// Recursively (top-down) determines the actual isize of child contexts and fragments. When called
    /// on this context, the context has had its isize set by the parent context.
    fn assign_isizes(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_isizes({}): assigning isize for flow", "table_row");

        // The position was set to the containing block by the flow's parent.
        let containing_block_isize = self.block_flow.base.position.size.isize;
        // FIXME: In case of border-collapse: collapse, istart_content_edge should be border-istart
        let istart_content_edge = Au::new(0);

        let isize_computer = InternalTable;
        isize_computer.compute_used_isize(&mut self.block_flow, ctx, containing_block_isize);

        self.block_flow.propagate_assigned_isize_to_children(istart_content_edge, Au(0), Some(self.col_isizes.clone()));
    }

    fn assign_bsize(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_bsize: assigning bsize for table_row");
        self.assign_bsize_table_row_base(ctx);
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
