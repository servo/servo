#[doc = "

The layout task. Performs layout on the dom, builds display lists and sends
them to be rendered

"];

import task::*;
import comm::*;
import gfx::geometry::{au, au_to_px, box, px_to_au};
import geom::point::Point2D;
import geom::rect::Rect;
import gfx::renderer;
import dom::base::node;
import dom::rcu::scope;
import dom::style::stylesheet;
import layout::base::*;
import layout::style::apply::apply_style_methods;
import layout::style::style::style_methods;
import box_builder::box_builder_methods;
import dl = display_list;
import util::color::methods;

enum msg {
    build(node, stylesheet),
    ping(chan<content::ping>),
    exit
}

fn layout(to_renderer: chan<renderer::msg>) -> chan<msg> {
    spawn_listener::<msg> { |po|
        loop {
            alt po.recv() {
              ping(ch) { ch.send(content::pong); }
              exit { break; }
              build(node, styles) {
                #debug("layout: received layout request for:");
                node.dump();

                node.recompute_style_for_subtree(styles);

                let this_box = node.construct_boxes();
                this_box.dump();

		this_box.apply_style_for_subtree();
                this_box.reflow(px_to_au(800));

                let dlist = build_display_list(this_box);
                to_renderer.send(renderer::render(dlist));
              }
            }
        }
    }
}

#[doc="

Builds a display list for a box and all its children.

# Arguments

* `box` - The box to build the display list for.
* `origin` - The coordinates of upper-left corner of the box containing the
             passed-in box.

"]
fn build_display_list_from_origin(box: @base::box, origin: Point2D<au>)
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

fn build_display_list(box : @base::box) -> dl::display_list {
    ret build_display_list_from_origin(box, Point2D(au(0), au(0)));
}

#[doc="

Creates a display list item for a single block. 
Args: 
-box: the box to build the display list for
-origin: the coordinates of upper-left corner of the passed in box.

"]
fn box_to_display_item(box: @base::box, origin: Point2D<au>)
    -> dl::display_item {
    let mut item;

    #debug("request to display a box from origin %?", origin);

    let bounds = Rect(origin, copy box.bounds.size);

    alt (box.appearance.background_image, box.appearance.background_color) {
      (some(image), some(*)) | (some(image), none) {
	item = dl::display_item({
	    item_type: dl::display_item_image(~copy *image),
	    bounds: bounds
	});
      }
      (none, some(col)) {
        #debug("Assigning color %? to box with bounds %?", col, bounds);
	item = dl::display_item({
	    item_type: dl::display_item_solid_color(col.red, col.green,
                                                    col.blue),
	    bounds: bounds
	});
      }
      (none, none) {
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
