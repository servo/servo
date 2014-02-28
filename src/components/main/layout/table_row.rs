/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::block::BlockFlow;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{BaseFlow, TableRowFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow;
use layout::model::{MaybeAuto, Specified, Auto};
use layout::float_context::{FloatContext};

use std::cell::RefCell;
use geom::{Point2D, Rect};
use gfx::display_list::DisplayList;
use servo_util::geometry::Au;
use servo_util::geometry;

/// A table formatting context.
pub struct TableRowFlow {
    block_flow: BlockFlow,

    /// Column widths.
    col_widths: ~[Au],
}

impl TableRowFlow {
    pub fn new(base: BaseFlow) -> TableRowFlow {
        TableRowFlow {
            block_flow: BlockFlow::new(base),
            col_widths: ~[],
        }
    }

    pub fn from_box(base: BaseFlow, box_: Box) -> TableRowFlow {
        TableRowFlow {
            block_flow: BlockFlow::from_box(base, box_, false),
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

    fn initialize_offset(&mut self) -> (Au, Au, Au) {
        // TODO: If border-collapse: collapse, top_offset, bottom_offset, and left_offset
        // should be updated. Currently, they are set as Au(0).
        (Au(0), Au(0), Au(0))
    }

    // inline(always) because this is only ever called by in-order or non-in-order top-level
    // methods
    #[inline(always)]
    fn assign_height_table_row_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {
        let (top_offset, bottom_offset, left_offset) = self.initialize_offset();

        let mut float_ctx = self.block_flow.handle_children_floats_if_inorder(ctx, Point2D(-left_offset, -top_offset), inorder);
        let mut cur_y = top_offset;

        // Per CSS 2.1 § 17.5.3, find max_y = max( computed `height`, minimum height of all cells )
        let mut max_y = Au::new(0);
        for kid in self.block_flow.base.child_iter() {
            for child_box in kid.as_table_cell().box_().iter() {
                // TODO: Percentage height
                let child_specified_height = MaybeAuto::from_style(child_box.style().Box.height,
                                                                   Au::new(0)).specified_or_zero();
                max_y = geometry::max(max_y, child_specified_height + child_box.noncontent_height());
            }
            let child_node = flow::mut_base(*kid);
            child_node.position.origin.y = cur_y;
            max_y = geometry::max(max_y, child_node.position.size.height);
        }
        cur_y = cur_y + max_y;

        let mut height = max_y;
        for box_ in self.block_flow.box_.iter() {
            // TODO: Percentage height
            height = match MaybeAuto::from_style(box_.style().Box.height, Au(0)) {
                Auto => height,
                Specified(value) => geometry::max(value, height)
            };
        }

        // Assign the height of own box
        for box_ in self.block_flow.box_.iter() {
            let mut position = box_.position.get();
            position.size.height = height;
            box_.position.set(position);
        }
        self.block_flow.base.position.size.height = height;

        // Assign the height of kid boxes, which is the same value as own height.
        for kid in self.block_flow.base.child_iter() {
            for kid_box_ in kid.as_table_cell().box_().iter() {
                let mut position = kid_box_.position.get();
                position.size.height = height;
                kid_box_.position.set(position);
            }
            let child_node = flow::mut_base(*kid);
            child_node.position.size.height = height;
        }

        self.block_flow.set_floats_out(&mut float_ctx, height, cur_y, top_offset,
                                       bottom_offset, left_offset, inorder);
    }

    pub fn propagate_assigned_width_to_children(&mut self, x_offset: Au) {
        let mut x_offset = x_offset;
        let has_inorder_children = self.block_flow.base.flags_info.flags.inorder() || 
                                   self.block_flow.base.num_floats > 0;

        // FIXME(ksh8281): avoid copy
        let flags_info = self.block_flow.base.flags_info.clone();
        for (i, kid) in self.block_flow.base.child_iter().enumerate() {
            assert!(kid.is_table_cell());

            let child_base = flow::mut_base(*kid);
            child_base.position.origin.x = x_offset;
            child_base.position.size.width = self.col_widths[i];
            x_offset = x_offset + self.col_widths[i];
            child_base.flags_info.flags.set_inorder(has_inorder_children);

            if !child_base.flags_info.flags.inorder() {
                child_base.floats_in = FloatContext::new(0);
            }

            // Per CSS 2.1 § 16.3.1, text decoration propagates to all children in flow.
            //
            // TODO(pcwalton): When we have out-of-flow children, don't unconditionally propagate.
            child_base.flags_info.propagate_text_decoration_from_parent(&flags_info);
            child_base.flags_info.propagate_text_alignment_from_parent(&flags_info)
        }
    }

    pub fn build_display_list_table_row<E:ExtraDisplayListData>(
                                        &mut self,
                                        builder: &DisplayListBuilder,
                                        dirty: &Rect<Au>,
                                        list: &RefCell<DisplayList<E>>)
                                        -> bool {
        debug!("build_display_list_table_row: same process as block flow");
        self.block_flow.build_display_list_block(builder, dirty, list)
    }
}

impl Flow for TableRowFlow {
    fn class(&self) -> FlowClass {
        TableRowFlowClass
    }

    fn as_table_row<'a>(&'a mut self) -> &'a mut TableRowFlow {
        self
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
                let child_specified_width = MaybeAuto::from_style(child_box.style().Box.width,
                                                                  Au::new(0)).specified_or_zero();
                self.col_widths.push(child_specified_width);
            }
        }

        // TODO: calculate min_width & pref_width for automatic table layout calculation
        self.block_flow.bubble_widths(ctx);
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    fn assign_widths(&mut self, _: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow {}", "table_row", self.block_flow.base.id);

        // The position was set to the containing block by the flow's parent.
        let remaining_width = self.block_flow.base.position.size.width;
        // In case of border-collapse: collapse, x_offset should be border-left
        let x_offset = Au::new(0);

        for box_ in self.block_flow.box_.iter() {
            let style = box_.style();

            // The text alignment of a table_row flow is the text alignment of its box's style.
            self.block_flow.base.flags_info.flags.set_text_align(style.Text.text_align);
            self.block_flow.initial_box_setting(box_, style, remaining_width, false, false);
            self.block_flow.set_box_x_and_width(box_, Au(0), remaining_width);
        }

        self.propagate_assigned_width_to_children(x_offset);
    }

    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height_inorder: assigning height for table_row {}", self.block_flow.base.id);
        self.assign_height_table_row_base(ctx, true);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height: assigning height for table_row {}", self.block_flow.base.id);
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

