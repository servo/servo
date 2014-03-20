/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::block::BlockFlow;
use layout::block::WidthAndMarginsComputer;
use layout::construct::FlowConstructor;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{TableRowFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow;
use layout::table::InternalTable;
use layout::model::{MaybeAuto, Specified, Auto};
use layout::wrapper::ThreadSafeLayoutNode;

use std::cell::RefCell;
use geom::{Point2D, Rect, Size2D};
use gfx::display_list::DisplayListCollection;
use servo_util::geometry::Au;
use servo_util::geometry;

/// A table formatting context.
pub struct TableRowFlow {
    block_flow: BlockFlow,

    /// Column widths.
    col_widths: ~[Au],
}

impl TableRowFlow {
    pub fn from_node_and_box(node: &ThreadSafeLayoutNode,
                             box_: Box)
                             -> TableRowFlow {
        TableRowFlow {
            block_flow: BlockFlow::from_node_and_box(node, box_),
            col_widths: ~[],
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableRowFlow {
        TableRowFlow {
            block_flow: BlockFlow::from_node(constructor, node),
            col_widths: ~[],
        }
    }

    pub fn teardown(&mut self) {
        self.block_flow.teardown();
        self.col_widths = ~[];
    }

    pub fn box_<'a>(&'a mut self) -> &'a Option<Box>{
        &self.block_flow.box_
    }

    fn initialize_offsets(&mut self) -> (Au, Au, Au) {
        // TODO: If border-collapse: collapse, top_offset, bottom_offset, and left_offset
        // should be updated. Currently, they are set as Au(0).
        (Au(0), Au(0), Au(0))
    }

    /// Assign height for table-row flow.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_height_table_row_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {
        let (top_offset, bottom_offset, left_offset) = self.initialize_offsets();

        self.block_flow.handle_children_floats_if_necessary(ctx, inorder,
                                                            left_offset, top_offset);
        let mut cur_y = top_offset;

        // Per CSS 2.1 § 17.5.3, find max_y = max( computed `height`, minimum height of all cells )
        let mut max_y = Au::new(0);
        for kid in self.block_flow.base.child_iter() {
            for child_box in kid.as_table_cell().box_().iter() {
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
        for box_ in self.block_flow.box_.iter() {
            // TODO: Percentage height
            height = match MaybeAuto::from_style(box_.style().Box.get().height, Au(0)) {
                Auto => height,
                Specified(value) => geometry::max(value, height)
            };
        }
        cur_y = cur_y + height;

        // Assign the height of own box
        for box_ in self.block_flow.box_.iter() {
            let mut position = box_.border_box.get();
            position.size.height = height;
            box_.border_box.set(position);
        }
        self.block_flow.base.position.size.height = height;

        // Assign the height of kid boxes, which is the same value as own height.
        for kid in self.block_flow.base.child_iter() {
            for kid_box_ in kid.as_table_cell().box_().iter() {
                let mut position = kid_box_.border_box.get();
                position.size.height = height;
                kid_box_.border_box.set(position);
            }
            let child_node = flow::mut_base(kid);
            child_node.position.size.height = height;
        }

        self.block_flow.set_floats_out_if_inorder(inorder, height, cur_y,
                                                  top_offset, bottom_offset, left_offset);
    }

    pub fn build_display_list_table_row<E:ExtraDisplayListData>(
                                        &mut self,
                                        builder: &DisplayListBuilder,
                                        container_block_size: &Size2D<Au>,
                                        absolute_cb_abs_position: Point2D<Au>,
                                        dirty: &Rect<Au>,
                                        index: uint,
                                        lists: &RefCell<DisplayListCollection<E>>)
                                        -> uint {
        debug!("build_display_list_table_row: same process as block flow");
        self.block_flow.build_display_list_block(builder, container_block_size,
                                                 absolute_cb_abs_position,
                                                 dirty, index, lists)
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

    /// Recursively (bottom-up) determines the context's preferred and minimum widths. When called
    /// on this context, all child contexts have had their min/pref widths set. This function must
    /// decide min/pref widths based on child context widths and dimensions of any boxes it is
    /// responsible for flowing.
    /// Min/pref widths set by this function are used in automatic table layout calculation.
    /// Also, this function collects the specified column widths of children cells. Those are used
    /// in fixed table layout calculation
    fn bubble_widths(&mut self, ctx: &mut LayoutContext) {
        /* find the specified widths from child table-cell contexts */
        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_cell());

            for child_box in kid.as_table_cell().box_().iter() {
                let child_specified_width = MaybeAuto::from_style(child_box.style().Box.get().width,
                                                                  Au::new(0)).specified_or_zero();
                self.col_widths.push(child_specified_width);
            }
        }

        // TODO: calculate min_width & pref_width for automatic table layout calculation
        self.block_flow.bubble_widths(ctx);
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

    /// TableRowBox and their parents(TableBox) do not have margins.
    /// Therefore, margins to be collapsed do not exist.
    fn collapse_margins(&mut self, _: bool, _: &mut bool, _: &mut Au,
                        _: &mut Au, _: &mut Au, _: &mut Au) {
    }

    fn debug_str(&self) -> ~str {
        let txt = ~"TableRowFlow: ";
        txt.append(match self.block_flow.box_ {
            Some(ref rb) => rb.debug_str(),
            None => ~"",
        })
    }
}

