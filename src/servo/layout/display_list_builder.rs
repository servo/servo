export build_display_list;

use css::values::{BgColor, BgTransparent, Specified};
use base::{Box, BTree, ImageHolder, TextBoxKind};
use dl = layout::display_list;
use dom::base::{Text, NodeScope};
use dom::rcu::Scope;
use either::{Left, Right};
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::geometry::{au, au_to_px, box, px_to_au};
use util::tree;

use dvec::DVec;
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
        px_to_au(au_to_px(origin.x) + au_to_px(box.bounds.origin.x)),
        px_to_au(au_to_px(origin.y) + au_to_px(box.bounds.origin.y)));
    #debug("Handed origin %?, box has bounds %?, starting with origin %?", origin, copy box.bounds, box_origin);

    box_to_display_items(list, box, box_origin);

    for BTree.each_child(box) |c| {
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
    #debug("request to display a box from origin %?", origin);

    let bounds = Rect(origin, copy box.bounds.size);

    match box.kind {
      TextBoxKind(subbox) => {
        let run = copy subbox.run;
        assert run.is_some();
        list.push(dl::DisplayItem {
            item: dl::SolidColor(255u8, 255u8, 255u8),
            bounds: bounds
        } );
        list.push(dl::DisplayItem {
            item: dl::Text(run.get()),
            bounds: bounds
        });
        return;
      }
      _ => {
        // Fall through
      }
    };

    // Check if there is a background image, if not set the background color.
    let image = box.appearance.get_image();
    
    if image.is_some() {
        let display_item = dl::DisplayItem {
            item: dl::Image(option::unwrap(image)),
            bounds: bounds
        };
        list.push(display_item);
    } else {
        // DAC
        // TODO: shouldn't need to unbox CSSValue by now
        let boxed_color = box.node.get_specified_style().background_color;
        let color = match boxed_color {
            Specified(BgColor(c)) => c,
            Specified(BgTransparent) | _ => util::color::rgba(0,0,0,0.0)
        };
        #debug("Assigning color %? to box with bounds %?", color, bounds);
        list.push(dl::DisplayItem {
            item: dl::SolidColor(color.red, color.green, color.blue),
            bounds: bounds
        });
    }
}


fn should_convert_text_boxes_to_solid_color_background_items() {
    #[test];

    let s = Scope();
    let n = s.new_node(Text(~"firecracker"));
    let b = n.construct_boxes().get();

    let subbox = match b.kind {
      TextBoxKind(subbox) => subbox,
      _ => fail
    };

    b.reflow_text(subbox);
    let list = DVec();
    box_to_display_items(list, b, Point2D(px_to_au(0), px_to_au(0)));

    do list.borrow |l| {
        match l[0].item {
            dl::SolidColor(*) => { }
            _ => { fail }
        }
    }    
}

fn should_convert_text_boxes_to_text_items() {
    #[test];

    let s = Scope();
    let n = s.new_node(Text(~"firecracker"));
    let b = n.construct_boxes().get();

    let subbox = match b.kind {
      TextBoxKind(subbox) => { subbox },
      _ => fail
    };

    b.reflow_text(subbox);
    let list = DVec();
    box_to_display_items(list, b, Point2D(px_to_au(0), px_to_au(0)));

    do list.borrow |l| {
        match l[1].item {
            dl::Text(_) => { }
            _ => { fail }
        }
    }
}

fn should_calculate_the_bounds_of_the_text_box_background_color() {
    #[test];
    #[ignore(cfg(target_os = "macos"))];

    let s = Scope();
    let n = s.new_node(Text(~"firecracker"));
    let b = n.construct_boxes().get();

    let subbox = match b.kind {
      TextBoxKind(subbox) => { subbox },
      _ => fail
    };

    b.reflow_text(subbox);
    let list = DVec();
    box_to_display_items(list, b, Point2D(px_to_au(0), px_to_au(0)));

    let expected = Rect(
        Point2D(px_to_au(0), px_to_au(0)),
        Size2D(px_to_au(84), px_to_au(20))
    );

    do list.borrow |l| { assert l[0].bounds == expected }
}

fn should_calculate_the_bounds_of_the_text_items() {
    #[test];
    #[ignore(cfg(target_os = "macos"))];

    let s = Scope();
    let n = s.new_node(Text(~"firecracker"));
    let b = n.construct_boxes().get();

    let subbox = match b.kind {
      TextBoxKind(subbox) => { subbox },
      _ => fail
    };

    b.reflow_text(subbox);
    let list = DVec();
    box_to_display_items(list, b, Point2D(px_to_au(0), px_to_au(0)));

    let expected = Rect(
        Point2D(px_to_au(0), px_to_au(0)),
        Size2D(px_to_au(84), px_to_au(20))
    );

    do list.borrow |l| { assert l[1].bounds == expected; }
}
