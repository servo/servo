use au = gfx::geometry;
use base::RenderBox;
use core::dvec::DVec;
use css::values::{BoxAuto, BoxLength, Px};
use dl = gfx::display_list;
use dom::rcu;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::geometry::au;
use layout::context::LayoutContext;
use layout::base::{FlowContext, InlineFlow, RenderBoxTree, ImageBox, TextBox, GenericBox};
use num::Num;
use util::tree;

/*
Tentative design: (may not line up with reality)

Lineboxes are represented as offsets into the child list, rather than
as an object that "owns" boxes. Choosing a different set of line
breaks requires a new list of offsets, and possibly some splitting and
merging of TextBoxes.

A similar list will keep track of the mapping between CSS boxes and
the corresponding render boxes in the inline flow.

After line breaks are determined, lender boxes in the inline flow may
overlap visually. For example, in the case of nested inline CSS boxes,
outer inlines must be at least as large as the inner inlines, for
purposes of drawing noninherited things like backgrounds, borders,
outlines.

N.B. roc has an alternative design where the list instead consists of
things like "start outer box, text, start inner box, text, end inner
box, text, end outer box, text". This seems a little complicated to
serve as the starting point, but the current design doesn't make it
hard to try out that alternative.
*/

struct InlineFlowData {
    boxes: ~DVec<@RenderBox>
}

fn InlineFlowData() -> InlineFlowData {
    InlineFlowData {
        boxes: ~DVec()
    }
}

trait InlineLayout {
    pure fn starts_inline_flow() -> bool;

    pure fn access_inline<T>(fn(&&InlineFlowData) -> T) -> T;
    fn bubble_widths_inline(ctx: &LayoutContext);
    fn assign_widths_inline(ctx: &LayoutContext);
    fn assign_height_inline(ctx: &LayoutContext);
    fn build_display_list_inline(a: &dl::DisplayListBuilder, b: &Rect<au>, c: &Point2D<au>, d: &dl::DisplayList);
}

impl @FlowContext : InlineLayout {
    pure fn starts_inline_flow() -> bool { match self.kind { InlineFlow(*) => true, _ => false } }

    pure fn access_inline<T>(cb: fn(&&InlineFlowData) -> T) -> T {
        match self.kind {
            InlineFlow(d) => cb(d),
            _  => fail fmt!("Tried to access() data of InlineFlow, but this is a %?", self.kind)
        }
    }

    fn bubble_widths_inline(_ctx: &LayoutContext) {
        assert self.starts_inline_flow();

        let mut min_width = au(0);
        let mut pref_width = au(0);

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
    fn assign_widths_inline(_ctx: &LayoutContext) {
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
                cur_y = cur_y.add(au::max(line_height, box.data.position.size.height));
            } // for boxes.each |box|
        }

        self.data.position.size.height = cur_y;
        
        /* There are no child contexts, so stop here. */

        // TODO: once there are 'inline-block' elements, this won't be
        // true.  In that case, perform inline flow, and then set the
        // block flow context's width as the width of the
        // 'inline-block' box that created this flow.

    } // fn assign_widths_inline

    fn assign_height_inline(_ctx: &LayoutContext) {
        // Don't need to set box or ctx heights, since that is done
        // during inline flowing.
    }

    fn build_display_list_inline(builder: &dl::DisplayListBuilder, dirty: &Rect<au>, 
                                 offset: &Point2D<au>, list: &dl::DisplayList) {

        assert self.starts_inline_flow();

        // TODO: if the CSS box introducing this inline context is *not* anonymous,
        // we need to draw it too, in a way similar to BlowFlowContext

        // TODO: once we form line boxes and have their cached bounds, we can be 
        // smarter and not recurse on a line if nothing in it can intersect dirty
        do self.access_inline |d| {
            for d.boxes.each |box| {
                box.build_display_list(builder, dirty, offset, list)
            }
        }

        // TODO: should inline-block elements have flows as children
        // of the inline flow, or should the flow be nested inside the
        // box somehow? Maybe it's best to unify flows and boxes into
        // the same enum, so inline-block flows are normal
        // (indivisible) children in the inline flow child list.
    }

} // @FlowContext : InlineLayout
