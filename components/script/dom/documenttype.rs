/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentTypeBinding;
use dom::bindings::codegen::Bindings::DocumentTypeBinding::DocumentTypeMethods;
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::ErrorResult;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::node::Node;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;

// https://dom.spec.whatwg.org/#documenttype
/// The `DOCTYPE` tag.
#[dom_struct]
pub struct DocumentType<TH: TypeHolderTrait> {
    node: Node<TH>,
    name: DOMString,
    public_id: DOMString,
    system_id: DOMString,
}

impl<TH: TypeHolderTrait> DocumentType<TH> {
    fn new_inherited(name: DOMString,
                     public_id: Option<DOMString>,
                     system_id: Option<DOMString>,
                     document: &Document<TH>)
                     -> DocumentType<TH> {
        DocumentType {
            node: Node::<TH>::new_inherited(document),
            name: name,
            public_id: public_id.unwrap_or_default(),
            system_id: system_id.unwrap_or_default(),
        }
    }
    #[allow(unrooted_must_root)]
    pub fn new(name: DOMString,
               public_id: Option<DOMString>,
               system_id: Option<DOMString>,
               document: &Document<TH>)
               -> DomRoot<DocumentType<TH>> {
        Node::<TH>::reflect_node(Box::new(DocumentType::new_inherited(name, public_id, system_id, document)),
                           document,
                           DocumentTypeBinding::Wrap)
    }

    #[inline]
    pub fn name(&self) -> &DOMString {
        &self.name
    }

    #[inline]
    pub fn public_id(&self) -> &DOMString {
        &self.public_id
    }

    #[inline]
    pub fn system_id(&self) -> &DOMString {
        &self.system_id
    }
}

impl<TH: TypeHolderTrait> DocumentTypeMethods<TH> for DocumentType<TH> {
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
    fn Before(&self, nodes: Vec<NodeOrString<TH>>) -> ErrorResult {
        self.upcast::<Node<TH>>().before(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-after
    fn After(&self, nodes: Vec<NodeOrString<TH>>) -> ErrorResult {
        self.upcast::<Node<TH>>().after(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-replacewith
    fn ReplaceWith(&self, nodes: Vec<NodeOrString<TH>>) -> ErrorResult {
        self.upcast::<Node<TH>>().replace_with(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-remove
    fn Remove(&self) {
        self.upcast::<Node<TH>>().remove_self();
    }
}
