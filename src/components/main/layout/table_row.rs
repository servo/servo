/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::block::BlockFlow;
use layout::block::WidthAndMarginsComputer;
use layout::construct::FlowConstructor;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, DisplayListBuildingInfo};
use layout::flow::{TableRowFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow;
use layout::table::InternalTable;
use layout::model::{MaybeAuto, Specified, Auto};
use layout::wrapper::ThreadSafeLayoutNode;

use gfx::display_list::StackingContext;
use servo_util::geometry::Au;
use servo_util::geometry;

/// A table formatting context.
pub struct TableRowFlow {
    pub block_flow: BlockFlow,

    /// Column widths.
    pub col_widths: ~[Au],

    /// Column min widths.
    pub col_min_widths: ~[Au],

    /// Column pref widths.
    pub col_pref_widths: ~[Au],
}

impl TableRowFlow {
    pub fn from_node_and_box(node: &ThreadSafeLayoutNode,
                             box_: Box)
                             -> TableRowFlow {
        TableRowFlow {
            block_flow: BlockFlow::from_node_and_box(node, box_),
            col_widths: ~[],
            col_min_widths: ~[],
            col_pref_widths: ~[],
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableRowFlow {
        TableRowFlow {
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

    /// Assign height for table-row flow.
    ///
    /// TODO(pcwalton): This doesn't handle floats and positioned elements right.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_height_table_row_base(&mut self, layout_context: &mut LayoutContext, inorder: bool) {
        let (top_offset, _, _) = self.initialize_offsets();

        let /* mut */ cur_y = top_offset;

        // Per CSS 2.1 § 17.5.3, find max_y = max( computed `height`, minimum height of all cells )
        let mut max_y = Au::new(0);
        for kid in self.block_flow.base.child_iter() {
            if inorder {
                kid.assign_height_inorder(layout_context)
            }

            {
                let child_box = kid.as_table_cell().box_();
                // TODO: Percentage height
                let child_specified_height = MaybeAuto::from_style(child_box.style().Box.get().height,
                                                                   Au::new(0)).specified_or_zero();
                max_y = geometry::max(max_y, child_specified_height + child_box.noncontent_height());
            }
            let child_node = flow::mut_base(kid);
            child_node.position.origin.y = cur_y;
            max_y = geometry::max(max_y, child_node.position.size.height);
        }

        let mut height = max_y;
        // TODO: Percentage height
        height = match MaybeAuto::from_style(self.block_flow.box_.style().Box.get().height, Au(0)) {
            Auto => height,
            Specified(value) => geometry::max(value, height)
        };
        // cur_y = cur_y + height;

        // Assign the height of own box
        //
        // FIXME(pcwalton): Take `cur_y` into account.
        let mut position = *self.block_flow.box_.border_box.borrow();
        position.size.height = height;
        *self.block_flow.box_.border_box.borrow_mut() = position;
        self.block_flow.base.position.size.height = height;

        // Assign the height of kid boxes, which is the same value as own height.
        for kid in self.block_flow.base.child_iter() {
            {
                let kid_box_ = kid.as_table_cell().box_();
                let mut position = *kid_box_.border_box.borrow();
                position.size.height = height;
                *kid_box_.border_box.borrow_mut() = position;
            }
            let child_node = flow::mut_base(kid);
            child_node.position.size.height = height;
        }
    }

    pub fn build_display_list_table_row(&mut self,
                                        stacking_context: &mut StackingContext,
                                        builder: &mut DisplayListBuilder,
                                        info: &DisplayListBuildingInfo) {
        debug!("build_display_list_table_row: same process as block flow");
        self.block_flow.build_display_list_block(stacking_context, builder, info)
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
    /// The specified column widths of children cells are used in fixed table layout calculation.
    fn bubble_widths(&mut self, _: &mut LayoutContext) {
        let mut min_width = Au(0);
        let mut pref_width = Au(0);
        let mut num_floats = 0;
        /* find the specified widths from child table-cell contexts */
        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_cell());

            // collect the specified column widths of cells. These are used in fixed table layout calculation.
            {
                let child_box = kid.as_table_cell().box_();
                let child_specified_width = MaybeAuto::from_style(child_box.style().Box.get().width,
                                                                  Au::new(0)).specified_or_zero();
                self.col_widths.push(child_specified_width);
            }

            // collect min_width & pref_width of children cells for automatic table layout calculation.
            let child_base = flow::mut_base(kid);
            self.col_min_widths.push(child_base.intrinsic_widths.minimum_width);
            self.col_pref_widths.push(child_base.intrinsic_widths.preferred_width);
            min_width = min_width + child_base.intrinsic_widths.minimum_width;
            pref_width = pref_width + child_base.intrinsic_widths.preferred_width;
            num_floats = num_floats + child_base.num_floats;
        }
        self.block_flow.base.num_floats = num_floats;
        self.block_flow.base.intrinsic_widths.minimum_width = min_width;
        self.block_flow.base.intrinsic_widths.preferred_width = geometry::max(min_width,
                                                                              pref_width);
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    fn assign_widths(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow", "table_row");

        // The position was set to the containing block by the flow's parent.
        let containing_block_width = self.block_flow.base.position.size.width;
        // FIXME: In case of border-collapse: collapse, left_content_edge should be border-left
        let left_content_edge = Au::new(0);

        let width_computer = InternalTable;
        width_computer.compute_used_width(&mut self.block_flow, ctx, containing_block_width);

        self.block_flow.propagate_assigned_width_to_children(left_content_edge, Au(0), Some(self.col_widths.clone()));
    }

    /// This is called on kid flows by a parent.
    ///
    /// Hence, we can assume that assign_height has already been called on the
    /// kid (because of the bottom-up traversal).
    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height_inorder: assigning height for table_row");
        self.assign_height_table_row_base(ctx, true);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height: assigning height for table_row");
        self.assign_height_table_row_base(ctx, false);
    }

    fn debug_str(&self) -> ~str {
        let txt = ~"TableRowFlow: ";
        txt.append(self.block_flow.box_.debug_str())
    }
}

