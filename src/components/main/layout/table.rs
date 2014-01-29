/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{BaseFlow, TableFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow;
use layout::model::{MaybeAuto, Specified, Auto};
use layout::float_context::{FloatContext, PlacementInfo, Invalid, FloatType};

use std::cell::RefCell;
use style::computed_values::table_layout;
use geom::{Point2D, Rect};
use gfx::display_list::DisplayList;
use servo_util::geometry::Au;
use servo_util::geometry;

/// Information specific to floated blocks.
pub struct FloatedTableInfo {
    containing_width: Au,

    /// Offset relative to where the parent tried to position this flow
    rel_pos: Point2D<Au>,

    /// Index into the box list for inline floats
    index: Option<uint>,

    /// Number of floated children
    floated_children: uint,

    /// Left or right?
    float_type: FloatType
}

impl FloatedTableInfo {
    pub fn new(float_type: FloatType) -> FloatedTableInfo {
        FloatedTableInfo {
            containing_width: Au(0),
            rel_pos: Point2D(Au(0), Au(0)),
            index: None,
            floated_children: 0,
            float_type: float_type
        }
    }
}

/// A table formatting context.
pub struct TableFlow {
    /// Data common to all flows.
    base: BaseFlow,

    /// The associated box.
    box_: Option<Box>,

    //TODO: is_fixed should be bit fields to conserve memory.
    /// Position property
    is_fixed: bool,

    /// Additional floating flow members.
    float: Option<~FloatedTableInfo>,

    /// Column widths
    col_widths: ~[Au],

    /// Table-layout property
    is_fixed_table_layout: bool,
}

impl TableFlow {
    pub fn new(base: BaseFlow) -> TableFlow {
        TableFlow {
            base: base,
            box_: None,
            is_fixed: false,
            float: None,
            col_widths: ~[],
            is_fixed_table_layout: false,

        }
    }

    pub fn from_box(base: BaseFlow, box_: Box, is_fixed: bool) -> TableFlow {
        let is_fixed_table_layout = box_.style().Table.table_layout == table_layout::fixed;
        TableFlow {
            base: base,
            box_: Some(box_),
            is_fixed: is_fixed,
            float: None,
            col_widths: ~[],
            is_fixed_table_layout: is_fixed_table_layout,
        }
    }

    pub fn float_from_box(base: BaseFlow, float_type: FloatType, box_: Box) -> TableFlow {
        TableFlow {
            base: base,
            box_: Some(box_),
            is_fixed: false,
            float: Some(~FloatedTableInfo::new(float_type)),
            col_widths: ~[],
            is_fixed_table_layout: false,
        }
    }

    pub fn new_float(base: BaseFlow, float_type: FloatType) -> TableFlow {
        TableFlow {
            base: base,
            box_: None,
            is_fixed: false,
            float: Some(~FloatedTableInfo::new(float_type)),
            col_widths: ~[],
            is_fixed_table_layout: false,
        }
    }

    pub fn is_float(&self) -> bool {
        self.float.is_some()
    }

    pub fn teardown(&mut self) {
        for box_ in self.box_.iter() {
            box_.teardown();
        }
        self.box_ = None;
        self.float = None;
    }

    // inline(always) because this is only ever called by in-order or non-in-order top-level
    // methods
    #[inline(always)]
    fn assign_height_table_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {
        let mut cur_y = Au::new(0);
        let mut clearance = Au::new(0);
        let mut top_offset = Au::new(0);
        let mut bottom_offset = Au::new(0);
        let mut left_offset = Au::new(0);
        let mut float_ctx = Invalid;

        for box_ in self.box_.iter() {
            clearance = match box_.clear() {
                None => Au::new(0),
                Some(clear) => {
                    self.base.floats_in.clearance(clear)
                }
            };

            top_offset = clearance + box_.margin.get().top + box_.border.get().top +
                box_.padding.get().top;
            cur_y = cur_y + top_offset;
            bottom_offset = box_.margin.get().bottom + box_.border.get().bottom +
                box_.padding.get().bottom;
            left_offset = box_.offset();
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
        let mut margin_top = Au::new(0);
        let mut margin_bottom = Au::new(0);
        let mut top_margin_collapsible = false;
        let mut bottom_margin_collapsible = false;
        let mut first_in_flow = true;
        for box_ in self.box_.iter() {
            if box_.border.get().top == Au(0) && box_.padding.get().top == Au(0) {
                collapsible = box_.margin.get().top;
                top_margin_collapsible = true;
            }
            if box_.border.get().bottom == Au(0) &&
                    box_.padding.get().bottom == Au(0) {
                bottom_margin_collapsible = true;
            }
            margin_top = box_.margin.get().top;
            margin_bottom = box_.margin.get().bottom;
        }

        for kid in self.base.child_iter() {
            kid.collapse_margins(top_margin_collapsible,
                                 &mut first_in_flow,
                                 &mut margin_top,
                                 &mut top_offset,
                                 &mut collapsing,
                                 &mut collapsible);

            let child_node = flow::mut_base(*kid);
            cur_y = cur_y - collapsing;
            child_node.position.origin.y = cur_y;
            cur_y = cur_y + child_node.position.size.height;
        }

        // The bottom margin collapses with its last in-flow block-level child's bottom margin
        // if the parent has no bottom boder, no bottom padding.
        collapsing = if bottom_margin_collapsible {
            if margin_bottom < collapsible {
                margin_bottom = collapsible;
            }
            collapsible
        } else {
            Au::new(0)
        };

        // TODO: A box's own margins collapse if the 'min-height' property is zero, and it has neither
        // top or bottom borders nor top or bottom padding, and it has a 'height' of either 0 or 'auto',
        // and it does not contain a line box, and all of its in-flow children's margins (if any) collapse.


        let mut height = cur_y - top_offset - collapsing;

        for box_ in self.box_.iter() {
            let style = box_.style();

            // At this point, `height` is the height of the containing block, so passing `height`
            // as the second argument here effectively makes percentages relative to the containing
            // block per CSS 2.1 § 10.5.
            height = match MaybeAuto::from_style(style.Box.height, height) {
                Auto => height,
                Specified(value) => value
            };
        }

        let mut noncontent_height = Au::new(0);
        let screen_height = ctx.screen_size.height;
        for box_ in self.box_.iter() {
            let mut position = box_.position.get();
            let mut margin = box_.margin.get();

            // The associated box is the border box of this flow.
            margin.top = margin_top;
            margin.bottom = margin_bottom;

            noncontent_height = box_.padding.get().top + box_.padding.get().bottom +
                box_.border.get().top + box_.border.get().bottom;

            let (y, h) = box_.get_y_coord_and_new_height_if_fixed(screen_height,
                                                                  height,
                                                                  clearance + margin.top,
                                                                  self.is_fixed);
            position.origin.y = y;
            height = h;

            if self.is_fixed {
                for kid in self.base.child_iter() {
                    let child_node = flow::mut_base(*kid);
                    child_node.position.origin.y = position.origin.y + top_offset;
                }
            }

            position.size.height = if self.is_fixed {
                height
            } else {
                height + noncontent_height
            };

            noncontent_height = noncontent_height + clearance + margin.top + margin.bottom;

            box_.position.set(position);
            box_.margin.set(margin);
        }

        self.base.position.size.height = if self.is_fixed {
            height
        } else {
            height + noncontent_height
        };

        if inorder {
            let extra_height = height - (cur_y - top_offset) + bottom_offset;
            self.base.floats_out = float_ctx.translate(Point2D(left_offset, -extra_height));
        } else {
            self.base.floats_out = self.base.floats_in.clone();
        }
    }

    fn assign_height_float_inorder(&mut self) {
        // assign_height_float was already called by the traversal function
        // so this is well-defined

        let mut height = Au(0);
        let mut clearance = Au(0);
        let mut full_noncontent_width = Au(0);
        let mut margin_height = Au(0);

        for box_ in self.box_.iter() {
            height = box_.position.get().size.height;
            clearance = match box_.clear() {
                None => Au(0),
                Some(clear) => self.base.floats_in.clearance(clear),
            };

            let noncontent_width = box_.padding.get().left + box_.padding.get().right +
                box_.border.get().left + box_.border.get().right;

            full_noncontent_width = noncontent_width + box_.margin.get().left +
                box_.margin.get().right;
            margin_height = box_.margin.get().top + box_.margin.get().bottom;
        }

        let info = PlacementInfo {
            width: self.base.position.size.width + full_noncontent_width,
            height: height + margin_height,
            ceiling: clearance,
            max_width: self.float.get_ref().containing_width,
            f_type: self.float.get_ref().float_type,
        };

        // Place the float and return the FloatContext back to the parent flow.
        // After, grab the position and use that to set our position.
        self.base.floats_out = self.base.floats_in.add_float(&info);
        self.float.get_mut_ref().rel_pos = self.base.floats_out.last_float_pos();
    }

    fn assign_height_float(&mut self, ctx: &mut LayoutContext) {
        // Now that we've determined our height, propagate that out.
        let has_inorder_children = self.base.num_floats > 0;
        if has_inorder_children {
            let mut float_ctx = FloatContext::new(self.float.get_ref().floated_children);
            for kid in self.base.child_iter() {
                flow::mut_base(*kid).floats_in = float_ctx.clone();
                kid.assign_height_inorder(ctx);
                float_ctx = flow::mut_base(*kid).floats_out.clone();
            }
        }
        let mut cur_y = Au(0);
        let mut top_offset = Au(0);

        for box_ in self.box_.iter() {
            top_offset = box_.margin.get().top + box_.border.get().top + box_.padding.get().top;
            cur_y = cur_y + top_offset;
        }

        for kid in self.base.child_iter() {
            let child_base = flow::mut_base(*kid);
            child_base.position.origin.y = cur_y;
            cur_y = cur_y + child_base.position.size.height;
        }

        let mut height = cur_y - top_offset;

        let mut noncontent_height;
        let box_ = self.box_.as_ref().unwrap();
        let mut position = box_.position.get();

        // The associated box is the border box of this flow.
        position.origin.y = box_.margin.get().top;

        noncontent_height = box_.padding.get().top + box_.padding.get().bottom +
            box_.border.get().top + box_.border.get().bottom;

        //TODO(eatkinson): compute heights properly using the 'height' property.
        let height_prop = MaybeAuto::from_style(box_.style().Box.height,
                                                Au::new(0)).specified_or_zero();

        height = geometry::max(height, height_prop) + noncontent_height;
        debug!("assign_height_float -- height: {}", height);

        position.size.height = height;
        box_.position.set(position);
    }

    pub fn build_display_list_table<E:ExtraDisplayListData>(
                                    &mut self,
                                    builder: &DisplayListBuilder,
                                    dirty: &Rect<Au>,
                                    list: &RefCell<DisplayList<E>>)
                                    -> bool {
        if self.is_float() {
            return self.build_display_list_float(builder, dirty, list);
        }

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

    pub fn build_display_list_float<E:ExtraDisplayListData>(
                                    &mut self,
                                    builder: &DisplayListBuilder,
                                    dirty: &Rect<Au>,
                                    list: &RefCell<DisplayList<E>>)
                                    -> bool {
        let abs_rect = Rect(self.base.abs_position, self.base.position.size);
        if !abs_rect.intersects(dirty) {
            return true
        }

        let offset = self.base.abs_position + self.float.get_ref().rel_pos;
        // add box that starts table context
        for box_ in self.box_.iter() {
            box_.build_display_list(builder, dirty, offset, (&*self) as &Flow, list)
        }


        // TODO: handle any out-of-flow elements

        // go deeper into the flow tree
        for child in self.base.child_iter() {
            let child_base = flow::mut_base(*child);
            child_base.abs_position = offset + child_base.position.origin;
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

    /* Recursively (bottom-up) determine the context's preferred and
    minimum widths.  When called on this context, all child contexts
    have had their min/pref widths set. This function must decide
    min/pref widths based on child context widths and dimensions of
    any boxes it is responsible for flowing.  */

    /* TODO: absolute contexts */
    /* TODO: inline-blocks */
    fn bubble_widths(&mut self, _: &mut LayoutContext) {
        let mut min_width = Au::new(0);
        let mut pref_width = Au::new(0);
        let mut num_floats = 0;

        /* find max width from child block contexts */
        for kid in self.base.child_iter() {
            assert!(kid.starts_block_flow() || kid.starts_table_flow());

            if kid.is_table_colgroup() {
                self.col_widths.push_all(kid.as_table_colgroup().widths);
            } else if kid.is_table_rowgroup() {
                // read column widths from table-row-group, and assign
                // width=0 for the columns not defined in column-group
                // FIXME: need to read widths from either table-header-group OR
                // first table-row
                if self.is_fixed_table_layout {
                    let mut child_widths = kid.as_table_rowgroup().col_widths.mut_iter();
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
                }
                let num_child_cols = kid.as_table_rowgroup().col_widths.len();
                let num_cols = self.col_widths.len();
                debug!("{:?} column(s) from colgroup, but the child has {:?} column(s)", num_cols, num_child_cols);
                for i in range(num_cols, num_child_cols) {
                    self.col_widths.push( kid.as_table_rowgroup().col_widths[i] );
                }
            }

            let child_base = flow::mut_base(*kid);
            min_width = geometry::max(min_width, child_base.min_width);
            pref_width = geometry::max(pref_width, child_base.pref_width);
            num_floats = num_floats + child_base.num_floats;
        }

        if self.is_float() {
            self.base.num_floats = 1;
            self.float.get_mut_ref().floated_children = num_floats;
        } else {
            self.base.num_floats = num_floats;
        }

        /* if not an anonymous block context, add in block box's widths.
           these widths will not include child elements, just padding etc. */
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
    ///
    /// Dual boxes consume some width first, and the remainder is assigned to all child (block)
    /// contexts.
    fn assign_widths(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow {}",
               if self.is_float() {
                   "float"
               } else {
                   "table"
               },
               self.base.id);

        // The position was set to the containing block by the flow's parent.
        let mut remaining_width = self.base.position.size.width;
        let mut x_offset = Au::new(0);

        let mut num_unspecified_widths = 0;
        let mut total_cell_widths = Au::new(0);
        for col_width in self.col_widths.iter() {
            if *col_width == Au::new(0) {
                num_unspecified_widths += 1;
            } else {
                total_cell_widths = total_cell_widths.add(col_width);
            }
        }

        if self.is_float() {
            self.float.get_mut_ref().containing_width = remaining_width;

            // Parent usually sets this, but floats are never inorder
            self.base.flags_info.flags.set_inorder(false);
        }

        let mut padding_and_borders = Au::new(0);

        for box_ in self.box_.iter() {
            let style = box_.style();

            // The text alignment of a table_wrapper flow is the text alignment of its box's style.
            self.base.flags_info.flags.set_text_align(style.Text.text_align);

            // Can compute padding here since we know containing block width.
            box_.compute_padding(style, remaining_width);

            let screen_size = ctx.screen_size;
            let (x, _w) = box_.get_x_coord_and_new_width_if_fixed(screen_size.width,
                                                                  screen_size.height,
                                                                  remaining_width,
                                                                  box_.offset(),
                                                                  self.is_fixed);
            x_offset = x;
            padding_and_borders = box_.padding.get().left + box_.padding.get().right +
                                  box_.border.get().left + box_.border.get().right;

            // The associated box is the border box of this flow.
            let mut position_ref = box_.position.borrow_mut();
            if self.is_fixed {
                position_ref.get().origin.x = x_offset;
                x_offset = x_offset + box_.padding.get().left;
            }

            position_ref.get().size.width = remaining_width;
        }

        if self.is_float() {
            self.base.position.size.width = remaining_width;
        }

        remaining_width = remaining_width - padding_and_borders;

        let final_cell_width = if (total_cell_widths < remaining_width) &&
                                    (num_unspecified_widths == 0) {
            for col_width in self.col_widths.mut_iter() {
                *col_width = *col_width * remaining_width / total_cell_widths;
            }
            Au(0)
        } else if num_unspecified_widths != 0 {
            (remaining_width - total_cell_widths) / Au::new(num_unspecified_widths)
        } else {
            Au(0)
        };

        let has_inorder_children = if self.is_float() {
            self.base.num_floats > 0
        } else {
            self.base.flags_info.flags.inorder() || self.base.num_floats > 0
        };

        // FIXME(ksh8281): avoid copy
        let flags_info = self.base.flags_info.clone();
        for kid in self.base.child_iter() {
            assert!(kid.starts_block_flow() || kid.starts_table_flow());

            if kid.is_table_rowgroup() {
                kid.as_table_rowgroup().col_widths = self.col_widths.map(|width| {
                    if *width == Au(0) {
                        final_cell_width
                    } else {
                        *width
                    }
                });
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
        if self.is_float() {
            debug!("assign_height_inorder_float: assigning height for float {}", self.base.id);
            self.assign_height_float_inorder();
        } else {
            debug!("assign_height_inorder: assigning height for table {}", self.base.id);
            self.assign_height_table_base(ctx, true);
        }
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        if self.is_float() {
            debug!("assign_height_float: assigning height for float {}", self.base.id);
            self.assign_height_float(ctx);
        } else {
            debug!("assign_height: assigning height for table {}", self.base.id);
            self.assign_height_table_base(ctx, false);
        }
    }

    fn collapse_margins(&mut self,
                        top_margin_collapsible: bool,
                        first_in_flow: &mut bool,
                        margin_top: &mut Au,
                        top_offset: &mut Au,
                        collapsing: &mut Au,
                        collapsible: &mut Au) {
        if self.is_float() {
            // Margins between a floated box and any other box do not collapse.
            *collapsing = Au::new(0);
            return;
        }

        for box_ in self.box_.iter() {
            // The top margin collapses with its first in-flow block-level child's
            // top margin if the parent has no top border, no top padding.
            if *first_in_flow && top_margin_collapsible {
                // If top-margin of parent is less than top-margin of its first child,
                // the parent box goes down until its top is aligned with the child.
                if *margin_top < box_.margin.get().top {
                    // TODO: The position of child floats should be updated and this
                    // would influence clearance as well. See #725
                    let extra_margin = box_.margin.get().top - *margin_top;
                    *top_offset = *top_offset + extra_margin;
                    *margin_top = box_.margin.get().top;
                }
            }
            // The bottom margin of an in-flow block-level element collapses
            // with the top margin of its next in-flow block-level sibling.
            *collapsing = geometry::min(box_.margin.get().top, *collapsible);
            *collapsible = box_.margin.get().bottom;
        }

        *first_in_flow = false;
    }

    fn debug_str(&self) -> ~str {
        let txt = if self.is_float() {
            ~"FloatFlow: "
        } else {
            ~"TableFlow: "
        };
        txt.append(match self.box_ {
            Some(ref rb) => rb.debug_str(),
            None => ~"",
        })
    }
}

