use au = gfx::geometry;
use css::values::*;
use dl = gfx::display_list;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::geometry::au;
use layout::box::RenderBox;
use layout::context::LayoutContext;
use layout::flow::{FlowContext, FlowTree, InlineBlockFlow, BlockFlow, RootFlow};
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
    fn build_display_list_root(@self, a: &dl::DisplayListBuilder, b: &Rect<au>,
                               c: &Point2D<au>, d: &dl::DisplayList);
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

        self.d().position = copy ctx.screen_size;
        self.assign_widths_block(ctx)
    }

    fn assign_height_root(@self, ctx: &LayoutContext) {
        assert self.starts_root_flow();

        self.assign_height_block(ctx);
    }

    fn build_display_list_root(@self, builder: &dl::DisplayListBuilder, dirty: &Rect<au>, 
                               offset: &Point2D<au>, list: &dl::DisplayList) {
        assert self.starts_root_flow();
        
        self.build_display_list_block(builder, dirty, offset, list);
    }
}
