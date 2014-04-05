/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::DocumentTypeDerived;
use dom::bindings::codegen::DocumentTypeBinding;
use dom::bindings::js::JS;
use dom::document::Document;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::node::{Node, DoctypeNodeTypeId};
use servo_util::str::DOMString;

/// The `DOCTYPE` tag.
#[deriving(Encodable)]
pub struct DocumentType {
    pub node: Node,
    pub name: DOMString,
    pub public_id: DOMString,
    pub system_id: DOMString,
}

impl DocumentTypeDerived for EventTarget {
    fn is_documenttype(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(DoctypeNodeTypeId) => true,
            _ => false
        }
    }
}

impl DocumentType {
    pub fn new_inherited(name: DOMString,
                         public_id: Option<DOMString>,
                         system_id: Option<DOMString>,
                         document: JS<Document>)
            -> DocumentType {
        DocumentType {
            node: Node::new_inherited(DoctypeNodeTypeId, document),
            name: name,
            public_id: public_id.unwrap_or(~""),
            system_id: system_id.unwrap_or(~"")
        }
    }

    pub fn new(name: DOMString,
               public_id: Option<DOMString>,
               system_id: Option<DOMString>,
               document: &JS<Document>)
               -> JS<DocumentType> {
        let documenttype = DocumentType::new_inherited(name,
                                                       public_id,
                                                       system_id,
                                                       document.clone());
        Node::reflect_node(~documenttype, document, DocumentTypeBinding::Wrap)
    }
}

impl DocumentType {
    pub fn Name(&self) -> DOMString {
        self.name.clone()
    }

    pub fn PublicId(&self) -> DOMString {
        self.public_id.clone()
    }

    pub fn SystemId(&self) -> DOMString {
        self.system_id.clone()
    }
}
