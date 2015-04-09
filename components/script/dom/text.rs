/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TextBinding::{self, TextMethods};
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::{CharacterDataCast, TextDerived};
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::characterdata::{CharacterData, CharacterDataHelpers};
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::node::{Node, NodeHelpers, NodeTypeId};
use util::str::DOMString;

/// An HTML text node.
#[dom_struct]
pub struct Text {
    characterdata: CharacterData,
}

impl TextDerived for EventTarget {
    fn is_text(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Text)
    }
}

impl Text {
    fn new_inherited(text: DOMString, document: JSRef<Document>) -> Text {
        Text {
            characterdata: CharacterData::new_inherited(NodeTypeId::Text, text, document)
        }
    }

    pub fn new(text: DOMString, document: JSRef<Document>) -> Temporary<Text> {
        Node::reflect_node(box Text::new_inherited(text, document),
                           document, TextBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef, text: DOMString) -> Fallible<Temporary<Text>> {
        let document = global.as_window().Document().root();
        Ok(Text::new(text, document.r()))
    }
}

impl<'a> TextMethods for JSRef<'a, Text> {
    // https://dom.spec.whatwg.org/#dom-text-wholetext
    fn WholeText(self) -> DOMString {
        let first = NodeCast::from_ref(self).inclusively_preceding_siblings()
                                            .map(|node| node.root())
                                            .take_while(|node| node.r().is_text())
                                            .last().unwrap();
        let nodes = first.r().inclusively_following_siblings().map(|node| node.root())
                             .take_while(|node| node.r().is_text());
        let mut text = DOMString::new();
        for node in nodes {
            let cdata = CharacterDataCast::to_ref(node.r()).unwrap();
            text.push_str(&cdata.data());
        }
        text
    }
}

