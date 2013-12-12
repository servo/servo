/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::DocumentFragmentBinding;
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
    pub fn new_inherited(document: AbstractDocument) -> DocumentFragment {
        DocumentFragment {
            node: Node::new_inherited(DocumentFragmentNodeTypeId, document),
        }
    }

    pub fn new(document: AbstractDocument) -> AbstractNode<ScriptView> {
        let node = DocumentFragment::new_inherited(document);
        Node::reflect_node(@mut node, document, DocumentFragmentBinding::Wrap)
    }
}

impl DocumentFragment {
    pub fn Constructor(owner: @mut Window) -> Fallible<AbstractNode<ScriptView>> {
        Ok(DocumentFragment::new(owner.Document()))
    }
}
