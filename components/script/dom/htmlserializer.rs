/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrHelpers};
use dom::bindings::codegen::InheritTypes::{ElementCast, TextCast, CommentCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{DocumentTypeCast, CharacterDataCast};
use dom::bindings::codegen::InheritTypes::ProcessingInstructionCast;
use dom::bindings::js::JSRef;
use dom::characterdata::CharacterData;
use dom::comment::Comment;
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementHelpers};
use dom::node::{Node, NodeIterator};
use dom::node::{DoctypeNodeTypeId, DocumentFragmentNodeTypeId, CommentNodeTypeId};
use dom::node::{DocumentNodeTypeId, ElementNodeTypeId, ProcessingInstructionNodeTypeId};
use dom::node::{TextNodeTypeId, NodeHelpers};
use dom::processinginstruction::ProcessingInstruction;
use dom::text::Text;

use string_cache::Atom;

#[allow(unrooted_must_root)]
pub fn serialize(iterator: &mut NodeIterator) -> String {
    let mut html = String::new();
    let mut open_elements: Vec<String> = vec!();
    let depth = iterator.depth;
    for node in *iterator {
        while open_elements.len() > depth {
            html.push_str("</");
            html.push_str(open_elements.pop().unwrap().as_slice());
            html.push_str(">");
        }
        match node.type_id() {
            ElementNodeTypeId(..) => {
                let elem: JSRef<Element> = ElementCast::to_ref(node).unwrap();
                serialize_elem(elem, &mut open_elements, &mut html)
            }
            CommentNodeTypeId => {
                let comment: JSRef<Comment> = CommentCast::to_ref(node).unwrap();
                serialize_comment(comment, &mut html)
            }
            TextNodeTypeId => {
                let text: JSRef<Text> = TextCast::to_ref(node).unwrap();
                serialize_text(text, &mut html)
            }
            DoctypeNodeTypeId => {
                let doctype: JSRef<DocumentType> = DocumentTypeCast::to_ref(node).unwrap();
                serialize_doctype(doctype, &mut html)
            }
            ProcessingInstructionNodeTypeId => {
                let processing_instruction: JSRef<ProcessingInstruction> =
                    ProcessingInstructionCast::to_ref(node).unwrap();
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
    html
}

fn serialize_comment(comment: JSRef<Comment>, html: &mut String) {
    html.push_str("<!--");
    html.push_str(comment.deref().characterdata.data.borrow().as_slice());
    html.push_str("-->");
}

fn serialize_text(text: JSRef<Text>, html: &mut String) {
    let text_node: JSRef<Node> = NodeCast::from_ref(text);
    match text_node.parent_node().map(|node| node.root()) {
        Some(ref parent) if parent.is_element() => {
            let elem: JSRef<Element> = ElementCast::to_ref(**parent).unwrap();
            match elem.deref().local_name.as_slice() {
                "style" | "script" | "xmp" | "iframe" |
                "noembed" | "noframes" | "plaintext" |
                "noscript" if elem.deref().namespace == ns!(HTML)
                => html.push_str(text.deref().characterdata.data.borrow().as_slice()),
                _ => escape(text.deref().characterdata.data.borrow().as_slice(), false, html)
            }
        }
        _ => escape(text.deref().characterdata.data.borrow().as_slice(), false, html)
    }
}

fn serialize_processing_instruction(processing_instruction: JSRef<ProcessingInstruction>,
                                    html: &mut String) {
    html.push_str("<?");
    html.push_str(processing_instruction.deref().target.as_slice());
    html.push_char(' ');
    html.push_str(processing_instruction.deref().characterdata.data.borrow().as_slice());
    html.push_str("?>");
}

fn serialize_doctype(doctype: JSRef<DocumentType>, html: &mut String) {
    html.push_str("<!DOCTYPE");
    html.push_str(doctype.deref().name.as_slice());
    html.push_char('>');
}

fn serialize_elem(elem: JSRef<Element>, open_elements: &mut Vec<String>, html: &mut String) {
    html.push_char('<');
    html.push_str(elem.deref().local_name.as_slice());
    for attr in elem.deref().attrs.borrow().iter() {
        let attr = attr.root();
        serialize_attr(*attr, html);
    };
    html.push_char('>');

    match elem.deref().local_name.as_slice() {
        "pre" | "listing" | "textarea" if elem.deref().namespace == ns!(HTML) => {
            let node: JSRef<Node> = NodeCast::from_ref(elem);
            match node.first_child().map(|child| child.root()) {
                Some(ref child) if child.is_text() => {
                    let text: JSRef<CharacterData> = CharacterDataCast::to_ref(**child).unwrap();
                    if text.deref().data.borrow().len() > 0 && text.deref().data.borrow().as_slice().char_at(0) == '\n' {
                        html.push_char('\x0A');
                    }
                },
                _ => {}
            }
        },
        _ => {}
    }

    if !(elem.is_void()) {
        open_elements.push(elem.deref().local_name.as_slice().to_string());
    }
}

fn serialize_attr(attr: JSRef<Attr>, html: &mut String) {
    html.push_char(' ');
    if attr.deref().namespace == ns!(XML) {
        html.push_str("xml:");
        html.push_str(attr.local_name().as_slice());
    } else if attr.deref().namespace == ns!(XMLNS) &&
        *attr.local_name() == Atom::from_slice("xmlns") {
        html.push_str("xmlns");
    } else if attr.deref().namespace == ns!(XMLNS) {
        html.push_str("xmlns:");
        html.push_str(attr.local_name().as_slice());
    } else if attr.deref().namespace == ns!(XLink) {
        html.push_str("xlink:");
        html.push_str(attr.local_name().as_slice());
    } else {
        html.push_str(attr.deref().name.as_slice());
    };
    html.push_str("=\"");
    escape(attr.value().as_slice(), true, html);
    html.push_char('"');
}

fn escape(string: &str, attr_mode: bool, html: &mut String) {
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
