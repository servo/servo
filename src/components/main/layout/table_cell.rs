/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{BaseFlow, TableCellFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow;
use layout::float_context::{FloatContext, Invalid};

use std::cell::RefCell;
use geom::{Point2D, Rect};
use gfx::display_list::DisplayList;
use servo_util::geometry::Au;
use servo_util::geometry;

/// A table formatting context.
pub struct TableCellFlow {
    /// Data common to all flows.
    base: BaseFlow,

    /// The associated box.
    box_: Option<Box>,
}

impl TableCellFlow {
    pub fn new(base: BaseFlow) -> TableCellFlow {
        TableCellFlow {
            base: base,
            box_: None,
        }
    }

    pub fn from_box(base: BaseFlow, box_: Box) -> TableCellFlow {
        TableCellFlow {
            base: base,
            box_: Some(box_),
        }
    }

    pub fn teardown(&mut self) {
        for box_ in self.box_.iter() {
            box_.teardown();
        }
        self.box_ = None;
    }

    // inline(always) because this is only ever called by in-order or non-in-order top-level
    // methods
    #[inline(always)]
    fn assign_height_table_cell_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {
        let mut cur_y = Au::new(0);
        let mut top_offset = Au::new(0);
        let mut bottom_offset = Au::new(0);
        let mut left_offset = Au::new(0);
        let mut float_ctx = Invalid;

        for box_ in self.box_.iter() {
            top_offset = box_.noncontent_top();
            cur_y = cur_y + top_offset;
            bottom_offset = box_.noncontent_bottom();
            left_offset = box_.noncontent_left();
        }

        if inorder {
            // Floats for blocks work like this:
            // self.floats_in -> child[0].floats_in
            // visit child[0]
            // child[i-1].floats_out -> child[i].floats_in
            // visit child[i]
            // repeat until all children are visited.
            // last_child.floats_out -> self.floats_out (done at the end of this method)
            float_ctx = self.base.floats_in.translate(Point2D(-left_offset, -top_offset));
            for kid in self.base.child_iter() {
                flow::mut_base(*kid).floats_in = float_ctx.clone();
                kid.assign_height_inorder(ctx);
                float_ctx = flow::mut_base(*kid).floats_out.clone();
            }
        }

        let mut collapsible = Au::new(0);
        let mut collapsing = Au::new(0);
        let mut first_in_flow = true;

        for kid in self.base.child_iter() {
            // Since table cell does not have `margin`, the first child's top margin and
            // the last child's bottom margin do not collapse.
            kid.collapse_margins(false,
                                 &mut first_in_flow,
                                 &mut Au(0),
                                 &mut top_offset,
                                 &mut collapsing,
                                 &mut collapsible);

            let child_node = flow::mut_base(*kid);
            cur_y = cur_y - collapsing;
            child_node.position.origin.y = cur_y;
            cur_y = cur_y + child_node.position.size.height;
        }

        // CSS 2.1 § 17.5.3. Table cell box height is the minimum height required by the content.
        let height = cur_y - top_offset;

        // TODO(june0cho): vertical-align of table-cell should be calculated.
        let mut noncontent_height = Au::new(0);
        for box_ in self.box_.iter() {
            let mut position = box_.position.get();

            noncontent_height = box_.noncontent_height();

            position.origin.y = Au(0);
            position.size.height = height + noncontent_height;

            box_.position.set(position);
        }

        self.base.position.size.height = height + noncontent_height;

        if inorder {
            let extra_height = height - (cur_y - top_offset) + bottom_offset;
            self.base.floats_out = float_ctx.translate(Point2D(left_offset, -extra_height));
        } else {
            self.base.floats_out = self.base.floats_in.clone();
        }
    }

    pub fn build_display_list_table_cell<E:ExtraDisplayListData>(
                                    &mut self,
                                    builder: &DisplayListBuilder,
                                    dirty: &Rect<Au>,
                                    list: &RefCell<DisplayList<E>>)
                                    -> bool {
        let abs_rect = Rect(self.base.abs_position, self.base.position.size);
        if !abs_rect.intersects(dirty) {
            return true;
        }

        debug!("build_display_list_table_cell: adding display element");

        for box_ in self.box_.iter() {
            box_.build_display_list(builder, dirty, self.base.abs_position, (&*self) as &Flow, list)
        }
        // TODO: handle any out-of-flow elements
        let this_position = self.base.abs_position;

        for child in self.base.child_iter() {
            let child_base = flow::mut_base(*child);
            child_base.abs_position = this_position + child_base.position.origin;
        }

        false
    }
}

impl Flow for TableCellFlow {
    fn class(&self) -> FlowClass {
        TableCellFlowClass
    }

    fn as_table_cell<'a>(&'a mut self) -> &'a mut TableCellFlow {
        self
    }

    /// Recursively (bottom-up) determines the context's preferred and minimum widths. When called
    /// on this context, all child contexts have had their min/pref widths set. This function must
    /// decide min/pref widths based on child context widths and dimensions of any boxes it is
    /// responsible for flowing.
    /// Min/pref widths set by this function are used in automatic table layout calculation.
    fn bubble_widths(&mut self, _: &mut LayoutContext) {
        let mut min_width = Au::new(0);
        let mut pref_width = Au::new(0);
        let mut num_floats = 0;

        /* find max width from child block contexts */
        for kid in self.base.child_iter() {
            assert!(kid.starts_block_flow() || kid.starts_inline_flow() || kid.is_table_kind());

            let child_base = flow::mut_base(*kid);
            min_width = geometry::max(min_width, child_base.min_width);
            pref_width = geometry::max(pref_width, child_base.pref_width);
            num_floats = num_floats + child_base.num_floats;
        }

        self.base.num_floats = num_floats;

        for box_ in self.box_.iter() {
            {
                // Can compute border width here since it doesn't depend on anything.
                box_.compute_borders(box_.style())
            }

            let (this_minimum_width, this_preferred_width) = box_.minimum_and_preferred_widths();
            min_width = min_width + this_minimum_width;
            pref_width = pref_width + this_preferred_width;
        }

        self.base.min_width = min_width;
        self.base.pref_width = pref_width;
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent table row.
    fn assign_widths(&mut self, _: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow {}", "table_cell", self.base.id);

        // The position was set to the column width by the parent flow, table row flow.
        let mut remaining_width = self.base.position.size.width;
        let mut x_offset = Au::new(0);

        for box_ in self.box_.iter() {
            let style = box_.style();

            // The text alignment of a table_cell flow is the text alignment of its box's style.
            self.base.flags_info.flags.set_text_align(style.Text.text_align);

            // Can compute padding here since we know containing block width.
            box_.compute_padding(style, remaining_width);

            x_offset = box_.padding.get().left + box_.border.get().left;

            // The associated box is the border box of this flow.
            let mut position_ref = box_.position.borrow_mut();
            position_ref.get().size.width = remaining_width;

            let padding_and_borders = box_.padding.get().left + box_.padding.get().right +
                                      box_.border.get().left + box_.border.get().right;
            remaining_width = remaining_width - padding_and_borders;
        }

        let has_inorder_children = self.base.flags_info.flags.inorder() || self.base.num_floats > 0;

        // FIXME(ksh8281): avoid copy
        let flags_info = self.base.flags_info.clone();
        for kid in self.base.child_iter() {
            assert!(kid.starts_block_flow() || kid.starts_inline_flow() || kid.is_table_kind());

            let child_base = flow::mut_base(*kid);
            child_base.position.origin.x = x_offset;
            child_base.position.size.width = remaining_width;
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

    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height_inorder: assigning height for table_cell {}", self.base.id);
        self.assign_height_table_cell_base(ctx, true);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height: assigning height for table_cell {}", self.base.id);
        self.assign_height_table_cell_base(ctx, false);
    }

    /// TableCellBox and their parents(TableRowBox) do not have margins.
    /// Therefore, margins to be collapsed do not exist.
    fn collapse_margins(&mut self, _: bool, _: &mut bool, _: &mut Au,
                        _: &mut Au, _: &mut Au, _: &mut Au) {
    }

    fn debug_str(&self) -> ~str {
        let txt = ~"TableCellFlow: ";
        txt.append(match self.box_ {
            Some(ref rb) => rb.debug_str(),
            None => ~"",
        })
    }
}

