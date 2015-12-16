/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding;
use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::document::Document;
use dom::documentfragment::DocumentFragment;
use dom::htmlelement::HTMLElement;
use dom::node::{CloneChildrenFlag, Node, document_from_node};
use dom::virtualmethods::VirtualMethods;
use string_cache::Atom;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLTemplateElement {
    htmlelement: HTMLElement,

    /// https://html.spec.whatwg.org/multipage/#template-contents
    contents: MutNullableHeap<JS<DocumentFragment>>,
}

impl HTMLTemplateElement {
    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLTemplateElement {
        HTMLTemplateElement {
            htmlelement:
                HTMLElement::new_inherited(localName, prefix, document),
            contents: MutNullableHeap::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
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

impl VirtualMethods for HTMLTemplateElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    /// https://html.spec.whatwg.org/multipage/#template-adopting-steps
    fn adopting_steps(&self, old_doc: &Document) {
        self.super_type().unwrap().adopting_steps(old_doc);
        // Step 1.
        let doc = document_from_node(self).appropriate_template_contents_owner_document();
        // Step 2.
        Node::adopt(self.Content().upcast(), &doc);
    }

    /// https://html.spec.whatwg.org/multipage/#the-template-element:concept-node-clone-ext
    fn cloning_steps(&self, copy: &Node, maybe_doc: Option<&Document>,
                     clone_children: CloneChildrenFlag) {
        self.super_type().unwrap().cloning_steps(copy, maybe_doc, clone_children);
        if clone_children == CloneChildrenFlag::DoNotCloneChildren {
            // Step 1.
            return;
        }
        let copy = copy.downcast::<HTMLTemplateElement>().unwrap();
        // Steps 2-3.
        let copy_contents = Root::upcast::<Node>(copy.Content());
        let copy_contents_doc = copy_contents.owner_doc();
        for child in self.Content().upcast::<Node>().children() {
            let copy_child = Node::clone(
                &child, Some(&copy_contents_doc), CloneChildrenFlag::CloneChildren);
            copy_contents.AppendChild(&copy_child).unwrap();
        }
    }
}
