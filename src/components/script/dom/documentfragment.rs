/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::{DocumentFragmentDerived, NodeCast};
use dom::bindings::codegen::BindingDeclarations::DocumentFragmentBinding;
use dom::bindings::js::JS;
use dom::bindings::error::Fallible;
use dom::document::Document;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::node::{DocumentFragmentNodeTypeId, Node};
use dom::window::Window;

#[deriving(Encodable)]
pub struct DocumentFragment {
    pub node: Node,
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
    pub fn new_inherited(document: JS<Document>) -> DocumentFragment {
        DocumentFragment {
            node: Node::new_inherited(DocumentFragmentNodeTypeId, document),
        }
    }

    pub fn new(document: &JS<Document>) -> JS<DocumentFragment> {
        let node = DocumentFragment::new_inherited(document.clone());
        Node::reflect_node(~node, document, DocumentFragmentBinding::Wrap)
    }
}

impl DocumentFragment {
    pub fn Constructor(owner: &JS<Window>) -> Fallible<JS<DocumentFragment>> {
        Ok(DocumentFragment::new(&owner.get().Document()))
    }
}

impl DocumentFragment {
    pub fn Children(&self, abstract_self: &JS<DocumentFragment>) -> JS<HTMLCollection> {
        let doc = self.node.owner_doc();
        let doc = doc.get();
        HTMLCollection::children(&doc.window, &NodeCast::from(abstract_self))
    }
}
