/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLBaseElementBinding;
use dom::bindings::codegen::Bindings::HTMLBaseElementBinding::HTMLBaseElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, UnbindContext, document_from_node};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use servo_url::ServoUrl;

#[dom_struct]
pub struct HTMLBaseElement {
    htmlelement: HTMLElement
}

impl HTMLBaseElement {
    fn new_inherited(local_name: LocalName, prefix: Option<Prefix>, document: &Document) -> HTMLBaseElement {
        HTMLBaseElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document) -> DomRoot<HTMLBaseElement> {
        Node::reflect_node(Box::new(HTMLBaseElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLBaseElementBinding::Wrap)
    }

    /// https://html.spec.whatwg.org/multipage/#frozen-base-url
    pub fn frozen_base_url(&self) -> ServoUrl {
        let href = self.upcast::<Element>().get_attribute(&ns!(), &local_name!("href"))
            .expect("The frozen base url is only defined for base elements \
                     that have a base url.");
        let document = document_from_node(self);
        let base = document.fallback_base_url();
        let parsed = base.join(&href.value());
        parsed.unwrap_or(base)
    }

    /// Update the cached base element in response to binding or unbinding from
    /// a tree.
    pub fn bind_unbind(&self, tree_in_doc: bool) {
        if !tree_in_doc {
            return;
        }

        if self.upcast::<Element>().has_attribute(&local_name!("href")) {
            let document = document_from_node(self);
            document.refresh_base_element();
        }
    }
}

impl HTMLBaseElementMethods for HTMLBaseElement {
    // https://html.spec.whatwg.org/multipage/#dom-base-href
    fn Href(&self) -> DOMString {
        // Step 1.
        let document = document_from_node(self);

        // Step 2.
        let attr = self.upcast::<Element>().get_attribute(&ns!(), &local_name!("href"));
        let value = attr.as_ref().map(|attr| attr.value());
        let url = value.as_ref().map_or("", |value| &**value);

        // Step 3.
        let url_record = document.fallback_base_url().join(url);

        match url_record {
            Err(_) => {
                // Step 4.
                url.into()
            }
            Ok(url_record) => {
                // Step 5.
                url_record.into_string().into()
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-base-href
    make_setter!(SetHref, "href");
}

impl VirtualMethods for HTMLBaseElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        if *attr.local_name() == local_name!("href") {
            document_from_node(self).refresh_base_element();
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        self.super_type().unwrap().bind_to_tree(tree_in_doc);
        self.bind_unbind(tree_in_doc);
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);
        self.bind_unbind(context.tree_in_doc);
    }
}
