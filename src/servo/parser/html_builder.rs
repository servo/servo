#[doc="Constructs a DOM tree from an incoming token stream."]

import dom::base::{Attr, Element, ElementData, ElementKind, HTMLDivElement, HTMLHeadElement};
import dom::base::{HTMLImageElement, Node, NodeScope, Text, TreeReadMethods, TreeWriteMethods};
import dom::base::{UnknownElement};
import dom::rcu::WriterMethods;
import geom::size::Size2D;
import gfx::geometry;
import gfx::geometry::au;
import parser = parser::lexer::html;
import parser::token;

import dvec::extensions;

#[warn(no_non_implicitly_copyable_typarams)]
fn link_up_attribute(scope: NodeScope, node: Node, -key: str, -value: str) {
    // TODO: Implement atoms so that we don't always perform string comparisons.
    scope.read(node) {
        |node_contents|
        alt *node_contents.kind {
            Element(element) {
                element.attrs.push(~Attr(copy key, copy value));
                alt *element.kind {
                    HTMLImageElement(img) if key == "width" {
                        alt int::from_str(value) {
                            none {
                                // Drop on the floor.
                            }
                            some(s) { img.size.width = geometry::px_to_au(s); }
                        }
                    }
                    HTMLImageElement(img) if key == "height" {
                        alt int::from_str(value) {
                            none {
                                // Drop on the floor.
                            }
                            some(s) {
                                img.size.height = geometry::px_to_au(s);
                            }
                        }
                    }
                    HTMLDivElement | HTMLImageElement(*) | HTMLHeadElement | UnknownElement {
                        // Drop on the floor.
                    }
                }
            }

            Text(*) {
                fail "attempt to link up an attribute to a text node"
            }
        }
    }
}

fn build_element_kind(tag_name: str) -> ~ElementKind {
    alt tag_name {
        "div"   { ~HTMLDivElement }
        "img"   {
            ~HTMLImageElement({
                mut size: Size2D(geometry::px_to_au(100),
                                 geometry::px_to_au(100))
            })
        }
        "head"  { ~HTMLHeadElement }
        _       { ~UnknownElement  }
    }
}

fn build_dom(scope: NodeScope, stream: port<token>) -> Node {
    // The current reference node.
    let mut cur = scope.new_node(Element(ElementData("html", ~HTMLDivElement)));
    loop {
        let token = stream.recv();
        alt token {
            parser::to_eof { break; }
            parser::to_start_opening_tag(tag_name) {
                #debug["starting tag %s", tag_name];
                let element_kind = build_element_kind(tag_name);
                let new_node = scope.new_node(Element(ElementData(tag_name, element_kind)));
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
                let s <- s;
                let new_node = scope.new_node(Text(s));
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

