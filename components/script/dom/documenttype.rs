/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentTypeBinding;
use dom::bindings::codegen::Bindings::DocumentTypeBinding::DocumentTypeMethods;
use dom::bindings::codegen::InheritTypes::{DocumentTypeDerived, NodeCast};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::node::{Node, DoctypeNodeTypeId, NodeHelpers};
use servo_util::str::DOMString;

/// The `DOCTYPE` tag.
#[deriving(Encodable)]
#[must_root]
pub struct DocumentType {
    pub node: Node,
    pub name: DOMString,
    pub public_id: DOMString,
    pub system_id: DOMString,
}

impl DocumentTypeDerived for EventTarget {
    fn is_documenttype(&self) -> bool {
        self.type_id == NodeTargetTypeId(DoctypeNodeTypeId)
    }
}

impl DocumentType {
    pub fn new_inherited(name: DOMString,
                         public_id: Option<DOMString>,
                         system_id: Option<DOMString>,
                         document: JSRef<Document>)
            -> DocumentType {
        DocumentType {
            node: Node::new_inherited(DoctypeNodeTypeId, document),
            name: name,
            public_id: public_id.unwrap_or("".to_string()),
            system_id: system_id.unwrap_or("".to_string())
        }
    }
    #[allow(unrooted_must_root)]
    pub fn new(name: DOMString,
               public_id: Option<DOMString>,
               system_id: Option<DOMString>,
               document: JSRef<Document>)
               -> Temporary<DocumentType> {
        let documenttype = DocumentType::new_inherited(name,
                                                       public_id,
                                                       system_id,
                                                       document);
        Node::reflect_node(box documenttype, document, DocumentTypeBinding::Wrap)
    }
}

impl<'a> DocumentTypeMethods for JSRef<'a, DocumentType> {
    fn Name(self) -> DOMString {
        self.name.clone()
    }

    fn PublicId(self) -> DOMString {
        self.public_id.clone()
    }

    fn SystemId(self) -> DOMString {
        self.system_id.clone()
    }

    // http://dom.spec.whatwg.org/#dom-childnode-remove
    fn Remove(self) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.remove_self();
    }
}

impl Reflectable for DocumentType {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.node.reflector()
    }
}
