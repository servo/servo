/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::TextBinding::{self, TextMethods};
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::conversions::Castable;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::js::{RootedReference};
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::node::Node;
use util::str::DOMString;

/// An HTML text node.
#[dom_struct]
pub struct Text {
    characterdata: CharacterData,
}

impl Text {
    fn new_inherited(text: DOMString, document: &Document) -> Text {
        Text {
            characterdata: CharacterData::new_inherited(text, document)
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

impl TextMethods for Text {
    // https://dom.spec.whatwg.org/#dom-text-splittextoffset
    fn SplitText(&self, offset: u32) -> Fallible<Root<Text>> {
        let cdata = self.upcast::<CharacterData>();
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
        let node = self.upcast::<Node>();
        let owner_doc = node.owner_doc();
        let new_node = owner_doc.r().CreateTextNode(new_data);
        // Step 6.
        let parent = node.GetParentNode();
        if let Some(ref parent) = parent {
            // Step 7.
            parent.InsertBefore(new_node.upcast(), node.GetNextSibling().r()).unwrap();
            // TODO: Ranges.
        }
        // Step 8.
        cdata.DeleteData(offset, count).unwrap();
        if parent.is_none() {
            // Step 9.
            // TODO: Ranges
        }
        // Step 10.
        Ok(new_node)
    }

    // https://dom.spec.whatwg.org/#dom-text-wholetext
    fn WholeText(&self) -> DOMString {
        let first = self.upcast::<Node>().inclusively_preceding_siblings()
                                         .take_while(|node| node.r().is::<Text>())
                                         .last().unwrap();
        let nodes = first.r().inclusively_following_siblings()
                             .take_while(|node| node.r().is::<Text>());
        let mut text = DOMString::new();
        for ref node in nodes {
            let cdata = node.downcast::<CharacterData>().unwrap();
            text.0.push_str(&cdata.data());
        }
        text
    }
}
