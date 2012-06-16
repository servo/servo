export build_display_list;

import dl = display_list;
import dom::rcu::Scope;
import dom::base::{Text, NodeScope};
import gfx::geometry::{au, au_to_px, box, px_to_au};
import gfx::renderer;
import layout::base::*;
import util::color::methods;
import util::tree;
import box_builder::box_builder_methods;
import text::text_layout_methods;
import geom::size::Size2D;
import geom::point::Point2D;
import geom::rect::Rect;

#[doc = "

Builds a display list for a box and all its children

"]
fn build_display_list(box : @Box) -> dl::display_list {
    ret build_display_list_from_origin(box, Point2D(au(0), au(0)));
}

#[doc="

Builds a display list for a box and all its children.

# Arguments

* `box` - The box to build the display list for.
* `origin` - The coordinates of upper-left corner of the box containing the
             passed-in box.

"]
fn build_display_list_from_origin(box: @Box, origin: Point2D<au>)
    -> dl::display_list {
    let box_origin = Point2D(
        px_to_au(au_to_px(origin.x) + au_to_px(box.bounds.origin.x)),
        px_to_au(au_to_px(origin.y) + au_to_px(box.bounds.origin.y)));
    #debug("Handed origin %?, box has bounds %?, starting with origin %?", origin, copy box.bounds, box_origin);

    let mut list = [box_to_display_item(box, box_origin)];

    for btree.each_child(box) {|c|
        #debug("Recursively building display list with origin %?", box_origin);
        list += build_display_list_from_origin(c, box_origin);
    }

    #debug("display_list: %?", list);
    ret list;
}

#[doc="

Creates a display list item for a single block. 
Args: 
-box: the box to build the display list for
-origin: the coordinates of upper-left corner of the passed in box.

"]
fn box_to_display_item(box: @Box, origin: Point2D<au>) -> dl::display_item {
    let mut item;

    #debug("request to display a box from origin %?", origin);

    let bounds = Rect(origin, copy box.bounds.size);

    alt (box.kind, box.appearance.background_image, box.appearance.background_color) {
      (TextBox(subbox), _, _) {
        let run = copy subbox.run;
        assert run.is_some();
        item = dl::display_item({
            item_type: dl::display_item_text(run.get()),
            bounds: bounds
        });
      }
      (_, some(image), some(*)) | (_, some(image), none) {
        item = dl::display_item({
            item_type: dl::display_item_image(~copy *image),
            bounds: bounds
        });
      }
      (_, none, some(col)) {
        #debug("Assigning color %? to box with bounds %?", col, bounds);
        item = dl::display_item({
            item_type: dl::display_item_solid_color(col.red, col.green, col.blue),
            bounds: bounds
        });
      }
      (_, none, none) {
        let r = rand::rng();
        item = dl::display_item({
            item_type: dl::display_item_solid_color(r.next() as u8,
                                                    r.next() as u8,
                                                    r.next() as u8),
            bounds: bounds
        });
      }
    }

    #debug("layout: display item: %?", item);
    ret item;
}

fn should_convert_text_boxes_to_text_items() {
    #[test];

    let s = Scope();
    let n = s.new_node(Text("firecracker"));
    let b = n.construct_boxes();
    let subbox = alt check b.kind { TextBox(subbox) { subbox } };
    b.reflow_text(px_to_au(800), subbox);
    let di = box_to_display_item(b, Point2D(px_to_au(0), px_to_au(0)));

    alt di.item_type {
      dl::display_item_text(_) { }
      _ { fail }
    }
}

fn should_calculate_the_bounds_of_the_text_box() {
    #[test];

    let s = Scope();
    let n = s.new_node(Text("firecracker"));
    let b = n.construct_boxes();
    let subbox = alt check b.kind { TextBox(subbox) { subbox } };
    b.reflow_text(px_to_au(800), subbox);
    let di = box_to_display_item(b, Point2D(px_to_au(0), px_to_au(0)));

    let expected = Rect(
        Point2D(px_to_au(0), px_to_au(0)),
        Size2D(px_to_au(110), px_to_au(14))
    );

    #error("%?", di.bounds);
    #error("%?", expected);

    assert di.bounds == expected;
}
