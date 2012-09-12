use au = gfx::geometry;
use base::Box;
use core::dvec::DVec;
use css::values::{BoxAuto, BoxLength, Px};
use dom::rcu;
use geom::point::Point2D;
use geom::size::Size2D;
use gfx::geometry::au;
use layout::base::{FlowContext, InlineFlow, BoxTree, ImageBox, TextBox, GenericBox};
use num::Num;
use util::tree;

struct InlineFlowData {
    boxes: ~DVec<@Box>
}

fn InlineFlowData() -> InlineFlowData {
    InlineFlowData {
        boxes: ~DVec()
    }
}

trait InlineLayout {
    pure fn starts_inline_flow() -> bool;

    pure fn access_inline<T>(fn(&&InlineFlowData) -> T) -> T;
    fn bubble_widths_inline();
    fn assign_widths_inline();
    fn assign_height_inline();
}

impl @FlowContext : InlineLayout {
    pure fn starts_inline_flow() -> bool { match self.kind { InlineFlow(*) => true, _ => false } }

    pure fn access_inline<T>(cb: fn(&&InlineFlowData) -> T) -> T {
        match self.kind {
            InlineFlow(d) => cb(d),
            _  => fail fmt!("Tried to access() data of InlineFlow, but this is a %?", self.kind)
        }
    }

    fn bubble_widths_inline() {
        assert self.starts_inline_flow();

        let mut min_width = au(0);
        let mut pref_width = au(0);

        /* TODO: implement a "line sizes" API. Each inline element
        should report its longest possible chunk (i.e., text run) and
        shortest chunk (i.e., smallest word or hyphenatable segment). 

        Until this exists, pretend that the text is indivisible, just
        like a replaced element.  */

        do self.access_inline |d| {
            for d.boxes.each |box| {
                min_width = au::max(min_width, box.get_min_width());
                pref_width = au::max(pref_width, box.get_pref_width());
            }
        }

        self.data.min_width = min_width;
        self.data.pref_width = pref_width;
    }

    /* Recursively (top-down) determines the actual width of child
    contexts and boxes. When called on this context, the context has
    had its width set by the parent context. */
    fn assign_widths_inline() {
        assert self.starts_inline_flow();

        /* Perform inline flow with the available width. */
        //let avail_width = self.data.position.size.width;

        let line_height = au::from_px(20);
        //let mut cur_x = au(0);
        let mut cur_y = au(0);
        
        do self.access_inline |d| {
            for d.boxes.each |box| {
                /* TODO: actually do inline flow.
                - Create a working linebox, and successively put boxes
                into it, splitting if necessary.
                
                - Set width and height for each positioned element based on 
                where its chunks ended up.

                - Save the dvec of this context's lineboxes. */
            
                box.data.position.size.width = match box.kind {
                    ImageBox(sz) => sz.width,
                    TextBox(d) => d.runs[0].size().width,
                    // TODO: this should be set to the extents of its children
                    GenericBox(*) => au(0)
                };

                box.data.position.size.height = match box.kind {
                    ImageBox(sz) => sz.height,
                    TextBox(d) => d.runs[0].size().height,
                    // TODO: this should be set to the extents of its children
                    GenericBox(*) => au(0)
                };
                
                box.data.position.origin = Point2D(au(0), cur_y);
                cur_y += au::max(line_height, box.data.position.size.height);
            } // for boxes.each |box|
        }

        self.data.position.size.height = cur_y;
        
        /* There are no child contexts, so stop here. */

        // TODO: once there are 'inline-block' elements, this won't be
        // true.  In that case, perform inline flow, and then set the
        // block flow context's width as the width of the
        // 'inline-block' box that created this flow.

    } // fn assign_widths_inline

    fn assign_height_inline() {
        // Don't need to set box or ctx heights, since that is done
        // during inline flowing.
    }

} // @FlowContext : InlineLayout
