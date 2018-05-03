/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding;
use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::document::Document;
use dom::documentfragment::DocumentFragment;
use dom::htmlelement::HTMLElement;
use dom::node::{CloneChildrenFlag, Node, document_from_node};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLTemplateElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>,

    /// <https://html.spec.whatwg.org/multipage/#template-contents>
    contents: MutNullableDom<DocumentFragment<TH>>,
}

impl<TH: TypeHolderTrait> HTMLTemplateElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLTemplateElement<TH> {
        HTMLTemplateElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document),
            contents: MutNullableDom::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLTemplateElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLTemplateElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLTemplateElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> HTMLTemplateElementMethods<TH> for HTMLTemplateElement<TH> {
    /// <https://html.spec.whatwg.org/multipage/#dom-template-content>
    fn Content(&self) -> DomRoot<DocumentFragment<TH>> {
        self.contents.or_init(|| {
            let doc = document_from_node(self);
            doc.appropriate_template_contents_owner_document().CreateDocumentFragment()
        })
    }
}

impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLTemplateElement<TH> {
    fn super_type(&self) -> Option<&VirtualMethods<TH>> {
        Some(self.upcast::<HTMLElement<TH>>() as &VirtualMethods<TH>)
    }

    /// <https://html.spec.whatwg.org/multipage/#template-adopting-steps>
    fn adopting_steps(&self, old_doc: &Document<TH>) {
        self.super_type().unwrap().adopting_steps(old_doc);
        // Step 1.
        let doc = document_from_node(self).appropriate_template_contents_owner_document();
        // Step 2.
        Node::<TH>::adopt(self.Content().upcast(), &doc);
    }

    /// <https://html.spec.whatwg.org/multipage/#the-template-element:concept-node-clone-ext>
    fn cloning_steps(&self, copy: &Node<TH>, maybe_doc: Option<&Document<TH>>,
                     clone_children: CloneChildrenFlag) {
        self.super_type().unwrap().cloning_steps(copy, maybe_doc, clone_children);
        if clone_children == CloneChildrenFlag::DoNotCloneChildren {
            // Step 1.
            return;
        }
        let copy = copy.downcast::<HTMLTemplateElement<TH>>().unwrap();
        // Steps 2-3.
        let copy_contents = DomRoot::upcast::<Node<TH>>(copy.Content());
        let copy_contents_doc = copy_contents.owner_doc();
        for child in self.Content().upcast::<Node<TH>>().children() {
            let copy_child = Node::<TH>::clone(
                &child, Some(&copy_contents_doc), CloneChildrenFlag::CloneChildren);
            copy_contents.AppendChild(&copy_child).unwrap();
        }
    }
}
