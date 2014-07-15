/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::{DocumentFragmentDerived, NodeCast};
use dom::bindings::codegen::Bindings::DocumentFragmentBinding;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::Element;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::node::{DocumentFragmentNodeTypeId, Node, NodeHelpers, window_from_node};
use dom::nodelist::NodeList;
use dom::window::WindowMethods;
use servo_util::str::DOMString;

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
        Node::reflect_node(box node, document, DocumentFragmentBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalRef) -> Fallible<Temporary<DocumentFragment>> {
        let document = global.as_window().Document();
        let document = document.root();

        Ok(DocumentFragment::new(&document.root_ref()))
    }
}

pub trait DocumentFragmentMethods {
    fn Children(&self) -> Temporary<HTMLCollection>;
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>>;
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<Temporary<NodeList>>;
}

impl<'a> DocumentFragmentMethods for JSRef<'a, DocumentFragment> {
    // http://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(&self) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::children(&window.root_ref(), NodeCast::from_ref(self))
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>> {
        let root: &JSRef<Node> = NodeCast::from_ref(self);
        root.query_selector(selectors)
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<Temporary<NodeList>> {
        let root: &JSRef<Node> = NodeCast::from_ref(self);
        root.query_selector_all(selectors)
    }

}

impl Reflectable for DocumentFragment {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.node.reflector()
    }
}
