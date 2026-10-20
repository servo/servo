/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::domstring::TracedDOMString;

use crate::dom::bindings::codegen::Bindings::DocumentTypeBinding::DocumentTypeMethods;
use crate::dom::bindings::codegen::UnionTypes::NodeOrString;
use crate::dom::bindings::error::ErrorResult;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::node::Node;

// https://dom.spec.whatwg.org/#documenttype
/// The `DOCTYPE` tag.
#[dom_struct]
pub(crate) struct DocumentType {
    node: Node,
    name: TracedDOMString,
    public_id: TracedDOMString,
    system_id: TracedDOMString,
}

impl DocumentType {
    fn new_inherited(
        name: DOMString,
        public_id: Option<DOMString>,
        system_id: Option<DOMString>,
        document: &Document,
    ) -> DocumentType {
        DocumentType {
            node: Node::new_inherited(document),
            name: name.traced(),
            public_id: public_id.unwrap_or_default().traced(),
            system_id: system_id.unwrap_or_default().traced(),
        }
    }
    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        name: DOMString,
        public_id: Option<DOMString>,
        system_id: Option<DOMString>,
        document: &Document,
    ) -> DomRoot<DocumentType> {
        Node::reflect_node(
            cx,
            Box::new(DocumentType::new_inherited(
                name, public_id, system_id, document,
            )),
            document,
        )
    }

    #[inline]
    pub(crate) fn name(&self) -> &TracedDOMString {
        &self.name
    }

    #[inline]
    pub(crate) fn public_id(&self) -> &TracedDOMString {
        &self.public_id
    }

    #[inline]
    pub(crate) fn system_id(&self) -> &TracedDOMString {
        &self.system_id
    }
}

impl DocumentTypeMethods<crate::DomTypeHolder> for DocumentType {
    /// <https://dom.spec.whatwg.org/#dom-documenttype-name>
    fn Name(&self) -> DOMString {
        self.name.clone().root()
    }

    /// <https://dom.spec.whatwg.org/#dom-documenttype-publicid>
    fn PublicId(&self) -> DOMString {
        self.public_id.clone().root()
    }

    /// <https://dom.spec.whatwg.org/#dom-documenttype-systemid>
    fn SystemId(&self) -> DOMString {
        self.system_id.clone().root()
    }

    /// <https://dom.spec.whatwg.org/#dom-childnode-before>
    fn Before(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().before(cx, nodes)
    }

    /// <https://dom.spec.whatwg.org/#dom-childnode-after>
    fn After(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().after(cx, nodes)
    }

    /// <https://dom.spec.whatwg.org/#dom-childnode-replacewith>
    fn ReplaceWith(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().replace_with(cx, nodes)
    }

    /// <https://dom.spec.whatwg.org/#dom-childnode-remove>
    fn Remove(&self, cx: &mut JSContext) {
        self.upcast::<Node>().remove_self(cx);
    }
}
