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
use layout::flow::{TableRowGroupFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow;
use layout::table::InternalTable;
use layout::wrapper::ThreadSafeLayoutNode;

use std::cell::RefCell;
use geom::{Point2D, Rect, Size2D};
use gfx::display_list::DisplayListCollection;
use servo_util::geometry::Au;

/// A table formatting context.
pub struct TableRowGroupFlow {
    block_flow: BlockFlow,

    /// Column widths
    col_widths: ~[Au],
}

impl TableRowGroupFlow {
    pub fn from_node_and_box(node: &ThreadSafeLayoutNode,
                             box_: Box)
                             -> TableRowGroupFlow {
        TableRowGroupFlow {
            block_flow: BlockFlow::from_node_and_box(node, box_),
            col_widths: ~[],
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableRowGroupFlow {
        TableRowGroupFlow {
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

    /// Assign height for table-rowgroup flow.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_height_table_rowgroup_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {
        let (top_offset, bottom_offset, left_offset) = self.initialize_offsets();

        self.block_flow.handle_children_floats_if_necessary(ctx, inorder,
                                                            left_offset, top_offset);
        let mut cur_y = top_offset;

        for kid in self.block_flow.base.child_iter() {
            let child_node = flow::mut_base(kid);
            child_node.position.origin.y = cur_y;
            cur_y = cur_y + child_node.position.size.height;
        }

        let height = cur_y - top_offset;

        for box_ in self.block_flow.box_.iter() {
            let mut position = box_.border_box.get();
            position.size.height = height;
            box_.border_box.set(position);
        }
        self.block_flow.base.position.size.height = height;

        self.block_flow.set_floats_out_if_inorder(inorder, height, cur_y,
                                                  top_offset, bottom_offset, left_offset);
    }

    pub fn build_display_list_table_rowgroup<E:ExtraDisplayListData>(
                                            &mut self,
                                            builder: &DisplayListBuilder,
                                            container_block_size: &Size2D<Au>,
                                            absolute_cb_abs_position: Point2D<Au>,
                                            dirty: &Rect<Au>,
                                            index: uint,
                                            lists: &RefCell<DisplayListCollection<E>>)
                                            -> uint {
        debug!("build_display_list_table_rowgroup: same process as block flow");
        self.block_flow.build_display_list_block(builder, container_block_size,
                                                 absolute_cb_abs_position,
                                                 dirty, index, lists)
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

    /// Recursively (bottom-up) determines the context's preferred and minimum widths. When called
    /// on this context, all child contexts have had their min/pref widths set. This function must
    /// decide min/pref widths based on child context widths and dimensions of any boxes it is
    /// responsible for flowing.
    /// Min/pref widths set by this function are used in automatic table layout calculation.
    /// Also, this function finds the specified column widths from the first row.
    /// Those are used in fixed table layout calculation
    fn bubble_widths(&mut self, ctx: &mut LayoutContext) {
        /* find the specified column widths from the first table-row.
           update the number of column widths from other table-rows. */
        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_row());
            if self.col_widths.is_empty() {
                self.col_widths = kid.as_table_row().col_widths.clone();
            } else {
                let num_cols = self.col_widths.len();
                let num_child_cols = kid.as_table_row().col_widths.len();
                for _ in range(num_cols, num_child_cols) {
                    self.col_widths.push(Au::new(0));
                }
            }
        }

        // TODO: calculate min_width & pref_width for automatic table layout calculation
        self.block_flow.bubble_widths(ctx);
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

    /// This is called on kid flows by a parent.
    ///
    /// Hence, we can assume that assign_height has already been called on the
    /// kid (because of the bottom-up traversal).
    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height_inorder: assigning height for table_rowgroup");
        self.assign_height_table_rowgroup_base(ctx, true);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height: assigning height for table_rowgroup");
        self.assign_height_table_rowgroup_base(ctx, false);
    }

    /// TableRowBox and their parents(TableBox) do not have margins.
    /// Therefore, margins to be collapsed do not exist.
    fn collapse_margins(&mut self, _: bool, _: &mut bool, _: &mut Au,
                        _: &mut Au, _: &mut Au, _: &mut Au) {
    }

    fn debug_str(&self) -> ~str {
        let txt = ~"TableRowGroupFlow: ";
        txt.append(match self.block_flow.box_ {
            Some(ref rb) => rb.debug_str(),
            None => ~"",
        })
    }
}

