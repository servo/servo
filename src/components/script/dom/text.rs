/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::TextBinding;
use dom::bindings::codegen::InheritTypes::TextDerived;
use dom::bindings::js::JS;
use dom::bindings::error::Fallible;
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::node::{Node, TextNodeTypeId};
use dom::window::Window;
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
    pub fn new_inherited(text: DOMString, document: JS<Document>) -> Text {
        Text {
            characterdata: CharacterData::new_inherited(TextNodeTypeId, text, document)
        }
    }

    pub fn new(text: DOMString, document: &JS<Document>) -> JS<Text> {
        let node = Text::new_inherited(text, document.clone());
        Node::reflect_node(~node, document, TextBinding::Wrap)
    }

    pub fn Constructor(owner: &JS<Window>, text: DOMString) -> Fallible<JS<Text>> {
        Ok(Text::new(text.clone(), &owner.get().Document()))
    }

    pub fn SplitText(&self, _offset: u32) -> Fallible<JS<Text>> {
        fail!("unimplemented")
    }

    pub fn GetWholeText(&self) -> Fallible<DOMString> {
        Ok(~"")
    }
}
