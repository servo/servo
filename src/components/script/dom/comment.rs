/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::CommentDerived;
use dom::bindings::codegen::BindingDeclarations::CommentBinding;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::Fallible;
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::node::{CommentNodeTypeId, Node};
use dom::window::{Window, WindowMethods};
use servo_util::str::DOMString;

/// An HTML comment.
#[deriving(Encodable)]
pub struct Comment {
    pub characterdata: CharacterData,
}

impl CommentDerived for EventTarget {
    fn is_comment(&self) -> bool {
        self.type_id == NodeTargetTypeId(CommentNodeTypeId)
    }
}

impl Comment {
    pub fn new_inherited(text: DOMString, document: &JSRef<Document>) -> Comment {
        Comment {
            characterdata: CharacterData::new_inherited(CommentNodeTypeId, text, document)
        }
    }

    pub fn new(text: DOMString, document: &JSRef<Document>) -> Temporary<Comment> {
        let node = Comment::new_inherited(text, document);
        Node::reflect_node(box node, document, CommentBinding::Wrap)
    }

    pub fn Constructor(owner: &JSRef<Window>, data: DOMString) -> Fallible<Temporary<Comment>> {
        let document = owner.Document().root();
        Ok(Comment::new(data, &*document))
    }
}

pub trait CommentMethods {
}
