#[doc="Constructs a DOM tree from an incoming token stream."]

import dom::rcu::writer_methods;
import dom::base::{attr, element, element_subclass, es_div, es_head, es_img};
import dom::base::{es_unknown, methods, nk_element, nk_text, rd_tree_ops};
import dom::base::{wr_tree_ops};
import dom = dom::base;
import parser = parser::lexer::html;
import parser::token;
import gfx::geom;
import dvec::extensions;

fn link_up_attribute(scope: dom::node_scope, node: dom::node, key: str,
                     value: str) {
    // TODO: Implement atoms so that we don't always perform string
    // comparisons.
    scope.rd(node) {
        |node_contents|
        alt *node_contents.kind {
            dom::nk_element(element) {
                element.attrs.push(~attr(key, value));
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
                    es_div | es_img(*) | es_head | es_unknown {
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

fn build_element_subclass(tag_name: str) -> ~element_subclass {
    alt tag_name {
        "div" { ret ~es_div; }
        "img" {
            ret ~es_img({
                mut width: geom::px_to_au(100),
                mut height: geom::px_to_au(100)
            });
        }
        "head" { ret ~es_head; }
        _ { ret ~es_unknown; }
    }
}

fn build_dom(scope: dom::node_scope,
             stream: port<token>) -> dom::node {
    // The current reference node.
    let mut cur = scope.new_node(dom::nk_element(element("html", ~es_div)));
    loop {
        let token = stream.recv();
        alt token {
            parser::to_eof { break; }
            parser::to_start_opening_tag(tag_name) {
                #debug["starting tag %s", tag_name];
                let element_subclass = build_element_subclass(tag_name);
                let new_node =
                    scope.new_node(dom::nk_element(element(tag_name,
                                                           element_subclass)));
                scope.add_child(cur, new_node);
                cur = new_node;
            }
            parser::to_attr(key, value) {
                #debug["attr: %? = %?", key, value];
                link_up_attribute(scope, cur, key, value);
            }
            parser::to_end_opening_tag {
                #debug("end opening tag");
            }
            parser::to_end_tag(_) | parser::to_self_close_tag {
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

