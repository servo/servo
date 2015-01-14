/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CommentBinding;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::CommentDerived;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::node::{Node, NodeTypeId};
use servo_util::str::DOMString;

/// An HTML comment.
#[dom_struct]
pub struct Comment {
    characterdata: CharacterData,
}

impl CommentDerived for EventTarget {
    fn is_comment(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Comment)
    }
}

impl Comment {
    fn new_inherited(text: DOMString, document: JSRef<Document>) -> Comment {
        Comment {
            characterdata: CharacterData::new_inherited(NodeTypeId::Comment, text, document)
        }
    }

    pub fn new(text: DOMString, document: JSRef<Document>) -> Temporary<Comment> {
        Node::reflect_node(box Comment::new_inherited(text, document),
                           document, CommentBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef, data: DOMString) -> Fallible<Temporary<Comment>> {
        let document = global.as_window().Document().root();
        Ok(Comment::new(data, document.r()))
    }

    #[inline]
    pub fn characterdata<'a>(&'a self) -> &'a CharacterData {
        &self.characterdata
    }
}

