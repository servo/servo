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
use layout::flow::{FlowContext, RootFlow};
use layout::display_list_builder::DisplayListBuilder;

pub struct RootFlowData {
    box: Option<@mut RenderBox>
}

pub fn RootFlowData() -> RootFlowData {
    RootFlowData {
        box: None
    }
}

pub trait RootLayout {
    fn starts_root_flow(&self) -> bool;

    fn bubble_widths_root(@mut self, ctx: &LayoutContext);
    fn assign_widths_root(@mut self, ctx: &LayoutContext);
    fn assign_height_root(@mut self, ctx: &LayoutContext);
    fn build_display_list_root(@mut self, a: &DisplayListBuilder, b: &Rect<Au>,
                               c: &Point2D<Au>, d: &Cell<DisplayList>);
}

impl RootLayout for FlowContext {
    fn starts_root_flow(&self) -> bool {
        match *self {
            RootFlow(*) => true,
            _ => false 
        }
    }

    /* defer to the block algorithm */
    fn bubble_widths_root(@mut self, ctx: &LayoutContext) {
        assert!(self.starts_root_flow());
        self.bubble_widths_block(ctx)
    }
 
    fn assign_widths_root(@mut self, ctx: &LayoutContext) { 
        assert!(self.starts_root_flow());

        self.d().position.origin = Au::zero_point();
        self.d().position.size.width = ctx.screen_size.size.width;

        self.assign_widths_block(ctx)
    }

    fn assign_height_root(@mut self, ctx: &LayoutContext) {
        assert!(self.starts_root_flow());

        // this is essentially the same as assign_height_block(), except
        // the root adjusts self height to at least cover the viewport.
        let mut cur_y = Au(0);

        for self.each_child |child_ctx| {
            child_ctx.d().position.origin.y = cur_y;
            cur_y += child_ctx.d().position.size.height;
        }

        self.d().position.size.height = Au::max(ctx.screen_size.size.height, cur_y);

        do self.with_block_box |box| {
            box.d().position.origin.y = Au(0);
            box.d().position.size.height = Au::max(ctx.screen_size.size.height, cur_y);
            let (_used_top, _used_bot) = box.get_used_height();
        }
    }

    fn build_display_list_root(@mut self, builder: &DisplayListBuilder, dirty: &Rect<Au>, 
                               offset: &Point2D<Au>, list: &Cell<DisplayList>) {
        assert!(self.starts_root_flow());

        self.build_display_list_block(builder, dirty, offset, list);
    }
}
