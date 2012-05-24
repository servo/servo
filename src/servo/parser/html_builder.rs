#[doc="Constructs a DOM tree from an incoming token stream."]

import dom::rcu::writer_methods;
import dom::base::{element, es_div, es_img, methods, nk_element, nk_text};
import dom::base::{rd_tree_ops, wr_tree_ops};
import dom = dom::base;
import parser = parser::html;
import html::token;
import gfx::geom;

fn link_up_attribute(scope: dom::node_scope, node: dom::node, key: str,
                     value: str) {
    // TODO: Implement atoms so that we don't always perform string
    // comparisons.
    scope.rd(node) {
        |node_contents|
        alt *node_contents.kind {
            dom::nk_element(element) {
                alt *element.subclass {
                    es_img(dimensions) if key == "width" {
                        alt int::from_str(value) {
                            none { /* drop on the floor */ }
                            some(s) { dimensions.width = geom::px_to_au(s); }
                        }
                    }
                    es_img(dimensions) if key == "height" {
                        alt int::from_str(value) {
                            none { /* drop on the floor */ }
                            some(s) { dimensions.height = geom::px_to_au(s); }
                        }
                    }
                    es_div | es_img(*) {
                        // Drop on the floor.
                    }
                }
            }
            dom::nk_text(*) {
                fail "attempt to link up an attribute to a text node"
            }
        }
    }
}

fn build_dom(scope: dom::node_scope,
             stream: port<token>) -> dom::node {
    // The current reference node.
    let mut cur = scope.new_node(dom::nk_element(element("html", ~es_div)));
    loop {
        let token = stream.recv();
        #debug["token=%?", token];
        alt token {
            parser::to_eof { break; }
            parser::to_start_opening_tag("div") {
                #debug["DIV"];
                let new_node =
                    scope.new_node(dom::nk_element(element("div", ~es_div)));
                scope.add_child(cur, new_node);
                cur = new_node;
            }
            parser::to_start_opening_tag("img") {
                #debug["IMG"];
                let new_node =
                    scope.new_node(dom::nk_element(element("img",
                        ~es_img({mut width: geom::px_to_au(100),
                                 mut height: geom::px_to_au(100)}))));
                scope.add_child(cur, new_node);
                cur = new_node;
            }
            parser::to_start_opening_tag(t) {
                fail ("Unrecognized tag: " + t);
            }
            parser::to_attr(key, value) {
                #debug["attr: %? = %?", key, value];
                link_up_attribute(scope, cur, key, value);
            }
            parser::to_end_opening_tag {
                #debug("end opening tag");
            }
            parser::to_end_tag(_) {
                // TODO: Assert that the closing tag has the right name.
                // TODO: Fail more gracefully (i.e. according to the HTML5
                //       spec) if we close more tags than we open.
                cur = scope.get_parent(cur).get();
            }
            parser::to_text(s) if !s.is_whitespace() {
                let new_node = scope.new_node(dom::nk_text(s));
                scope.add_child(cur, new_node);
            }
            parser::to_text(_) {
                // FIXME: Whitespace should not be ignored.
            }
            parser::to_doctype {
                // TODO: Do something here...
            }
        }
    }
    ret cur;
}

