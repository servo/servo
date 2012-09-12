export build_display_list;

use au = gfx::geometry;
use base::Box;
use css::values::{BgColor, BgTransparent, Specified};
use dl = gfx::display_list;
use dom::base::{Text, NodeScope};
use dom::rcu::Scope;
use dvec::DVec;
use either::{Left, Right};
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::geometry::au;
use layout::text::TextBoxData;
use layout::base::{TextBox, BoxTree};
use servo_text::text_run::TextRun;
use util::tree;
use vec::push;

#[doc = "

Builds a display list for a box and all its children

"]
fn build_display_list(box : @Box) -> dl::DisplayList {
    let list = DVec();
    build_display_list_from_origin(list, box, Point2D(au(0), au(0)));
    return list;
}

#[doc="

Builds a display list for a box and all its children.

# Arguments

* `box` - The box to build the display list for.
* `origin` - The coordinates of upper-left corner of the box containing the
             passed-in box.

"]
fn build_display_list_from_origin(list: dl::DisplayList, box: @Box, origin: Point2D<au>) {
    let box_origin = Point2D(
        au::from_px(au::to_px(origin.x) + au::to_px(box.data.position.origin.x)),
        au::from_px(au::to_px(origin.y) + au::to_px(box.data.position.origin.y)));
    #debug("Handed origin %?, box has bounds %?, starting with origin %?", origin, box.data.position.size, box_origin);

    box_to_display_items(list, box, box_origin);

    for BoxTree.each_child(box) |c| {
        #debug("Recursively building display list with origin %?", box_origin);
        build_display_list_from_origin(list, c, box_origin);
    }
}

#[doc="

Creates a display list item for a single block. 

# Arguments 

* `box` - The box to build the display list for
* `origin` - The coordinates of upper-left corner of the passed in box.

"]
#[allow(non_implicitly_copyable_typarams)]
fn box_to_display_items(list: dl::DisplayList, box: @Box, origin: Point2D<au>) {

    // TODO: each box should know how to make its own display items.
    // The display list builder should mainly hold information about
    // the initial request and desired result---for example, is the 
    // DisplayList to be used for painting or hit testing. This can
    // influence which boxes are created.

    // TODO: to implement stacking contexts correctly, we need to
    // create a set of display lists, one per each layer of a stacking
    // context. (CSS 2.1, Section 9.9.1). Each box is passed the list
    // set representing the box's stacking context. When asked to
    // construct its constituent display items, each box puts its
    // DisplayItems into the correct stack layer (according to CSS 2.1
    // Appendix E).  and then builder flattens the list at the end.
    
    #debug("request to display a box from origin %?", origin);

    let bounds : Rect<au> = Rect(origin, copy box.data.position.size);

    match box.kind {
      TextBox(d) => {
        let mut runs = d.runs;
        list.push(~dl::SolidColor(bounds, 255u8, 255u8, 255u8));

        let mut bounds = bounds;
        for uint::range(0, runs.len()) |i| {
            bounds.size.height = runs[i].size().height;
            let glyph_run = text_run_to_dl_glyph_run(& runs[i]);
            list.push(~dl::Glyphs(bounds, glyph_run));
            bounds.origin.y += bounds.size.height;
        }
        return;

        pure fn text_run_to_dl_glyph_run(text_run: &TextRun) ->
                dl::GlyphRun {
            dl::GlyphRun {
                glyphs: copy text_run.glyphs
            }
        }
      }
      _ => {
        // Fall through
      }
    };

    // Check if there is a background image, if not set the background color.
    let image = box.get_image();
    
    if image.is_some() {
        list.push(~dl::Image(bounds, option::unwrap(image)))
    } else {
        // DAC
        // TODO: shouldn't need to unbox CSSValue by now
        let boxed_color = box.node.style().background_color;
        let color = match boxed_color {
            Specified(BgColor(c)) => c,
            Specified(BgTransparent) | _ => util::color::rgba(0,0,0,0.0)
        };
        #debug("Assigning color %? to box with bounds %?", color, bounds);
        list.push(~dl::SolidColor(bounds, color.red, color.green, color.blue));
    }
}


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
