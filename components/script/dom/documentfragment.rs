/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentFragmentBinding;
use dom::bindings::codegen::Bindings::DocumentFragmentBinding::DocumentFragmentMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, NodeCast};
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::Element;
use dom::htmlcollection::HTMLCollection;
use dom::node::{Node, window_from_node};
use dom::nodelist::NodeList;
use string_cache::Atom;
use util::str::DOMString;

// https://dom.spec.whatwg.org/#documentfragment
#[dom_struct]
pub struct DocumentFragment {
    node: Node,
}

impl DocumentFragment {
    /// Creates a new DocumentFragment.
    fn new_inherited(document: &Document) -> DocumentFragment {
        DocumentFragment {
            node: Node::new_inherited(document),
        }
    }

    pub fn new(document: &Document) -> Root<DocumentFragment> {
        Node::reflect_node(box DocumentFragment::new_inherited(document),
                           document, DocumentFragmentBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef) -> Fallible<Root<DocumentFragment>> {
        let document = global.as_window().Document();

        Ok(DocumentFragment::new(document.r()))
    }
}

impl DocumentFragmentMethods for DocumentFragment {
    // https://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(&self) -> Root<HTMLCollection> {
        let window = window_from_node(self);
        HTMLCollection::children(window.r(), NodeCast::from_ref(self))
    }

    // https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    fn GetElementById(&self, id: DOMString) -> Option<Root<Element>> {
        let node = NodeCast::from_ref(self);
        let id = Atom::from_slice(&id);
        node.traverse_preorder().filter_map(ElementCast::to_root).find(|descendant| {
            match descendant.get_attribute(&ns!(""), &atom!(id)) {
                None => false,
                Some(attr) => *attr.value().as_atom() == id,
            }
        })
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild
    fn GetFirstElementChild(&self) -> Option<Root<Element>> {
        NodeCast::from_ref(self).child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild
    fn GetLastElementChild(&self) -> Option<Root<Element>> {
        NodeCast::from_ref(self).rev_children().filter_map(ElementCast::to_root).next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-childelementcount
    fn ChildElementCount(&self) -> u32 {
        NodeCast::from_ref(self).child_elements().count() as u32
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-prepend
    fn Prepend(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).prepend(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-append
    fn Append(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).append(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<Root<Element>>> {
        let root = NodeCast::from_ref(self);
        root.query_selector(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<Root<NodeList>> {
        let root = NodeCast::from_ref(self);
        root.query_selector_all(selectors)
    }
}
