/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::DocumentTypeDerived;
use dom::bindings::codegen::BindingDeclarations::DocumentTypeBinding;
use dom::bindings::js::{JSRef, Temporary};
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
                         document: &JSRef<Document>)
            -> DocumentType {
        DocumentType {
            node: Node::new_inherited(DoctypeNodeTypeId, document),
            name: name,
            public_id: public_id.unwrap_or("".to_owned()),
            system_id: system_id.unwrap_or("".to_owned())
        }
    }

    pub fn new(name: DOMString,
               public_id: Option<DOMString>,
               system_id: Option<DOMString>,
               document: &JSRef<Document>)
               -> Temporary<DocumentType> {
        let documenttype = DocumentType::new_inherited(name,
                                                       public_id,
                                                       system_id,
                                                       document);
        Node::reflect_node(~documenttype, document, DocumentTypeBinding::Wrap)
    }
}

pub trait DocumentTypeMethods {
    fn Name(&self) -> DOMString;
    fn PublicId(&self) -> DOMString;
    fn SystemId(&self) -> DOMString;
}

impl<'a> DocumentTypeMethods for JSRef<'a, DocumentType> {
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    fn PublicId(&self) -> DOMString {
        self.public_id.clone()
    }

    fn SystemId(&self) -> DOMString {
        self.system_id.clone()
    }
}
