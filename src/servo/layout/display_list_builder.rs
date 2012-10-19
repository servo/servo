export DisplayListBuilder;

use au = gfx::geometry;
use au::Au;
use css::values::{BgColor, BgColorTransparent, Specified};
use dl = gfx::display_list;
use dom::node::{Text, NodeScope};
use dom::cow::Scope;
use dvec::DVec;
use either::{Left, Right};
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use layout::box::{RenderBox, TextBox};
use layout::context::LayoutContext;
use layout::flow::FlowContext;
use layout::text::TextBoxData;
use servo_text::text_run::TextRun;
use util::tree;
use vec::push;

/** A builder object that manages display list builder should mainly
 hold information about the initial request and desired result---for
 example, whether the DisplayList to be used for painting or hit
 testing. This can affect which boxes are created.

 Right now, the builder isn't used for much, but it  establishes the
 pattern we'll need once we support DL-based hit testing &c.  */
pub struct DisplayListBuilder {
    ctx:  &LayoutContext,
}


trait FlowDisplayListBuilderMethods {
    fn build_display_list(@self, a: &DisplayListBuilder, b: &Rect<Au>, c: &dl::DisplayList);

    fn build_display_list_for_child(@self, a: &DisplayListBuilder, b: @FlowContext,
                                    c: &Rect<Au>, d: &Point2D<Au>, e: &dl::DisplayList);
}

impl FlowContext: FlowDisplayListBuilderMethods {
    fn build_display_list(@self, builder: &DisplayListBuilder, dirty: &Rect<Au>, list: &dl::DisplayList) {
        let zero = au::zero_point();
        self.build_display_list_recurse(builder, dirty, &zero, list);
    }

    fn build_display_list_for_child(@self, builder: &DisplayListBuilder, child_flow: @FlowContext,
                                    dirty: &Rect<Au>, offset: &Point2D<Au>,
                                    list: &dl::DisplayList) {

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

/* TODO: redo unit tests, if possible?gn

fn should_convert_text_boxes_to_solid_color_background_items() {
    #[test];

    use layout::box_builder::LayoutTreeBuilder;

    let s = Scope();
    let n = s.new_node(Text(~"firecracker"));
    let builder = LayoutTreeBuilder();
    let b = builder.construct_trees(n).get();

    b.reflow_text();
    let list = DVec();
    box_to_display_items(list, b, Point2D(au::from_px(0), au::from_px(0)));

    do list.borrow |l| {
        match l[0].data {
            dl::SolidColorData(*) => { }
            _ => { fail }
        }
    }    
}

fn should_convert_text_boxes_to_text_items() {
    #[test];
    use layout::box_builder::LayoutTreeBuilder;

    let s = Scope();
    let n = s.new_node(Text(~"firecracker"));
    let builder = LayoutTreeBuilder();
    let b = builder.construct_trees(n).get();

    b.reflow_text();
    let list = DVec();
    box_to_display_items(list, b, Point2D(au::from_px(0), au::from_px(0)));

    do list.borrow |l| {
        match l[1].data {
            dl::GlyphData(_) => { }
            _ => { fail }
        }
    }
}

fn should_calculate_the_bounds_of_the_text_box_background_color() {
    #[test];
    #[ignore(cfg(target_os = "macos"))];
    use layout::box_builder::LayoutTreeBuilder;

    let s = Scope();
    let n = s.new_node(Text(~"firecracker"));
    let builder = LayoutTreeBuilder();
    let b = builder.construct_trees(n).get();

    b.reflow_text();
    let list = DVec();
    box_to_display_items(list, b, Point2D(au::from_px(0), au::from_px(0)));

    let expected = Rect(
        Point2D(au::from_px(0), au::from_px(0)),
        Size2D(au::from_px(84), au::from_px(20))
    );

    do list.borrow |l| { assert l[0].bounds == expected }
}

fn should_calculate_the_bounds_of_the_text_items() {
    #[test];
    #[ignore(reason = "busted")];
    use layout::box_builder::LayoutTreeBuilder;

    let s = Scope();
    let n = s.new_node(Text(~"firecracker"));
    let builder = LayoutTreeBuilder();
    let b = builder.construct_trees(n).get();

    b.reflow_text();
    let list = DVec();
    box_to_display_items(list, b, Point2D(au::from_px(0), au::from_px(0)));

    let expected = Rect(
        Point2D(au::from_px(0), au::from_px(0)),
        Size2D(au::from_px(84), au::from_px(20))
    );

    do list.borrow |l| { assert l[1].bounds == expected; }
}
*/
