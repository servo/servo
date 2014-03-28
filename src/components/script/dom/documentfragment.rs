/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::{DocumentFragmentDerived, NodeCast};
use dom::bindings::codegen::BindingDeclarations::DocumentFragmentBinding;
use dom::bindings::js::{JS, JSRef, RootCollection};
use dom::bindings::error::Fallible;
use dom::document::Document;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::node::{DocumentFragmentNodeTypeId, Node, window_from_node};
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

    pub fn new(document: &JSRef<Document>) -> JS<DocumentFragment> {
        let node = DocumentFragment::new_inherited(document.unrooted());
        Node::reflect_node(~node, document, DocumentFragmentBinding::Wrap)
    }
}

impl DocumentFragment {
    pub fn Constructor(owner: &JSRef<Window>) -> Fallible<JS<DocumentFragment>> {
        let roots = RootCollection::new();
        let document = owner.get().Document();
        let document = document.root(&roots);

        Ok(DocumentFragment::new(&document.root_ref()))
    }
}

impl DocumentFragment {
    pub fn Children(&self, abstract_self: &JSRef<DocumentFragment>) -> JS<HTMLCollection> {
        let roots = RootCollection::new();
        let window = window_from_node(&abstract_self.unrooted()).root(&roots);
        HTMLCollection::children(&window.root_ref(), NodeCast::from_ref(abstract_self))
    }
}
