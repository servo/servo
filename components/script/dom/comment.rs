/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::characterdata::CharacterData;
use crate::dom::document::Document;
use crate::dom::node::Node;
use crate::dom::window::Window;

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

    pub fn new(
        text: DOMString,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<Comment> {
        Node::reflect_node_with_proto(
            Box::new(Comment::new_inherited(text, document)),
            document,
            proto,
        )
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        data: DOMString,
    ) -> Fallible<DomRoot<Comment>> {
        let document = window.Document();
        Ok(Comment::new(data, &document, proto))
    }
}
