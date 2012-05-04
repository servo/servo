export msg;
export content;

import gfx::geom::*;
import dom::rcu::*;
import dom::base::*;
import layout::base::{rd_tree_ops, wr_tree_ops};

enum msg {
    exit
}

fn content(layout: chan<layout::layout::msg>) -> chan<msg> {

    task::spawn_listener::<msg> {|po|
        // TODO: Get a DOM from the parser
        // let s: int = scope();

        // TODO: RCU this stuff over to layout
        loop {
            if po.peek() {
                break;
            } else {
                #debug("content: requesting layout");
                layout.send(layout::layout::build);
                std::timer::sleep(1000u);
            }
        }
    }
}
