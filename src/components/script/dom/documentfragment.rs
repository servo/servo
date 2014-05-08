/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::{DocumentFragmentDerived, NodeCast};
use dom::bindings::codegen::BindingDeclarations::DocumentFragmentBinding;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::Fallible;
use dom::document::Document;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::node::{DocumentFragmentNodeTypeId, Node, window_from_node};
use dom::window::{Window, WindowMethods};

#[deriving(Encodable)]
pub struct DocumentFragment {
    pub node: Node,
}

impl DocumentFragmentDerived for EventTarget {
    fn is_documentfragment(&self) -> bool {
        self.type_id == NodeTargetTypeId(DocumentFragmentNodeTypeId)
    }
}

impl DocumentFragment {
    /// Creates a new DocumentFragment.
    pub fn new_inherited(document: &JSRef<Document>) -> DocumentFragment {
        DocumentFragment {
            node: Node::new_inherited(DocumentFragmentNodeTypeId, document),
        }
    }

    pub fn new(document: &JSRef<Document>) -> Temporary<DocumentFragment> {
        let node = DocumentFragment::new_inherited(document);
        Node::reflect_node(~node, document, DocumentFragmentBinding::Wrap)
    }

    pub fn Constructor(owner: &JSRef<Window>) -> Fallible<Temporary<DocumentFragment>> {
        let document = owner.Document();
        let document = document.root();

        Ok(DocumentFragment::new(&document.root_ref()))
    }
}

pub trait DocumentFragmentMethods {
    fn Children(&self) -> Temporary<HTMLCollection>;
}

impl<'a> DocumentFragmentMethods for JSRef<'a, DocumentFragment> {
    fn Children(&self) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::children(&window.root_ref(), NodeCast::from_ref(self))
    }
}
