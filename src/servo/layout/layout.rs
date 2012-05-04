#[doc = "

The layout task. Performs layout on the dom, builds display lists and sends
them to be rendered

"];

import task::*;
import comm::*;
import gfx::geom;
import gfx::geom::*;
import gfx::renderer;
import dom::base::*;
import display_list::*;
import dom::rcu::scope;
import base::tree;

enum msg {
    build,
    exit
}

fn layout(renderer: chan<renderer::msg>) -> chan<msg> {

    spawn_listener::<msg> {|po|

        let s = scope();
        let n0 = s.new_node(nk_img(size(int_to_au(10),int_to_au(10))));
        let n1 = s.new_node(nk_img(size(int_to_au(10),int_to_au(15))));
        let n2 = s.new_node(nk_img(size(int_to_au(10),int_to_au(20))));
        let n3 = s.new_node(nk_div);

        tree::add_child(n3, n0);
        tree::add_child(n3, n1);
        tree::add_child(n3, n2);

        let b0 = base::linked_box(n0);
        let b1 = base::linked_box(n1);
        let b2 = base::linked_box(n2);
        let b3 = base::linked_box(n3);

        tree::add_child(b3, b0);
        tree::add_child(b3, b1);
        tree::add_child(b3, b2);

        loop {
            alt recv(po) {
              build {
                #debug("layout: received layout request");
                base::reflow_block(b3, int_to_au(800));
                let dlist = build_display_list(b3);

                send(renderer, gfx::renderer::render(dlist));
              }
              exit {
                break;
              }
            }
        }
    }

}

fn build_display_list(box: @base::box) -> display_list::display_list {
    let mut list = [box_to_display_item(box)];

    for tree::each_child(box) {|c|
        list += build_display_list(c);
    }

    #debug("display_list: %?", list);
    ret list;
}

fn box_to_display_item(box: @base::box) -> display_item {
    let r = rand::rng();
    let item = display_item({
        item_type: solid_color(r.next() as u8, r.next() as u8, r.next() as u8),
        bounds: box.bounds
    });
    #debug("layout: display item: %?", item);
    ret item;
}
