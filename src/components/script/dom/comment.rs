/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, str, null_string, ErrorResult};
use dom::characterdata::CharacterData;
use dom::node::{AbstractNode, ScriptView, CommentNodeTypeId, Node};
use dom::window::Window;

/// An HTML comment.
pub struct Comment {
    parent: CharacterData,
}

impl Comment {
    /// Creates a new HTML comment.
    pub fn new(text: ~str) -> Comment {
        Comment {
            parent: CharacterData::new(CommentNodeTypeId, text)
        }
    }

    pub fn Constructor(owner: @mut Window, data: &DOMString, _rv: &mut ErrorResult) -> AbstractNode<ScriptView> {
        let s = match *data {
            str(ref s) => s.clone(),
            null_string => ~""
        };
        unsafe {
            let compartment = (*owner.page).js_info.get_ref().js_compartment;
            Node::as_abstract_node(compartment.cx.ptr, @Comment::new(s))
        }
    }
}
