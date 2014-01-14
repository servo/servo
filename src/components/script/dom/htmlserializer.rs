/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::namespace;
use dom::attr::Attr;
use dom::node::NodeIterator;
use dom::node::{DoctypeNodeTypeId, DocumentFragmentNodeTypeId, CommentNodeTypeId, DocumentNodeTypeId, ElementNodeTypeId, TextNodeTypeId, AbstractNode};

pub fn serialize(iterator: &mut NodeIterator) -> ~str {
    let mut html = ~"";
    let mut open_elements: ~[~str] = ~[];

    for node in *iterator {
        while open_elements.len() > iterator.depth {
            html.push_str(~"</" + open_elements.pop() + ">");
        }
        html.push_str(
            match node.type_id() {
                ElementNodeTypeId(..) => {
                    serialize_elem(node, &mut open_elements)
                }
                CommentNodeTypeId => {
                    serialize_comment(node)
                }
                TextNodeTypeId => {
                    serialize_text(node)
                }
                DoctypeNodeTypeId => {
                    serialize_doctype(node)
                }
                DocumentFragmentNodeTypeId => {
                    ~""
                }
                DocumentNodeTypeId(_) => {
                    fail!("It shouldn't be possible to serialize a document node")
                }
            }
            );
    }
    while open_elements.len() > 0 {
        html.push_str(~"</" + open_elements.pop() + ">");
    }
    html
}

fn serialize_comment(node: AbstractNode) -> ~str {
    node.with_imm_characterdata(|comment| {
        ~"<!--" + comment.data + "-->"
    })
}

fn serialize_text(node: AbstractNode) -> ~str {
    node.with_imm_characterdata(|text| {
        match node.parent_node() {
            Some(parent) if parent.is_element() => {
                parent.with_imm_element(|elem| {
                    match elem.tag_name.as_slice() {
                        "style" | "script" | "xmp" | "iframe" |
                        "noembed" | "noframes" | "plaintext" |
                        "noscript" if elem.namespace == namespace::HTML => {
                            text.data.clone()
                        },
                        _ => escape(text.data, false)
                    }
               })
            },
            _ => escape(text.data, false)
        }
    })
}

fn serialize_doctype(node: AbstractNode) -> ~str {
    node.with_imm_doctype(|doctype| {
        ~"<!DOCTYPE" + doctype.name + ">"
    })
}

fn serialize_elem(node: AbstractNode, open_elements: &mut ~[~str]) -> ~str {
    node.with_imm_element(|elem| {
        let mut rv = ~"<" + elem.tag_name;
        for attr in elem.attrs.iter() {
            rv.push_str(serialize_attr(attr));
        };
        rv.push_str(">");
        match elem.tag_name.as_slice() {
            "pre" | "listing" | "textarea" if
                elem.namespace == namespace::HTML => {
                    match node.first_child() {
                        Some(child) if child.is_text() => {
                            child.with_imm_characterdata(|text| {
                                if text.data[0] == 0x0A as u8 {
                                    rv.push_str("\x0A");
                                }
                            })
                        },
                        _ => {}
                    }
            },
            _ => {}
        }
        if !elem.is_void() {
            open_elements.push(elem.tag_name.clone());
        }
        rv
    })
}

fn serialize_attr(attr: &@mut Attr) -> ~str {
    let attr_name = if attr.namespace == namespace::XML {
        ~"xml:" + attr.local_name.clone()
    } else if attr.namespace == namespace::XMLNS &&
        attr.local_name.as_slice() == "xmlns" {
        ~"xmlns"
    } else if attr.namespace == namespace::XMLNS {
        ~"xmlns:" + attr.local_name.clone()
    } else if attr.namespace == namespace::XLink {
        ~"xlink:" + attr.local_name.clone()
    } else {
        attr.name.clone()
    };
    ~" " + attr_name + "=\"" + escape(attr.value, true) + "\""
}

fn escape(string: &str, attr_mode: bool) -> ~str {
    let replaced = string.replace("&", "&amp;").replace("\xA0", "&nbsp;");
    match attr_mode {
        true => {
            replaced.replace("\"", "&quot;")
        },
        false => {
            replaced.replace("<", "&lt;").replace(">", "&gt;")
        }
    }
}
