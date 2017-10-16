/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CommentBinding;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::Fallible;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::node::Node;
use dom::window::Window;
use dom_struct::dom_struct;

/// An HTML comment.
#[dom_struct]
pub struct Comment {
    characterdata: CharacterData,
}

impl Comment {
    fn new_inherited(text: DOMString, document: &Document) -> Comment {
        Comment {
            characterdata: CharacterData::new_inherited(text, document),
        }
    }

    pub fn new(text: DOMString, document: &Document) -> DomRoot<Comment> {
        Node::reflect_node(Box::new(Comment::new_inherited(text, document)),
                           document,
                           CommentBinding::Wrap)
    }

    pub fn Constructor(window: &Window, data: DOMString) -> Fallible<DomRoot<Comment>> {
        let document = window.Document();
        Ok(Comment::new(data, &document))
    }
}
