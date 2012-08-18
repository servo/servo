export build_display_list;

import base::{Box, BTree, ImageHolder, TextBoxKind};
import dl = display_list;
import dom::base::{Text, NodeScope};
import dom::rcu::Scope;
import either::{Left, Right};
import geom::point::Point2D;
import geom::rect::Rect;
import geom::size::Size2D;
import gfx::geometry::{au, au_to_px, box, px_to_au};
import gfx::renderer;
import util::tree;

import dvec::dvec;
import vec::push;

#[doc = "

Builds a display list for a box and all its children

"]
fn build_display_list(box : @Box) -> dl::display_list {
    let list = dvec();
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
fn build_display_list_from_origin(list: dl::display_list, box: @Box, origin: Point2D<au>) {
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
fn box_to_display_items(list: dl::display_list, box: @Box, origin: Point2D<au>) {
    #debug("request to display a box from origin %?", origin);

    let bounds = Rect(origin, copy box.bounds.size);
    let col = box.appearance.background_color;

    match box.kind {
      TextBoxKind(subbox) => {
        let run = copy subbox.run;
        assert run.is_some();
        list.push(dl::display_item({
            item_type: dl::display_item_solid_color(255u8, 255u8, 255u8),
            bounds: bounds
        }));
        list.push(dl::display_item({
            item_type: dl::display_item_text(run.get()),
            bounds: bounds
        }));
        return;
      }
      _ => {
        // Fall through
      }
    };

    // Check if there is a background image, if not set the background color.
    let image = box.appearance.get_image();
    
    if image.is_some() {
        let display_item = dl::display_item({
            item_type: dl::display_item_image(option::unwrap(image)),
            bounds: bounds
        });
        list.push(display_item);
    } else {
        #debug("Assigning color %? to box with bounds %?", col, bounds);
        let col = box.appearance.background_color;
        list.push(dl::display_item({
            item_type: dl::display_item_solid_color(col.red, col.green, col.blue),
            bounds: bounds
        }));
    }
}


fn should_convert_text_boxes_to_solid_color_background_items() {
    #[test];
    #[ignore(reason = "crashy")];

    let s = Scope();
    let n = s.new_node(Text(~"firecracker"));
    let b = n.construct_boxes();

    let subbox = match check b.kind { TextBoxKind(subbox) => subbox };

    b.reflow_text(subbox);
    let list = dvec();
    box_to_display_items(list, b, Point2D(px_to_au(0), px_to_au(0)));

    match list[0].item_type {
      dl::display_item_solid_color(*) => { }
      _ => { fail }
    }
    
}

fn should_convert_text_boxes_to_text_items() {
    #[test];
    #[ignore(reason = "crashy")];

    let s = Scope();
    let n = s.new_node(Text(~"firecracker"));
    let b = n.construct_boxes();

    let subbox = match check b.kind { TextBoxKind(subbox) => { subbox } };

    b.reflow_text(subbox);
    let list = dvec();
    box_to_display_items(list, b, Point2D(px_to_au(0), px_to_au(0)));

    match list[1].item_type {
      dl::display_item_text(_) => { }
      _ => { fail }
    }
}

fn should_calculate_the_bounds_of_the_text_box_background_color() {
    #[test];
    #[ignore];

    let s = Scope();
    let n = s.new_node(Text(~"firecracker"));
    let b = n.construct_boxes();

    let subbox = match check b.kind { TextBoxKind(subbox) => { subbox } };

    b.reflow_text(subbox);
    let list = dvec();
    box_to_display_items(list, b, Point2D(px_to_au(0), px_to_au(0)));

    let expected = Rect(
        Point2D(px_to_au(0), px_to_au(0)),
        Size2D(px_to_au(84), px_to_au(20))
    );

    assert list[0].bounds == expected;
}

fn should_calculate_the_bounds_of_the_text_items() {
    #[test];
    #[ignore];

    let s = Scope();
    let n = s.new_node(Text(~"firecracker"));
    let b = n.construct_boxes();

    let subbox = match check b.kind { TextBoxKind(subbox) => { subbox } };

    b.reflow_text(subbox);
    let list = dvec();
    box_to_display_items(list, b, Point2D(px_to_au(0), px_to_au(0)));

    let expected = Rect(
        Point2D(px_to_au(0), px_to_au(0)),
        Size2D(px_to_au(84), px_to_au(20))
    );

    assert list[1].bounds == expected;
}
