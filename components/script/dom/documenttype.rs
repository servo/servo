/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::DocumentTypeBinding::DocumentTypeMethods;
use crate::dom::bindings::codegen::UnionTypes::NodeOrString;
use crate::dom::bindings::error::ErrorResult;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::node::Node;
use crate::script_runtime::CanGc;

// https://dom.spec.whatwg.org/#documenttype
/// The `DOCTYPE` tag.
#[dom_struct]
pub(crate) struct DocumentType {
    node: Node,
    name: DOMString,
    public_id: DOMString,
    system_id: DOMString,
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
            name,
            public_id: public_id.unwrap_or_default(),
            system_id: system_id.unwrap_or_default(),
        }
    }
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        name: DOMString,
        public_id: Option<DOMString>,
        system_id: Option<DOMString>,
        document: &Document,
        can_gc: CanGc,
    ) -> DomRoot<DocumentType> {
        Node::reflect_node(
            Box::new(DocumentType::new_inherited(
                name, public_id, system_id, document,
            )),
            document,
            can_gc,
        )
    }

    #[inline]
    pub(crate) fn name(&self) -> &DOMString {
        &self.name
    }

    #[inline]
    pub(crate) fn public_id(&self) -> &DOMString {
        &self.public_id
    }

    #[inline]
    pub(crate) fn system_id(&self) -> &DOMString {
        &self.system_id
    }
}

impl DocumentTypeMethods<crate::DomTypeHolder> for DocumentType {
    // https://dom.spec.whatwg.org/#dom-documenttype-name
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    // https://dom.spec.whatwg.org/#dom-documenttype-publicid
    fn PublicId(&self) -> DOMString {
        self.public_id.clone()
    }

    // https://dom.spec.whatwg.org/#dom-documenttype-systemid
    fn SystemId(&self) -> DOMString {
        self.system_id.clone()
    }

    // https://dom.spec.whatwg.org/#dom-childnode-before
    fn Before(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().before(nodes, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-after
    fn After(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().after(nodes, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-replacewith
    fn ReplaceWith(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().replace_with(nodes, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-remove
    fn Remove(&self) {
        self.upcast::<Node>().remove_self(CanGc::note());
    }
}
