/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::DocumentTypeBinding;
use dom::bindings::utils::DOMString;
use dom::document::AbstractDocument;
use dom::node::{AbstractNode, Node, DoctypeNodeTypeId};

/// The `DOCTYPE` tag.
pub struct DocumentType {
    node: Node,
    name: DOMString,
    public_id: DOMString,
    system_id: DOMString,
    force_quirks: bool
}

impl DocumentType {
    pub fn new_inherited(name: ~str,
                         public_id: Option<~str>,
                         system_id: Option<~str>,
                         force_quirks: bool,
                         document: AbstractDocument)
            -> DocumentType {
        DocumentType {
            node: Node::new_inherited(DoctypeNodeTypeId, document),
            name: name,
            public_id: public_id.unwrap_or(~""),
            system_id: system_id.unwrap_or(~""),
            force_quirks: force_quirks,
        }
    }

    pub fn new(name: ~str,
               public_id: Option<~str>,
               system_id: Option<~str>,
               force_quirks: bool,
               document: AbstractDocument)
               -> AbstractNode {
        let documenttype = DocumentType::new_inherited(name,
                                                       public_id,
                                                       system_id,
                                                       force_quirks,
                                                       document);
        Node::reflect_node(@mut documenttype, document, DocumentTypeBinding::Wrap)
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
