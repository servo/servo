// Constructs a DOM tree from an incoming token stream.

import dom::rcu::writer_methods;
import dom::base::{methods, rd_tree_ops, wr_tree_ops};
import dom = dom::base;
import parser = parser::html;
import html::token;
import gfx::geom;

fn build_dom(scope: dom::node_scope,
             stream: port<token>) -> dom::node {
    // The current reference node.
    let mut cur = scope.new_node(dom::nk_div);
    loop {
        let token = stream.recv();
        #debug["token=%?", token];
        alt token {
            parser::to_eof { break; }
            parser::to_start_tag("div") {
                #debug["DIV"];
                let new_node = scope.new_node(
                    dom::nk_div);
                scope.add_child(cur, new_node);
                cur = new_node;
            }
            parser::to_start_tag("img") {
                #debug["IMG"];
                let new_node = scope.new_node(
                    dom::nk_img({mut width: geom::int_to_au(100),
                                 mut height: geom::int_to_au(100)}));
                scope.add_child(cur, new_node);
                cur = new_node;
            }
            parser::to_start_tag(t) {
              fail ("Unrecognized tag: " + t);
            }
            parser::to_end_tag(_) {
                // TODO: Assert that the closing tag has the right name.
                // TODO: Fail more gracefully (i.e. according to the HTML5
                //       spec) if we close more tags than we open.
                cur = scope.get_parent(cur).get();
            }
            parser::to_text(_) {
                // TODO
            }
            parser::to_doctype {
                // TODO: Do something here...
            }
        }
    }
    ret cur;
}
