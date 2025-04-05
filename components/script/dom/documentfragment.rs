/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use stylo_atoms::Atom;

use super::bindings::trace::HashMapTracedValues;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DocumentFragmentBinding::DocumentFragmentMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::UnionTypes::NodeOrString;
use crate::dom::bindings::error::{ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::htmlcollection::HTMLCollection;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::nodelist::NodeList;
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

// https://dom.spec.whatwg.org/#documentfragment
#[dom_struct]
pub(crate) struct DocumentFragment {
    node: Node,
    /// Caches for the getElement methods
    id_map: DomRefCell<HashMapTracedValues<Atom, Vec<Dom<Element>>>>,
}

impl DocumentFragment {
    /// Creates a new DocumentFragment.
    pub(crate) fn new_inherited(document: &Document) -> DocumentFragment {
        DocumentFragment {
            node: Node::new_inherited(document),
            id_map: DomRefCell::new(HashMapTracedValues::new()),
        }
    }

    pub(crate) fn new(document: &Document, can_gc: CanGc) -> DomRoot<DocumentFragment> {
        Self::new_with_proto(document, None, can_gc)
    }

    fn new_with_proto(
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<DocumentFragment> {
        Node::reflect_node_with_proto(
            Box::new(DocumentFragment::new_inherited(document)),
            document,
            proto,
            can_gc,
        )
    }

    pub(crate) fn id_map(&self) -> &DomRefCell<HashMapTracedValues<Atom, Vec<Dom<Element>>>> {
        &self.id_map
    }
}

impl DocumentFragmentMethods<crate::DomTypeHolder> for DocumentFragment {
    // https://dom.spec.whatwg.org/#dom-documentfragment-documentfragment
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<DocumentFragment>> {
        let document = window.Document();

        Ok(DocumentFragment::new_with_proto(&document, proto, can_gc))
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(&self) -> DomRoot<HTMLCollection> {
        let window = self.owner_window();
        HTMLCollection::children(&window, self.upcast(), CanGc::note())
    }

    // https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    fn GetElementById(&self, id: DOMString) -> Option<DomRoot<Element>> {
        let id = Atom::from(id);
        self.id_map
            .borrow()
            .get(&id)
            .map(|elements| DomRoot::from_ref(&*elements[0]))
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild
    fn GetFirstElementChild(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>().child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild
    fn GetLastElementChild(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>()
            .rev_children()
            .filter_map(DomRoot::downcast::<Element>)
            .next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-childelementcount
    fn ChildElementCount(&self) -> u32 {
        self.upcast::<Node>().child_elements().count() as u32
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-prepend
    fn Prepend(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().prepend(nodes, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-append
    fn Append(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().append(nodes, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-replacechildren
    fn ReplaceChildren(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().replace_children(nodes, can_gc)
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

impl VirtualMethods for DocumentFragment {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<Node>() as &dyn VirtualMethods)
    }
}
