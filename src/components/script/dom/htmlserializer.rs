/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_util::namespace;
use dom::attr::Attr;
use dom::bindings::codegen::InheritTypes::{ElementCast, TextCast, CommentCast};
use dom::bindings::codegen::InheritTypes::{DocumentTypeCast, CharacterDataCast};
use dom::bindings::codegen::InheritTypes::ProcessingInstructionCast;
use dom::bindings::js::{JS, JSRef, RootCollection};
use dom::characterdata::CharacterData;
use dom::comment::Comment;
use dom::documenttype::DocumentType;
use dom::element::Element;
use dom::node::NodeIterator;
use dom::node::{DoctypeNodeTypeId, DocumentFragmentNodeTypeId, CommentNodeTypeId};
use dom::node::{DocumentNodeTypeId, ElementNodeTypeId, ProcessingInstructionNodeTypeId};
use dom::node::{TextNodeTypeId, NodeHelpers};
use dom::processinginstruction::ProcessingInstruction;
use dom::text::Text;

pub fn serialize(iterator: &mut NodeIterator) -> ~str {
    let roots = RootCollection::new();
    let mut html = ~"";
    let mut open_elements: Vec<~str> = vec!();

    for node in *iterator {
        while open_elements.len() > iterator.depth {
            html.push_str(~"</" + open_elements.pop().unwrap().as_slice() + ">");
        }
        html.push_str(
            match node.type_id() {
                ElementNodeTypeId(..) => {
                    let elem: JS<Element> = ElementCast::to(&node).unwrap();
                    let elem = elem.root(&roots);
                    serialize_elem(&elem.root_ref(), &mut open_elements)
                }
                CommentNodeTypeId => {
                    let comment: JS<Comment> = CommentCast::to(&node).unwrap();
                    let comment = comment.root(&roots);
                    serialize_comment(&comment.root_ref())
                }
                TextNodeTypeId => {
                    let text: JS<Text> = TextCast::to(&node).unwrap();
                    let text = text.root(&roots);
                    serialize_text(&text.root_ref())
                }
                DoctypeNodeTypeId => {
                    let doctype: JS<DocumentType> = DocumentTypeCast::to(&node).unwrap();
                    let doctype = doctype.root(&roots);
                    serialize_doctype(&doctype.root_ref())
                }
                ProcessingInstructionNodeTypeId => {
                    let processing_instruction: JS<ProcessingInstruction> = ProcessingInstructionCast::to(&node).unwrap();
                    let processing_instruction = processing_instruction.root(&roots);
                    serialize_processing_instruction(&processing_instruction.root_ref())
                }
                DocumentFragmentNodeTypeId => {
                    ~""
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
    ~"<!--" + comment.get().characterdata.data + "-->"
}

fn serialize_text(text: &JSRef<Text>) -> ~str {
    match text.get().characterdata.node.parent_node {
        Some(ref parent) if parent.is_element() => {
            let elem: JS<Element> = ElementCast::to(parent).unwrap();
            match elem.get().local_name.as_slice() {
                "style" | "script" | "xmp" | "iframe" |
                "noembed" | "noframes" | "plaintext" |
                "noscript" if elem.get().namespace == namespace::HTML => {
                    text.get().characterdata.data.clone()
                },
                _ => escape(text.get().characterdata.data, false)
            }
        }
        _ => escape(text.get().characterdata.data, false)
    }
}

fn serialize_processing_instruction(processing_instruction: &JSRef<ProcessingInstruction>) -> ~str {
    ~"<?" + processing_instruction.get().target + " " + processing_instruction.get().characterdata.data + "?>"
}

fn serialize_doctype(doctype: &JSRef<DocumentType>) -> ~str {
    ~"<!DOCTYPE" + doctype.get().name + ">"
}

fn serialize_elem(elem: &JSRef<Element>, open_elements: &mut Vec<~str>) -> ~str {
    let roots = RootCollection::new();
    let mut rv = ~"<" + elem.get().local_name;
    for attr in elem.get().attrs.iter() {
        let attr = attr.root(&roots);
        rv.push_str(serialize_attr(&attr.root_ref()));
    };
    rv.push_str(">");
    match elem.get().local_name.as_slice() {
        "pre" | "listing" | "textarea" if elem.get().namespace == namespace::HTML => {
            match elem.get().node.first_child {
                Some(ref child) if child.is_text() => {
                    let text: JS<CharacterData> = CharacterDataCast::to(child).unwrap();
                    if text.get().data.len() > 0 && text.get().data[0] == 0x0A as u8 {
                        rv.push_str("\x0A");
                    }
                },
                _ => {}
            }
        },
        _ => {}
    }
    if !elem.get().is_void() {
        open_elements.push(elem.get().local_name.clone());
    }
    rv
}

fn serialize_attr(attr: &JSRef<Attr>) -> ~str {
    let attr_name = if attr.get().namespace == namespace::XML {
        ~"xml:" + attr.get().local_name.clone()
    } else if attr.get().namespace == namespace::XMLNS &&
        attr.get().local_name.as_slice() == "xmlns" {
        ~"xmlns"
    } else if attr.get().namespace == namespace::XMLNS {
        ~"xmlns:" + attr.get().local_name.clone()
    } else if attr.get().namespace == namespace::XLink {
        ~"xlink:" + attr.get().local_name.clone()
    } else {
        attr.get().name.clone()
    };
    ~" " + attr_name + "=\"" + escape(attr.get().value, true) + "\""
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
