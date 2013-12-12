/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::CommentBinding;
use dom::bindings::utils::{DOMString, Fallible};
use dom::characterdata::CharacterData;
use dom::document::AbstractDocument;
use dom::node::{AbstractNode, ScriptView, CommentNodeTypeId, Node};
use dom::window::Window;

/// An HTML comment.
pub struct Comment {
    element: CharacterData,
}

impl Comment {
    pub fn new_inherited(text: ~str, document: AbstractDocument) -> Comment {
        Comment {
            element: CharacterData::new_inherited(CommentNodeTypeId, text, document)
        }
    }

    pub fn new(text: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let node = Comment::new_inherited(text, document);
        Node::reflect_node(@mut node, document, CommentBinding::Wrap)
    }

    pub fn Constructor(owner: @mut Window, data: DOMString) -> Fallible<AbstractNode<ScriptView>> {
        Ok(Comment::new(data, owner.Document()))
    }
}
