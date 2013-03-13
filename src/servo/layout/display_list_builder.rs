///
/// Constructs display lists from render boxes.
///

use layout::box::{RenderBox, TextBox};
use layout::context::LayoutContext;
use layout::flow::FlowContext;
use layout::text::TextBoxData;
use newcss::values::Specified;
use newcss::values::{CSSBackgroundColorColor, CSSBackgroundColorTransparent};
use util::tree;

use core::either::{Left, Right};
use core::mutable::Mut;
use core::vec::push;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
use gfx;

/** A builder object that manages display list builder should mainly
 hold information about the initial request and desired result---for
 example, whether the DisplayList to be used for painting or hit
 testing. This can affect which boxes are created.

 Right now, the builder isn't used for much, but it  establishes the
 pattern we'll need once we support DL-based hit testing &c.  */
pub struct DisplayListBuilder {
    ctx:  &'self LayoutContext,
}

pub trait FlowDisplayListBuilderMethods {
    fn build_display_list(@mut self, a: &DisplayListBuilder, b: &Rect<Au>, c: &Mut<DisplayList>);
    fn build_display_list_for_child(@mut self,
                                    a: &DisplayListBuilder,
                                    b: @mut FlowContext,
                                    c: &Rect<Au>,
                                    d: &Point2D<Au>,
                                    e: &Mut<DisplayList>);
}

impl FlowDisplayListBuilderMethods for FlowContext {
    fn build_display_list(@mut self,
                          builder: &DisplayListBuilder,
                          dirty: &Rect<Au>,
                          list: &Mut<DisplayList>) {
        let zero = gfx::geometry::zero_point();
        self.build_display_list_recurse(builder, dirty, &zero, list);
    }

    fn build_display_list_for_child(@mut self,
                                    builder: &DisplayListBuilder,
                                    child_flow: @mut FlowContext,
                                    dirty: &Rect<Au>, offset: &Point2D<Au>,
                                    list: &Mut<DisplayList>) {

        // adjust the dirty rect to child flow context coordinates
        let abs_flow_bounds = child_flow.d().position.translate(offset);
        let adj_offset = offset.add(&child_flow.d().position.origin);

        debug!("build_display_list_for_child: rel=%?, abs=%?",
               child_flow.d().position, abs_flow_bounds);
        debug!("build_display_list_for_child: dirty=%?, offset=%?",
               dirty, offset);

        if dirty.intersects(&abs_flow_bounds) {
            debug!("build_display_list_for_child: intersected. recursing into child flow...");
            child_flow.build_display_list_recurse(builder, dirty, &adj_offset, list);
        } else {
            debug!("build_display_list_for_child: Did not intersect...");
        }
    }
}

