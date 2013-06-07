/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS block layout.

use layout::box::{RenderBox};
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, FlowDisplayListBuilderMethods};
use layout::flow::{BlockFlow, FlowContext, FlowData, InlineBlockFlow};
use layout::inline::InlineLayout;

use au = gfx::geometry;
use core::cell::Cell;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
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
                min_width = au::max(min_width, child_node.min_width);
                pref_width = au::max(pref_width, child_node.pref_width);
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

        let mut remaining_width = self.common.position.size.width;
        let left_used = Au(0);

        // Let the box consume some width. It will return the amount remaining for its children.
        self.box.map(|&box| {
            do box.with_mut_base |base| {
                base.position.size.width = remaining_width;

                let (left_used, right_used) = box.get_used_width();
                remaining_width -= left_used.add(&right_used);
            }
        });

        for BlockFlow(self).each_child |kid| {
            assert!(kid.starts_block_flow() || kid.starts_inline_flow());

            do kid.with_mut_base |child_node| {
                child_node.position.origin.x = left_used;
                child_node.position.size.width = remaining_width;
            }
        }
    }

    pub fn assign_height_block(@mut self, ctx: &LayoutContext) {
        let mut cur_y = Au(0);

        for BlockFlow(self).each_child |kid| {
            do kid.with_mut_base |child_node| {
                child_node.position.origin.y = cur_y;
                cur_y += child_node.position.size.height;
            }
        }

        let height = if self.is_root { Au::max(ctx.screen_size.size.height, cur_y) }
                     else            { cur_y };

        self.common.position.size.height = height;

        let _used_top = Au(0);
        let _used_bot = Au(0);
        
        self.box.map(|&box| {
            do box.with_mut_base |base| {
                base.position.origin.y = Au(0);
                base.position.size.height = height;
                let (_used_top, _used_bot) = box.get_used_height();
            }
        });
    }

    pub fn build_display_list_block(@mut self,
                                builder: &DisplayListBuilder,
                                dirty: &Rect<Au>, 
                                offset: &Point2D<Au>,
                                list: &Cell<DisplayList>) {
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
