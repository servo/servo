/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentTypeBinding;
use dom::bindings::codegen::Bindings::DocumentTypeBinding::DocumentTypeMethods;
use dom::bindings::codegen::InheritTypes::{DocumentTypeDerived, NodeCast};
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::ErrorResult;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::node::{Node, NodeTypeId};
use util::str::DOMString;

use std::borrow::ToOwned;

// https://dom.spec.whatwg.org/#documenttype
/// The `DOCTYPE` tag.
#[dom_struct]
pub struct DocumentType {
    node: Node,
    name: DOMString,
    public_id: DOMString,
    system_id: DOMString,
}

impl DocumentTypeDerived for EventTarget {
    fn is_documenttype(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::DocumentType)
    }
}

impl DocumentType {
    fn new_inherited(name: DOMString,
                         public_id: Option<DOMString>,
                         system_id: Option<DOMString>,
                         document: &Document)
            -> DocumentType {
        DocumentType {
            node: Node::new_inherited(NodeTypeId::DocumentType, document),
            name: name,
            public_id: public_id.unwrap_or("".to_owned()),
            system_id: system_id.unwrap_or("".to_owned())
        }
    }
    #[allow(unrooted_must_root)]
    pub fn new(name: DOMString,
               public_id: Option<DOMString>,
               system_id: Option<DOMString>,
               document: &Document)
               -> Root<DocumentType> {
        let documenttype = DocumentType::new_inherited(name,
                                                       public_id,
                                                       system_id,
                                                       document);
        Node::reflect_node(box documenttype, document, DocumentTypeBinding::Wrap)
    }

    #[inline]
    pub fn name<'a>(&'a self) -> &'a DOMString {
        &self.name
    }

    #[inline]
    pub fn public_id<'a>(&'a self) -> &'a DOMString {
        &self.public_id
    }

    #[inline]
    pub fn system_id<'a>(&'a self) -> &'a DOMString {
        &self.system_id
    }
}

impl<'a> DocumentTypeMethods for &'a DocumentType {
    // https://dom.spec.whatwg.org/#dom-documenttype-name
    fn Name(self) -> DOMString {
        self.name.clone()
    }

    // https://dom.spec.whatwg.org/#dom-documenttype-publicid
    fn PublicId(self) -> DOMString {
        self.public_id.clone()
    }

    // https://dom.spec.whatwg.org/#dom-documenttype-systemid
    fn SystemId(self) -> DOMString {
        self.system_id.clone()
    }

    // https://dom.spec.whatwg.org/#dom-childnode-before
    fn Before(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).before(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-after
    fn After(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).after(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-replacewith
    fn ReplaceWith(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).replace_with(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-remove
    fn Remove(self) {
        let node = NodeCast::from_ref(self);
        node.remove_self();
    }
}
