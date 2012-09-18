use au = gfx::geometry;
use css::values::*;
use gfx::geometry::au;
use layout::base::{Box, FlowContext, FlowTree, InlineBlockFlow, BlockFlow, RootFlow};
use layout::context::LayoutContext;
use util::tree;

struct RootFlowData {
    mut box: Option<@Box>
}

fn RootFlowData() -> RootFlowData {
    RootFlowData {
        box: None
    }
}

trait RootLayout {
    pure fn starts_root_flow() -> bool;
    pure fn access_root<T>(fn(&&RootFlowData) -> T) -> T;

    fn bubble_widths_root(ctx: &LayoutContext);
    fn assign_widths_root(ctx: &LayoutContext);
    fn assign_height_root(ctx: &LayoutContext);
}

impl @FlowContext : RootLayout {

    pure fn starts_root_flow() -> bool {
        match self.kind {
            RootFlow(*) => true,
            _ => false 
        }
    }

    pure fn access_root<T>(cb:fn(&&RootFlowData) -> T) -> T {
        match self.kind {
            RootFlow(d) => cb(d),
            _  => fail fmt!("Tried to access() data of RootFlow, but this is a %?", self.kind)
        }
    }

    /* defer to the block algorithm */
    fn bubble_widths_root(ctx: &LayoutContext) {
        assert self.starts_root_flow();
        self.bubble_widths_block(ctx)
    }
 
    fn assign_widths_root(ctx: &LayoutContext) { 
        assert self.starts_root_flow();

        self.data.position = copy ctx.screen_size;
        self.assign_widths_block(ctx)
    }

    fn assign_height_root(ctx: &LayoutContext) {
        assert self.starts_root_flow();

        self.assign_height_block(ctx);
    }
}
