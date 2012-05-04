export msg;
export content;

import gfx::geom::*;
import dom::rcu::*;
import dom::base::*;
import layout::base::tree; // method implementations of tree for box and node

enum msg {
    exit
}

fn content(layout: chan<layout::layout::msg>) -> chan<msg> {

    task::spawn_listener::<msg> {|po|
        let s = scope();
        let n0 = s.new_node(nk_img(size(int_to_au(10),int_to_au(10))));
        let n1 = s.new_node(nk_img(size(int_to_au(10),int_to_au(15))));
        let n2 = s.new_node(nk_img(size(int_to_au(10),int_to_au(20))));
        let n3 = s.new_node(nk_div);

        tree::add_child(n3, n0);
        tree::add_child(n3, n1);
        tree::add_child(n3, n2);

        let b0 = layout::base::linked_box(n0);
        let b1 = layout::base::linked_box(n1);
        let b2 = layout::base::linked_box(n2);
        let b3 = layout::base::linked_box(n3);

        tree::add_child(b3, b0);
        tree::add_child(b3, b1);
        tree::add_child(b3, b2);

        // TODO: RCU this stuff over to layout
        loop {
            if po.peek() {
                break;
            } else {
                #debug("content: requesting layout");
                layout.send(layout::layout::build);
                std::timer::sleep(100u);
            }
        }
    }
}
