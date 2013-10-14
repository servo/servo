/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::Fallible;
use dom::document::AbstractDocument;
use dom::node::{ScriptView, Node, DocumentFragmentNodeTypeId};
use dom::node::{AbstractNode};
use dom::window::Window;

pub struct DocumentFragment {
    node: Node<ScriptView>,
}

impl DocumentFragment {
    /// Creates a new DocumentFragment.
    pub fn new(document: AbstractDocument) -> DocumentFragment {
        DocumentFragment {
            node: Node::new(DocumentFragmentNodeTypeId, document),
        }
    }

    pub fn Constructor(owner: @mut Window) -> Fallible<AbstractNode<ScriptView>> {
        let cx = owner.get_cx();
        let fragment = @DocumentFragment::new(owner.Document());
        Ok(unsafe { Node::as_abstract_node(cx, fragment) })
    }
}
