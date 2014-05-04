/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::block::BlockFlow;
use layout::block::WidthAndMarginsComputer;
use layout::construct::FlowConstructor;
use layout::context::LayoutContext;
use layout::flow::{TableRowGroupFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow;
use layout::table::{InternalTable, TableFlow};
use layout::wrapper::ThreadSafeLayoutNode;

use servo_util::geometry::Au;
use servo_util::geometry;

/// A table formatting context.
pub struct TableRowGroupFlow {
    pub block_flow: BlockFlow,

    /// Column widths
    pub col_widths: ~[Au],

    /// Column min widths.
    pub col_min_widths: ~[Au],

    /// Column pref widths.
    pub col_pref_widths: ~[Au],
}

impl TableRowGroupFlow {
    pub fn from_node_and_box(node: &ThreadSafeLayoutNode,
                             box_: Box)
                             -> TableRowGroupFlow {
        TableRowGroupFlow {
            block_flow: BlockFlow::from_node_and_box(node, box_),
            col_widths: ~[],
            col_min_widths: ~[],
            col_pref_widths: ~[],
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableRowGroupFlow {
        TableRowGroupFlow {
            block_flow: BlockFlow::from_node(constructor, node),
            col_widths: ~[],
            col_min_widths: ~[],
            col_pref_widths: ~[],
        }
    }

    pub fn teardown(&mut self) {
        self.block_flow.teardown();
        self.col_widths = ~[];
        self.col_min_widths = ~[];
        self.col_pref_widths = ~[];
    }

    pub fn box_<'a>(&'a mut self) -> &'a Box {
        &self.block_flow.box_
    }

    fn initialize_offsets(&mut self) -> (Au, Au, Au) {
        // TODO: If border-collapse: collapse, top_offset, bottom_offset, and left_offset
        // should be updated. Currently, they are set as Au(0).
        (Au(0), Au(0), Au(0))
    }

    /// Assign height for table-rowgroup flow.
    ///
    /// FIXME(pcwalton): This doesn't handle floats right.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_height_table_rowgroup_base(&mut self, layout_context: &mut LayoutContext) {
        let (top_offset, _, _) = self.initialize_offsets();

        let mut cur_y = top_offset;

        for kid in self.block_flow.base.child_iter() {
            kid.assign_height_for_inorder_child_if_necessary(layout_context);

            let child_node = flow::mut_base(kid);
            child_node.position.origin.y = cur_y;
            cur_y = cur_y + child_node.position.size.height;
        }

        let height = cur_y - top_offset;

        let mut position = self.block_flow.box_.border_box;
        position.size.height = height;
        self.block_flow.box_.border_box = position;
        self.block_flow.base.position.size.height = height;
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

    fn col_widths<'a>(&'a mut self) -> &'a mut ~[Au] {
        &mut self.col_widths
    }

    fn col_min_widths<'a>(&'a self) -> &'a ~[Au] {
        &self.col_min_widths
    }

    fn col_pref_widths<'a>(&'a self) -> &'a ~[Au] {
        &self.col_pref_widths
    }

    /// Recursively (bottom-up) determines the context's preferred and minimum widths. When called
    /// on this context, all child contexts have had their min/pref widths set. This function must
    /// decide min/pref widths based on child context widths and dimensions of any boxes it is
    /// responsible for flowing.
    /// Min/pref widths set by this function are used in automatic table layout calculation.
    /// Also, this function finds the specified column widths from the first row.
    /// Those are used in fixed table layout calculation
    fn bubble_widths(&mut self, _: &mut LayoutContext) {
        let mut min_width = Au(0);
        let mut pref_width = Au(0);

        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_row());

            // calculate min_width & pref_width for automatic table layout calculation
            // 'self.col_min_widths' collects the maximum value of cells' min-widths for each column.
            // 'self.col_pref_widths' collects the maximum value of cells' pref-widths for each column.
            if self.col_widths.is_empty() { // First Row
                assert!(self.col_min_widths.is_empty() && self.col_pref_widths.is_empty());
                // 'self.col_widths' collects the specified column widths from the first table-row for fixed table layout calculation.
                self.col_widths = kid.col_widths().clone();
                self.col_min_widths = kid.col_min_widths().clone();
                self.col_pref_widths = kid.col_pref_widths().clone();
            } else {
                min_width = TableFlow::update_col_widths(&mut self.col_min_widths, kid.col_min_widths());
                pref_width = TableFlow::update_col_widths(&mut self.col_pref_widths, kid.col_pref_widths());

                // update the number of column widths from table-rows.
                let num_cols = self.col_widths.len();
                let num_child_cols = kid.col_min_widths().len();
                for i in range(num_cols, num_child_cols) {
                    self.col_widths.push(Au::new(0));
                    let new_kid_min = kid.col_min_widths()[i];
                    self.col_min_widths.push(kid.col_min_widths()[i]);
                    let new_kid_pref = kid.col_pref_widths()[i];
                    self.col_pref_widths.push(kid.col_pref_widths()[i]);
                    min_width = min_width + new_kid_min;
                    pref_width = pref_width + new_kid_pref;
                }
            }
        }

        self.block_flow.base.intrinsic_widths.minimum_width = min_width;
        self.block_flow.base.intrinsic_widths.preferred_width = geometry::max(min_width,
                                                                              pref_width);
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    fn assign_widths(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow", "table_rowgroup");

        // The position was set to the containing block by the flow's parent.
        let containing_block_width = self.block_flow.base.position.size.width;
        // FIXME: In case of border-collapse: collapse, left_content_edge should be border-left
        let left_content_edge = Au::new(0);
        let content_width = containing_block_width;

        let width_computer = InternalTable;
        width_computer.compute_used_width(&mut self.block_flow, ctx, containing_block_width);

        self.block_flow.propagate_assigned_width_to_children(left_content_edge, content_width, Some(self.col_widths.clone()));
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height: assigning height for table_rowgroup");
        self.assign_height_table_rowgroup_base(ctx);
    }

    fn compute_absolute_position(&mut self) {
        self.block_flow.compute_absolute_position()
    }

    fn debug_str(&self) -> ~str {
        let txt = "TableRowGroupFlow: ".to_owned();
        txt.append(self.block_flow.box_.debug_str())
    }
}

