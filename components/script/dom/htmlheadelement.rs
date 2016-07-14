/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::HTMLHeadElementBinding;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{Root, RootedReference};
use dom::bindings::str::DOMString;
use dom::document::{Document, determine_policy_for_token};
use dom::element::Element;
use dom::htmlelement::HTMLElement;
use dom::htmlmetaelement::HTMLMetaElement;
use dom::node::{Node, document_from_node};
use dom::userscripts::load_script;
use dom::virtualmethods::VirtualMethods;
use string_cache::Atom;

#[dom_struct]
pub struct HTMLHeadElement {
    htmlelement: HTMLElement
}

impl HTMLHeadElement {
    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLHeadElement {
        HTMLHeadElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLHeadElement> {
        Node::reflect_node(box HTMLHeadElement::new_inherited(localName, prefix, document),
                           document,
                           HTMLHeadElementBinding::Wrap)
    }

    /// https://html.spec.whatwg.org/multipage/#meta-referrer
    pub fn set_document_referrer(&self) {
        let doc = document_from_node(self);

        if doc.GetHead().r() != Some(self) {
            return;
        }

        let node = self.upcast::<Node>();
        let candidates = node.traverse_preorder()
                             .filter_map(Root::downcast::<Element>)
                             .filter(|elem| elem.is::<HTMLMetaElement>())
                             .filter(|elem| elem.get_string_attribute(&atom!("name")) == "referrer")
                             .filter(|elem| elem.get_attribute(&ns!(), &atom!("content")).is_some());

        for meta in candidates {
            if let Some(content) = meta.get_attribute(&ns!(), &atom!("content")).r() {
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
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }
    fn bind_to_tree(&self, _tree_in_doc: bool) {
        load_script(self);
    }
}
