/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CommentBinding;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::CommentDerived;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::node::{CommentNodeTypeId, Node};
use servo_util::str::DOMString;

/// An HTML comment.
#[deriving(Encodable)]
#[must_root]
pub struct Comment {
    pub characterdata: CharacterData,
}

impl CommentDerived for EventTarget {
    fn is_comment(&self) -> bool {
        self.type_id == NodeTargetTypeId(CommentNodeTypeId)
    }
}

impl Comment {
    pub fn new_inherited(text: DOMString, document: JSRef<Document>) -> Comment {
        Comment {
            characterdata: CharacterData::new_inherited(CommentNodeTypeId, text, document)
        }
    }

    pub fn new(text: DOMString, document: JSRef<Document>) -> Temporary<Comment> {
        Node::reflect_node(box Comment::new_inherited(text, document),
                           document, CommentBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalRef, data: DOMString) -> Fallible<Temporary<Comment>> {
        let document = global.as_window().Document().root();
        Ok(Comment::new(data, *document))
    }
}

impl Reflectable for Comment {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.characterdata.reflector()
    }
}
