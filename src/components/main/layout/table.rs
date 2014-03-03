/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{BaseFlow, TableFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow;
use layout::float_context::{FloatContext, Invalid};
use layout::table_wrapper::{TableLayout, FixedLayout, AutoLayout};

use std::cell::RefCell;
use style::computed_values::table_layout;
use geom::{Point2D, Rect};
use gfx::display_list::DisplayList;
use servo_util::geometry::Au;
use servo_util::geometry;

/// A table flow corresponded to the table's internal table box under a table wrapper flow.
/// The properties `position`, `float`, and `margin-*` are used on the table wrapper box,
/// not table box per CSS 2.1 § 10.5.
pub struct TableFlow {
    /// Data common to all flows.
    base: BaseFlow,

    /// The associated box.
    box_: Option<Box>,

    /// Column widths
    col_widths: ~[Au],

    /// Table-layout property
    table_layout: TableLayout,
}

impl TableFlow {
    pub fn new(base: BaseFlow) -> TableFlow {
        TableFlow {
            base: base,
            box_: None,
            col_widths: ~[],
            table_layout: AutoLayout,
        }
    }

    pub fn from_box(base: BaseFlow, box_: Box) -> TableFlow {
        let table_layout = if (box_.style().Table.table_layout == table_layout::fixed) {
            FixedLayout
        } else {
            AutoLayout
        };
        TableFlow {
            base: base,
            box_: Some(box_),
            col_widths: ~[],
            table_layout: table_layout,
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

        for kid in self.base.child_iter() {
            let child_node = flow::mut_base(*kid);
            child_node.position.origin.y = cur_y;
            cur_y = cur_y + child_node.position.size.height;
        }

        let height = cur_y - top_offset;

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

        debug!("build_display_list_table[TABLE]: adding display element");

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

impl Flow for TableFlow {
    fn class(&self) -> FlowClass {
        TableFlowClass
    }

    fn as_table<'a>(&'a mut self) -> &'a mut TableFlow {
        self
    }

    /// This function finds the specified column widths from column group and the first row.
    /// Those are used in fixed table layout calculation.
    /* FIXME: automatic table layout calculation */
    fn bubble_widths(&mut self, _: &mut LayoutContext) {
        let mut min_width = Au::new(0);
        let mut pref_width = Au::new(0);
        let mut num_floats = 0;
        let mut did_first_row = false;

        /* find max width from child block contexts */
        for kid in self.base.child_iter() {
            assert!(kid.is_proper_table_child());

            if kid.is_table_colgroup() {
                self.col_widths.push_all(kid.as_table_colgroup().widths);
            } else if kid.is_table_rowgroup() || kid.is_table_row() {
                // read column widths from table-row-group/table-row, and assign
                // width=0 for the columns not defined in column-group
                // FIXME: need to read widths from either table-header-group OR
                // first table-row
                let kid_col_widths = if kid.is_table_rowgroup() {
                    &kid.as_table_rowgroup().col_widths
                } else {
                    &kid.as_table_row().col_widths
                };
                match self.table_layout {
                    FixedLayout if !did_first_row => {
                        did_first_row = true;
                        let mut child_widths = kid_col_widths.iter();
                        for col_width in self.col_widths.mut_iter() {
                            match child_widths.next() {
                                Some(child_width) => {
                                    if *col_width == Au::new(0) {
                                        *col_width = *child_width;
                                    }
                                },
                                None => break
                            }
                        }
                    },
                    _ => {}
                }
                /*
                if self.table_layout == FixedLayout && !did_first_row {
                    did_first_row = true;
                    let mut child_widths = kid_col_widths.iter();
                    for col_width in self.col_widths.mut_iter() {
                        match child_widths.next() {
                            Some(child_width) => {
                                if *col_width == Au::new(0) {
                                    *col_width = *child_width;
                                }
                            },
                            None => break
                        }
                    }
                }*/
                let num_child_cols = kid_col_widths.len();
                let num_cols = self.col_widths.len();
                debug!("colgroup has {} column(s) and child has {} column(s)", num_cols, num_child_cols);
                for i in range(num_cols, num_child_cols) {
                    self.col_widths.push( kid_col_widths[i] );
                }
            }

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
    /// on this context, the context has had its width set by the parent context.
    fn assign_widths(&mut self, _: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow {}", "table", self.base.id);

        // The position was set to the containing block by the flow's parent.
        let mut remaining_width = self.base.position.size.width;
        let mut x_offset = Au::new(0);

        let mut num_unspecified_widths = 0;
        let mut total_columns_widths = Au::new(0);
        for col_width in self.col_widths.iter() {
            if *col_width == Au::new(0) {
                num_unspecified_widths += 1;
            } else {
                total_columns_widths = total_columns_widths.add(col_width);
            }
        }

        let mut padding_and_borders = Au::new(0);

        for box_ in self.box_.iter() {
            let style = box_.style();

            // The text alignment of a table_wrapper flow is the text alignment of its box's style.
            self.base.flags_info.flags.set_text_align(style.Text.text_align);

            // Can compute padding here since we know containing block width.
            box_.compute_padding(style, remaining_width);

            x_offset = box_.padding.get().left + box_.border.get().left;
            padding_and_borders = box_.padding.get().left + box_.padding.get().right +
                                  box_.border.get().left + box_.border.get().right;

            let mut position_ref = box_.position.borrow_mut();
            position_ref.get().size.width = remaining_width;
        }

        remaining_width = remaining_width - padding_and_borders;

        // In fixed table layout, we distribute extra space among the unspecified columns if there are
        // any, or among all the columns if all are specified.
        let extra_column_width = if (total_columns_widths < remaining_width) &&
                                    (num_unspecified_widths == 0) {
            let ratio = remaining_width.to_f64().unwrap() / total_columns_widths.to_f64().unwrap();
            for col_width in self.col_widths.mut_iter() {
                *col_width = (*col_width).scale_by(ratio);
            }
            Au(0)
        } else if num_unspecified_widths != 0 {
            (remaining_width - total_columns_widths) / Au::new(num_unspecified_widths)
        } else {
            Au(0)
        };

        let has_inorder_children = self.base.flags_info.flags.inorder() || self.base.num_floats > 0;

        // FIXME(ksh8281): avoid copy
        let flags_info = self.base.flags_info.clone();
        for kid in self.base.child_iter() {
            assert!(kid.is_proper_table_child());

            if kid.is_table_colgroup() {
                continue;
            }
            let final_col_widths = self.col_widths.map(|width| {
                if *width == Au(0) {
                    extra_column_width
                } else {
                    *width
                }
            });
            if kid.is_table_rowgroup() {
                kid.as_table_rowgroup().col_widths = final_col_widths;
            } else if kid.is_table_row() {
                kid.as_table_row().col_widths = final_col_widths;
            }

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
        debug!("assign_height_inorder: assigning height for table {}", self.base.id);
        self.assign_height_table_base(ctx, true);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height: assigning height for table {}", self.base.id);
        self.assign_height_table_base(ctx, false);
    }

    fn collapse_margins(&mut self,
                        _: bool,
                        _: &mut bool,
                        _: &mut Au,
                        _: &mut Au,
                        collapsing: &mut Au,
                        collapsible: &mut Au) {
        // `margin` is not used on table box.
        *collapsing = Au::new(0);
        *collapsible = Au::new(0);
    }

    fn debug_str(&self) -> ~str {
        let txt = ~"TableFlow: ";
        txt.append(match self.box_ {
            Some(ref rb) => rb.debug_str(),
            None => ~"",
        })
    }
}

