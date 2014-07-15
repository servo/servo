/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TextBinding;
use dom::bindings::codegen::InheritTypes::TextDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::Fallible;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::node::{Node, TextNodeTypeId};
use dom::window::{Window, WindowMethods};
use servo_util::str::DOMString;

/// An HTML text node.
#[deriving(Encodable)]
pub struct Text {
    pub characterdata: CharacterData,
}

impl TextDerived for EventTarget {
    fn is_text(&self) -> bool {
        self.type_id == NodeTargetTypeId(TextNodeTypeId)
    }
}

impl Text {
    pub fn new_inherited(text: DOMString, document: &JSRef<Document>) -> Text {
        Text {
            characterdata: CharacterData::new_inherited(TextNodeTypeId, text, document)
        }
    }

    pub fn new(text: DOMString, document: &JSRef<Document>) -> Temporary<Text> {
        let node = Text::new_inherited(text, document);
        Node::reflect_node(box node, document, TextBinding::Wrap)
    }

    pub fn Constructor(owner: &JSRef<Window>, text: DOMString) -> Fallible<Temporary<Text>> {
        let document = owner.Document().root();
        Ok(Text::new(text, &*document))
    }
}

pub trait TextMethods {
}

impl Reflectable for Text {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.characterdata.reflector()
    }
}
