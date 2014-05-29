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
    let mut html = StrBuf::new();
    let mut open_elements: Vec<~str> = vec!();

    for node in *iterator {
        while open_elements.len() > iterator.depth {
            html.push_str("</");
            html.push_str(open_elements.pop().unwrap().as_slice());
            html.push_str(">");
        }
        match node.type_id() {
            ElementNodeTypeId(..) => {
                let elem: &JSRef<Element> = ElementCast::to_ref(&node).unwrap();
                serialize_elem(elem, &mut open_elements, &mut html)
            }
            CommentNodeTypeId => {
                let comment: &JSRef<Comment> = CommentCast::to_ref(&node).unwrap();
                serialize_comment(comment, &mut html)
            }
            TextNodeTypeId => {
                let text: &JSRef<Text> = TextCast::to_ref(&node).unwrap();
                serialize_text(text, &mut html)
            }
            DoctypeNodeTypeId => {
                let doctype: &JSRef<DocumentType> = DocumentTypeCast::to_ref(&node).unwrap();
                serialize_doctype(doctype, &mut html)
            }
            ProcessingInstructionNodeTypeId => {
                let processing_instruction: &JSRef<ProcessingInstruction> =
                    ProcessingInstructionCast::to_ref(&node).unwrap();
                serialize_processing_instruction(processing_instruction, &mut html)
            }
            DocumentFragmentNodeTypeId => {}
            DocumentNodeTypeId => {
                fail!("It shouldn't be possible to serialize a document node")
            }
        }
    }
    while open_elements.len() > 0 {
        html.push_str("</");
        html.push_str(open_elements.pop().unwrap().as_slice());
        html.push_str(">");
    }
    html.into_owned()
}

fn serialize_comment(comment: &JSRef<Comment>, html: &mut StrBuf) {
    html.push_str("<!--");
    html.push_str(comment.deref().characterdata.data);
    html.push_str("-->");
}

fn serialize_text(text: &JSRef<Text>, html: &mut StrBuf) {
    let text_node: &JSRef<Node> = NodeCast::from_ref(text);
    match text_node.parent_node().map(|node| node.root()) {
        Some(ref parent) if parent.is_element() => {
            let elem: &JSRef<Element> = ElementCast::to_ref(&**parent).unwrap();
            match elem.deref().local_name.as_slice() {
                "style" | "script" | "xmp" | "iframe" |
                "noembed" | "noframes" | "plaintext" |
                "noscript" if elem.deref().namespace == namespace::HTML
                => html.push_str(text.deref().characterdata.data),
                _ => escape(text.deref().characterdata.data, false, html)
            }
        }
        _ => escape(text.deref().characterdata.data, false, html)
    }
}

fn serialize_processing_instruction(processing_instruction: &JSRef<ProcessingInstruction>,
                                    html: &mut StrBuf) {
    html.push_str("<?");
    html.push_str(processing_instruction.deref().target);
    html.push_char(' ');
    html.push_str(processing_instruction.deref().characterdata.data);
    html.push_str("?>");
}

fn serialize_doctype(doctype: &JSRef<DocumentType>, html: &mut StrBuf) {
    html.push_str("<!DOCTYPE");
    html.push_str(doctype.deref().name);
    html.push_char('>');
}

fn serialize_elem(elem: &JSRef<Element>, open_elements: &mut Vec<~str>, html: &mut StrBuf) {
    html.push_char('<');
    html.push_str(elem.deref().local_name);
    for attr in elem.deref().attrs.borrow().iter() {
        let attr = attr.root();
        serialize_attr(&*attr, html);
    };
    html.push_char('>');

    match elem.deref().local_name.as_slice() {
        "pre" | "listing" | "textarea" if elem.deref().namespace == namespace::HTML => {
            let node: &JSRef<Node> = NodeCast::from_ref(elem);
            match node.first_child().map(|child| child.root()) {
                Some(ref child) if child.is_text() => {
                    let text: &JSRef<CharacterData> = CharacterDataCast::to_ref(&**child).unwrap();
                    if text.deref().data.len() > 0 && text.deref().data[0] == 0x0A as u8 {
                        html.push_char('\x0A');
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
}

fn serialize_attr(attr: &JSRef<Attr>, html: &mut StrBuf) {
    html.push_char(' ');
    if attr.deref().namespace == namespace::XML {
        html.push_str("xml:");
        html.push_str(attr.deref().local_name);
    } else if attr.deref().namespace == namespace::XMLNS &&
        attr.deref().local_name.as_slice() == "xmlns" {
        html.push_str("xmlns");
    } else if attr.deref().namespace == namespace::XMLNS {
        html.push_str("xmlns:");
        html.push_str(attr.deref().local_name);
    } else if attr.deref().namespace == namespace::XLink {
        html.push_str("xlink:");
        html.push_str(attr.deref().local_name);
    } else {
        html.push_str(attr.deref().name);
    };
    html.push_str("=\"");
    escape(attr.deref().value, true, html);
    html.push_char('"');
}

fn escape(string: &str, attr_mode: bool, html: &mut StrBuf) {
    for c in string.chars() {
        match c {
            '&' => html.push_str("&amp;"),
            '\xA0' => html.push_str("&nbsp;"),
            '"' if attr_mode => html.push_str("&quot;"),
            '<' if !attr_mode => html.push_str("&lt;"),
            '>' if !attr_mode => html.push_str("&gt;"),
            c => html.push_char(c),
        }
    }
}
