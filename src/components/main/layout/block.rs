/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS block layout.

use layout::box::{RenderBox};
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::display_list_builder::{FlowDisplayListBuilderMethods};
use layout::flow::{BlockFlow, FlowContext, FlowData, InlineBlockFlow};
use layout::inline::InlineLayout;
use layout::model::{MaybeAuto, Specified, Auto};

use core::cell::Cell;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
use gfx::geometry;
use servo_util::tree::{TreeNodeRef, TreeUtils};

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
        self.common.teardown();
        for self.box.each |box| {
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
            BlockFlow(info) => info.is_root,
            _ => false
        }
    }

    fn starts_block_flow(&self) -> bool {
        match *self {
            BlockFlow(*) | InlineBlockFlow(*) => true,
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
    pub fn bubble_widths_block(@mut self, ctx: &LayoutContext) {
        let mut min_width = Au(0);
        let mut pref_width = Au(0);

        /* find max width from child block contexts */
        for BlockFlow(self).each_child |child_ctx| {
            assert!(child_ctx.starts_block_flow() || child_ctx.starts_inline_flow());

            do child_ctx.with_base |child_node| {
                min_width = geometry::max(min_width, child_node.min_width);
                pref_width = geometry::max(pref_width, child_node.pref_width);
            }
        }

        /* if not an anonymous block context, add in block box's widths.
           these widths will not include child elements, just padding etc. */
        self.box.map(|&box| {
            //Can compute border width here since it doesn't depend on anything
            let style = box.style();
            do box.with_model |model| {
                model.compute_borders(style)
            }
            min_width = min_width.add(&box.get_min_width(ctx));
            pref_width = pref_width.add(&box.get_pref_width(ctx));
        });

        self.common.min_width = min_width;
        self.common.pref_width = pref_width;
    }
 
    /// Computes left and right margins and width based on CSS 2.1 secion 10.3.3.
    /// Requires borders and padding to already be computed
    priv fn compute_horiz( &self, 
                            width: MaybeAuto, 
                            left_margin: MaybeAuto, 
                            right_margin: MaybeAuto, 
                            available_width: Au) -> (Au, Au, Au) {

        //If width is not 'auto', and width + margins > available_width, all 'auto' margins are treated as '0'
        let (left_margin, right_margin) = match width{
            Auto => (left_margin, right_margin),
            Specified(width) => {
                let left = left_margin.spec_or_default(Au(0));
                let right = right_margin.spec_or_default(Au(0));
                
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
    pub fn assign_widths_block(@mut self, ctx: &LayoutContext) { 
        debug!("assign_widths_block: assigning width for flow %?",  self.common.id);
        if self.is_root {
            debug!("Setting root position");
            self.common.position.origin = Au::zero_point();
            self.common.position.size.width = ctx.screen_size.size.width;
        }

        //position was set to the containing block by the flow's parent
        let mut remaining_width = self.common.position.size.width;
        let mut x_offset = Au(0);

        for self.box.each |&box| {
            let style = box.style();
            do box.with_model |model| {
                // Can compute padding here since we know containing block width.
                model.compute_padding(style, remaining_width);

                // Margins are 0 right now so model.noncontent_width() is just borders + padding.
                let available_width = remaining_width - model.noncontent_width();

                // Top and bottom margins for blocks are 0 if auto.
                let margin_top = MaybeAuto::from_margin(style.margin_top(),
                                                        remaining_width).spec_or_default(Au(0));
                let margin_bottom = MaybeAuto::from_margin(style.margin_bottom(),
                                                           remaining_width).spec_or_default(Au(0));

                let (width, margin_left, margin_right) =
                    (MaybeAuto::from_width(style.width(), remaining_width),
                     MaybeAuto::from_margin(style.margin_left(), remaining_width),
                     MaybeAuto::from_margin(style.margin_right(), remaining_width));

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

        for BlockFlow(self).each_child |kid| {
            assert!(kid.starts_block_flow() || kid.starts_inline_flow());

            do kid.with_mut_base |child_node| {
                child_node.position.origin.x = x_offset;
                child_node.position.size.width = remaining_width;
            }
        }
    }

    pub fn assign_height_block(@mut self, ctx: &LayoutContext) {
        let mut cur_y = Au(0);
        let mut top_offset = Au(0);

        for self.box.each |&box| {
            do box.with_model |model| {
                top_offset = model.margin.top + model.border.top + model.padding.top;
                cur_y += top_offset;
            }
        }

        for BlockFlow(self).each_child |kid| {
            do kid.with_mut_base |child_node| {
                child_node.position.origin.y = cur_y;
                cur_y += child_node.position.size.height;
            }
        }

        let height = if self.is_root {
            Au::max(ctx.screen_size.size.height, cur_y)
        } else {
            cur_y - top_offset
        };
        
        let mut noncontent_height = Au(0);
        self.box.map(|&box| {
            do box.with_mut_base |base| {
                //The associated box is the border box of this flow
                base.position.origin.y = base.model.margin.top;

                noncontent_height = base.model.padding.top + base.model.padding.bottom +
                    base.model.border.top + base.model.border.bottom;
                base.position.size.height = height + noncontent_height;

                noncontent_height += base.model.margin.top + base.model.margin.bottom;
            }
        });

        //TODO(eatkinson): compute heights using the 'height' property.
        self.common.position.size.height = height + noncontent_height;

    }

    pub fn build_display_list_block<E:ExtraDisplayListData>(@mut self,
                                                            builder: &DisplayListBuilder,
                                                            dirty: &Rect<Au>, 
                                                            offset: &Point2D<Au>,
                                                            list: &Cell<DisplayList<E>>) {
        // add box that starts block context
        self.box.map(|&box| {
            box.build_display_list(builder, dirty, offset, list)
        });


        // TODO: handle any out-of-flow elements

        // go deeper into the flow tree
        let flow = BlockFlow(self);
        for flow.each_child |child| {
            flow.build_display_list_for_child(builder, child, dirty, offset, list)
        }
    }
}

