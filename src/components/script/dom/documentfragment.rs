/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::DocumentFragmentDerived;
use dom::bindings::codegen::DocumentFragmentBinding;
use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::Fallible;
use dom::document::Document;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::node::{DocumentFragmentNodeTypeId, Node};
use dom::window::Window;

pub struct DocumentFragment {
    node: Node,
}

impl DocumentFragmentDerived for EventTarget {
    fn is_documentfragment(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(DocumentFragmentNodeTypeId) => true,
            _ => false
        }
    }
}

impl DocumentFragment {
    /// Creates a new DocumentFragment.
    pub fn new_inherited(document: JSManaged<Document>) -> DocumentFragment {
        DocumentFragment {
            node: Node::new_inherited(DocumentFragmentNodeTypeId, document),
        }
    }

    pub fn new(document: JSManaged<Document>) -> JSManaged<DocumentFragment> {
        let node = DocumentFragment::new_inherited(document);
        Node::reflect_node(~node, document, DocumentFragmentBinding::Wrap)
    }
}

impl DocumentFragment {
    pub fn Constructor(owner: @mut Window) -> Fallible<JSManaged<DocumentFragment>> {
        Ok(DocumentFragment::new(owner.Document()))
    }
}
