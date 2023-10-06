/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::document::Document;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{document_from_node, CloneChildrenFlag, Node};
use crate::dom::virtualmethods::VirtualMethods;

#[dom_struct]
pub struct HTMLTemplateElement {
    htmlelement: HTMLElement,

    /// <https://html.spec.whatwg.org/multipage/#template-contents>
    contents: MutNullableDom<DocumentFragment>,
}

impl HTMLTemplateElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLTemplateElement {
        HTMLTemplateElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            contents: MutNullableDom::new(None),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLTemplateElement> {
        let n = Node::reflect_node_with_proto(
            Box::new(HTMLTemplateElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        );

        n.upcast::<Node>().set_weird_parser_insertion_mode();
        n
    }
}

impl HTMLTemplateElementMethods for HTMLTemplateElement {
    /// <https://html.spec.whatwg.org/multipage/#dom-template-content>
    fn Content(&self) -> DomRoot<DocumentFragment> {
        self.contents.or_init(|| {
            let doc = document_from_node(self);
            doc.appropriate_template_contents_owner_document()
                .CreateDocumentFragment()
        })
    }
}

impl VirtualMethods for HTMLTemplateElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    /// <https://html.spec.whatwg.org/multipage/#template-adopting-steps>
    fn adopting_steps(&self, old_doc: &Document) {
        self.super_type().unwrap().adopting_steps(old_doc);
        // Step 1.
        let doc = document_from_node(self).appropriate_template_contents_owner_document();
        // Step 2.
        Node::adopt(self.Content().upcast(), &doc);
    }

    /// <https://html.spec.whatwg.org/multipage/#the-template-element:concept-node-clone-ext>
    fn cloning_steps(
        &self,
        copy: &Node,
        maybe_doc: Option<&Document>,
        clone_children: CloneChildrenFlag,
    ) {
        self.super_type()
            .unwrap()
            .cloning_steps(copy, maybe_doc, clone_children);
        if clone_children == CloneChildrenFlag::DoNotCloneChildren {
            // Step 1.
            return;
        }
        let copy = copy.downcast::<HTMLTemplateElement>().unwrap();
        // Steps 2-3.
        let copy_contents = DomRoot::upcast::<Node>(copy.Content());
        let copy_contents_doc = copy_contents.owner_doc();
        for child in self.Content().upcast::<Node>().children() {
            let copy_child = Node::clone(
                &child,
                Some(&copy_contents_doc),
                CloneChildrenFlag::CloneChildren,
            );
            copy_contents.AppendChild(&copy_child).unwrap();
        }
    }
}
