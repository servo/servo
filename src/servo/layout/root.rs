/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::cell::Cell;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
use layout::block::BlockLayout;
use layout::box::RenderBox;
use layout::context::LayoutContext;
use layout::flow::{FlowContext, FlowData, RootFlow};
use layout::display_list_builder::DisplayListBuilder;

use servo_util::tree::{TreeNodeRef, TreeUtils};

pub struct RootFlowData {
    /// Data common to all flows.
    common: FlowData,

    /// The render box at the root of the tree.
    box: Option<RenderBox>
}

impl RootFlowData {
    pub fn new(common: FlowData) -> RootFlowData {
        RootFlowData {
            common: common,
            box: None,
        }
    }
}

pub trait RootLayout {
    fn starts_root_flow(&self) -> bool;
}

impl RootLayout for FlowContext {
    fn starts_root_flow(&self) -> bool {
        match *self {
            RootFlow(*) => true,
            _ => false 
        }
    }
}

impl RootFlowData {
    /// Defer to the block algorithm.
    pub fn bubble_widths_root(@mut self, ctx: &LayoutContext) {
        RootFlow(self).bubble_widths_block(ctx)
    }
 
    pub fn assign_widths_root(@mut self, ctx: &LayoutContext) { 
        self.common.position.origin = Au::zero_point();
        self.common.position.size.width = ctx.screen_size.size.width;

        RootFlow(self).assign_widths_block(ctx)
    }

    pub fn assign_height_root(@mut self, ctx: &LayoutContext) {
        // this is essentially the same as assign_height_block(), except
        // the root adjusts self height to at least cover the viewport.
        let mut cur_y = Au(0);

        for RootFlow(self).each_child |child_flow| {
            do child_flow.with_mut_node |child_node| {
                child_node.position.origin.y = cur_y;
                cur_y += child_node.position.size.height;
            }
        }

        self.common.position.size.height = Au::max(ctx.screen_size.size.height, cur_y);

        do RootFlow(self).with_block_box |box| {
            do box.with_mut_base |base| {
                base.position.origin.y = Au(0);
                base.position.size.height = Au::max(ctx.screen_size.size.height, cur_y);
                let (_used_top, _used_bot) = box.get_used_height();
            }
        }
    }

    pub fn build_display_list_root(@mut self,
                                   builder: &DisplayListBuilder,
                                   dirty: &Rect<Au>, 
                                   offset: &Point2D<Au>,
                                   list: &Cell<DisplayList>) {
        RootFlow(self).build_display_list_block(builder, dirty, offset, list);
    }
}
