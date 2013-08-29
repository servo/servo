/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult, null_string};
use dom::characterdata::CharacterData;
use dom::node::{AbstractNode, ScriptView, Node, TextNodeTypeId};
use dom::window::Window;

/// An HTML text node.
pub struct Text {
    parent: CharacterData,
}

impl Text {
    /// Creates a new HTML text node.
    pub fn new(text: ~str) -> Text {
        Text {
            parent: CharacterData::new(TextNodeTypeId, text)
        }
    }

    pub fn Constructor(owner: @mut Window, text: &DOMString, _rv: &mut ErrorResult) -> AbstractNode<ScriptView> {
        let cx = unsafe {(*owner.page).js_info.get_ref().js_compartment.cx.ptr};
        unsafe { Node::as_abstract_node(cx, @Text::new(text.to_str())) }
    }

    pub fn SplitText(&self, _offset: u32, _rv: &mut ErrorResult) -> AbstractNode<ScriptView> {
        fail!("unimplemented")
    }

    pub fn GetWholeText(&self, _rv: &mut ErrorResult) -> DOMString {
        null_string
    }
}
