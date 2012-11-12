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

        self.d().position = copy ctx.screen_size;
        self.assign_widths_block(ctx)
    }

    fn assign_height_root(@self, ctx: &LayoutContext) {
        assert self.starts_root_flow();

        self.assign_height_block(ctx);
    }

    fn build_display_list_root(@self, builder: &DisplayListBuilder, dirty: &Rect<Au>, 
                               offset: &Point2D<Au>, list: &mut DisplayList) {
        assert self.starts_root_flow();
        
        self.build_display_list_block(builder, dirty, offset, list);
    }
}
