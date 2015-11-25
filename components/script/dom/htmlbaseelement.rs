/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLBaseElementBinding;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, document_from_node};
use dom::virtualmethods::VirtualMethods;
use url::{Url, UrlParser};
use util::str::DOMString;

#[dom_struct]
pub struct HTMLBaseElement {
    htmlelement: HTMLElement
}

impl HTMLBaseElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document) -> HTMLBaseElement {
        HTMLBaseElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLBaseElement> {
        let element = HTMLBaseElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLBaseElementBinding::Wrap)
    }

    /// https://html.spec.whatwg.org/multipage/#frozen-base-url
    pub fn frozen_base_url(&self) -> Url {
        let href = self.upcast::<Element>().get_attribute(&ns!(), &atom!("href"))
            .expect("The frozen base url is only defined for base elements \
                     that have a base url.");
        let document = document_from_node(self);
        let base = document.fallback_base_url();
        let parsed = UrlParser::new().base_url(&base).parse(&href.value());
        parsed.unwrap_or(base)
    }

    /// Update the cached base element in response to binding or unbinding from
    /// a tree.
    pub fn bind_unbind(&self, tree_in_doc: bool) {
        if !tree_in_doc {
            return;
        }

        if self.upcast::<Element>().has_attribute(&atom!("href")) {
            let document = document_from_node(self);
            document.refresh_base_element();
        }
    }
}

impl VirtualMethods for HTMLBaseElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        if *attr.local_name() == atom!("href") {
            document_from_node(self).refresh_base_element();
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        self.super_type().unwrap().bind_to_tree(tree_in_doc);
        self.bind_unbind(tree_in_doc);
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        self.super_type().unwrap().unbind_from_tree(tree_in_doc);
        self.bind_unbind(tree_in_doc);
    }
}
