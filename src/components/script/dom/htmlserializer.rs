/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_util::namespace;
use dom::attr::Attr;
use dom::bindings::codegen::InheritTypes::{ElementCast, TextCast, CommentCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{DocumentTypeCast, CharacterDataCast};
use dom::bindings::codegen::InheritTypes::ProcessingInstructionCast;
use dom::bindings::js::JSRef;
use dom::characterdata::CharacterData;
use dom::comment::Comment;
use dom::documenttype::DocumentType;
use dom::element::Element;
use dom::node::{Node, NodeIterator};
use dom::node::{DoctypeNodeTypeId, DocumentFragmentNodeTypeId, CommentNodeTypeId};
use dom::node::{DocumentNodeTypeId, ElementNodeTypeId, ProcessingInstructionNodeTypeId};
use dom::node::{TextNodeTypeId, NodeHelpers};
use dom::processinginstruction::ProcessingInstruction;
use dom::text::Text;

pub fn serialize(iterator: &mut NodeIterator) -> ~str {
    let mut html = "".to_owned();
    let mut open_elements: Vec<~str> = vec!();

    for node in *iterator {
        while open_elements.len() > iterator.depth {
            html.push_str(~"</" + open_elements.pop().unwrap().as_slice() + ">");
        }
        html.push_str(
            match node.type_id() {
                ElementNodeTypeId(..) => {
                    let elem: &JSRef<Element> = ElementCast::to_ref(&node).unwrap();
                    serialize_elem(elem, &mut open_elements)
                }
                CommentNodeTypeId => {
                    let comment: &JSRef<Comment> = CommentCast::to_ref(&node).unwrap();
                    serialize_comment(comment)
                }
                TextNodeTypeId => {
                    let text: &JSRef<Text> = TextCast::to_ref(&node).unwrap();
                    serialize_text(text)
                }
                DoctypeNodeTypeId => {
                    let doctype: &JSRef<DocumentType> = DocumentTypeCast::to_ref(&node).unwrap();
                    serialize_doctype(doctype)
                }
                ProcessingInstructionNodeTypeId => {
                    let processing_instruction: &JSRef<ProcessingInstruction> =
                        ProcessingInstructionCast::to_ref(&node).unwrap();
                    serialize_processing_instruction(processing_instruction)
                }
                DocumentFragmentNodeTypeId => {
                    "".to_owned()
                }
                DocumentNodeTypeId => {
                    fail!("It shouldn't be possible to serialize a document node")
                }
            }
        );
    }
    while open_elements.len() > 0 {
        html.push_str(~"</" + open_elements.pop().unwrap().as_slice() + ">");
    }
    html
}

fn serialize_comment(comment: &JSRef<Comment>) -> ~str {
    ~"<!--" + comment.deref().characterdata.data + "-->"
}

fn serialize_text(text: &JSRef<Text>) -> ~str {
    let text_node: &JSRef<Node> = NodeCast::from_ref(text);
    match text_node.parent_node().map(|node| node.root()) {
        Some(ref parent) if parent.is_element() => {
            let elem: &JSRef<Element> = ElementCast::to_ref(&**parent).unwrap();
            match elem.deref().local_name.as_slice() {
                "style" | "script" | "xmp" | "iframe" |
                "noembed" | "noframes" | "plaintext" |
                "noscript" if elem.deref().namespace == namespace::HTML => {
                    text.deref().characterdata.data.clone()
                },
                _ => escape(text.deref().characterdata.data, false)
            }
        }
        _ => escape(text.deref().characterdata.data, false)
    }
}

fn serialize_processing_instruction(processing_instruction: &JSRef<ProcessingInstruction>) -> ~str {
    ~"<?" + processing_instruction.deref().target + " " + processing_instruction.deref().characterdata.data + "?>"
}

fn serialize_doctype(doctype: &JSRef<DocumentType>) -> ~str {
    ~"<!DOCTYPE" + doctype.deref().name + ">"
}

fn serialize_elem(elem: &JSRef<Element>, open_elements: &mut Vec<~str>) -> ~str {
    let mut rv = ~"<" + elem.deref().local_name;
    for attr in elem.deref().attrs.iter() {
        let attr = attr.root();
        rv.push_str(serialize_attr(&*attr));
    };
    rv.push_str(">");
    match elem.deref().local_name.as_slice() {
        "pre" | "listing" | "textarea" if elem.deref().namespace == namespace::HTML => {
            let node: &JSRef<Node> = NodeCast::from_ref(elem);
            match node.first_child().map(|child| child.root()) {
                Some(ref child) if child.is_text() => {
                    let text: &JSRef<CharacterData> = CharacterDataCast::to_ref(&**child).unwrap();
                    if text.deref().data.len() > 0 && text.deref().data[0] == 0x0A as u8 {
                        rv.push_str("\x0A");
                    }
                },
                _ => {}
            }
        },
        _ => {}
    }
    if !elem.deref().is_void() {
        open_elements.push(elem.deref().local_name.clone());
    }
    rv
}

fn serialize_attr(attr: &JSRef<Attr>) -> ~str {
    let attr_name = if attr.deref().namespace == namespace::XML {
        ~"xml:" + attr.deref().local_name.clone()
    } else if attr.deref().namespace == namespace::XMLNS &&
        attr.deref().local_name.as_slice() == "xmlns" {
        ~"xmlns"
    } else if attr.deref().namespace == namespace::XMLNS {
        ~"xmlns:" + attr.deref().local_name.clone()
    } else if attr.deref().namespace == namespace::XLink {
        ~"xlink:" + attr.deref().local_name.clone()
    } else {
        attr.deref().name.clone()
    };
    ~" " + attr_name + "=\"" + escape(attr.deref().value, true) + "\""
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
