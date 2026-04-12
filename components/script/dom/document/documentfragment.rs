/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use rustc_hash::FxBuildHasher;
use stylo_atoms::Atom;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DocumentFragmentBinding::DocumentFragmentMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::UnionTypes::NodeOrString;
use crate::dom::bindings::error::{ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::HashMapTracedValues;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::html::htmlcollection::HTMLCollection;
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
    id_map: DomRefCell<HashMapTracedValues<Atom, Vec<Dom<Element>>, FxBuildHasher>>,

    /// <https://dom.spec.whatwg.org/#concept-documentfragment-host>
    host: MutNullableDom<Element>,
}

impl DocumentFragment {
    /// Creates a new DocumentFragment.
    pub(crate) fn new_inherited(document: &Document, host: Option<&Element>) -> DocumentFragment {
        DocumentFragment {
            node: Node::new_inherited(document),
            id_map: DomRefCell::new(HashMapTracedValues::new_fx()),
            host: MutNullableDom::new(host),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        document: &Document,
    ) -> DomRoot<DocumentFragment> {
        Self::new_with_proto(cx, document, None)
    }

    fn new_with_proto(
        cx: &mut js::context::JSContext,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<DocumentFragment> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(DocumentFragment::new_inherited(document, None)),
            document,
            proto,
        )
    }

    pub(crate) fn id_map(
        &self,
    ) -> &DomRefCell<HashMapTracedValues<Atom, Vec<Dom<Element>>, FxBuildHasher>> {
        &self.id_map
    }

    pub(crate) fn host(&self) -> Option<DomRoot<Element>> {
        self.host.get()
    }

    pub(crate) fn set_host(&self, host: &Element) {
        self.host.set(Some(host));
    }
}

impl<'dom> LayoutDom<'dom, DocumentFragment> {
    #[inline]
    pub(crate) fn shadowroot_host_for_layout(self) -> LayoutDom<'dom, Element> {
        #[expect(unsafe_code)]
        unsafe {
            // https://dom.spec.whatwg.org/#shadowroot
            // > Shadow roots’s associated host is never null.
            self.unsafe_get()
                .host
                .get_inner_as_layout()
                .expect("Shadow roots's associated host is never null")
        }
    }
}

impl DocumentFragmentMethods<crate::DomTypeHolder> for DocumentFragment {
    /// <https://dom.spec.whatwg.org/#dom-documentfragment-documentfragment>
    fn Constructor(
        cx: &mut js::context::JSContext,
        window: &Window,
        proto: Option<HandleObject>,
    ) -> Fallible<DomRoot<DocumentFragment>> {
        let document = window.Document();

        Ok(DocumentFragment::new_with_proto(cx, &document, proto))
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-children>
    fn Children(&self, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        let window = self.owner_window();
        HTMLCollection::children(&window, self.upcast(), can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid>
    fn GetElementById(&self, id: DOMString) -> Option<DomRoot<Element>> {
        let id = Atom::from(id);
        self.id_map
            .borrow()
            .get(&id)
            .map(|elements| DomRoot::from_ref(&*elements[0]))
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild>
    fn GetFirstElementChild(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>().child_elements().next()
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild>
    fn GetLastElementChild(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>()
            .rev_children()
            .find_map(DomRoot::downcast::<Element>)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-childelementcount>
    fn ChildElementCount(&self) -> u32 {
        self.upcast::<Node>().child_elements().count() as u32
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-prepend>
    fn Prepend(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().prepend(cx, nodes)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-append>
    fn Append(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().append(cx, nodes)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-replacechildren>
    fn ReplaceChildren(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().replace_children(cx, nodes)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-movebefore>
    fn MoveBefore(&self, cx: &mut JSContext, node: &Node, child: Option<&Node>) -> ErrorResult {
        self.upcast::<Node>().move_before(cx, node, child)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-queryselector>
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<DomRoot<Element>>> {
        self.upcast::<Node>().query_selector(selectors)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall>
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<DomRoot<NodeList>> {
        self.upcast::<Node>().query_selector_all(selectors)
    }
}

impl VirtualMethods for DocumentFragment {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<Node>() as &dyn VirtualMethods)
    }
}
