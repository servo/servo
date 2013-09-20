/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, Fallible, null_str_as_empty};
use dom::characterdata::CharacterData;
use dom::node::{AbstractNode, ScriptView, Node, TextNodeTypeId};
use dom::window::Window;

/// An HTML text node.
pub struct Text {
    element: CharacterData,
}

impl Text {
    /// Creates a new HTML text node.
    pub fn new(text: ~str) -> Text {
        Text {
            element: CharacterData::new(TextNodeTypeId, text)
        }
    }

    pub fn Constructor(owner: @mut Window, text: &DOMString) -> Fallible<AbstractNode<ScriptView>> {
        let cx = owner.page.js_info.get_ref().js_compartment.cx.ptr;
        unsafe { Ok(Node::as_abstract_node(cx, @Text::new(null_str_as_empty(text)))) }
    }

    pub fn SplitText(&self, _offset: u32) -> Fallible<AbstractNode<ScriptView>> {
        fail!("unimplemented")
    }

    pub fn GetWholeText(&self) -> Fallible<DOMString> {
        Ok(None)
    }
}
