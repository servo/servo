#[doc = "

The layout task. Performs layout on the dom, builds display lists and sends
them to be rendered

"];

import task::*;
import comm::*;
import gfx::geom;
import gfx::renderer;
import dom::base::node;
import dom::rcu::scope;
import dom::style::stylesheet;
import /*layout::*/base::*;
import /*layout::*/style::apply::apply_style_methods;
import /*layout::*/style::style::style_methods;
import box_builder::box_builder_methods;
import dl = display_list;

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
                this_box.reflow(geom::px_to_au(800));

                let dlist = build_display_list(this_box);
                to_renderer.send(renderer::render(dlist));
              }
            }
        }
    }
}

fn build_display_list(box: @base::box) -> display_list::display_list {
    let mut list = [box_to_display_item(box)];

    for btree.each_child(box) {|c|
        list += build_display_list(c);
    }

    #debug("display_list: %?", list);
    ret list;
}

fn box_to_display_item(box: @base::box) -> dl::display_item {
    let mut item;
    alt box.appearance.background_image {
      some(image) {
	item = dl::display_item({
	    item_type: dl::display_item_image(~copy *image),
	    bounds: copy box.bounds
	});
      }
      none {
        alt box.appearance.background_color {
          some(col) {
            let red_col = (col >> 16u) & 255u;
            let green_col = (col >> 8u) & 255u;
            let blue_col = col & 255u;

	    item = dl::display_item({
	        item_type: dl::display_item_solid_color(red_col as u8,
                                                        green_col as u8,
                                                        blue_col as u8),
	        bounds: copy box.bounds
	    });
          }
	  none {
            let r = rand::rng();
	    item = dl::display_item({
	        item_type: dl::display_item_solid_color(r.next() as u8,
						        r.next() as u8,
						        r.next() as u8),
	        bounds: copy box.bounds
	    });
          }
        }
      }
    }

    #debug("layout: display item: %?", item);
    ret item;
}
