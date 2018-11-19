/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::HTMLHeadElementBinding;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, RootedReference};
use crate::dom::document::{determine_policy_for_token, Document};
use crate::dom::element::Element;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlmetaelement::HTMLMetaElement;
use crate::dom::node::{document_from_node, Node};
use crate::dom::userscripts::load_script;
use crate::dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};

#[dom_struct]
pub struct HTMLHeadElement {
    htmlelement: HTMLElement,
}

impl HTMLHeadElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLHeadElement {
        HTMLHeadElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> DomRoot<HTMLHeadElement> {
        Node::reflect_node(
            Box::new(HTMLHeadElement::new_inherited(local_name, prefix, document)),
            document,
            HTMLHeadElementBinding::Wrap,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#meta-referrer>
    pub fn set_document_referrer(&self) {
        let doc = document_from_node(self);

        if doc.GetHead().r() != Some(self) {
            return;
        }

        let node = self.upcast::<Node>();
        let candidates = node
            .traverse_preorder()
            .filter_map(DomRoot::downcast::<Element>)
            .filter(|elem| elem.is::<HTMLMetaElement>())
            .filter(|elem| elem.get_string_attribute(&local_name!("name")) == "referrer")
            .filter(|elem| {
                elem.get_attribute(&ns!(), &local_name!("content"))
                    .is_some()
            });

        for meta in candidates {
            if let Some(content) = meta.get_attribute(&ns!(), &local_name!("content")).r() {
                let content = content.value();
                let content_val = content.trim();
                if !content_val.is_empty() {
                    doc.set_referrer_policy(determine_policy_for_token(content_val));
                    return;
                }
            }
        }
    }
}

impl VirtualMethods for HTMLHeadElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }
    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }
        load_script(self);
    }
}
