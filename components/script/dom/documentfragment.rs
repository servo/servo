/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentFragmentBinding;
use dom::bindings::codegen::Bindings::DocumentFragmentBinding::DocumentFragmentMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::Element;
use dom::htmlcollection::HTMLCollection;
use dom::node::{Node, window_from_node};
use dom::nodelist::NodeList;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_atoms::Atom;

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

    pub fn new(document: &Document) -> DomRoot<DocumentFragment> {
        Node::reflect_node(Box::new(DocumentFragment::new_inherited(document)),
                           document,
                           DocumentFragmentBinding::Wrap)
    }

    pub fn Constructor(window: &Window) -> Fallible<DomRoot<DocumentFragment>> {
        let document = window.Document();

        Ok(DocumentFragment::new(&document))
    }
}

impl DocumentFragmentMethods for DocumentFragment {
    // https://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(&self) -> DomRoot<HTMLCollection> {
        let window = window_from_node(self);
        HTMLCollection::children(&window, self.upcast())
    }

    // https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    fn GetElementById(&self, id: DOMString) -> Option<DomRoot<Element>> {
        let node = self.upcast::<Node>();
        let id = Atom::from(id);
        node.traverse_preorder().filter_map(DomRoot::downcast::<Element>).find(|descendant| {
            match descendant.get_attribute(&ns!(), &local_name!("id")) {
                None => false,
                Some(attr) => *attr.value().as_atom() == id,
            }
        })
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild
    fn GetFirstElementChild(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>().child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild
    fn GetLastElementChild(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>().rev_children().filter_map(DomRoot::downcast::<Element>).next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-childelementcount
    fn ChildElementCount(&self) -> u32 {
        self.upcast::<Node>().child_elements().count() as u32
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-prepend
    fn Prepend(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().prepend(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-append
    fn Append(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().append(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<DomRoot<Element>>> {
        self.upcast::<Node>().query_selector(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<DomRoot<NodeList>> {
        self.upcast::<Node>().query_selector_all(selectors)
    }
}
