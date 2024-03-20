/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::TextBinding::TextMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::characterdata::CharacterData;
use crate::dom::document::Document;
use crate::dom::node::Node;
use crate::dom::window::Window;

/// An HTML text node.
#[dom_struct]
pub struct Text {
    characterdata: CharacterData,
}

impl Text {
    pub fn new_inherited(text: DOMString, document: &Document) -> Text {
        Text {
            characterdata: CharacterData::new_inherited(text, document),
        }
    }

    pub fn new(text: DOMString, document: &Document) -> DomRoot<Text> {
        Self::new_with_proto(text, document, None)
    }

    fn new_with_proto(
        text: DOMString,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<Text> {
        Node::reflect_node_with_proto(
            Box::new(Text::new_inherited(text, document)),
            document,
            proto,
        )
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        text: DOMString,
    ) -> Fallible<DomRoot<Text>> {
        let document = window.Document();
        Ok(Text::new_with_proto(text, &document, proto))
    }
}

impl TextMethods for Text {
    // https://dom.spec.whatwg.org/#dom-text-splittext
    // https://dom.spec.whatwg.org/#concept-text-split
    fn SplitText(&self, offset: u32) -> Fallible<DomRoot<Text>> {
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
        let new_node = owner_doc.CreateTextNode(new_data);
        // Step 6.
        let parent = node.GetParentNode();
        if let Some(ref parent) = parent {
            // Step 7.1.
            parent
                .InsertBefore(new_node.upcast(), node.GetNextSibling().as_deref())
                .unwrap();
            // Steps 7.2-3.
            node.ranges()
                .move_to_following_text_sibling_above(node, offset, new_node.upcast());
            // Steps 7.4-5.
            parent.ranges().increment_at(parent, node.index() + 1);
        }
        // Step 8.
        cdata.DeleteData(offset, count).unwrap();
        // Step 9.
        Ok(new_node)
    }

    // https://dom.spec.whatwg.org/#dom-text-wholetext
    fn WholeText(&self) -> DOMString {
        let first = self
            .upcast::<Node>()
            .inclusively_preceding_siblings()
            .take_while(|node| node.is::<Text>())
            .last()
            .unwrap();
        let nodes = first
            .inclusively_following_siblings()
            .take_while(|node| node.is::<Text>());
        let mut text = String::new();
        for ref node in nodes {
            let cdata = node.downcast::<CharacterData>().unwrap();
            text.push_str(&cdata.data());
        }
        DOMString::from(text)
    }
}
