/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::TextBinding;
use dom::bindings::codegen::InheritTypes::TextDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::Fallible;
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
        match self.type_id {
            NodeTargetTypeId(TextNodeTypeId) => true,
            _ => false
        }
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
        Node::reflect_node(~node, document, TextBinding::Wrap)
    }

    pub fn Constructor(owner: &JSRef<Window>, text: DOMString) -> Fallible<Temporary<Text>> {
        let document = owner.Document().root();
        Ok(Text::new(text.clone(), &*document))
    }
}

pub trait TextMethods {
    fn SplitText(&self, _offset: u32) -> Fallible<Temporary<Text>>;
    fn GetWholeText(&self) -> Fallible<DOMString>;
}

impl<'a> TextMethods for JSRef<'a, Text> {
    fn SplitText(&self, _offset: u32) -> Fallible<Temporary<Text>> {
        fail!("unimplemented")
    }

    fn GetWholeText(&self) -> Fallible<DOMString> {
        Ok("".to_owned())
    }
}
