use au = gfx::geometry;
use newcss::values::*;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
use layout::box::RenderBox;
use layout::context::LayoutContext;
use layout::flow::{FlowContext, FlowTree, InlineBlockFlow, BlockFlow, RootFlow};
use layout::display_list_builder::DisplayListBuilder;
use util::tree;

struct RootFlowData {
    mut box: Option<@RenderBox>
}

fn RootFlowData() -> RootFlowData {
    RootFlowData {
        box: None
    }
}

trait RootLayout {
    pure fn starts_root_flow() -> bool;

    fn bubble_widths_root(@self, ctx: &LayoutContext);
    fn assign_widths_root(@self, ctx: &LayoutContext);
    fn assign_height_root(@self, ctx: &LayoutContext);
    fn build_display_list_root(@self, a: &DisplayListBuilder, b: &Rect<Au>,
                               c: &Point2D<Au>, d: &mut DisplayList);
}

impl FlowContext : RootLayout {

    pure fn starts_root_flow() -> bool {
        match self {
            RootFlow(*) => true,
            _ => false 
        }
    }

    /* defer to the block algorithm */
    fn bubble_widths_root(@self, ctx: &LayoutContext) {
        assert self.starts_root_flow();
        self.bubble_widths_block(ctx)
    }
 
    fn assign_widths_root(@self, ctx: &LayoutContext) { 
        assert self.starts_root_flow();

        self.d().position.origin = Au::zero_point();
        self.d().position.size.width = ctx.screen_size.size.width;

        self.assign_widths_block(ctx)
    }

    fn assign_height_root(@self, ctx: &LayoutContext) {
        assert self.starts_root_flow();

        // this is essentially the same as assign_height_block(), except
        // the root adjusts self height to at least cover the viewport.
        let mut cur_y = Au(0);

        for FlowTree.each_child(self) |child_ctx| {
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

    fn build_display_list_root(@self, builder: &DisplayListBuilder, dirty: &Rect<Au>, 
                               offset: &Point2D<Au>, list: &mut DisplayList) {
        assert self.starts_root_flow();

        self.build_display_list_block(builder, dirty, offset, list);
    }
}
