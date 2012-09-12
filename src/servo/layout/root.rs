use au = gfx::geometry;
use css::values::*;
use geom::point::Point2D;
use geom::size::Size2D;
use gfx::geometry::au;
use layout::base::{Box, FlowContext, FlowTree, InlineBlockFlow, BlockFlow, RootFlow};
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

    fn bubble_widths_root();
    fn assign_widths_root();
    fn assign_height_root();
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
    fn bubble_widths_root() {
        assert self.starts_root_flow();
        self.bubble_widths_block()
    }
 
    fn assign_widths_root() { 
        assert self.starts_root_flow();

        /* TODO: should determine frame width here, not in
        LayoutTask. Until then, defer to block.  */
        self.assign_widths_block() }

    fn assign_height_root() {
        assert self.starts_root_flow();

        self.assign_height_block();
    }
}
