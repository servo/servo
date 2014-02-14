/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{BaseFlow, TableRowFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow;
use layout::model::{MaybeAuto, Specified, Auto};
use layout::float_context::{FloatContext, Invalid};

use std::cell::RefCell;
use geom::{Point2D, Rect};
use gfx::display_list::DisplayList;
use servo_util::geometry::Au;
use servo_util::geometry;

/// A table formatting context.
pub struct TableRowFlow {
    /// Data common to all flows.
    base: BaseFlow,

    /// The associated box.
    box_: Option<Box>,

    /// Column widths.
    col_widths: ~[Au],
}

impl TableRowFlow {
    pub fn new(base: BaseFlow) -> TableRowFlow {
        TableRowFlow {
            base: base,
            box_: None,
            col_widths: ~[],
        }
    }

    pub fn from_box(base: BaseFlow, box_: Box) -> TableRowFlow {
        TableRowFlow {
            base: base,
            box_: Some(box_),
            col_widths: ~[],
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
    fn assign_height_table_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {
        let mut cur_y = Au::new(0);
        let top_offset = Au::new(0);
        let bottom_offset = Au::new(0);
        let left_offset = Au::new(0);
        let mut float_ctx = Invalid;

        // TODO: If border-collapse: collapse, top_offset, bottom_offset, and left_offset
        // should be updated. Currently, they are set as Au(0).

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

        let mut max_y = Au::new(0);
        for kid in self.base.child_iter() {
            let child_node = flow::mut_base(*kid);
            child_node.position.origin.y = cur_y;
            max_y = geometry::max(max_y, child_node.position.size.height);
        }
        cur_y = cur_y + max_y;

        let mut height = max_y;
        for box_ in self.box_.iter() {
            let style = box_.style();

            // TODO: Percentage height
            height = match MaybeAuto::from_style(style.Box.height, Au(0)) {
                Auto => height,
                Specified(value) => geometry::max(value, height)
            };
        }

        // TODO: Table height should be correctly calculated. Child cells' 'height' property
        // should be considered.

        // Assign the height of own box
        for box_ in self.box_.iter() {
            let mut position = box_.position.get();
            position.size.height = height;
            box_.position.set(position);
        }
        self.base.position.size.height = height;

        // Assign the height of kid boxes, which is the same value as own height.
        for kid in self.base.child_iter() {
            for kid_box_ in kid.as_table_cell().box_.iter() {
                let mut position = kid_box_.position.get();
                position.size.height = height;
                kid_box_.position.set(position);
            }
            let child_node = flow::mut_base(*kid);
            child_node.position.size.height = height;
        }

        if inorder {
            let extra_height = height - (cur_y - top_offset) + bottom_offset;
            self.base.floats_out = float_ctx.translate(Point2D(left_offset, -extra_height));
        } else {
            self.base.floats_out = self.base.floats_in.clone();
        }
    }

    pub fn build_display_list_table<E:ExtraDisplayListData>(
                                    &mut self,
                                    builder: &DisplayListBuilder,
                                    dirty: &Rect<Au>,
                                    list: &RefCell<DisplayList<E>>)
                                    -> bool {
        let abs_rect = Rect(self.base.abs_position, self.base.position.size);
        if !abs_rect.intersects(dirty) {
            return true;
        }

        debug!("build_display_list_table: adding display element");

        // add box that starts table context
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
    fn bubble_widths(&mut self, _: &mut LayoutContext) {
        let mut min_width = Au::new(0);
        let mut pref_width = Au::new(0);
        let mut num_floats = 0;

        /* find the specified widths from child table-cell contexts */
        for kid in self.base.child_iter() {
            assert!(kid.is_table_cell());

            for child_box in kid.as_table_cell().box_.iter() {
                let child_specified_width = MaybeAuto::from_style(child_box.style().Box.width,
                                                                  Au::new(0)).specified_or_zero();
                self.col_widths.push(child_specified_width);
            }

            let child_base = flow::mut_base(*kid);
            min_width = geometry::max(min_width, child_base.min_width);
            pref_width = geometry::max(pref_width, child_base.pref_width);
            num_floats = num_floats + child_base.num_floats;
        }

        self.base.num_floats = num_floats;

        // FIXME: automatic table layout calculation
        for box_ in self.box_.iter() {
            let (this_minimum_width, this_preferred_width) = box_.minimum_and_preferred_widths();
            min_width = min_width + this_minimum_width;
            pref_width = pref_width + this_preferred_width;
        }

        self.base.min_width = min_width;
        self.base.pref_width = pref_width;
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    fn assign_widths(&mut self, _: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow {}", "table_row", self.base.id);

        // The position was set to the containing block by the flow's parent.
        let remaining_width = self.base.position.size.width;
        let mut x_offset = Au::new(0);

        for box_ in self.box_.iter() {
            let style = box_.style();

            // The text alignment of a table_row flow is the text alignment of its box's style.
            self.base.flags_info.flags.set_text_align(style.Text.text_align);

            // In case of border-collapse: collapse, x_offset should be border-left
            x_offset = Au(0);

            let mut position_ref = box_.position.borrow_mut();
            position_ref.get().size.width = remaining_width;
        }

        let has_inorder_children = self.base.flags_info.flags.inorder() || self.base.num_floats > 0;

        // FIXME(ksh8281): avoid copy
        let flags_info = self.base.flags_info.clone();
        let mut i = 0;
        for kid in self.base.child_iter() {
            assert!(kid.is_table_cell());

            let child_base = flow::mut_base(*kid);
            child_base.position.origin.x = x_offset;
            child_base.position.size.width = self.col_widths[i];
            child_base.flags_info.flags.set_inorder(has_inorder_children);

            x_offset = x_offset + self.col_widths[i];
            i += 1;

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
        debug!("assign_height_inorder: assigning height for table_row {}", self.base.id);
        self.assign_height_table_base(ctx, true);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height: assigning height for table_row {}", self.base.id);
        self.assign_height_table_base(ctx, false);
    }

    fn collapse_margins(&mut self, _: bool, _: &mut bool, _: &mut Au,
                        _: &mut Au, _: &mut Au, _: &mut Au) {
    }

    fn debug_str(&self) -> ~str {
        let txt = ~"TableRowFlow: ";
        txt.append(match self.box_ {
            Some(ref rb) => rb.debug_str(),
            None => ~"",
        })
    }
}

