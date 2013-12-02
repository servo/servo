/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS block layout.

use layout::box::{RenderBox, RenderBoxUtils};
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{BlockFlowClass, FlowClass, Flow, FlowData, ImmutableFlowUtils};
use layout::flow;
use layout::model::{MaybeAuto, Specified, Auto, specified_or_none, specified};
use layout::float_context::{FloatContext, Invalid};

use std::cell::Cell;
use geom::point::Point2D;
use geom::size::Size2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use servo_util::geometry::{Au, to_frac_px};
use servo_util::geometry;

pub struct BlockFlow {
    /// Data common to all flows.
    base: FlowData,

    /// The associated render box.
    box: Option<@RenderBox>,

    /// Whether this block flow is the root flow.
    is_root: bool
}

impl BlockFlow {
    pub fn new(base: FlowData) -> BlockFlow {
        BlockFlow {
            base: base,
            box: None,
            is_root: false
        }
    }

    pub fn from_box(base: FlowData, box: @RenderBox) -> BlockFlow {
        BlockFlow {
            base: base,
            box: Some(box),
            is_root: false
        }
    }

    pub fn new_root(base: FlowData) -> BlockFlow {
        BlockFlow {
            base: base,
            box: None,
            is_root: true
        }
    }

    pub fn teardown(&mut self) {
        for box in self.box.iter() {
            box.teardown();
        }
        self.box = None;
    }

    /// Computes left and right margins and width based on CSS 2.1 section 10.3.3.
    /// Requires borders and padding to already be computed.
    fn compute_horiz(&self,
                     width: MaybeAuto, 
                     left_margin: MaybeAuto, 
                     right_margin: MaybeAuto, 
                     available_width: Au)
                     -> (Au, Au, Au) {
        // If width is not 'auto', and width + margins > available_width, all 'auto' margins are
        // treated as 0.
        let (left_margin, right_margin) = match width {
            Auto => (left_margin, right_margin),
            Specified(width) => {
                let left = left_margin.specified_or_zero();
                let right = right_margin.specified_or_zero();
                
                if((left + right + width) > available_width) {
                    (Specified(left), Specified(right))
                } else {
                    (left_margin, right_margin)
                }
            }
        };

        //Invariant: left_margin_Au + width_Au + right_margin_Au == available_width
        let (left_margin_Au, width_Au, right_margin_Au) = match (left_margin, width, right_margin) {
            //If all have a computed value other than 'auto', the system is over-constrained and we need to discard a margin.
            //if direction is ltr, ignore the specified right margin and solve for it. If it is rtl, ignore the specified 
            //left margin. FIXME(eatkinson): this assumes the direction is ltr
            (Specified(margin_l), Specified(width), Specified(_margin_r)) => (margin_l, width, available_width - (margin_l + width )),

            //If exactly one value is 'auto', solve for it
            (Auto, Specified(width), Specified(margin_r)) => (available_width - (width + margin_r), width, margin_r),
            (Specified(margin_l), Auto, Specified(margin_r)) => (margin_l, available_width - (margin_l + margin_r), margin_r),
            (Specified(margin_l), Specified(width), Auto) => (margin_l, width, available_width - (margin_l + width)),

            //If width is set to 'auto', any other 'auto' value becomes '0', and width is solved for
            (Auto, Auto, Specified(margin_r)) => (Au::new(0), available_width - margin_r, margin_r),
            (Specified(margin_l), Auto, Auto) => (margin_l, available_width - margin_l, Au::new(0)),
            (Auto, Auto, Auto) => (Au::new(0), available_width, Au::new(0)),

            //If left and right margins are auto, they become equal
            (Auto, Specified(width), Auto) => {
                let margin = (available_width - width).scale_by(0.5);
                (margin, width, margin)
            }

        };
        //return values in same order as params
        (width_Au, left_margin_Au, right_margin_Au)
    }

    // inline(always) because this is only ever called by in-order or non-in-order top-level
    // methods
    #[inline(always)]
    fn assign_height_block_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {
        let mut cur_y = Au::new(0);
        let mut clearance = Au::new(0);
        let mut top_offset = Au::new(0);
        let mut bottom_offset = Au::new(0);
        let mut left_offset = Au::new(0);
        let mut float_ctx = Invalid;

        for &box in self.box.iter() {
            let base = box.base();
            clearance = match base.clear() {
                None => Au::new(0),
                Some(clear) => {
                    self.base.floats_in.clearance(clear)
                }
            };

            top_offset = clearance + base.margin.top + base.border.top + base.padding.top;
            cur_y = cur_y + top_offset;
            bottom_offset = base.margin.bottom + base.border.bottom + base.padding.bottom;
            left_offset = base.offset();
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
        for &box in self.box.iter() {
            let base = box.base();
            if !self.is_root && base.border.top == Au::new(0) && base.padding.top == Au::new(0) {
                collapsible = base.margin.top;
                top_margin_collapsible = true;
            }
            if !self.is_root && base.border.bottom == Au::new(0) && base.padding.bottom == Au::new(0) {
                bottom_margin_collapsible = true;
            }
            margin_top = base.margin.top;
            margin_bottom = base.margin.bottom;
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


        let mut height = if self.is_root {
            Au::max(ctx.screen_size.size.height, cur_y)
        } else {
            cur_y - top_offset - collapsing
        };

        for &box in self.box.iter() {
            let base = box.base();
            let style = base.style();
            height = match MaybeAuto::from_style(style.Box.height, Au::new(0)) {
                Auto => height,
                Specified(value) => value
            };
        }

        let mut noncontent_height = Au::new(0);
        for box in self.box.iter() {
            let base = box.mut_base();
            let mut position_ref = base.position.mutate();
            let position = &mut position_ref.ptr;

            // The associated box is the border box of this flow.
            base.margin.top = margin_top;
            base.margin.bottom = margin_bottom;

            position.origin.y = clearance + base.margin.top;

            noncontent_height = base.padding.top + base.padding.bottom + base.border.top +
                base.border.bottom;
            position.size.height = height + noncontent_height;

            noncontent_height = noncontent_height + clearance + base.margin.top +
                base.margin.bottom;
        }

        self.base.position.size.height = height + noncontent_height;

        if inorder {
            let extra_height = height - (cur_y - top_offset) + bottom_offset; 
            self.base.floats_out = float_ctx.translate(Point2D(left_offset, -extra_height));
        } else {
            self.base.floats_out = self.base.floats_in.clone();
        }
    }

    pub fn build_display_list_block<E:ExtraDisplayListData>(
                                    &mut self,
                                    builder: &DisplayListBuilder,
                                    dirty: &Rect<Au>, 
                                    list: &Cell<DisplayList<E>>) 
                                    -> bool {
        if self.base.node.is_iframe_element() {
            let x = self.base.abs_position.x + do self.box.map_default(Au::new(0)) |box| {
                let base = box.base();
                base.margin.left + base.border.left + base.padding.left
            };
            let y = self.base.abs_position.y + do self.box.map_default(Au::new(0)) |box| {
                let base = box.base();
                base.margin.top + base.border.top + base.padding.top
            };
            let w = self.base.position.size.width - do self.box.map_default(Au::new(0)) |box| {
                box.base().noncontent_width()
            };
            let h = self.base.position.size.height - do self.box.map_default(Au::new(0)) |box| {
                box.base().noncontent_height()
            };
            do self.base.node.with_mut_iframe_element |iframe_element| {
                iframe_element.size.get_mut_ref().set_rect(Rect(Point2D(to_frac_px(x) as f32,
                                                                        to_frac_px(y) as f32),
                                                                Size2D(to_frac_px(w) as f32,
                                                                       to_frac_px(h) as f32)));
            }
        }

        let abs_rect = Rect(self.base.abs_position, self.base.position.size);
        if !abs_rect.intersects(dirty) {
            return true;
        }

        debug!("build_display_list_block: adding display element");

        // add box that starts block context
        for box in self.box.iter() {
            box.build_display_list(builder, dirty, &self.base.abs_position, list)
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

impl Flow for BlockFlow {
    fn class(&self) -> FlowClass {
        BlockFlowClass
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        self
    }

    /* Recursively (bottom-up) determine the context's preferred and
    minimum widths.  When called on this context, all child contexts
    have had their min/pref widths set. This function must decide
    min/pref widths based on child context widths and dimensions of
    any boxes it is responsible for flowing.  */

    /* TODO: floats */
    /* TODO: absolute contexts */
    /* TODO: inline-blocks */
    fn bubble_widths(&mut self, _: &mut LayoutContext) {
        let mut min_width = Au::new(0);
        let mut pref_width = Au::new(0);
        let mut num_floats = 0;

        /* find max width from child block contexts */
        for child_ctx in self.base.child_iter() {
            assert!(child_ctx.starts_block_flow() || child_ctx.starts_inline_flow());

            let child_base = flow::mut_base(*child_ctx);
            min_width = geometry::max(min_width, child_base.min_width);
            pref_width = geometry::max(pref_width, child_base.pref_width);

            num_floats = num_floats + child_base.num_floats;
        }

        /* if not an anonymous block context, add in block box's widths.
           these widths will not include child elements, just padding etc. */
        for box in self.box.iter() {
            {
                let mut_base = box.mut_base();
                let base = box.base();
                // Can compute border width here since it doesn't depend on anything.
                mut_base.compute_borders(base.style())
            }

            let (this_minimum_width, this_preferred_width) = box.minimum_and_preferred_widths();
            min_width = min_width + this_minimum_width;
            pref_width = pref_width + this_preferred_width;
        }

        self.base.min_width = min_width;
        self.base.pref_width = pref_width;
        self.base.num_floats = num_floats;
    }
 
    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    ///
    /// Dual boxes consume some width first, and the remainder is assigned to all child (block)
    /// contexts.
    fn assign_widths(&mut self, ctx: &mut LayoutContext) { 
        debug!("assign_widths_block: assigning width for flow {}",  self.base.id);
        if self.is_root {
            debug!("Setting root position");
            self.base.position.origin = Au::zero_point();
            self.base.position.size.width = ctx.screen_size.size.width;
            self.base.floats_in = FloatContext::new(self.base.num_floats);
            self.base.is_inorder = false;
        }

        //position was set to the containing block by the flow's parent
        let mut remaining_width = self.base.position.size.width;
        let mut x_offset = Au::new(0);

        for &box in self.box.iter() {
            let base = box.mut_base();
            let style = base.style();

            // Can compute padding here since we know containing block width.
            base.compute_padding(style, remaining_width);

            // Margins are 0 right now so base.noncontent_width() is just borders + padding.
            let available_width = remaining_width - base.noncontent_width();

            // Top and bottom margins for blocks are 0 if auto.
            let margin_top = MaybeAuto::from_style(style.Margin.margin_top,
                                                   remaining_width).specified_or_zero();
            let margin_bottom = MaybeAuto::from_style(style.Margin.margin_bottom,
                                                      remaining_width).specified_or_zero();

            let (width, maybe_margin_left, maybe_margin_right) =
                (MaybeAuto::from_style(style.Box.width, remaining_width),
                 MaybeAuto::from_style(style.Margin.margin_left, remaining_width),
                 MaybeAuto::from_style(style.Margin.margin_right, remaining_width));

            let (width, margin_left, margin_right) = self.compute_horiz(width,
                                                                        maybe_margin_left,
                                                                        maybe_margin_right,
                                                                        available_width);

            // If the tentative used width is greater than 'max-width', width should be recalculated,
            // but this time using the computed value of 'max-width' as the computed value for 'width'.
            let (width, margin_left, margin_right) = {
                match specified_or_none(style.Box.max_width, remaining_width) {
                    Some(value) if value < width => self.compute_horiz(Specified(value),
                                                                       maybe_margin_left,
                                                                       maybe_margin_right,
                                                                       available_width),
                    _ => (width, margin_left, margin_right)
                }
            };
            // If the resulting width is smaller than 'min-width', width should be recalculated,
            // but this time using the value of 'min-width' as the computed value for 'width'.
            let (width, margin_left, margin_right) = {
                let computed_min_width = specified(style.Box.min_width, remaining_width);
                if computed_min_width > width {
                    self.compute_horiz(Specified(computed_min_width),
                                       maybe_margin_left,
                                       maybe_margin_right,
                                       available_width)
                } else {
                    (width, margin_left, margin_right)
                }
            };

            base.margin.top = margin_top;
            base.margin.right = margin_right;
            base.margin.bottom = margin_bottom;
            base.margin.left = margin_left;

            x_offset = base.offset();
            remaining_width = width;

            //The associated box is the border box of this flow
            let position_ref = base.position.mutate();
            position_ref.ptr.origin.x = base.margin.left;
            let padding_and_borders = base.padding.left + base.padding.right +
                base.border.left + base.border.right;
            position_ref.ptr.size.width = remaining_width + padding_and_borders;
        }

        let has_inorder_children = self.base.is_inorder || self.base.num_floats > 0;
        for kid in self.base.child_iter() {
            assert!(kid.starts_block_flow() || kid.starts_inline_flow());

            let child_base = flow::mut_base(*kid);
            child_base.position.origin.x = x_offset;
            child_base.position.size.width = remaining_width;
            child_base.is_inorder = has_inorder_children;

            if !child_base.is_inorder {
                child_base.floats_in = FloatContext::new(0);
            }
        }
    }

    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height_inorder: assigning height for block {}", self.base.id);
        self.assign_height_block_base(ctx, true);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height: assigning height for block {}", self.base.id);
        // This is the only case in which a block flow can start an inorder
        // subtraversal.
        if self.is_root && self.base.num_floats > 0 {
            self.assign_height_inorder(ctx);
            return;
        }
        self.assign_height_block_base(ctx, false);
    }

    fn collapse_margins(&mut self,
                        top_margin_collapsible: bool,
                        first_in_flow: &mut bool,
                        margin_top: &mut Au,
                        top_offset: &mut Au,
                        collapsing: &mut Au,
                        collapsible: &mut Au) {
        for &box in self.box.iter() {
            let base = box.base();

            // The top margin collapses with its first in-flow block-level child's
            // top margin if the parent has no top border, no top padding.
            if *first_in_flow && top_margin_collapsible {
                // If top-margin of parent is less than top-margin of its first child, 
                // the parent box goes down until its top is aligned with the child.
                if *margin_top < base.margin.top {
                    // TODO: The position of child floats should be updated and this
                    // would influence clearance as well. See #725
                    let extra_margin = base.margin.top - *margin_top;
                    *top_offset = *top_offset + extra_margin;
                    *margin_top = base.margin.top;
                }
            }
            // The bottom margin of an in-flow block-level element collapses 
            // with the top margin of its next in-flow block-level sibling.
            *collapsing = geometry::min(base.margin.top, *collapsible);
            *collapsible = base.margin.bottom;
        }

        *first_in_flow = false;
    }

    fn mark_as_root(&mut self) {
        self.is_root = true
    }

    fn debug_str(&self) -> ~str {
        if self.is_root {
            ~"BlockFlow(root)"
        } else {
            let txt = ~"BlockFlow: ";
            txt.append(match self.box {
                Some(rb) => {
                    rb.debug_str()
                }
                None => { ~"" }
            })
        }
    }
}

