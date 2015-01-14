/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentFragmentBinding;
use dom::bindings::codegen::Bindings::DocumentFragmentBinding::DocumentFragmentMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::{DocumentFragmentDerived, NodeCast};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::document::Document;
use dom::element::Element;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::node::{Node, NodeHelpers, NodeTypeId, window_from_node};
use dom::nodelist::NodeList;
use servo_util::str::DOMString;

#[dom_struct]
pub struct DocumentFragment {
    node: Node,
}

impl DocumentFragmentDerived for EventTarget {
    fn is_documentfragment(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::DocumentFragment)
    }
}

impl DocumentFragment {
    /// Creates a new DocumentFragment.
    fn new_inherited(document: JSRef<Document>) -> DocumentFragment {
        DocumentFragment {
            node: Node::new_inherited(NodeTypeId::DocumentFragment, document),
        }
    }

    pub fn new(document: JSRef<Document>) -> Temporary<DocumentFragment> {
        Node::reflect_node(box DocumentFragment::new_inherited(document),
                           document, DocumentFragmentBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef) -> Fallible<Temporary<DocumentFragment>> {
        let document = global.as_window().Document();
        let document = document.root();

        Ok(DocumentFragment::new(document.r()))
    }
}

impl<'a> DocumentFragmentMethods for JSRef<'a, DocumentFragment> {
    // http://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(self) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::children(window.r(), NodeCast::from_ref(self))
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn QuerySelector(self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>> {
        let root: JSRef<Node> = NodeCast::from_ref(self);
        root.query_selector(selectors)
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn QuerySelectorAll(self, selectors: DOMString) -> Fallible<Temporary<NodeList>> {
        let root: JSRef<Node> = NodeCast::from_ref(self);
        root.query_selector_all(selectors)
    }
}

