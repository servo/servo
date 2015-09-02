/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding;
use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::codegen::InheritTypes::HTMLTemplateElementDerived;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::document::Document;
use dom::documentfragment::DocumentFragment;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId, document_from_node};
use util::str::DOMString;

#[dom_struct]
pub struct HTMLTemplateElement {
    htmlelement: HTMLElement,

    /// https://html.spec.whatwg.org/multipage/#template-contents
    contents: MutNullableHeap<JS<DocumentFragment>>,
}

impl HTMLTemplateElementDerived for EventTarget {
    fn is_htmltemplateelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTemplateElement)))
    }
}

impl HTMLTemplateElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLTemplateElement {
        HTMLTemplateElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLTemplateElement, localName, prefix, document),
            contents: MutNullableHeap::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLTemplateElement> {
        let element = HTMLTemplateElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTemplateElementBinding::Wrap)
    }
}

impl HTMLTemplateElementMethods for HTMLTemplateElement {
    /// https://html.spec.whatwg.org/multipage/#dom-template-content
    fn Content(&self) -> Root<DocumentFragment> {
        self.contents.or_init(|| {
            let doc = document_from_node(self);
            doc.appropriate_template_contents_owner_document().CreateDocumentFragment()
        })
    }
}
