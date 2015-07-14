/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TextBinding::{self, TextMethods};
use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::{CharacterDataCast, TextDerived};
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Root, RootedReference};
use dom::characterdata::{CharacterData, CharacterDataHelpers, CharacterDataTypeId};
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::node::{Node, NodeHelpers, NodeTypeId};
use util::str::DOMString;

/// An HTML text node.
#[dom_struct]
#[derive(HeapSizeOf)]
pub struct Text {
    characterdata: CharacterData,
}

impl TextDerived for EventTarget {
    fn is_text(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::CharacterData(CharacterDataTypeId::Text))
    }
}

impl Text {
    fn new_inherited(text: DOMString, document: &Document) -> Text {
        Text {
            characterdata: CharacterData::new_inherited(CharacterDataTypeId::Text, text, document)
        }
    }

    pub fn new(text: DOMString, document: &Document) -> Root<Text> {
        Node::reflect_node(box Text::new_inherited(text, document),
                           document, TextBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef, text: DOMString) -> Fallible<Root<Text>> {
        let document = global.as_window().Document();
        Ok(Text::new(text, document.r()))
    }
}

impl<'a> TextMethods for &'a Text {
    // https://dom.spec.whatwg.org/#dom-text-splittextoffset
    fn SplitText(self, offset: u32) -> Fallible<Root<Text>> {
        let cdata = CharacterDataCast::from_ref(self);
        // Step 1.
        let length = cdata.Length();
        if offset > length {
            // Step 2.
            return Err(Error::IndexSize);
        }
        // Step 3.
        let count = length - offset;
        // Step 4.
        let new_data = cdata.SubstringData(offset, count).unwrap();
        // Step 5.
        let node = NodeCast::from_ref(self);
        let owner_doc = node.owner_doc();
        let new_text = owner_doc.r().CreateTextNode(new_data);
        // Step 6.
        let parent = node.GetParentNode();
        if let Some(ref parent) = parent {
            // Step 7.1.
            let new_node = NodeCast::from_ref(new_text.r());
            parent.InsertBefore(new_node, node.GetNextSibling().r()).unwrap();
            for range in parent.owner_doc().ranges() {
                match range.upgrade() {
                    Some(range) => {
                        let (start_node, start_offset, end_node, end_offset) = {
                            let range = range.borrow();
                            let start = &range.start();
                            let end = &range.end();
                            (start.node(), start.offset(), end.node(), end.offset())
                        };

                        // Step 7.2.
                        if start_node.r() == node && start_offset > offset {
                            range.borrow_mut().set_start(new_node, start_offset - offset);
                        }

                        // Step 7.3.
                        if end_node.r() == node && end_offset > offset {
                            range.borrow_mut().set_end(new_node, end_offset - offset);
                        }

                        // Step 7.4.
                        if start_node == *parent && start_offset == node.index() + 1 {
                            range.borrow_mut().set_start(start_node.r(), start_offset + 1);
                        }

                        // Step 7.5.
                        if end_node == *parent && end_offset == node.index() + 1 {
                            range.borrow_mut().set_end(end_node.r(), end_offset + 1);
                        }
                    },
                    _ => ()
                }
            }
        }
        // Step 8.
        cdata.DeleteData(offset, count).unwrap();
        if parent.is_none() {
            for range in node.owner_doc().ranges() {
                match range.upgrade() {
                    Some(range) => {
                        let (start_node, start_offset, end_node, end_offset) = {
                            let range = range.borrow();
                            let start = &range.start();
                            let end = &range.end();
                            (start.node(), start.offset(), end.node(), end.offset())
                        };

                        // Step 9.1.
                        if start_node.r() == node && start_offset > offset {
                            range.borrow_mut().set_start(start_node.r(), offset);
                        }

                        // Step 9.2.
                        if end_node.r() == node && end_offset > offset {
                            range.borrow_mut().set_end(end_node.r(), offset);
                        }
                    },
                    _ => ()
                }
            }
        }
        // Step 10.
        Ok(new_text)
    }

    // https://dom.spec.whatwg.org/#dom-text-wholetext
    fn WholeText(self) -> DOMString {
        let first = NodeCast::from_ref(self).inclusively_preceding_siblings()
                                            .take_while(|node| node.r().is_text())
                                            .last().unwrap();
        let nodes = first.r().inclusively_following_siblings()
                             .take_while(|node| node.r().is_text());
        let mut text = DOMString::new();
        for ref node in nodes {
            let cdata = CharacterDataCast::to_ref(node.r()).unwrap();
            text.push_str(&cdata.data());
        }
        text
    }
}

