/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::CommentBinding;
use dom::bindings::utils::Fallible;
use dom::characterdata::CharacterData;
use dom::document::AbstractDocument;
use dom::node::{AbstractNode, CommentNodeTypeId, Node};
use dom::window::Window;
use servo_util::str::DOMString;

/// An HTML comment.
pub struct Comment {
    characterdata: CharacterData,
}

impl Comment {
    pub fn new_inherited(text: DOMString, document: AbstractDocument) -> Comment {
        Comment {
            characterdata: CharacterData::new_inherited(CommentNodeTypeId, text, document)
        }
    }

    pub fn new(text: DOMString, document: AbstractDocument) -> AbstractNode {
        let node = Comment::new_inherited(text, document);
        Node::reflect_node(@mut node, document, CommentBinding::Wrap)
    }

    pub fn Constructor(owner: @mut Window, data: DOMString) -> Fallible<AbstractNode> {
        Ok(Comment::new(data, owner.Document()))
    }
}
