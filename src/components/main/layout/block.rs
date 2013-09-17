/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS block layout.

use layout::box::{RenderBox};
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{BlockFlow, FlowContext, FlowData, InlineBlockFlow, FloatFlow, InlineFlow};
use layout::inline::InlineLayout;
use layout::model::{MaybeAuto, Specified, Auto};
use layout::float_context::{FloatContext, Invalid};

use std::cell::Cell;
use geom::point::Point2D;
use geom::size::Size2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use gfx::geometry::{Au, to_frac_px};
use gfx::geometry;

pub struct BlockFlowData {
    /// Data common to all flows.
    common: FlowData,

    /// The associated render box.
    box: Option<RenderBox>,

    /// Whether this block flow is the root flow.
    is_root: bool
}

impl BlockFlowData {
    pub fn new(common: FlowData) -> BlockFlowData {
        BlockFlowData {
            common: common,
            box: None,
            is_root: false
        }
    }

    pub fn new_root(common: FlowData) -> BlockFlowData {
        BlockFlowData {
            common: common,
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
}

pub trait BlockLayout {
    fn starts_root_flow(&self) -> bool;
    fn starts_block_flow(&self) -> bool;
}

impl BlockLayout for FlowContext {
    fn starts_root_flow(&self) -> bool {
        match *self {
            BlockFlow(ref info) => info.is_root,
            _ => false
        }
    }

    fn starts_block_flow(&self) -> bool {
        match *self {
            BlockFlow(*) | InlineBlockFlow(*) | FloatFlow(*) => true,
            _ => false 
        }
    }
}

impl BlockFlowData {
    /* Recursively (bottom-up) determine the context's preferred and
    minimum widths.  When called on this context, all child contexts
    have had their min/pref widths set. This function must decide
    min/pref widths based on child context widths and dimensions of
    any boxes it is responsible for flowing.  */

    /* TODO: floats */
    /* TODO: absolute contexts */
    /* TODO: inline-blocks */
    pub fn bubble_widths_block(&mut self, ctx: &LayoutContext) {
        let mut min_width = Au(0);
        let mut pref_width = Au(0);
        let mut num_floats = 0;

        /* find max width from child block contexts */
        for child_ctx in self.common.child_iter() {
            assert!(child_ctx.starts_block_flow() || child_ctx.starts_inline_flow());

            do child_ctx.with_mut_base |child_node| {
                min_width = geometry::max(min_width, child_node.min_width);
                pref_width = geometry::max(pref_width, child_node.pref_width);

                num_floats = num_floats + child_node.num_floats;
            }
        }

        /* if not an anonymous block context, add in block box's widths.
           these widths will not include child elements, just padding etc. */
        self.box.map(|&box| {
            min_width = min_width.add(&box.get_min_width(ctx));
            pref_width = pref_width.add(&box.get_pref_width(ctx));
        });

        self.common.min_width = min_width;
        self.common.pref_width = pref_width;
        self.common.num_floats = num_floats;
    }
 
    /// Computes left and right margins and width based on CSS 2.1 secion 10.3.3.
    /// Requires borders and padding to already be computed
    fn compute_horiz( &self,
                            width: MaybeAuto, 
                            left_margin: MaybeAuto, 
                            right_margin: MaybeAuto, 
                            available_width: Au) -> (Au, Au, Au) {

        //If width is not 'auto', and width + margins > available_width, all 'auto' margins are treated as '0'
        let (left_margin, right_margin) = match width{
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
            (Auto, Auto, Specified(margin_r)) => (Au(0), available_width - margin_r, margin_r),
            (Specified(margin_l), Auto, Auto) => (margin_l, available_width - margin_l, Au(0)),
            (Auto, Auto, Auto) => (Au(0), available_width, Au(0)),

            //If left and right margins are auto, they become equal
            (Auto, Specified(width), Auto) => {
                let margin = (available_width - width).scale_by(0.5);
                (margin, width, margin)
            }

        };
        //return values in same order as params
        (width_Au, left_margin_Au, right_margin_Au)
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    ///
    /// Dual boxes consume some width first, and the remainder is assigned to all child (block)
    /// contexts.
    pub fn assign_widths_block(&mut self, ctx: &LayoutContext) { 
        debug!("assign_widths_block: assigning width for flow %?",  self.common.id);
        if self.is_root {
            debug!("Setting root position");
            self.common.position.origin = Au::zero_point();
            self.common.position.size.width = ctx.screen_size.size.width;
            self.common.floats_in = FloatContext::new(self.common.num_floats);
            self.common.is_inorder = false;
        }

        //position was set to the containing block by the flow's parent
        let mut remaining_width = self.common.position.size.width;
        let mut x_offset = Au(0);

        for &box in self.box.iter() {
            let style = box.style();
            do box.with_model |model| {
                //Can compute border width here since it doesn't depend on anything
                model.compute_borders(style);

                // Can compute padding here since we know containing block width.
                model.compute_padding(style, remaining_width);

                // Margins are 0 right now so model.noncontent_width() is just borders + padding.
                let available_width = remaining_width - model.noncontent_width();

                // Top and bottom margins for blocks are 0 if auto.
                let margin_top = MaybeAuto::from_margin(style.margin_top(),
                                                        remaining_width,
                                                        style.font_size()).specified_or_zero();
                let margin_bottom = MaybeAuto::from_margin(style.margin_bottom(),
                                                           remaining_width,
                                                           style.font_size()).specified_or_zero();

                let (width, margin_left, margin_right) =
                    (MaybeAuto::from_width(style.width(), remaining_width, style.font_size()),
                     MaybeAuto::from_margin(style.margin_left(), remaining_width, style.font_size()),
                     MaybeAuto::from_margin(style.margin_right(), remaining_width, style.font_size()));

                let (width, margin_left, margin_right) = self.compute_horiz(width,
                                                                            margin_left,
                                                                            margin_right,
                                                                            available_width);

                model.margin.top = margin_top;
                model.margin.right = margin_right;
                model.margin.bottom = margin_bottom;
                model.margin.left = margin_left;

                x_offset = model.offset();
                remaining_width = width;
            }

            do box.with_mut_base |base| {
                //The associated box is the border box of this flow
                base.position.origin.x = base.model.margin.left;

                let pb = base.model.padding.left + base.model.padding.right +
                    base.model.border.left + base.model.border.right;
                base.position.size.width = remaining_width + pb;
            }
        }

        let has_inorder_children = self.common.is_inorder || self.common.num_floats > 0;
        for kid in self.common.child_iter() {
            assert!(kid.starts_block_flow() || kid.starts_inline_flow());

            do kid.with_mut_base |child_node| {
                child_node.position.origin.x = x_offset;
                child_node.position.size.width = remaining_width;
                child_node.is_inorder = has_inorder_children;

                if !child_node.is_inorder {
                    child_node.floats_in = FloatContext::new(0);
                }
            }
        }
    }

    pub fn assign_height_inorder_block(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height_inorder_block: assigning height for block %?", self.common.id);
        self.assign_height_block_base(ctx, true);
    }

    pub fn assign_height_block(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height_block: assigning height for block %?", self.common.id);
        // This is the only case in which a block flow can start an inorder
        // subtraversal.
        if self.is_root && self.common.num_floats > 0 {
            self.assign_height_inorder_block(ctx);
            return;
        }
        self.assign_height_block_base(ctx, false);
    }

    fn assign_height_block_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {
        let mut cur_y = Au(0);
        let mut clearance = Au(0);
        let mut top_offset = Au(0);
        let mut bottom_offset = Au(0);
        let mut left_offset = Au(0);
        let mut float_ctx = Invalid;

        for &box in self.box.iter() {
            clearance = match box.clear() {
                None => Au(0),
                Some(clear) => {
                    self.common.floats_in.clearance(clear)
                }
            };

            do box.with_model |model| {
                top_offset = clearance + model.margin.top + model.border.top + model.padding.top;
                cur_y = cur_y + top_offset;
                bottom_offset = model.margin.bottom + model.border.bottom + model.padding.bottom;
                left_offset = model.offset();
            };
        }

        if inorder {
            // Floats for blocks work like this:
            // self.floats_in -> child[0].floats_in
            // visit child[0]
            // child[i-1].floats_out -> child[i].floats_in
            // visit child[i]
            // repeat until all children are visited.
            // last_child.floats_out -> self.floats_out (done at the end of this method)
            float_ctx = self.common.floats_in.translate(Point2D(-left_offset, -top_offset));
            for kid in self.common.child_iter() {
                do kid.with_mut_base |child_node| {
                    child_node.floats_in = float_ctx.clone();
                }
                kid.assign_height_inorder(ctx);
                do kid.with_mut_base |child_node| {
                    float_ctx = child_node.floats_out.clone();
                }
            }
        }

        let mut collapsible = Au(0);
        let mut collapsing = Au(0);
        let mut margin_top = Au(0);
        let mut margin_bottom = Au(0);
        let mut top_margin_collapsible = false;
        let mut bottom_margin_collapsible = false;
        let mut first_inflow = true;
        for &box in self.box.iter() {
            do box.with_model |model| {
                if !self.is_root && model.border.top == Au(0) && model.padding.top == Au(0) {
                    collapsible = model.margin.top;
                    top_margin_collapsible = true;
                }
                if !self.is_root && model.border.bottom == Au(0) && model.padding.bottom == Au(0) {
                    bottom_margin_collapsible = true;
                }
                margin_top = model.margin.top;
                margin_bottom = model.margin.bottom;
            }
        }

        for kid in self.common.child_iter() {
            match *kid {
                BlockFlow(ref info) => {
                    for &box in info.box.iter() {
                        do box.with_model |model| {
                            // The top margin collapses with its first in-flow block-level child's
                            // top margin if the parent has no top boder, no top padding.
                            if first_inflow && top_margin_collapsible {
                                // If top-margin of parent is less than top-margin of its first child, 
                                // the parent box goes down until its top is aligned with the child.
                                if margin_top < model.margin.top {
                                    // TODO: The position of child floats should be updated and this
                                    // would influence clearance as well. See #725
                                    let extra_margin = model.margin.top - margin_top;
                                    top_offset = top_offset + extra_margin;
                                    margin_top = model.margin.top;
                                }
                            }
                            // The bottom margin of an in-flow block-level element collapses 
                            // with the top margin of its next in-flow block-level sibling.
                            collapsing = geometry::min(model.margin.top, collapsible);
                            collapsible = model.margin.bottom;
                        }
                    }
                    first_inflow = false;
                }
                InlineFlow(ref info) => {
                    collapsing = Au(0);
                    // Non-empty inline flows prevent collapsing between the previous margion and the next.
                    if info.common.position.size.height > Au(0) {
                        collapsible = Au(0);
                    }
                }
                // Margins between a floated box and any other box do not collapse.
                _ => {
                    collapsing = Au(0);
                }
                // TODO: Handling for AbsoluteFlow, InlineBlockFlow and TableFlow?
            }

            do kid.with_mut_base |child_node| {
                cur_y = cur_y - collapsing;
                child_node.position.origin.y = cur_y;
                cur_y = cur_y + child_node.position.size.height;
            };
        }

        // The bottom margin collapses with its last in-flow block-level child's bottom margin
        // if the parent has no bottom boder, no bottom padding.
        collapsing = if bottom_margin_collapsible {
            if margin_bottom < collapsible {
                margin_bottom = collapsible;
            }
            collapsible
        } else {
            Au(0)
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
            let style = box.style();
            let maybe_height = MaybeAuto::from_height(style.height(), Au(0), style.font_size());
            let maybe_height = maybe_height.specified_or_zero();
            height = geometry::max(height, maybe_height);
        }

        let mut noncontent_height = Au(0);
        self.box.map(|&box| {
            do box.with_mut_base |base| {
                //The associated box is the border box of this flow
                base.model.margin.top = margin_top;
                base.model.margin.bottom = margin_bottom;

                base.position.origin.y = clearance + base.model.margin.top;

                noncontent_height = base.model.padding.top + base.model.padding.bottom +
                    base.model.border.top + base.model.border.bottom;
                base.position.size.height = height + noncontent_height;

                noncontent_height = noncontent_height + clearance +
                    base.model.margin.top + base.model.margin.bottom;
            }
        });

        //TODO(eatkinson): compute heights using the 'height' property.
        self.common.position.size.height = height + noncontent_height;

        if inorder {
            let extra_height = height - (cur_y - top_offset) + bottom_offset; 
            self.common.floats_out = float_ctx.translate(Point2D(left_offset, -extra_height));
        } else {
            self.common.floats_out = self.common.floats_in.clone();
        }
    }

    pub fn build_display_list_block<E:ExtraDisplayListData>(&mut self,
                                                            builder: &DisplayListBuilder,
                                                            dirty: &Rect<Au>, 
                                                            list: &Cell<DisplayList<E>>) 
                                                            -> bool {

        if self.common.node.is_iframe_element() {
            let x = self.common.abs_position.x + do self.box.map_default(Au(0)) |box| {
                box.with_model(|model| model.margin.left + model.border.left + model.padding.left)
            };
            let y = self.common.abs_position.y + do self.box.map_default(Au(0)) |box| {
                box.with_model(|model| model.margin.top + model.border.top + model.padding.top)
            };
            let w = self.common.position.size.width - do self.box.map_default(Au(0)) |box| {
                box.with_model(|model| model.noncontent_width())
            };
            let h = self.common.position.size.height - do self.box.map_default(Au(0)) |box| {
                box.with_model(|model| model.noncontent_height())
            };
            do self.common.node.with_mut_iframe_element |iframe_element| {
                iframe_element.size.get_mut_ref().set_rect(Rect(Point2D(to_frac_px(x) as f32,
                                                                        to_frac_px(y) as f32),
                                                                Size2D(to_frac_px(w) as f32,
                                                                       to_frac_px(h) as f32)));
            }
        }

        let abs_rect = Rect(self.common.abs_position, self.common.position.size);
        if !abs_rect.intersects(dirty) {
            return true;
        }

        debug!("build_display_list_block: adding display element");

        // add box that starts block context
        self.box.map(|&box| {
            box.build_display_list(builder, dirty, &self.common.abs_position, list)
        });


        // TODO: handle any out-of-flow elements
        let this_position = self.common.abs_position;
        for child in self.common.child_iter() {
            do child.with_mut_base |base| {
                base.abs_position = this_position + base.position.origin;
            }
        }

        false
    }
}

