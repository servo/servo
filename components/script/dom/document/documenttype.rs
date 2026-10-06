/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;

use crate::dom::bindings::codegen::Bindings::DocumentTypeBinding::DocumentTypeMethods;
use crate::dom::bindings::codegen::UnionTypes::NodeOrString;
use crate::dom::bindings::error::ErrorResult;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::RootedDOMString;
use crate::dom::document::Document;
use crate::dom::node::Node;

// https://dom.spec.whatwg.org/#documenttype
/// The `DOCTYPE` tag.
#[dom_struct]
pub(crate) struct DocumentType {
    node: Node,
    name: RootedDOMString,
    public_id: RootedDOMString,
    system_id: RootedDOMString,
}

impl DocumentType {
    fn new_inherited(
        name: RootedDOMString,
        public_id: Option<RootedDOMString>,
        system_id: Option<RootedDOMString>,
        document: &Document,
    ) -> DocumentType {
        DocumentType {
            node: Node::new_inherited(document),
            name,
            public_id: public_id.unwrap_or_default(),
            system_id: system_id.unwrap_or_default(),
        }
    }
    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        name: RootedDOMString,
        public_id: Option<RootedDOMString>,
        system_id: Option<RootedDOMString>,
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
    pub(crate) fn name(&self) -> &RootedDOMString {
        &self.name
    }

    #[inline]
    pub(crate) fn public_id(&self) -> &RootedDOMString {
        &self.public_id
    }

    #[inline]
    pub(crate) fn system_id(&self) -> &RootedDOMString {
        &self.system_id
    }
}

impl DocumentTypeMethods<crate::DomTypeHolder> for DocumentType {
    /// <https://dom.spec.whatwg.org/#dom-documenttype-name>
    fn Name(&self) -> RootedDOMString {
        self.name.clone()
    }

    /// <https://dom.spec.whatwg.org/#dom-documenttype-publicid>
    fn PublicId(&self) -> RootedDOMString {
        self.public_id.clone()
    }

    /// <https://dom.spec.whatwg.org/#dom-documenttype-systemid>
    fn SystemId(&self) -> RootedDOMString {
        self.system_id.clone()
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
