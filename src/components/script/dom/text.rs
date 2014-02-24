/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::TextBinding;
use dom::bindings::utils::Fallible;
use dom::characterdata::CharacterData;
use dom::document::AbstractDocument;
use dom::node::{AbstractNode, Node, TextNodeTypeId};
use dom::window::Window;
use servo_util::str::DOMString;

/// An HTML text node.
pub struct Text {
    characterdata: CharacterData,
}

impl Text {
    pub fn new_inherited(text: DOMString, document: AbstractDocument) -> Text {
        Text {
            characterdata: CharacterData::new_inherited(TextNodeTypeId, text, document)
        }
    }

    pub fn new_layout_pseudo(text: ~str) -> Text {
        Text {
            characterdata: CharacterData::new_layout_pseudo(TextNodeTypeId, text)
        }
    }

    pub fn new(text: ~str, document: AbstractDocument) -> AbstractNode {
        let node = Text::new_inherited(text, document);
        Node::reflect_node(@mut node, document, TextBinding::Wrap)
    }

    pub fn Constructor(owner: @mut Window, text: DOMString) -> Fallible<AbstractNode> {
        Ok(Text::new(text.clone(), owner.Document()))
    }

    pub fn SplitText(&self, _offset: u32) -> Fallible<AbstractNode> {
        fail!("unimplemented")
    }

    pub fn GetWholeText(&self) -> Fallible<DOMString> {
        Ok(~"")
    }
}
